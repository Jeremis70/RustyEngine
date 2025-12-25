use crate::core::assets::id::AssetId;

/// Marker type for sound assets.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct SoundMarker;

/// Unique identifier for a loaded sound asset.
pub type SoundId = AssetId<SoundMarker>;
