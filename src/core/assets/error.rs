use std::path::PathBuf;
use thiserror::Error;

/// Result type for asset loading operations.
pub type AssetResult<T> = Result<T, AssetError>;

/// Error type for asset-related failures.
#[derive(Debug, Error)]
pub enum AssetError {
    #[error("I/O error while loading asset {path:?}: {source}")]
    Io {
        #[source]
        source: std::io::Error,
        path: PathBuf,
    },
    #[error("Image decode error while loading asset {path:?}: {source}")]
    Image {
        #[source]
        source: image::ImageError,
        path: PathBuf,
    },
    #[error("Asset memory limit exceeded: {current} / {limit} bytes")]
    MemoryExceeded { current: usize, limit: usize },

    #[error("Invalid Spritesheet format in asset {path:?}: {reason}")]
    InvalidSpritesheet { path: PathBuf, reason: String },

    #[error("Invalid font format")]
    InvalidFont,

    #[error("Out of memory")]
    OutOfMemory,
}
