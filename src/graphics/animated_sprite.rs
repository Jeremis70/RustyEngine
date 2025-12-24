use crate::core::assets::ImageId;
use crate::core::events::callbacks::Callbacks;
use crate::graphics::animation::Animation;
use crate::math::color::Color;
use crate::math::vec2::Vec2;
use crate::render::{Drawable, RenderContext, SpriteDrawData, Transform2d};
use std::collections::VecDeque;
use std::time::Duration;

/// Playback state of an animated sprite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Finished,
}

/// An animated sprite that cycles through animation frames with events and transitions.
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
    playback_state: PlaybackState,

    /// Fallback image when animation is empty (optional safety).
    fallback_image: Option<ImageId>,

    /// Queue of animations to play after current finishes.
    animation_queue: VecDeque<Animation>,

    /// Called when current animation completes (non-looping only).
    pub on_animation_finished: Callbacks<()>,
    /// Called when the sprite is truly finished (queue empty + animation done).
    pub on_sprite_finished: Callbacks<()>,
    /// Called when animation loops back to frame 0.
    pub on_loop: Callbacks<()>,
    /// Called every time the frame changes (passes new frame index).
    pub on_frame_changed: Callbacks<usize>,
}

impl AnimatedSprite {
    /// Create a new animated sprite from an animation.
    /// Panics if animation has no frames (use `with_fallback` for safety).
    pub fn new(animation: Animation, width: u32, height: u32) -> Self {
        assert!(
            !animation.frames.is_empty(),
            "Animation must have at least one frame"
        );

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
            playback_state: PlaybackState::Playing,
            fallback_image: None,
            animation_queue: VecDeque::new(),
            on_animation_finished: Callbacks::new(),
            on_sprite_finished: Callbacks::new(),
            on_loop: Callbacks::new(),
            on_frame_changed: Callbacks::new(),
        }
    }

    /// Create a sprite with a fallback image for empty animations.
    pub fn with_fallback(animation: Animation, width: u32, height: u32, fallback: ImageId) -> Self {
        let mut sprite = Self {
            animation,
            size: Vec2::new(width as f32, height as f32),
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
            origin: Vec2::new(0.5, 0.5),
            tint: Color::WHITE,
            current_frame: 0,
            elapsed: Duration::ZERO,
            playback_state: PlaybackState::Playing,
            fallback_image: Some(fallback),
            animation_queue: VecDeque::new(),
            on_animation_finished: Callbacks::new(),
            on_sprite_finished: Callbacks::new(),
            on_loop: Callbacks::new(),
            on_frame_changed: Callbacks::new(),
        };

        if sprite.animation.frames.is_empty() {
            sprite.playback_state = PlaybackState::Paused;
        }

        sprite
    }

    /// Play a temporary animation, then return to current animation.
    /// Example: play_once(yawn_animation) will play yawn then return to idle.
    /// Forces the animation to be non-looping to ensure it returns.
    pub fn play_once(&mut self, mut animation: Animation) {
        // Force non-looping to ensure the animation finishes
        animation.looped = false;

        // Queue current animation to return to it
        self.animation_queue.push_front(self.animation.clone());

        // Start the temporary animation
        self.animation = animation;
        self.reset();
    }

    /// Queue an animation to play after the current one finishes.
    pub fn queue_animation(&mut self, animation: Animation) {
        self.animation_queue.push_back(animation);
    }

    /// Change to a new animation immediately, clearing the queue.
    pub fn set_animation(&mut self, animation: Animation) {
        if !animation.frames.is_empty() {
            self.animation = animation;
            self.animation_queue.clear();
            self.reset();
        }
    }

    /// Check if animation has truly finished (no more animations queued).
    pub fn is_finished(&self) -> bool {
        self.playback_state == PlaybackState::Finished && self.animation_queue.is_empty()
    }

    /// Get current playback state.
    pub fn state(&self) -> PlaybackState {
        self.playback_state
    }

    /// Check if currently playing.
    pub fn is_playing(&self) -> bool {
        self.playback_state == PlaybackState::Playing
    }

    /// Pause the animation.
    pub fn pause(&mut self) {
        if self.playback_state == PlaybackState::Playing {
            self.playback_state = PlaybackState::Paused;
        }
    }

    /// Resume the animation.
    pub fn resume(&mut self) {
        if self.playback_state == PlaybackState::Paused {
            self.playback_state = PlaybackState::Playing;
        }
    }

    /// Get a reference to the current animation.
    pub fn animation(&self) -> &Animation {
        &self.animation
    }

    /// Reset the animation to the first frame without auto-playing.
    /// Useful for preloading, rewinding, or timeline editing.
    pub fn reset_paused(&mut self) {
        self.current_frame = 0;
        self.elapsed = Duration::ZERO;
        self.playback_state = PlaybackState::Paused;
    }

    /// Update the animation state based on delta time.
    pub fn update(&mut self, dt: Duration) {
        if self.playback_state != PlaybackState::Playing || self.animation.frames.is_empty() {
            return;
        }

        self.elapsed += dt;

        let mut frame_changed = false;

        loop {
            // Recalculate frame duration each iteration (fix bug)
            let current_frame_duration = self.animation.frames[self.current_frame].duration;

            // Protection against zero-duration frames (infinite loop)
            debug_assert!(
                current_frame_duration > Duration::ZERO,
                "Animation frame duration must be > 0"
            );
            let current_frame_duration = current_frame_duration.max(Duration::from_millis(1));

            if self.elapsed < current_frame_duration {
                break;
            }

            self.elapsed -= current_frame_duration;
            self.current_frame += 1;
            frame_changed = true;

            if self.current_frame >= self.animation.frames.len() {
                if self.animation.looped {
                    self.current_frame = 0;
                    self.on_loop.invoke(&());
                } else {
                    self.current_frame = self.animation.frames.len() - 1;
                    self.playback_state = PlaybackState::Finished;

                    // Call animation finished callback
                    self.on_animation_finished.invoke(&());

                    // Auto-transition to next queued animation
                    if let Some(next) = self.animation_queue.pop_front() {
                        self.animation = next;
                        self.reset();
                    } else {
                        // No more animations - sprite is truly finished
                        self.on_sprite_finished.invoke(&());
                    }

                    break;
                }
            }
        }

        if frame_changed {
            self.on_frame_changed.invoke(&self.current_frame);
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
        self.playback_state = PlaybackState::Playing;
    }

    /// Convert to sprite draw data for rendering.
    pub fn to_draw_data(&self) -> SpriteDrawData {
        let image_id = if self.animation.frames.is_empty() {
            self.fallback_image
                .expect("AnimatedSprite has no frames and no fallback image")
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
