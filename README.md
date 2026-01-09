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
