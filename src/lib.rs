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