use std::collections::HashMap;

use crate::{core::assets::ImageId, math::vec2::Vec2};

use super::id::AssetId;

/// Marker type for font assets.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct FontMarker;

/// Unique identifier for a font asset.
pub type FontId = AssetId<FontMarker>;

#[derive(Debug, Clone, Copy)]
pub struct Glyph {
    /// UV min dans l’atlas (0–1)
    pub uv_min: Vec2,

    /// UV max dans l’atlas (0–1)
    pub uv_max: Vec2,

    /// Taille du glyph en pixels
    pub size: Vec2,

    /// Décalage depuis le curseur (baseline)
    pub bearing: Vec2,

    /// Avance du curseur après ce caractère
    pub advance: f32,
}

/// Representation of a font asset.
#[derive(Debug, Clone)]
pub struct FontAsset {
    pub data: Vec<u8>,
    /// Texture contenant tous les glyphes (font atlas)
    pub atlas: ImageId,

    /// Infos par caractère
    pub glyphs: HashMap<char, Glyph>,
    /// Taille de la font en pixels
    pub font_size: f32,
    /// Hauteur d’une ligne (baseline → baseline)
    pub line_height: f32,
}
