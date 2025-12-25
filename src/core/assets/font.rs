use super::id::AssetId;

/// Marker type for font assets.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct FontMarker;

/// Unique identifier for a font asset.
pub type FontId = AssetId<FontMarker>;

/// Representation of a font asset.
pub struct FontAsset {
    pub data: Vec<u8>, // Raw font data
}
