use std::fmt;
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

pub struct PrettyVec<'a, T>(pub &'a [T]);

impl<'a, T: fmt::Debug> fmt::Debug for PrettyVec<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[")?;
        for item in self.0 {
            writeln!(f, "    {:?},", item)?; // 4-space indent
        }
        write!(f, "]")
    }
}
