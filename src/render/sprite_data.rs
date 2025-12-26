use crate::core::assets::ImageId;
use crate::math::color::Color;
use crate::math::vec2::Vec2;

/// Generic sprite drawing data - decoupled from the Sprite type itself.
/// This allows any system (Sprite, AnimatedSprite, custom renderers, etc.)
/// to provide draw data without coupling.
#[derive(Clone, Debug)]
pub struct SpriteDrawData {
    pub image_id: ImageId,
    pub size: Vec2,
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    pub origin: Vec2,
    pub tint: Color,

    // UV coordinates for atlas support
    pub uv_min: Vec2,
    pub uv_max: Vec2,
}

impl SpriteDrawData {
    /// Create sprite draw data with common defaults.
    pub fn new(image_id: ImageId, width: u32, height: u32) -> Self {
        Self {
            image_id,
            size: Vec2::new(width as f32, height as f32),
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
            origin: Vec2::new(0.5, 0.5),
            tint: Color::WHITE,
            uv_min: Vec2::new(0.0, 0.0),
            uv_max: Vec2::new(1.0, 1.0),
        }
    }
}
