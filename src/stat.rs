use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Stat {
    data: AtomicUsize,
}

impl Stat {
    pub fn new() -> Self {
        Self {
            data: AtomicUsize::new(0),
        }
    }

    pub fn add(&self, n: usize) -> usize {
        self.data.fetch_add(n, Ordering::Relaxed)
    }

    pub fn sub(&self, n: usize) -> usize {
        self.data.fetch_sub(n, Ordering::Relaxed)
    }

    pub fn count(&self) -> usize {
        self.data.load(Ordering::Relaxed)
    }
}
