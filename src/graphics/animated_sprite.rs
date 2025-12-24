use crate::core::assets::ImageId;
use crate::graphics::animation::Animation;
use crate::math::color::Color;
use crate::math::vec2::Vec2;
use crate::render::{Drawable, RenderContext, SpriteDrawData, Transform2d};
use std::time::Duration;

/// An animated sprite that cycles through animation frames.
#[derive(Clone)]
pub struct AnimatedSprite {
    pub animation: Animation,
    pub size: Vec2,
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    pub origin: Vec2,
    pub tint: Color,

    current_frame: usize,
    elapsed: Duration,
    pub playing: bool,
}

impl AnimatedSprite {
    /// Create a new animated sprite from an animation.
    pub fn new(animation: Animation, width: u32, height: u32) -> Self {
        Self {
            animation,
            size: Vec2::new(width as f32, height as f32),
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
            origin: Vec2::new(0.5, 0.5),
            tint: Color::WHITE,
            current_frame: 0,
            elapsed: Duration::ZERO,
            playing: true,
        }
    }

    /// Update the animation state based on delta time.
    pub fn update(&mut self, dt: Duration) {
        if !self.playing || self.animation.frames.is_empty() {
            return;
        }

        self.elapsed += dt;

        let current_frame_duration = self.animation.frames[self.current_frame].duration;

        while self.elapsed >= current_frame_duration {
            self.elapsed -= current_frame_duration;
            self.current_frame += 1;

            if self.current_frame >= self.animation.frames.len() {
                if self.animation.looped {
                    self.current_frame = 0;
                } else {
                    self.current_frame = self.animation.frames.len() - 1;
                    self.playing = false;
                    break;
                }
            }
        }
    }

    /// Get the current frame index.
    pub fn current_frame(&self) -> usize {
        self.current_frame
    }

    /// Reset the animation to the first frame.
    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.elapsed = Duration::ZERO;
        self.playing = true;
    }

    /// Convert to sprite draw data for rendering.
    pub fn to_draw_data(&self) -> SpriteDrawData {
        let image_id = if self.animation.frames.is_empty() {
            ImageId(0)
        } else {
            self.animation.frames[self.current_frame].image_id
        };

        SpriteDrawData {
            image_id,
            size: self.size,
            position: self.position,
            rotation: self.rotation,
            scale: self.scale,
            origin: self.origin,
            tint: self.tint,
        }
    }
}

impl Transform2d for AnimatedSprite {
    fn position(&self) -> Vec2 {
        self.position
    }

    fn position_mut(&mut self) -> &mut Vec2 {
        &mut self.position
    }

    fn rotation(&self) -> f32 {
        self.rotation
    }

    fn rotation_mut(&mut self) -> &mut f32 {
        &mut self.rotation
    }

    fn scale(&self) -> Vec2 {
        self.scale
    }

    fn scale_mut(&mut self) -> &mut Vec2 {
        &mut self.scale
    }

    fn origin(&self) -> Vec2 {
        self.origin
    }

    fn origin_mut(&mut self) -> &mut Vec2 {
        &mut self.origin
    }
}

impl Drawable for AnimatedSprite {
    fn draw(&self, ctx: &mut RenderContext) {
        ctx.draw_sprite(self.to_draw_data());
    }
}

impl From<&AnimatedSprite> for SpriteDrawData {
    fn from(sprite: &AnimatedSprite) -> Self {
        sprite.to_draw_data()
    }
}

impl From<AnimatedSprite> for SpriteDrawData {
    fn from(sprite: AnimatedSprite) -> Self {
        sprite.to_draw_data()
    }
}
