import timeit
import collections
import os
import sys
import random
import ctypes

try:
    import polyglot_compute_lab
except ImportError:
    print("❌ Error: 'polyglot_compute_lab' not found.")
    sys.exit(1)


WASM_ZIG_ZIPPER_PATH = "www/zig_zipper.wasm"
zig_zipper_binary = None

try:
    with open(WASM_ZIG_ZIPPER_PATH, "rb") as f:
        zig_zipper_binary = f.read()
except FileNotFoundError:
    print(f"⚠️ Warning: {WASM_ZIG_ZIPPER_PATH} not found.")

# --- Zig Native (DLL) Load ---
ZIG_LIB_PATH = "./zig_zipper.so"
zig_native_lib = None

try:
    zig_native_lib = ctypes.CDLL(ZIG_LIB_PATH)
    # 引数と戻り値の型定義
    zig_native_lib.run_zig_zipper.argtypes = [ctypes.c_int32]
    zig_native_lib.run_zig_zipper.restype = ctypes.c_int32
    print(f"Loaded {ZIG_LIB_PATH}")
except OSError:
    print(f"⚠️ Warning: {ZIG_LIB_PATH} not found. Skipping Native Zig.")


# ==========================================
# Python Implementations
# ==========================================
def run_python_list_insert(iterations):
    # Dynamic Array (C-optimized memmove)
    l = []
    seed = 123456789
    current_len = 0
    for i in range(iterations):
        pos = 0 if current_len == 0 else seed % current_len
        l.insert(pos, i)
        seed = (seed * 1103515245 + 12345) & 0x7FFFFFFF
        current_len += 1


def run_python_deque_insert(iterations):
    # Linked Block List
    d = collections.deque()
    seed = 123456789
    current_len = 0
    for i in range(iterations):
        pos = 0 if current_len == 0 else seed % current_len
        d.insert(pos, i)
        seed = (seed * 1103515245 + 12345) & 0x7FFFFFFF
        current_len += 1


# ==========================================
# Rust Implementations (Random logic is inside Rust)
# ==========================================
def run_rust_safe_insert(iterations):
    polyglot_compute_lab.run_rust_safe_insert_py(iterations)


def run_rust_unsafe_insert(iterations):
    polyglot_compute_lab.run_rust_unsafe_insert_py(iterations)


def run_rust_bump_insert(iterations):
    polyglot_compute_lab.run_rust_bump_insert_py(iterations)


def run_rust_zipper_insert(iterations):
    polyglot_compute_lab.run_rust_zipper_insert_py(iterations)


def run_rust_unsafe_zipper_insert(iterations):
    polyglot_compute_lab.run_rust_unsafe_zipper_insert_py(iterations)


# ==========================================
# Zig Implementations (Random logic is inside Zig)
# ==========================================
def run_zig_zipper_insert(iterations):
    if zig_zipper_binary:
        # Rust経由でWASMを実行 (polyglot_compute_lab.run_wasm_py を利用)
        # 引数: (wasmバイナリ, エクスポート関数名, イテレーション数)
        polyglot_compute_lab.run_wasm_py(
            zig_zipper_binary, "run_zig_zipper", iterations
        )
    else:
        print("Zig binary not loaded")


def run_zig_native_insert(iterations):
    if zig_native_lib:
        # ネイティブ関数を直接呼ぶ（WASMのようなオーバーヘッドなし）
        res = zig_native_lib.run_zig_zipper(iterations)
        if res != 0:
            raise RuntimeError("Zig execution failed")


# ==========================================
# Benchmark
# ==========================================
def main():
    # 挿入はO(N^2)になりがちなので回数を減らす
    ITERATIONS = 30_000
    REPEAT = 5

    print(f"--- Random Insertion Benchmark (N={ITERATIONS:,}, Repeat={REPEAT}) ---")
    print("Note: Simulating O(N) random insertions.")

    results = []

    def benchmark(name, func):
        try:
            total_time = timeit.timeit(lambda: func(ITERATIONS), number=REPEAT)
            avg_ms = (total_time / REPEAT) * 1000.0
            return {"name": name, "time_ms": avg_ms}
        except Exception as e:
            print(f"⚠️ {name}: {e}")
            return None

    # 1. Python List (Baseline)
    # 実はPythonのlist.insertはmemmoveを使うので意外と速い
    res_py = benchmark("Python (list)", run_python_list_insert)
    results.append(res_py)

    # 2. Python deque
    results.append(benchmark("Python (deque)", run_python_deque_insert))

    # 3. Rust Safe (Rc/RefCell)
    results.append(benchmark("Rust (Safe)", run_rust_safe_insert))

    # 4. Rust Unsafe (Raw Pointer)
    results.append(benchmark("Rust (Unsafe)", run_rust_unsafe_insert))

    # 5. Rust Bump (Bump Allocator)
    results.append(benchmark("Rust (Unsafe-Bump)", run_rust_bump_insert))

    # 6. Rust Zipper (Vec Stack)
    results.append(benchmark("Rust (Safe-Zipper)", run_rust_zipper_insert))

    # 7. Rust Unsafe Zipper (Vec Stack with Unsafe)
    results.append(benchmark("Rust (Unsafe-Zipper)", run_rust_unsafe_zipper_insert))

    # 8. Zig Zipper (ArrayList Stack)
    if zig_zipper_binary:
        results.append(benchmark("Zig (Zipper-WASM)", run_zig_zipper_insert))

    # 9. Zig (Native DLL)
    if zig_native_lib:
        results.append(benchmark("Zig (Zipper-Native)", run_zig_native_insert))

    # Sort and Display
    valid_results = [r for r in results if r is not None]
    valid_results.sort(key=lambda x: x["time_ms"])

    print("-" * 60)
    print(f"{'Implementation':<20} | {'Time (avg)':<10} | {'vs Py List':<10}")
    print("-" * 60)

    base_time = res_py["time_ms"]
    for r in valid_results:
        speedup = base_time / r["time_ms"]
        print(f"{r['name']:<20} | {r['time_ms']:>8.2f} ms | {speedup:>8.2f}x")
    print("-" * 60)


if __name__ == "__main__":
    main()
