use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

static TMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn make_temporary() -> String {
    let i = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("tmp.{}", i)
}

pub fn make_label(prefix: &str) -> String {
    let i = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}.{}", prefix, i)
}
