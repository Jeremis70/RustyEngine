use crate::{
    core::assets::{font::FontId, manager::AssetManager},
    math::{Color, Transform, Vec2},
    render::{Drawable, RenderContext, SpriteDrawData, Transform2d},
};

use crate::core::assets::font::FontAsset;

pub struct Text {
    pub font: FontId,
    pub content: String,
    pub font_size: u32,
    pub color: Color,
    pub transform: Transform,

    /// Line height multiplier. 1.0 = default font line height.
    pub line_height: f32,

    /// Additional spacing between letters in pixels.
    pub letter_spacing: f32,

    /// Cached sprite data for rendering. Updated via layout().
    sprites: Vec<SpriteDrawData>,
}

impl Text {
    pub fn new(font: FontId, content: &str, font_size: u32, color: Color) -> Self {
        Self {
            font,
            content: content.to_string(),
            font_size,
            color,
            transform: Transform::new(),
            line_height: 1.0,
            letter_spacing: 0.0,
            sprites: Vec::new(),
        }
    }

    /// Create text with custom line height and letter spacing.
    pub fn with_spacing(
        font: FontId,
        content: &str,
        font_size: u32,
        color: Color,
        line_height: f32,
        letter_spacing: f32,
    ) -> Self {
        Self {
            font,
            content: content.to_string(),
            font_size,
            color,
            transform: Transform::new(),
            line_height,
            letter_spacing,
            sprites: Vec::new(),
        }
    }

    /// Layout the text by computing sprite data from the font atlas.
    /// Must be called after creating or modifying the text, and requires access to AssetManager.
    /// After calling this, draw() can be used without needing AssetManager.
    pub fn layout(&mut self, assets: &AssetManager) {
        let Some(font) = assets.get_font(self.font) else {
            self.sprites.clear();
            return;
        };

        self.layout_with_font_asset(font);
    }

    /// Layout using a previously retrieved `FontAsset`.
    /// Useful for dynamic text in render callbacks where `AssetManager` isn't available.
    pub fn layout_with_font_asset(&mut self, font: &FontAsset) {
        self.sprites.clear();

        // Calculate scale factor: target size / atlas size
        let scale = self.font_size as f32 / font.font_size;

        let mut pen_x = 0.0;
        let mut pen_y = 0.0;

        for ch in self.content.chars() {
            if ch == '\n' {
                pen_x = 0.0;
                pen_y += font.line_height * self.line_height * scale;
                continue;
            }

            let Some(glyph) = font.glyphs.get(&ch) else {
                continue;
            };

            // Skip glyphs with no visual representation
            if glyph.size.x == 0.0 || glyph.size.y == 0.0 {
                pen_x += (glyph.advance + self.letter_spacing) * scale;
                continue;
            }

            // Calculate glyph position: pen + bearing (relative to text origin), scaled
            let glyph_pos = Vec2::new(
                pen_x + glyph.bearing.x * scale,
                pen_y + glyph.bearing.y * scale,
            );

            // Store sprite data (position is relative, will be transformed in draw())
            self.sprites.push(SpriteDrawData {
                image_id: font.atlas,
                size: glyph.size * scale,
                position: glyph_pos,
                rotation: 0.0,
                scale: Vec2::new(1.0, 1.0),
                origin: Vec2::new(0.0, 0.0),
                tint: self.color,
                uv_min: glyph.uv_min,
                uv_max: glyph.uv_max,
            });

            pen_x += (glyph.advance + self.letter_spacing) * scale;
        }
    }
}

impl Transform2d for Text {
    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}

impl Drawable for Text {
    fn draw(&self, ctx: &mut RenderContext) {
        // Draw all cached sprites, applying the text's transform to each
        for sprite in &self.sprites {
            let mut sprite_data = sprite.clone();
            // Apply text transform: position is relative to text origin, add text position
            sprite_data.position.x += self.transform.position.x;
            sprite_data.position.y += self.transform.position.y;
            // Note: rotation and scale from transform could also be applied if needed
            ctx.draw_sprite(sprite_data);
        }
    }
}
