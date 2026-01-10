# Polyglot Compute Lab: Doubly Linked List Benchmark

This repository benchmarks the implementation of a **Doubly Linked List** across four different layers of abstraction:
1.  **Python** (High-Level, GC managed)
2.  **Rust** (Safe Systems, Ownership model via `Rc<RefCell>`)
3.  **Zig** (Modern C-style, Manual Allocator, Safety checks)
4.  **WebAssembly Text (WAT)** (Raw Assembly, Manual pointer arithmetic)

## Prerequisites
* Rust (`rustup`, `cargo`)
* Python 3.8+ (`pip install maturin`)
* Zig (`0.11.0` or later)
* WABT (`wat2wasm` tool for assembling WAT)
* Node.js (for local web server)

## Build & Run

### 1. Build WASM Targets (Rust, Zig, WAT)
```bash
./build_all.sh

```

### 2. Run Web Benchmark (Rust WASM vs Zig vs WAT)

```bash
cd www
npx http-server .
# Open http://localhost:8080

```

### 3. Run Python Benchmark (Python vs Rust Native)

```bash
maturin develop --release --features python
python python/benchmark.py

```

# Result

## Sum
```
$ python python/benchmark.py
--- Polyglot Benchmark (N=100,000, Repeat=10) ---
Running benchmarks...

Implementation            | Time (avg)   | Speedup (vs Py Class) 
-----------------------------------------------------------------
🛡️ Rust (Zipper (Safe Rust)) |    0.08 ms |            251.04x
🚀 Rust (Bump)             |    0.36 ms |             57.29x
📦 Zig (WASM)              |    1.04 ms |             19.67x
⚡ Rust (Unsafe)           |    1.33 ms |             15.33x
📦 WAT (WASM)              |    1.46 ms |             14.01x
🛡️ Rust (Safe)             |    2.81 ms |              7.28x
🐍 Python (deque)          |    3.18 ms |              6.42x
🐌 Python (Pure Class)     |   20.42 ms |              1.00x
🎮 WGPU (WebGPU)           |  314.98 ms |              0.06x
-----------------------------------------------------------------
```

## Random Insert
```
$ python python/benchmark2.py
Loaded ./zig_zipper.so
--- Random Insertion Benchmark (N=30,000, Repeat=5) ---
Note: Simulating O(N) random insertions.
------------------------------------------------------------
Implementation       | Time (avg) | vs Py List
------------------------------------------------------------
Rust (Unsafe-Zipper) |    10.09 ms |     4.09x
Zig (Zipper-Native)  |    10.59 ms |     3.90x
Rust (Safe-Zipper)   |    18.91 ms |     2.18x
Python (list)        |    41.28 ms |     1.00x
Zig (Zipper-WASM)    |    48.32 ms |     0.85x
Python (deque)       |    58.84 ms |     0.70x
Rust (Unsafe-Bump)   |  1524.71 ms |     0.03x
Rust (Unsafe)        |  1960.06 ms |     0.02x
Rust (Safe)          |  2555.37 ms |     0.02x
------------------------------------------------------------
```
