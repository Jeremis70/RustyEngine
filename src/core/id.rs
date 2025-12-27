use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU32, Ordering};

static GLOBAL_ID_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Strongly-typed ID backed by an integer.
///
/// This is used across the engine for assets (ImageId, FontId, SoundId), input actions,
/// and any other handle-like identifiers.
///
/// Trait impls intentionally do *not* require any bounds on `T`.
pub struct Id<T> {
    raw: u32,
    _phantom: PhantomData<fn() -> T>,
}

impl<T> Copy for Id<T> {}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Id").field("raw", &self.raw).finish()
    }
}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<T> Eq for Id<T> {}

impl<T> Hash for Id<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.raw.cmp(&other.raw)
    }
}

impl<T> Id<T> {
    /// Create a new globally-unique id.
    ///
    /// Note: intended for engine internals (assets). Prefer scoped allocators for
    /// systems that want per-instance ID spaces (e.g. ActionMap).
    pub(crate) fn new() -> Self {
        let raw = GLOBAL_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self {
            raw,
            _phantom: PhantomData,
        }
    }

    pub fn as_u32(&self) -> u32 {
        self.raw
    }

    pub fn as_usize(&self) -> usize {
        self.raw as usize
    }
}

impl<T> From<Id<T>> for u32 {
    fn from(id: Id<T>) -> u32 {
        id.raw
    }
}

impl<T> From<Id<T>> for usize {
    fn from(id: Id<T>) -> usize {
        id.raw as usize
    }
}

impl<T> fmt::Display for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

/// Thread-safe scoped ID allocator.
///
/// Use this when you need IDs that are unique *within a system instance* (e.g. one ActionMap),
/// while still supporting multi-threaded construction/configuration.
#[derive(Debug)]
pub struct IdAllocator {
    next: AtomicU32,
}

impl IdAllocator {
    pub fn new() -> Self {
        Self {
            next: AtomicU32::new(0),
        }
    }

    pub fn alloc<T>(&self) -> Id<T> {
        let raw = self.next.fetch_add(1, Ordering::Relaxed);
        Id {
            raw,
            _phantom: PhantomData,
        }
    }
}

impl Default for IdAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for IdAllocator {
    fn clone(&self) -> Self {
        Self {
            next: AtomicU32::new(self.next.load(Ordering::Relaxed)),
        }
    }
}
