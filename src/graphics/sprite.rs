use crate::core::assets::ImageAsset;
use crate::core::assets::ImageId;
use crate::math::Transform;
use crate::math::color::Color;
use crate::math::vec2::Vec2;
use crate::render::{Drawable, RenderContext, SpriteDrawData, Transform2d};

/// Simple 2D sprite similar to pygame's Sprite.
/// Holds a reference id to a texture and basic transform properties.
#[derive(Clone, Debug)]
pub struct Sprite {
    pub transform: Transform,
    pub image_id: ImageId,
    pub size: Vec2,
    pub tint: Color,
}

impl Sprite {
    /// Create a sprite from an already loaded image asset.
    pub fn from_image(id: ImageId, image: &ImageAsset) -> Self {
        let size = Vec2::new(image.width as f32, image.height as f32);
        Self {
            transform: Transform::new(),
            image_id: id,
            size,
            tint: Color::WHITE,
        }
    }

    /// Convenience constructor from explicit size.
    pub fn new(id: ImageId, width: u32, height: u32) -> Self {
        let image = ImageAsset {
            width,
            height,
            data: Vec::new(),
        };
        Self::from_image(id, &image)
    }

    /// Compute world-space corners of the sprite quad in pixel coordinates.
    /// Order: top-left, top-right, bottom-right, bottom-left.
    pub fn world_corners(&self) -> [Vec2; 4] {
        let size = self.size;
        let tl = self.transform.transform_point(Vec2::new(0.0, 0.0), size);
        let tr = self.transform.transform_point(Vec2::new(size.x, 0.0), size);
        let br = self
            .transform
            .transform_point(Vec2::new(size.x, size.y), size);
        let bl = self.transform.transform_point(Vec2::new(0.0, size.y), size);
        [tl, tr, br, bl]
    }

    /// Convert this sprite to draw data for rendering.
    pub fn to_draw_data(&self) -> SpriteDrawData {
        SpriteDrawData {
            image_id: self.image_id,
            size: self.size,
            position: self.transform.position,
            rotation: self.transform.rotation,
            scale: self.transform.scale,
            origin: self.transform.origin,
            tint: self.tint,
            uv_min: Vec2::new(0.0, 0.0),
            uv_max: Vec2::new(1.0, 1.0),
        }
    }
}

impl Transform2d for Sprite {
    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}

impl Drawable for Sprite {
    fn draw(&self, ctx: &mut RenderContext) {
        ctx.draw_sprite(self.to_draw_data());
    }
}

impl From<&Sprite> for SpriteDrawData {
    fn from(sprite: &Sprite) -> Self {
        sprite.to_draw_data()
    }
}

impl From<Sprite> for SpriteDrawData {
    fn from(sprite: Sprite) -> Self {
        SpriteDrawData {
            image_id: sprite.image_id,
            size: sprite.size,
            position: sprite.transform.position,
            rotation: sprite.transform.rotation,
            scale: sprite.transform.scale,
            origin: sprite.transform.origin,
            tint: sprite.tint,
            uv_min: Vec2::new(0.0, 0.0),
            uv_max: Vec2::new(1.0, 1.0),
        }
    }
}
