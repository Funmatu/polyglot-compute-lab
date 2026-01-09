import timeit
import polyglot_compute_lab  # Rust module


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

    def print_list(self):
        current = self.head
        while current:
            print(f"{current.value} <-> ", end="")
            current = current.next
        print("None")


def run_python_dll(iterations):
    dll = DoublyLinkedList()
    for i in range(iterations):
        dll.append(i)
    return dll.sum()


ITERATIONS = 100_000

print(f"--- Benchmark (N={ITERATIONS}) ---")

# Python
py_time = timeit.timeit(lambda: run_python_dll(ITERATIONS), number=10)
print(f"Python (Pure): {py_time / 10 * 1000:.2f} ms")

# Rust
rs_time = timeit.timeit(
    lambda: polyglot_compute_lab.run_rust_dll_py(ITERATIONS), number=10
)
print(f"Rust (Native): {rs_time / 10 * 1000:.2f} ms")

print(f"Speedup: {py_time / rs_time:.2f}x")
