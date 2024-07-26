use std::{
    sync::atomic::{AtomicU64, Ordering},
    sync::LazyLock,
    time::{SystemTime, UNIX_EPOCH},
};

static N: LazyLock<AtomicU64> = LazyLock::new(|| AtomicU64::new(1));

pub fn uniqid() -> u64 {
    N.fetch_add(1, Ordering::Relaxed)
}

pub fn unix_time() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap_or_default();
    since_the_epoch.as_secs()
}
