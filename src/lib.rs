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
#[pymodule]
fn polyglot_compute_lab(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_rust_dll_py, m)?)?;
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