use std::collections::HashMap;

use crate::math::Vec2;

use super::image::ImageId;

use super::id::AssetId;

/// Marker type for font assets.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct FontMarker;

/// Unique identifier for a font asset.
pub type FontId = AssetId<FontMarker>;

/// Which characters should be rasterized into the font atlas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FontCharset {
    /// ASCII printable characters (U+0020..=U+007E).
    Ascii,
    /// Latin-1 (ISO-8859-1) printable-ish characters (U+0020..=U+00FF).
    Latin1,
    /// Explicit list of characters to rasterize.
    ///
    /// Tip: include at least ' ' and '?' for spacing + fallback.
    Custom(Vec<char>),
}

#[derive(Debug, Clone, Copy)]
pub struct Glyph {
    /// UV min in the atlas (0–1)
    pub uv_min: Vec2,

    /// UV max in the atlas (0–1)
    pub uv_max: Vec2,

    /// Glyph size in pixels
    pub size: Vec2,

    /// Offset from the pen position (baseline)
    pub bearing: Vec2,

    /// Pen advance after this character
    pub advance: f32,
}

/// Representation of a font asset.
#[derive(Debug, Clone)]
pub struct FontAsset {
    pub data: Vec<u8>,
    /// Texture containing all glyphs (font atlas)
    pub atlas: ImageId,

    /// Per-character glyph information
    pub glyphs: HashMap<char, Glyph>,
    /// Font size in pixels
    pub font_size: f32,
    /// Line height (baseline → baseline)
    pub line_height: f32,
}
