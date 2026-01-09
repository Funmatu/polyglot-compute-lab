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

// ========================================================
// WGPU (WebGPU) Implementation
// Impl: GPU Compute Shader with Atomic Bump Allocator
// ========================================================

//#[cfg(feature = "wasm")]
//use wasm_bindgen_futures::spawn_local;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub async fn run_wgpu_dll(iterations: u32) -> f64 { // Result is passed as f64 for simplicity
    use wgpu::util::DeviceExt;

    // 1. Initialize WebGPU
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .expect("Failed to find an appropriate adapter");

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(), // WebGL互換性のため緩く
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        )
        .await
        .expect("Failed to create device");

    // 2. Define WGSL Shader
    // GPU上で動く「第6の言語」のソースコード
    let shader_source = format!(r#"
        struct Node {{
            value: i32,
            next: u32, // Pointer (Index)
            prev: u32, // Pointer (Index)
            padding: u32, // Alignment
        }};

        struct Allocator {{
            counter: atomic<u32>,
        }};

        struct Result {{
            sum: i32,
        }};

        @group(0) @binding(0) var<storage, read_write> heap: array<Node>;
        @group(0) @binding(1) var<storage, read_write> alloc: Allocator;
        @group(0) @binding(2) var<storage, read_write> head_tail: array<u32, 2>; // 0:head, 1:tail
        @group(0) @binding(3) var<storage, read_write> result: Result;

        // GPU Allocator (Atomic Bump)
        fn alloc_node(val: i32) -> u32 {{
            // アトミックにインデックスを取得 (ここがポインタ生成)
            let ptr = atomicAdd(&alloc.counter, 1u);
            
            // 初期化
            heap[ptr].value = val;
            heap[ptr].next = 0u; // null
            heap[ptr].prev = 0u; // null
            return ptr;
        }}

        @compute @workgroup_size(1)
        fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
            // シングルスレッドで実行 (Linked Listは直列構造なので)
            // ※ここを並列化するのがGPUの醍醐味だが、今回はCPUと同じロジックを再現する
            
            let iters = {}u; // Rustから注入された定数

            // 1. Append Loop
            for (var i = 0u; i < iters; i++) {{
                let val = i32(i);
                let new_ptr = alloc_node(val);
                let old_tail = head_tail[1]; // Get current tail

                if (old_tail != 0u) {{
                    heap[old_tail].next = new_ptr;
                    heap[new_ptr].prev = old_tail;
                    head_tail[1] = new_ptr; // Update tail
                }} else {{
                    head_tail[0] = new_ptr; // Head
                    head_tail[1] = new_ptr; // Tail
                }}
            }}

            // 2. Traversal Loop (Sum)
            var current = head_tail[0];
            var s = 0;
            
            // 安全装置: 無限ループ防止のため最大回数を制限
            for (var k = 0u; k < iters + 10u; k++) {{
                if (current == 0u) {{ break; }}
                s += heap[current].value;
                current = heap[current].next;
            }}
            
            result.sum = s;
        }}
    "#, iterations);

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("DLL Shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    // 3. Setup Buffers
    // Node Heap (128MB) - GPUメモリは大きいので豪勢に
    let heap_size = 128 * 1024 * 1024; 
    let buffer_heap = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Heap Buffer"),
        size: heap_size,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });

    // Allocator Counter (Initialize to 1, as 0 is null)
    let buffer_alloc = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Alloc Buffer"),
        contents: bytemuck::cast_slice(&[1u32]), // Start from index 1
        usage: wgpu::BufferUsages::STORAGE,
    });

    // Head/Tail [0, 0]
    let buffer_head_tail = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("HeadTail Buffer"),
        contents: bytemuck::cast_slice(&[0u32, 0u32]), 
        usage: wgpu::BufferUsages::STORAGE,
    });

    // Result Buffer (Output)
    let buffer_result = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Result Buffer"),
        size: 4, // i32
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    // Staging Buffer (For reading back to CPU)
    let buffer_staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Staging Buffer"),
        size: 4,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // 4. Pipeline & BindGroup
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None }, // Heap
            wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None }, // Alloc
            wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None }, // HeadTail
            wgpu::BindGroupLayoutEntry { binding: 3, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None }, // Result
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

    // 5. Execute
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(1, 1, 1); // Single Thread Execution!
    }
    
    // Copy result to staging
    encoder.copy_buffer_to_buffer(&buffer_result, 0, &buffer_staging, 0, 4);
    queue.submit(Some(encoder.finish()));

    // 6. Read back
    let buffer_slice = buffer_staging.slice(..);
    let (sender, receiver) = futures::channel::oneshot::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result: Result<(), wgpu::BufferAsyncError>| {
        sender.send(result).unwrap();
    });
    
    device.poll(wgpu::Maintain::Wait); // Wait for GPU
    
    if let Ok(Ok(())) = receiver.await {
        let data = buffer_slice.get_mapped_range();
        let result: i32 = *bytemuck::from_bytes(&data[..]);
        drop(data);
        buffer_staging.unmap();
        return result as f64;
    }

    return -1.0;
}