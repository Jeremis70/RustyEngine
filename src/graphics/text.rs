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

    /// Tab width expressed in number of spaces.
    pub tab_width_spaces: u32,

    /// Replacement character used when a glyph is missing from the atlas.
    pub fallback_char: char,

    /// Cached sprite data for rendering. Updated via layout().
    sprites: Vec<SpriteDrawData>,

    /// Local-space bounds of the laid out text (min/max).
    /// Used to apply `transform.origin` consistently.
    bounds_min: Vec2,
    bounds_max: Vec2,

    /// Layout size (pen-advance based), including line height and whitespace.
    layout_size: Vec2,
}

impl Text {
    pub fn new(font: FontId, content: &str, font_size: u32, color: Color) -> Self {
        let mut transform = Transform::new();
        // Text is typically anchored at top-left by default.
        transform.origin = Vec2::ZERO;
        Self {
            font,
            content: content.to_string(),
            font_size,
            color,
            transform,
            line_height: 1.0,
            letter_spacing: 0.0,
            tab_width_spaces: 4,
            fallback_char: '?',
            sprites: Vec::new(),
            bounds_min: Vec2::ZERO,
            bounds_max: Vec2::ZERO,
            layout_size: Vec2::ZERO,
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
        let mut transform = Transform::new();
        // Text is typically anchored at top-left by default.
        transform.origin = Vec2::ZERO;
        Self {
            font,
            content: content.to_string(),
            font_size,
            color,
            transform,
            line_height,
            letter_spacing,
            tab_width_spaces: 4,
            fallback_char: '?',
            sprites: Vec::new(),
            bounds_min: Vec2::ZERO,
            bounds_max: Vec2::ZERO,
            layout_size: Vec2::ZERO,
        }
    }

    /// Current laid-out size in pixels.
    /// Returns (0,0) if `layout()` has not been called or the content is empty.
    pub fn size(&self) -> Vec2 {
        Vec2::new(
            (self.bounds_max.x - self.bounds_min.x).max(0.0),
            (self.bounds_max.y - self.bounds_min.y).max(0.0),
        )
    }

    /// Layout size in pixels based on pen advance (includes whitespace and line height).
    /// Useful for UI layout where you want "reserved space".
    pub fn layout_size(&self) -> Vec2 {
        self.layout_size
    }

    fn transform_point(&self, local: Vec2) -> Vec2 {
        self.transform.transform_point(local, self.size())
    }

    fn world_corners(&self) -> [Vec2; 4] {
        let size = self.size();
        [
            self.transform_point(Vec2::ZERO),
            self.transform_point(Vec2::new(size.x, 0.0)),
            self.transform_point(Vec2::new(0.0, size.y)),
            self.transform_point(Vec2::new(size.x, size.y)),
        ]
    }

    // Corner getters in world space
    pub fn topleft(&self) -> Vec2 {
        self.transform_point(Vec2::ZERO)
    }

    pub fn topright(&self) -> Vec2 {
        let s = self.size();
        self.transform_point(Vec2::new(s.x, 0.0))
    }

    pub fn bottomleft(&self) -> Vec2 {
        let s = self.size();
        self.transform_point(Vec2::new(0.0, s.y))
    }

    pub fn bottomright(&self) -> Vec2 {
        let s = self.size();
        self.transform_point(Vec2::new(s.x, s.y))
    }

    // Corner setters (keep size constant, adjust position accordingly)
    pub fn set_topleft(&mut self, p: Vec2) {
        let delta = p - self.topleft();
        self.transform.translate(delta);
    }

    pub fn set_topright(&mut self, p: Vec2) {
        let delta = p - self.topright();
        self.transform.translate(delta);
    }

    pub fn set_bottomleft(&mut self, p: Vec2) {
        let delta = p - self.bottomleft();
        self.transform.translate(delta);
    }

    pub fn set_bottomright(&mut self, p: Vec2) {
        let delta = p - self.bottomright();
        self.transform.translate(delta);
    }

    // Center getter/setter
    pub fn center(&self) -> Vec2 {
        let s = self.size();
        self.transform_point(Vec2::new(s.x * 0.5, s.y * 0.5))
    }

    pub fn set_center(&mut self, c: Vec2) {
        let delta = c - self.center();
        self.transform.translate(delta);
    }

    pub fn top(&self) -> f32 {
        self.world_corners()
            .iter()
            .map(|v| v.y)
            .fold(f32::INFINITY, f32::min)
    }

    pub fn bottom(&self) -> f32 {
        self.world_corners()
            .iter()
            .map(|v| v.y)
            .fold(f32::NEG_INFINITY, f32::max)
    }

    pub fn left(&self) -> f32 {
        self.world_corners()
            .iter()
            .map(|v| v.x)
            .fold(f32::INFINITY, f32::min)
    }

    pub fn right(&self) -> f32 {
        self.world_corners()
            .iter()
            .map(|v| v.x)
            .fold(f32::NEG_INFINITY, f32::max)
    }

    pub fn set_top(&mut self, t: f32) {
        let delta = t - self.top();
        self.transform.translate(Vec2::new(0.0, delta));
    }

    pub fn set_bottom(&mut self, b: f32) {
        let delta = b - self.bottom();
        self.transform.translate(Vec2::new(0.0, delta));
    }

    pub fn set_left(&mut self, l: f32) {
        let delta = l - self.left();
        self.transform.translate(Vec2::new(delta, 0.0));
    }

    pub fn set_right(&mut self, r: f32) {
        let delta = r - self.right();
        self.transform.translate(Vec2::new(delta, 0.0));
    }

    /// Layout the text by computing sprite data from the font atlas.
    /// Must be called after creating or modifying the text, and requires access to AssetManager.
    /// After calling this, draw() can be used without needing AssetManager.
    pub fn layout(&mut self, assets: &AssetManager) {
        let Some(font) = assets.get_font(self.font) else {
            self.sprites.clear();
            self.bounds_min = Vec2::ZERO;
            self.bounds_max = Vec2::ZERO;
            self.layout_size = Vec2::ZERO;
            return;
        };

        self.layout_with_font_asset(font);
    }

    /// Layout using a previously retrieved `FontAsset`.
    /// Useful for dynamic text in render callbacks where `AssetManager` isn't available.
    pub fn layout_with_font_asset(&mut self, font: &FontAsset) {
        self.sprites.clear();

        // Reset bounds; will be expanded while laying out.
        self.bounds_min = Vec2::ZERO;
        self.bounds_max = Vec2::ZERO;
        self.layout_size = Vec2::ZERO;

        if self.font_size == 0 {
            return;
        }

        if !font.font_size.is_finite() || font.font_size <= 0.0 {
            return;
        }

        let line_height_mul = if self.line_height.is_finite() && self.line_height > 0.0 {
            self.line_height
        } else {
            1.0
        };

        let letter_spacing = if self.letter_spacing.is_finite() {
            self.letter_spacing
        } else {
            0.0
        };

        // Calculate scale factor: target size / atlas size
        let scale = self.font_size as f32 / font.font_size;

        if !scale.is_finite() || scale <= 0.0 {
            return;
        }

        let mut pen_x = 0.0;
        let mut pen_y = 0.0;

        // Track local-space bounds (may include negative extents due to bearings).
        let mut any_bounds = false;
        let mut min = Vec2::new(f32::INFINITY, f32::INFINITY);
        let mut max = Vec2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);

        let mut extend = |p: Vec2| {
            if !p.x.is_finite() || !p.y.is_finite() {
                return;
            }
            any_bounds = true;
            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
        };

        let default_line_advance = font.line_height * line_height_mul * scale;

        let space_advance = font.glyphs.get(&' ').map(|g| g.advance).unwrap_or(0.0);

        let mut max_line_width = 0.0f32;

        for ch in self.content.chars() {
            if ch == '\n' {
                max_line_width = max_line_width.max(pen_x);
                pen_x = 0.0;
                pen_y += default_line_advance;
                continue;
            }

            if ch == '\t' {
                let tab_adv = (space_advance + letter_spacing) * scale;
                pen_x += tab_adv * self.tab_width_spaces.max(1) as f32;
                continue;
            }

            // Missing glyph handling: try the requested char, then fallback, then advance like space.
            let glyph = match font.glyphs.get(&ch) {
                Some(g) => g,
                None => match font.glyphs.get(&self.fallback_char) {
                    Some(g) => g,
                    None => {
                        pen_x += (space_advance + letter_spacing) * scale;
                        continue;
                    }
                },
            };

            // Skip glyphs with no visual representation
            if glyph.size.x == 0.0 || glyph.size.y == 0.0 {
                pen_x += (glyph.advance + letter_spacing) * scale;
                continue;
            }

            // Calculate glyph position in local space.
            //
            // fontdue metrics:
            // - xmin: offset of the *left-most* bitmap edge from the origin.
            // - ymin: offset of the *bottom-most* bitmap edge from the baseline (Y-up).
            //   So the bitmap top edge in Y-up is: (ymin + height).
            //
            // Our engine coordinates are Y-down. If `pen_y` represents the baseline in Y-down,
            // then bitmap_top_y_down = baseline_y_down - (ymin + height).
            let glyph_pos = Vec2::new(
                pen_x + glyph.bearing.x * scale,
                pen_y - (glyph.bearing.y + glyph.size.y) * scale,
            );

            let glyph_size = glyph.size * scale;

            extend(glyph_pos);
            extend(Vec2::new(
                glyph_pos.x + glyph_size.x,
                glyph_pos.y + glyph_size.y,
            ));

            // Store sprite data (position is relative, will be transformed in draw())
            self.sprites.push(SpriteDrawData {
                image_id: font.atlas,
                size: glyph_size,
                position: glyph_pos,
                rotation: 0.0,
                scale: Vec2::new(1.0, 1.0),
                origin: Vec2::new(0.0, 0.0),
                tint: self.color,
                uv_min: glyph.uv_min,
                uv_max: glyph.uv_max,
            });

            pen_x += (glyph.advance + letter_spacing) * scale;
        }

        max_line_width = max_line_width.max(pen_x);
        if !self.content.is_empty() {
            self.layout_size = Vec2::new(
                max_line_width.max(0.0),
                (pen_y + default_line_advance).max(0.0),
            );
        }

        if any_bounds {
            // Normalize so that (0,0) is the *tight* top-left of the text.
            // This makes `transform.position` behave like "text top-left".
            let offset = min;
            for sprite in &mut self.sprites {
                sprite.position = sprite.position - offset;
            }

            self.bounds_min = Vec2::ZERO;
            self.bounds_max = max - offset;
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
        if self.sprites.is_empty() {
            return;
        }

        // Use `transform.origin` like other drawables: position is the origin pivot.
        // Since layout normalizes bounds to start at (0,0), origin=(0,0) means top-left.
        let bounds_size = self.size();
        let pivot_local = bounds_size.hadamard(self.transform.origin);
        let pivot_world = self.transform.position + pivot_local.hadamard(self.transform.scale);

        for sprite in &self.sprites {
            let mut sprite_data = sprite.clone();

            // Keep tint in sync even if `color` changes post-layout.
            sprite_data.tint = self.color;

            // Apply the text transform exactly once.
            // We keep every glyph at the same world-space anchor (the text pivot) and encode the
            // glyph offset via `origin`. This makes rotation/scale happen around the *shared* pivot.
            //
            // Renderer math (simplified): world = rotate(scale*(local - origin_px)) + position
            // We want: world = rotate(scale*((glyph_pos + local) - pivot_local)) + text_position
            // => choose origin_px = pivot_local - glyph_pos, and position = text_position.
            let glyph_pos = sprite.position;
            let glyph_size = sprite.size;

            if glyph_size.x.abs() <= f32::EPSILON || glyph_size.y.abs() <= f32::EPSILON {
                continue;
            }

            let origin_px = pivot_local - glyph_pos;
            let origin = Vec2::new(origin_px.x / glyph_size.x, origin_px.y / glyph_size.y);

            // Match `Transform::transform_point` semantics (used by shapes like Rectangle):
            // `transform.position` is a translation, not the pivot point.
            // The sprite renderer expects `position` to be the *pivot* in world space,
            // so we add the scaled pivot offset.
            sprite_data.position = pivot_world;
            sprite_data.rotation = self.transform.rotation;
            sprite_data.scale = self.transform.scale;
            sprite_data.origin = origin;

            ctx.draw_sprite(sprite_data);
        }
    }
}
