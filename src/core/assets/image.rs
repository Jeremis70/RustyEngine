use std::sync::atomic::{AtomicUsize, Ordering};

/// Unique identifier for an image asset.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ImageId(usize);

impl ImageId {
    pub(crate) fn new() -> Self {
        static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        ImageId(id)
    }
}

/// CPU-side representation of an image (RGBA8).
#[derive(Debug, Clone)]
pub struct ImageAsset {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}
