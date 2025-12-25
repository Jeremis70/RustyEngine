use super::id::AssetId;

/// Marker type for image assets.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ImageMarker;

/// Unique identifier for an image asset.
pub type ImageId = AssetId<ImageMarker>;

/// CPU-side representation of an image (RGBA8).
#[derive(Debug, Clone)]
pub struct ImageAsset {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}
