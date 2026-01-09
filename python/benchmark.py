import timeit
import collections
import sys

# Rustモジュールのインポート確認
try:
    import polyglot_compute_lab
except ImportError:
    print("❌ Error: 'polyglot_compute_lab' module not found.")
    print("💡 Hint: Did you run `maturin develop --release --features python`?")
    sys.exit(1)

# WASMファイルのパス (ビルド済みのものを参照)
WASM_ZIG_PATH = "www/zig_dll.wasm"
WASM_WAT_PATH = "www/wat_dll.wasm"

# WASMファイルをメモリにロードしておく
zig_binary = None
wat_binary = None

try:
    with open(WASM_ZIG_PATH, "rb") as f:
        zig_binary = f.read()
    with open(WASM_WAT_PATH, "rb") as f:
        wat_binary = f.read()
except FileNotFoundError:
    print(f"⚠️ Warning: WASM files not found in 'www/'. skipping WASM benchmarks.")


# ==========================================
# 1. Pure Python Implementation (Class)
# ==========================================
class Node:
    def __init__(self, value):
        self.value = value
        self.prev = None
        self.next = None


class DoublyLinkedList:
    def __init__(self):
        self.head = None
        self.tail = None

    def append(self, value):
        new_node = Node(value)
        if not self.head:
            self.head = new_node
            self.tail = new_node
        else:
            new_node.prev = self.tail
            self.tail.next = new_node
            self.tail = new_node

    def sum(self):
        s = 0
        curr = self.head
        while curr:
            s += curr.value
            curr = curr.next
        return s


def run_python_class(iterations):
    dll = DoublyLinkedList()
    for i in range(iterations):
        dll.append(i)
    return dll.sum()


# ==========================================
# 2. Standard Library (collections.deque)
# ==========================================
def run_python_deque(iterations):
    # dequeはC言語で最適化された双方向リスト実装
    d = collections.deque()
    for i in range(iterations):
        d.append(i)
    return sum(d)


# ==========================================
# 3. Rust Implementations
# ==========================================
def run_rust_safe(iterations):
    return polyglot_compute_lab.run_rust_dll_py(iterations)


def run_rust_unsafe(iterations):
    return polyglot_compute_lab.run_rust_unsafe_py(iterations)


def run_rust_bump(iterations):
    return polyglot_compute_lab.run_rust_bump_py(iterations)


def run_rust_wgpu(iterations):
    return polyglot_compute_lab.run_wgpu_py(iterations)


# ==========================================
# 4. WASM Implementations (Running via Rust Wasmtime)
# ==========================================
def run_zig_wasm(iterations):
    if not zig_binary:
        return 0
    # "run_zig_dll" はZig側でexportした関数名
    return polyglot_compute_lab.run_wasm_py(zig_binary, "run_zig_dll", iterations)


def run_wat_wasm(iterations):
    if not wat_binary:
        return 0
    # "run_wat_dll" はWAT側でexportした関数名
    return polyglot_compute_lab.run_wasm_py(wat_binary, "run_wat_dll", iterations)


# ==========================================
# Benchmarking Engine
# ==========================================
def main():
    ITERATIONS = 100_000
    REPEAT = 10

    print(f"--- Polyglot Benchmark (N={ITERATIONS:,}, Repeat={REPEAT}) ---")
    print("Running benchmarks...\n")

    results = []

    # Helper function to measure and record
    def benchmark(name, func):
        try:
            # 実行時間を計測 (秒)
            total_time = timeit.timeit(lambda: func(ITERATIONS), number=REPEAT)
            avg_ms = (total_time / REPEAT) * 1000.0
            return {
                "name": name,
                "time_ms": avg_ms,
                "speedup": 0.0,  # 後で計算
            }
        except Exception as e:
            print(f"⚠️ Failed to run {name}: {e}")
            return None

    # 1. Python (Pure Class)
    res_py = benchmark("Python (Pure Class)", run_python_class)
    results.append(res_py)

    # 基準タイム
    baseline_time = res_py["time_ms"]

    # 2. Python (deque)
    results.append(benchmark("Python (deque)", run_python_deque))

    # 3. Rust (Safe)
    results.append(benchmark("Rust (Safe)", run_rust_safe))

    # 4. Rust (Unsafe)
    results.append(benchmark("Rust (Unsafe)", run_rust_unsafe))

    # 5. Rust (Bump)
    results.append(benchmark("Rust (Bump)", run_rust_bump))

    # 6. WGPU (WebGPU)
    results.append(benchmark("WGPU (WebGPU)", run_rust_wgpu))

    # 7. Zig (WASM via Rust)
    if zig_binary:
        results.append(benchmark("Zig (WASM)", run_zig_wasm))

    # 8. WAT (WASM via Rust)
    if wat_binary:
        results.append(benchmark("WAT (WASM)", run_wat_wasm))

    # Calculate Speedup & Sort
    valid_results = [r for r in results if r is not None]
    for r in valid_results:
        r["speedup"] = baseline_time / r["time_ms"]

    # 降順ソート (倍率が高い順)
    valid_results.sort(key=lambda x: x["speedup"], reverse=True)

    # ==========================================
    # Report Output
    # ==========================================
    print(
        f"{'Implementation':<25} | {'Time (avg)':<12} | {'Speedup (vs Py Class)':<22}"
    )
    print("-" * 65)

    for r in valid_results:
        name = r["name"]
        time_str = f"{r['time_ms']:.2f} ms"
        speedup_str = f"{r['speedup']:.2f}x"

        # アイコン分け
        if "Bump" in name:
            prefix = "🚀 "
        elif "Unsafe" in name:
            prefix = "⚡ "
        elif "Safe" in name:
            prefix = "🛡️ "
        elif "deque" in name:
            prefix = "🐍 "
        elif "WGPU" in name:
            prefix = "🎮 "
        elif "WASM" in name:
            prefix = "📦 "
        else:
            prefix = "🐌 "

        print(f"{prefix}{name:<23} | {time_str:>10} | {speedup_str:>18}")

    print("-" * 65)


if __name__ == "__main__":
    main()
