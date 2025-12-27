use std::path::{Path, PathBuf};

use super::cache::{AssetStore, FontKey, ImageKey, compute_asset_path_info};
use super::error::{AssetError, AssetResult};
use super::font::{FontAsset, FontId};
use super::image::{ImageAsset, ImageId};
use super::sound_tracking::{SoundAsset, SoundKey};
use crate::audio::SoundId;

mod bulk;
mod fonts;
mod images;
mod metrics;
mod sounds;
mod spritesheet;

/// Simple asset manager capable of loading and caching images, fonts, and sounds.
/// Tracks memory usage and supports unloading.
pub struct AssetManager {
    pub(crate) images: AssetStore<ImageId, ImageKey, ImageAsset>,
    pub(crate) fonts: AssetStore<FontId, FontKey, FontAsset>,
    pub(crate) sounds: AssetStore<SoundId, SoundKey, SoundAsset>,
    pub(crate) asset_root: PathBuf,
    pub(crate) path_policy: AssetPathPolicy,
    pub(crate) max_memory_bytes: usize,
    pub(crate) current_memory_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetPathPolicy {
    /// Allow paths outside the asset root, but emit a warning.
    AllowAndWarn,
    /// Reject paths outside the asset root.
    Deny,
    /// Allow paths outside the asset root without warnings.
    AllowSilent,
}

impl AssetManager {
    pub(crate) fn enforce_path_policy(
        &self,
        input: &Path,
        info: &super::cache::AssetPathInfo,
    ) -> AssetResult<()> {
        if info.is_portable {
            return Ok(());
        }

        match self.path_policy {
            AssetPathPolicy::AllowSilent => Ok(()),
            AssetPathPolicy::AllowAndWarn => {
                let reason = info.reason.unwrap_or("path is not portable");
                log::warn!(
                    "Asset path {:?} is not portable relative to asset_root {:?} ({reason}); loading is allowed but may be unsafe/non-portable",
                    input,
                    self.asset_root
                );
                Ok(())
            }
            AssetPathPolicy::Deny => {
                let reason = info.reason.unwrap_or("path is not portable");
                Err(AssetError::InvalidAssetPath {
                    input: input.to_path_buf(),
                    asset_root: self.asset_root.clone(),
                    reason: reason.to_string(),
                })
            }
        }
    }

    pub fn set_asset_path_policy(&mut self, policy: AssetPathPolicy) {
        self.path_policy = policy;
    }

    pub fn asset_path_policy(&self) -> AssetPathPolicy {
        self.path_policy
    }

    pub(crate) fn ensure_capacity_for(&self, additional_bytes: usize) -> AssetResult<()> {
        let new_total = self
            .current_memory_bytes
            .checked_add(additional_bytes)
            .ok_or(AssetError::MemoryExceeded {
                current: self.current_memory_bytes,
                limit: self.max_memory_bytes,
            })?;

        if new_total > self.max_memory_bytes {
            return Err(AssetError::MemoryExceeded {
                current: self.current_memory_bytes,
                limit: self.max_memory_bytes,
            });
        }

        Ok(())
    }

    /// Create a new asset manager with unlimited memory.
    pub fn new() -> Self {
        Self::with_limit(usize::MAX)
    }

    /// Create a new asset manager with a memory limit.
    ///
    /// # Arguments
    /// * `max_bytes` - Maximum memory in bytes (e.g., 512 * 1024 * 1024 for 512MB)
    pub fn with_limit(max_bytes: usize) -> Self {
        let asset_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::with_limit_and_root(max_bytes, asset_root)
    }

    /// Create a new asset manager with a memory limit and a custom asset root.
    ///
    /// The `asset_root` is used to compute stable cache keys (virtual paths) so that
    /// different representations of the same file (absolute/relative, separators, dot segments)
    /// deduplicate correctly.
    pub fn with_limit_and_root<P: Into<PathBuf>>(max_bytes: usize, asset_root: P) -> Self {
        let asset_root = asset_root.into();
        let asset_root = std::fs::canonicalize(&asset_root).unwrap_or(asset_root);
        Self {
            images: AssetStore::new(),
            fonts: AssetStore::new(),
            sounds: AssetStore::new(),
            asset_root,
            path_policy: AssetPathPolicy::AllowAndWarn,
            max_memory_bytes: max_bytes,
            current_memory_bytes: 0,
        }
    }

    pub(crate) fn compute_path_info(&self, input: &Path) -> super::cache::AssetPathInfo {
        compute_asset_path_info(&self.asset_root, input)
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}
