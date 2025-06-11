use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub usize);

static NEXT_ENTITY_ID: AtomicUsize = AtomicUsize::new(0);

impl Entity {
    pub fn new() -> Self {
        Entity(NEXT_ENTITY_ID.fetch_add(1, Ordering::Relaxed))
    }

    pub fn id(&self) -> usize {
        self.0
    }
}
