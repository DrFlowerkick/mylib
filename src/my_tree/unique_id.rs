// global unique id for tree nodes
use std::sync::atomic::{AtomicUsize, Ordering};

static GLOBAL_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn generate_unique_id() -> usize {
    GLOBAL_COUNTER.fetch_add(1, Ordering::SeqCst)
}
