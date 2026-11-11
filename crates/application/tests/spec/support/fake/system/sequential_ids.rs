use std::sync::atomic::{AtomicUsize, Ordering};

use dms_application::port::system::IdGenerator;

/// Ids `id-1`, `id-2`, … so tests can predict them.
pub struct SequentialIds {
    next: AtomicUsize,
}

impl Default for SequentialIds {
    fn default() -> Self {
        Self { next: AtomicUsize::new(1) }
    }
}

impl IdGenerator for SequentialIds {
    fn new_id(&self) -> String {
        format!("id-{}", self.next.fetch_add(1, Ordering::SeqCst))
    }
}
