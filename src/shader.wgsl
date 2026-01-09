struct Node {
    value: i32,
    next: u32,
    prev: u32,
    padding: u32,
}

struct Allocator {
    counter: atomic<u32>,
}

struct Result {
    sum: i32,
}

@group(0) @binding(0) var<storage, read_write> heap: array<Node>;
@group(0) @binding(1) var<storage, read_write> alloc: Allocator;
@group(0) @binding(2) var<storage, read_write> head_tail: array<u32, 2>;
@group(0) @binding(3) var<storage, read_write> result: Result;

fn alloc_node(val: i32) -> u32 {
    // 【修正】ptr -> node_idx に変更
    let node_idx = atomicAdd(&alloc.counter, 1u);
    
    heap[node_idx].value = val;
    heap[node_idx].next = 0u;
    heap[node_idx].prev = 0u;
    
    return node_idx;
}

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let iters = ITERATIONS_PLACEHOLDER; 

    for (var i = 0u; i < iters; i++) {
        let val = i32(i);
        // 【修正】受け取る変数名も変更
        let new_idx = alloc_node(val);
        let old_tail = head_tail[1];

        if (old_tail != 0u) {
            heap[old_tail].next = new_idx;
            heap[new_idx].prev = old_tail;
            head_tail[1] = new_idx;
        } else {
            head_tail[0] = new_idx;
            head_tail[1] = new_idx;
        }
    }

    var current = head_tail[0];
    var s = 0;
    for (var k = 0u; k < iters + 10u; k++) {
        if (current == 0u) { break; }
        s += heap[current].value;
        current = heap[current].next;
    }
    result.sum = s;
}