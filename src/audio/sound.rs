use std::fmt;
use std::sync::atomic::{AtomicU32, Ordering};

/// Unique identifier for a loaded sound asset.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct SoundId(u32);

impl SoundId {
    pub(crate) fn new() -> Self {
        static ID_COUNTER: AtomicU32 = AtomicU32::new(0);
        let id = ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        SoundId(id)
    }
}

impl fmt::Display for SoundId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
