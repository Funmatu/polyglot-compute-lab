import timeit
import collections
import sys
import random

try:
    import polyglot_compute_lab
except ImportError:
    print("❌ Error: 'polyglot_compute_lab' not found.")
    sys.exit(1)


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
    results.append(benchmark("Rust (Safe DLL)", run_rust_safe_insert))

    # 4. Rust Unsafe (Raw Pointer)
    results.append(benchmark("Rust (Unsafe DLL)", run_rust_unsafe_insert))

    # 5. Rust Bump (Bump Allocator)
    results.append(benchmark("Rust (Bump DLL)", run_rust_bump_insert))

    # 6. Rust Zipper (Vec Stack)
    results.append(benchmark("Rust (Zipper)", run_rust_zipper_insert))

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
