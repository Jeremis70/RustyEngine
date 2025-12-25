use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Generic asset identifier with type safety via phantom types.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct AssetId<T> {
    id: usize,
    _phantom: PhantomData<T>,
}

impl<T> AssetId<T> {
    pub(crate) fn new() -> Self {
        static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        AssetId {
            id,
            _phantom: PhantomData,
        }
    }

    /// Get the raw ID value.
    pub fn as_usize(&self) -> usize {
        self.id
    }
}

impl<T> From<AssetId<T>> for usize {
    fn from(id: AssetId<T>) -> usize {
        id.id
    }
}

impl<T> std::fmt::Display for AssetId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}
