use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

lazy_static! {
    static ref N: AtomicU64 = AtomicU64::new(1);
}

pub fn uniqid() -> u64 {
    N.fetch_add(1, Ordering::Relaxed)
}

pub fn unix_time() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap_or_default();
    since_the_epoch.as_secs()
}

pub fn recent(hours: u64) -> SystemTime {
    SystemTime::now() - Duration::from_secs(3600 * hours)
}
