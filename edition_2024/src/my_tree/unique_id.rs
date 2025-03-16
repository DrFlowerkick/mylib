// global unique id for tree nodes
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering as AtomicOrdering;

static GLOBAL_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn generate_unique_id() -> usize {
    GLOBAL_COUNTER.fetch_add(1, AtomicOrdering::SeqCst)
}
