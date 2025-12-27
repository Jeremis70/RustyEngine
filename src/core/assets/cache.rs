use std::collections::HashMap;
use std::hash::Hash;
use std::path::{Path, PathBuf};

/// Entry stored in an `AssetStore`.
///
/// `key` is present for assets loaded via a lookup key (e.g. from disk), and
/// absent for assets created/generated in-memory (e.g. sprites from a sheet).
#[derive(Debug, Clone)]
pub(crate) struct CacheEntry<Asset, Key> {
    pub(crate) asset: Asset,
    pub(crate) key: Option<Key>,
}

/// Common pattern used by all asset types:
/// - `by_id`: authoritative storage (Id -> Entry)
/// - `id_by_key`: dedup lookup (Key -> Id)
///
/// This keeps behavior consistent across images/fonts/sounds while staying at 2 HashMaps.
#[derive(Debug, Default)]
pub(crate) struct AssetStore<Id, Key, Asset>
where
    Id: Eq + Hash,
    Key: Eq + Hash,
{
    pub(crate) by_id: HashMap<Id, CacheEntry<Asset, Key>>,
    pub(crate) id_by_key: HashMap<Key, Id>,
}

impl<Id, Key, Asset> AssetStore<Id, Key, Asset>
where
    Id: Copy + Eq + Hash,
    Key: Clone + Eq + Hash,
{
    pub(crate) fn new() -> Self {
        Self {
            by_id: HashMap::new(),
            id_by_key: HashMap::new(),
        }
    }

    pub(crate) fn get_existing_id(&self, key: &Key) -> Option<Id> {
        self.id_by_key.get(key).copied()
    }

    pub(crate) fn insert_keyed(&mut self, id: Id, key: Key, asset: Asset) {
        self.by_id.insert(
            id,
            CacheEntry {
                asset,
                key: Some(key.clone()),
            },
        );
        self.id_by_key.insert(key, id);
    }

    pub(crate) fn insert_unkeyed(&mut self, id: Id, asset: Asset) {
        self.by_id.insert(id, CacheEntry { asset, key: None });
    }

    pub(crate) fn remove(&mut self, id: Id) -> Option<CacheEntry<Asset, Key>> {
        let entry = self.by_id.remove(&id)?;
        if let Some(key) = entry.key.as_ref() {
            self.id_by_key.remove(key);
        }
        Some(entry)
    }

    pub(crate) fn clear(&mut self) {
        self.by_id.clear();
        self.id_by_key.clear();
    }

    pub(crate) fn len(&self) -> usize {
        self.by_id.len()
    }

    pub(crate) fn contains_id(&self, id: Id) -> bool {
        self.by_id.contains_key(&id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ImageKey {
    pub(crate) path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct FontKey {
    pub(crate) path: PathBuf,
    pub(crate) size_bits: u32,
}

impl FontKey {
    pub(crate) fn new(path: PathBuf, font_size: f32) -> Self {
        Self {
            path,
            size_bits: font_size.to_bits(),
        }
    }
}

pub(crate) fn normalize_path(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}
