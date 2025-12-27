use std::path::PathBuf;

use crate::audio::LoadStrategy;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct SoundKey {
    pub(crate) path: PathBuf,
    pub(crate) strategy: LoadStrategy,
}

#[derive(Debug, Clone)]
pub(crate) struct SoundAsset {
    /// Best-effort memory estimate (currently uses file size on disk).
    pub(crate) estimated_bytes: usize,
}
