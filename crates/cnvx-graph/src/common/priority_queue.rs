use std::cmp::Ordering;
use std::collections::BinaryHeap;

struct Entry<T> {
    priority: f64,
    item: T,
}

impl<T> PartialEq for Entry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}
impl<T> Eq for Entry<T> {}
impl<T> PartialOrd for Entry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> Ord for Entry<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reversed, so `BinaryHeap` (a max-heap) pops the smallest
        // priority first.
        other
            .priority
            .partial_cmp(&self.priority)
            .expect("priority queue: NaN priority from weight function")
    }
}

/// A min-priority queue: `pop()` returns the item with the smallest
/// `f64` priority pushed so far.
pub(crate) struct PriorityQueue<T> {
    heap: BinaryHeap<Entry<T>>,
}

#[allow(dead_code)]
impl<T> PriorityQueue<T> {
    pub(crate) fn new() -> Self {
        Self { heap: BinaryHeap::new() }
    }

    pub(crate) fn push(&mut self, item: T, priority: f64) {
        self.heap.push(Entry { priority, item });
    }

    pub(crate) fn pop(&mut self) -> Option<(T, f64)> {
        self.heap.pop().map(|e| (e.item, e.priority))
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
}
