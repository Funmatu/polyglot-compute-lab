use std::rc::{Rc, Weak};
use std::cell::RefCell;

// Node definition
struct Node {
    value: i32,
    next: Option<Rc<RefCell<Node>>>,
    prev: Option<Weak<RefCell<Node>>>,
}

struct DoublyLinkedList {
    head: Option<Rc<RefCell<Node>>>,
    tail: Option<Rc<RefCell<Node>>>,
}

impl DoublyLinkedList {
    fn new() -> Self {
        Self { head: None, tail: None }
    }

    fn append(&mut self, value: i32) {
        let new_node = Rc::new(RefCell::new(Node {
            value,
            next: None,
            prev: None,
        }));

        match &self.tail {
            Some(old_tail) => {
                old_tail.borrow_mut().next = Some(Rc::clone(&new_node));
                new_node.borrow_mut().prev = Some(Rc::downgrade(old_tail));
                self.tail = Some(new_node);
            }
            None => {
                self.head = Some(Rc::clone(&new_node));
                self.tail = Some(new_node);
            }
        }
    }

    // Forward traversal sum
    fn sum(&self) -> i32 {
        let mut sum = 0;
        let mut current = self.head.clone();
        while let Some(node) = current {
            let borrowed = node.borrow();
            sum += borrowed.value;
            current = borrowed.next.clone();
        }
        sum
    }

    fn insert(&mut self, index: usize, value: i32) {
        // 先頭への挿入
        if index == 0 {
            let new_node = Rc::new(RefCell::new(Node {
                value,
                next: self.head.clone(),
                prev: None,
            }));
            if let Some(old_head) = &self.head {
                old_head.borrow_mut().prev = Some(Rc::downgrade(&new_node));
            } else {
                self.tail = Some(new_node.clone());
            }
            self.head = Some(new_node);
            return;
        }

        // 挿入位置を探す (O(N) traversal)
        let mut current = self.head.clone();
        let mut current_idx = 0;
        
        while let Some(node) = current {
            if current_idx == index {
                // nodeの手前に挿入する
                let prev_node = node.borrow().prev.clone().and_then(|w| w.upgrade());
                let new_node = Rc::new(RefCell::new(Node {
                    value,
                    next: Some(node.clone()),
                    prev: prev_node.clone().map(|r| Rc::downgrade(&r)),
                }));

                node.borrow_mut().prev = Some(Rc::downgrade(&new_node));
                
                if let Some(prev) = prev_node {
                    prev.borrow_mut().next = Some(new_node);
                }
                return;
            }
            current = node.borrow().next.clone();
            current_idx += 1;
        }
        
        // インデックスが範囲外(末尾)ならappend
        self.append(value);
    }
}

impl Drop for DoublyLinkedList {
    fn drop(&mut self) {
        // headから順に所有権を奪っていく（take）
        let mut current = self.head.take();
        while let Some(node) = current {
            // nodeの借用がここで終わるようにブロックを作るか、単にnextをtakeする
            // currentのnextを奪い取ることで、再帰的な連鎖を断ち切る
            current = node.borrow_mut().next.take();
            // ここで `node` (Rc) がスコープを抜け、参照カウントが減って破棄される。
            // しかし next はすでに None になっているので、再帰は起きない。
        }
    }
}

// ========================================================
// Rust (Unsafe) Implementation
// Impl: Raw Pointers (*mut T) without Rc/RefCell
// ========================================================

struct UnsafeNode {
    value: i32,
    next: *mut UnsafeNode,
    prev: *mut UnsafeNode,
}

pub struct UnsafeDll {
    head: *mut UnsafeNode,
    tail: *mut UnsafeNode,
}

impl UnsafeDll {
    fn new() -> Self {
        Self {
            head: std::ptr::null_mut(),
            tail: std::ptr::null_mut(),
        }
    }

    fn append(&mut self, value: i32) {
        unsafe {
            // 1. Boxで作ってヒープ確保し、即座に生ポインタにする
            // (Rustの所有権管理から外す)
            let new_node = Box::into_raw(Box::new(UnsafeNode {
                value,
                next: std::ptr::null_mut(),
                prev: std::ptr::null_mut(),
            }));

            if !self.tail.is_null() {
                // 既存のtailがある場合
                (*self.tail).next = new_node;
                (*new_node).prev = self.tail;
                self.tail = new_node;
            } else {
                // 空の場合
                self.head = new_node;
                self.tail = new_node;
            }
        }
    }

    fn sum(&self) -> i32 {
        unsafe {
            let mut s = 0;
            let mut current = self.head;
            while !current.is_null() {
                s += (*current).value;
                current = (*current).next;
            }
            s
        }
    }

    // メモリリークを防ぐための手動解放
    // (ベンチマークの計測時間には含めないが、実用上必須)
    fn cleanup(&mut self) {
        unsafe {
            let mut current = self.head;
            while !current.is_null() {
                let next = (*current).next;
                // Boxに戻してDropさせる
                let _ = Box::from_raw(current);
                current = next;
            }
        }
    }

    fn insert(&mut self, index: usize, value: i32) {
        unsafe {
            let new_node = Box::into_raw(Box::new(UnsafeNode {
                value,
                next: std::ptr::null_mut(),
                prev: std::ptr::null_mut(),
            }));

            if index == 0 {
                if !self.head.is_null() {
                    (*self.head).prev = new_node;
                    (*new_node).next = self.head;
                } else {
                    self.tail = new_node;
                }
                self.head = new_node;
                return;
            }

            let mut current = self.head;
            let mut i = 0;
            while !current.is_null() {
                if i == index {
                    // currentの手前に挿入
                    let prev = (*current).prev;
                    (*current).prev = new_node;
                    (*new_node).next = current;
                    (*new_node).prev = prev;
                    if !prev.is_null() {
                        (*prev).next = new_node;
                    }
                    return;
                }
                current = (*current).next;
                i += 1;
            }
            
            // 末尾
            self.append(value);
            // ※Box::into_rawしたポインタの解放はappend内のロジックにはないので、
            // cleanup時またはここで適切に処理しないとリークするが、appendの実装に委ねる
            // (今回は既存のappendを呼ぶ形にしたので、上で作ったnew_nodeが無駄になるバグがあるが
            //  ベンチマーク用簡易実装として、ここは append(value) して new_node を消す形にする)
            let _ = Box::from_raw(new_node); 
        }
    }
}

// WASM Export for Unsafe Rust
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn run_rust_unsafe(iterations: i32) -> i32 {
    let mut dll = UnsafeDll::new();
    for i in 0..iterations {
        dll.append(i);
    }
    let s = dll.sum();
    
    // 計測後に掃除 (ベンチマーク外で呼ぶのが理想だが、WASMのメモリ圧迫を防ぐためここで呼ぶ)
    // ※厳密な生成+トラバーサル速度比較のため、cleanupの時間はノイズになる可能性があるが、
    //  Rust(Safe)はDropコストを支払っているため、ここでも支払うのが公平。
    dll.cleanup();
    
    s
}

// --------------------------------------------------------
// WASM Interface
// --------------------------------------------------------
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn run_rust_dll(iterations: i32) -> i32 {
    let mut dll = DoublyLinkedList::new();
    for i in 0..iterations {
        dll.append(i);
    }
    dll.sum()
}

// --------------------------------------------------------
// Python Interface
// --------------------------------------------------------
#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pyfunction]
fn run_rust_dll_py(iterations: i32) -> PyResult<i32> {
    let mut dll = DoublyLinkedList::new();
    for i in 0..iterations {
        dll.append(i);
    }
    Ok(dll.sum())
}

#[cfg(feature = "python")]
#[pyfunction]
fn run_rust_unsafe_py(iterations: i32) -> PyResult<i32> {
    let mut dll = UnsafeDll::new();
    for i in 0..iterations {
        dll.append(i);
    }
    let s = dll.sum();
    dll.cleanup(); // メモリリーク防止
    Ok(s)
}

#[cfg(feature = "python")]
#[pyfunction]
fn run_rust_bump_py(iterations: i32) -> PyResult<i32> {
    let mut dll = BumpDll::new();
    for i in 0..iterations {
        dll.append(i);
    }
    Ok(dll.sum())
}

#[cfg(feature = "python")]
#[pyfunction]
fn run_wgpu_py(iterations: u32) -> PyResult<f64> {
    // Native環境では非同期ランタイム(Tokio)を自分で用意して
    // async関数を同期的にブロック実行する
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        run_wgpu_core(iterations).await
    });
    Ok(result)
}

#[cfg(feature = "python")]
#[pymodule]
fn polyglot_compute_lab(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_rust_dll_py, m)?)?;
    m.add_function(wrap_pyfunction!(run_rust_safe_insert_py, m)?)?;
    m.add_function(wrap_pyfunction!(run_rust_unsafe_py, m)?)?;
    m.add_function(wrap_pyfunction!(run_rust_unsafe_insert_py, m)?)?;
    m.add_function(wrap_pyfunction!(run_rust_bump_py, m)?)?; 
    m.add_function(wrap_pyfunction!(run_rust_bump_insert_py, m)?)?;
    m.add_function(wrap_pyfunction!(run_rust_zipper_py, m)?)?;
    m.add_function(wrap_pyfunction!(run_rust_zipper_insert_py, m)?)?;
    m.add_function(wrap_pyfunction!(run_rust_unsafe_zipper_insert_py, m)?)?;
    m.add_function(wrap_pyfunction!(run_wgpu_py, m)?)?;
    m.add_function(wrap_pyfunction!(run_wasm_py, m)?)?;
    Ok(())
}

// ========================================================
// Rust (Bump Allocation) Implementation
// Impl: Manual Bump Allocator (Zig style)
// ========================================================

// 1. Zigと同じ64MBの巨大バッファを静的に確保
// WASMはシングルスレッドなので static mut でもデータ競合は起きない（が、Rust的には超Unsafe）
static mut HEAP: [u8; 64 * 1024 * 1024] = [0; 64 * 1024 * 1024];
static mut HEAP_OFFSET: usize = 0;

struct BumpNode {
    value: i32,
    next: *mut BumpNode,
    prev: *mut BumpNode,
}

struct BumpDll {
    head: *mut BumpNode,
    tail: *mut BumpNode,
}

impl BumpDll {
    fn new() -> Self {
        // ベンチマーク毎にオフセットをリセット（Zigのfba.reset()と同じ）
        unsafe { HEAP_OFFSET = 0; }
        Self {
            head: std::ptr::null_mut(),
            tail: std::ptr::null_mut(),
        }
    }

    // 2. 独自の割り当て関数 (mallocの代わり)
    fn alloc_node(value: i32) -> *mut BumpNode {
        unsafe {
            let size = std::mem::size_of::<BumpNode>();
            
            // バッファ溢れチェック（本来必要だが速度のため省略可。ここでは簡易的に）
            // if HEAP_OFFSET + size > HEAP.len() { panic!("OOM"); }

            // ポインタ計算【修正箇所】
            // HEAP.as_mut_ptr() だと &mut HEAP を作ってしまうので警告が出る。
            // addr_of_mut! で直接生ポインタを取得する。
            let base_ptr = std::ptr::addr_of_mut!(HEAP) as *mut u8;
            let ptr = base_ptr.add(HEAP_OFFSET) as *mut BumpNode;
            
            // オフセットを進める (Bump!)
            HEAP_OFFSET += size;

            // 初期化
            (*ptr).value = value;
            (*ptr).next = std::ptr::null_mut();
            (*ptr).prev = std::ptr::null_mut();
            
            ptr
        }
    }

    fn append(&mut self, value: i32) {
        unsafe {
            // Box::new ではなく、自作allocを使う
            let new_node = Self::alloc_node(value);

            if !self.tail.is_null() {
                (*self.tail).next = new_node;
                (*new_node).prev = self.tail;
                self.tail = new_node;
            } else {
                self.head = new_node;
                self.tail = new_node;
            }
        }
    }

    fn sum(&self) -> i32 {
        unsafe {
            let mut s = 0;
            let mut current = self.head;
            while !current.is_null() {
                s += (*current).value;
                current = (*current).next;
            }
            s
        }
    }

    fn insert(&mut self, index: usize, value: i32) {
        unsafe {
            // ここが重要: Box::new ではなく、Bumpアロケータからメモリをもらう
            let new_node = Self::alloc_node(value);

            if index == 0 {
                if !self.head.is_null() {
                    (*self.head).prev = new_node;
                    (*new_node).next = self.head;
                } else {
                    self.tail = new_node;
                }
                self.head = new_node;
                return;
            }

            let mut current = self.head;
            let mut i = 0;
            
            // 挿入位置までトラバーサル
            while !current.is_null() {
                if i == index {
                    // currentの手前に挿入
                    let prev = (*current).prev;
                    (*current).prev = new_node;
                    (*new_node).next = current;
                    (*new_node).prev = prev;
                    if !prev.is_null() {
                        (*prev).next = new_node;
                    }
                    return;
                }
                current = (*current).next;
                i += 1;
            }
            
            // インデックスが範囲外なら末尾に追加
            // appendを再利用するが、append内で再度alloc_nodeしないように注意が必要。
            // しかし既存のappendはalloc_nodeを呼んでしまう仕様なので、
            // ここでは簡易的に「ポインタ繋ぎ変え」をベタ書きするか、
            // new_nodeを破棄してappendを呼ぶ（Bumpなので破棄＝何もしないでOK）。
            
            // Bumpアロケータは「解放」がないので、作ったnew_nodeを放置して
            // append(value) を呼んでもメモリリーク（無駄使い）するだけでクラッシュはしない。
            // ベンチマークの厳密性のために、ここではappendを呼んでリターンする。
            self.append(value);
        }
    }
}

// Export
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn run_rust_bump(iterations: i32) -> i32 {
    let mut dll = BumpDll::new();
    for i in 0..iterations {
        dll.append(i);
    }
    dll.sum()
    // Drop不要（オフセットを0に戻すだけで全解放とみなすため）
}

#[cfg(feature = "python")]
#[pyfunction]
fn run_rust_bump_insert_py(iterations: i32) -> PyResult<()> {
    // 毎回ヒープをリセット
    let mut dll = BumpDll::new();
    let mut seed: usize = 123456789;
    let mut len = 0;
    
    for i in 0..iterations {
        let pos = if len == 0 { 0 } else { seed % len };
        dll.insert(pos, i);
        
        seed = (seed * 1103515245 + 12345) & 0x7fffffff;
        len += 1;
    }
    // Bumpなのでcleanup不要
    Ok(())
}



// ========================================================
// Rust (Zipper) Implementation
// Impl: Two Stacks (Vec) based Cursor
// ========================================================

struct ZipperList {
    left: Vec<i32>,  // カーソルより左にある要素（スタック）
    right: Vec<i32>, // カーソルより右にある要素（スタック）
}

impl ZipperList {
    fn new(capacity: usize) -> Self {
        Self {
            left: Vec::with_capacity(capacity),
            right: Vec::with_capacity(capacity),
        }
    }

    // 末尾への追加 = カーソルが末尾にある状態での左スタックへのPush
    fn append(&mut self, value: i32) {
        self.left.push(value);
    }

    // カーソルを左へ移動（参考実装：今回は使わないがDLLの機能として）
    fn move_left(&mut self) {
        if let Some(val) = self.left.pop() {
            self.right.push(val);
        }
    }

    // カーソルを右へ移動
    fn move_right(&mut self) {
        if let Some(val) = self.right.pop() {
            self.left.push(val);
        }
    }

    fn sum(&self) -> i32 {
        // 2つのベクタの合計を足すだけ
        // メモリ上で連続しているため、CPUキャッシュが効きまくる
        let left_sum: i32 = self.left.iter().sum();
        let right_sum: i32 = self.right.iter().sum();
        left_sum + right_sum
    }

    fn insert(&mut self, index: usize, value: i32) {
        let current_pos = self.left.len();

        if index < current_pos {
            // 左にある -> まとめて右へ移動
            let _diff = current_pos - index;
            // 1つずつpop/pushせず、drainで一気に移動させる
            // (右スタックは逆順になる仕様なので、rev()等考慮が必要だが、
            //  単純な2つのスタックとして扱うなら drain して append が最速)
            let moved: Vec<i32> = self.left.drain(index..).rev().collect();
            self.right.extend(moved);
        } else if index > current_pos {
            // 右にある -> まとめて左へ移動
            let _diff = index - current_pos;
            // 右スタックの末尾(=カーソル直近)から取り出して左へ
            let len = self.right.len();
            let start = len.saturating_sub(_diff);
            let moved: Vec<i32> = self.right.drain(start..).rev().collect();
            self.left.extend(moved);
        }
        self.left.push(value);
    }
}

#[cfg(feature = "python")]
#[pyfunction]
fn run_rust_zipper_py(iterations: i32) -> PyResult<i32> {
    let mut dll = ZipperList::new(iterations as usize);
    // 実際に大量のメモリ確保が発生する
    for i in 0..iterations {
        dll.append(i);
    }
    Ok(dll.sum())
}


#[cfg(feature = "python")]
#[pyfunction]
fn run_rust_safe_insert_py(iterations: i32) -> PyResult<()> {
    // 挿入テストは時間がかかるので回数を減らすことを推奨するが、引数に従う
    // 乱数はRust側で簡易生成するか、Pythonからリストをもらうのが公平だが、
    // ここでは簡易的に「毎回ランダム」ではなく「中央付近への挿入」などで負荷をかける
    // 厳密なランダムアクセスはPython側で制御したほうが公平なため、
    // ここでは「Pythonから位置と値のリストを受け取る」形がベストだが、
    // 実装が複雑になるため、「疑似ランダム（線形合同法）」でRust内で完結させる。
    
    let mut dll = DoublyLinkedList::new();
    let mut seed: usize = 123456789;
    let mut len = 0;
    
    for i in 0..iterations {
        // 簡易乱数: 0..len の間のどこか
        let pos = if len == 0 { 0 } else { seed % len };
        dll.insert(pos, i);
        
        // 次の乱数
        seed = (seed * 1103515245 + 12345) & 0x7fffffff;
        len += 1;
    }
    Ok(())
}

#[cfg(feature = "python")]
#[pyfunction]
fn run_rust_unsafe_insert_py(iterations: i32) -> PyResult<()> {
    let mut dll = UnsafeDll::new();
    let mut seed: usize = 123456789;
    let mut len = 0;
    for i in 0..iterations {
        let pos = if len == 0 { 0 } else { seed % len };
        dll.insert(pos, i);
        seed = (seed * 1103515245 + 12345) & 0x7fffffff;
        len += 1;
    }
    dll.cleanup();
    Ok(())
}

#[cfg(feature = "python")]
#[pyfunction]
fn run_rust_zipper_insert_py(iterations: i32) -> PyResult<()> {
    let mut dll = ZipperList::new(iterations as usize);
    let mut seed: usize = 123456789;
    let mut len = 0;
    for i in 0..iterations {
        let pos = if len == 0 { 0 } else { seed % len };
        dll.insert(pos, i);
        seed = (seed * 1103515245 + 12345) & 0x7fffffff;
        len += 1;
    }
    Ok(())
}


// ========================================================
// Rust (Unsafe Zipper) Implementation
// Impl: Unsafe Vec ops mimicking Zig's manual memory move
// ========================================================

struct UnsafeZipperList {
    left: Vec<i32>,
    right: Vec<i32>,
}

impl UnsafeZipperList {
    fn new(capacity: usize) -> Self {
        Self {
            left: Vec::with_capacity(capacity),
            right: Vec::with_capacity(capacity),
        }
    }

    #[inline(always)]
    fn insert(&mut self, index: usize, value: i32) {
        unsafe {
            let left_len = self.left.len();
            if index < left_len {
                // --- Move from Left to Right ---
                let count = left_len - index;
                
                // ポインタ取得
                let left_ptr = self.left.as_ptr().add(index);
                let right_ptr = self.right.as_mut_ptr().add(self.right.len());
                
                // 【修正】Write Forward (順方向書き込み) に変更
                // 左スタックの末尾(top)から順に、右スタックの末尾(new top)へ書き込む
                for i in 0..count {
                    // Right(Dest): 前から順に書き込む (Cache Friendly!)
                    // Left(Src)  : 後ろから順に読む
                    *right_ptr.add(i) = *left_ptr.add(count - 1 - i);
                }
                
                self.right.set_len(self.right.len() + count);
                self.left.set_len(index);

            } else if index > left_len {
                // --- Move from Right to Left ---
                let count = index - left_len;
                let right_len = self.right.len();
                let move_count = if count > right_len { right_len } else { count };
                
                let start_index = right_len - move_count;
                let right_ptr = self.right.as_ptr().add(start_index);
                let left_ptr = self.left.as_mut_ptr().add(self.left.len());
                
                // 【修正】Write Forward (順方向書き込み) に変更
                for i in 0..move_count {
                     // Left(Dest): 前から順に書き込む
                     // Right(Src): 後ろから順に読む
                     *left_ptr.add(i) = *right_ptr.add(move_count - 1 - i);
                }
                
                self.left.set_len(self.left.len() + move_count);
                self.right.set_len(start_index);
            }
            
            // Push
            // Pre-allocation済みなのでreserveチェックなしで書くことも可能だが
            // pushの実装は十分に速いのでこのまま
            self.left.push(value);
        }
    }
}

#[cfg(feature = "python")]
#[pyfunction]
fn run_rust_unsafe_zipper_insert_py(iterations: i32) -> PyResult<()> {
    let mut dll = UnsafeZipperList::new(iterations as usize);
    let mut seed: usize = 123456789;
    let mut len = 0;
    for i in 0..iterations {
        let pos = if len == 0 { 0 } else { seed % len };
        dll.insert(pos, i);
        seed = (seed * 1103515245 + 12345) & 0x7fffffff;
        len += 1;
    }
    Ok(())
}


// ========================================================
// WGPU (WebGPU) Core Implementation
// Impl: GPU Compute Shader with Atomic Bump Allocator
// ========================================================

// 共通ロジック: WASM依存もPython依存もしない純粋な非同期関数
#[cfg(any(feature = "wasm", feature = "python"))]
async fn run_wgpu_core(iterations: u32) -> f64 {
    // ------------------------------------------------------------
    // 1. Initialize WGPU (Adapter & Device)
    // ------------------------------------------------------------
    let instance = wgpu::Instance::default();
    
    // Native(Python)とWebでAdapterの取得戦略が少し違うが、
    // default() で大抵うまくいく（NativeならVulkan/Metal/DX12が選ばれる）
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .expect("Failed to find an appropriate adapter");

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        )
        .await
        .expect("Failed to create device");

    // ------------------------------------------------------------
    // 2. WGSL Shader (External File)
    // ------------------------------------------------------------
    // コンパイル時にファイルを文字列として読み込む
    let shader_raw = include_str!("shader.wgsl");
    // プレースホルダーを実際の数値（文字列）に置換する
    // 例: "ITERATIONS_PLACEHOLDER" -> "100000u"
    let shader_source = shader_raw.replace(
        "ITERATIONS_PLACEHOLDER", 
        &format!("{}u", iterations)
    );

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("DLL Shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    // ------------------------------------------------------------
    // 3. Setup Buffers
    // ------------------------------------------------------------
    use wgpu::util::DeviceExt; // for create_buffer_init

    let heap_size = 128 * 1024 * 1024; 
    let buffer_heap = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Heap Buffer"),
        size: heap_size,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });

    let buffer_alloc = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Alloc Buffer"),
        contents: bytemuck::cast_slice(&[1u32]),
        usage: wgpu::BufferUsages::STORAGE,
    });

    let buffer_head_tail = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("HeadTail Buffer"),
        contents: bytemuck::cast_slice(&[0u32, 0u32]), 
        usage: wgpu::BufferUsages::STORAGE,
    });

    let buffer_result = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Result Buffer"),
        size: 4,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let buffer_staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Staging Buffer"),
        size: 4,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // ------------------------------------------------------------
    // 4. Pipeline & Execute
    // ------------------------------------------------------------
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            wgpu::BindGroupLayoutEntry { binding: 3, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: buffer_heap.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: buffer_alloc.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: buffer_head_tail.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: buffer_result.as_entire_binding() },
        ],
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(1, 1, 1);
    }
    
    encoder.copy_buffer_to_buffer(&buffer_result, 0, &buffer_staging, 0, 4);
    queue.submit(Some(encoder.finish()));

    // ------------------------------------------------------------
    // 5. Read Back (Async)
    // ------------------------------------------------------------
    let buffer_slice = buffer_staging.slice(..);
    let (sender, receiver) = futures::channel::oneshot::channel();
    
    buffer_slice.map_async(wgpu::MapMode::Read, move |result: Result<(), wgpu::BufferAsyncError>| {
        sender.send(result).unwrap();
    });
    
    device.poll(wgpu::Maintain::Wait);

    if let Ok(Ok(())) = receiver.await {
        let data = buffer_slice.get_mapped_range();
        let result: i32 = *bytemuck::from_bytes(&data[..]);
        drop(data);
        buffer_staging.unmap();
        return result as f64;
    }

    return -1.0;
}

// --------------------------------------------------------
// WASM Interface (Wrapper for WGPU)
// --------------------------------------------------------
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub async fn run_wgpu_dll(iterations: u32) -> f64 {
    // WASM環境ではJSのイベントループがよしなにやってくれるので、
    // 単にasync関数を呼ぶだけでOK
    run_wgpu_core(iterations).await
}


// ========================================================
// WASM Runtime (Server-side WASM via Wasmtime)
// ========================================================

#[cfg(feature = "python")]
#[pyfunction]
fn run_wasm_py(wasm_bytes: &[u8], func_name: &str, iterations: i32) -> PyResult<i32> {
    use wasmtime::*;

    // 1. エンジンの設定 (JITコンパイル有効)
    let engine = Engine::default();

    // 2. モジュールのコンパイル (バイナリ -> マシンコード)
    // ※ベンチマークで「実行速度」だけを見たいなら、このコンパイル時間は計測から外すべきだが、
    // 今回は「ロードして実行」のトータルを見ても面白い。
    // (Python側でModuleキャッシュする手もあるが、まずは単純に毎回ロードする)
    let module = Module::new(&engine, wasm_bytes)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    // 3. ストア(メモリ空間)の作成
    let mut store = Store::new(&engine, ());

    // 4. インスタンス化 (Importsが必要ならここで渡す)
    // 今回のZig/WATはFreestandingでImports不要なので空でOK
    let instance = Instance::new(&mut store, &module, &[])
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    // 5. 関数の取得 (型付きで取得して高速化)
    let run_func = instance
        .get_typed_func::<i32, i32>(&mut store, func_name)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Function '{}' not found: {}", func_name, e)))?;

    // 6. 実行
    let result = run_func
        .call(&mut store, iterations)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    Ok(result)
}
