use std::sync::atomic::{AtomicU64, Ordering};

lazy_static! {
    static ref N: AtomicU64 = AtomicU64::new(0);
}

pub fn uniqid() -> u64 {
    N.fetch_add(1, Ordering::Relaxed)
}
