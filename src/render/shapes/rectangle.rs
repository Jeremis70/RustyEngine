use crate::math::Transform;
use crate::math::color::Color;
use crate::math::vec2::Vec2;
use crate::render::Vertex;
use crate::render::context::RenderContext;

use super::{Collider, Drawable, ShapeRef, Transform2d};

pub struct Rectangle {
    pub transform: Transform,
    pub size: Vec2,
    pub color: Color,

    pub filled: bool,
    pub outline_thickness: f32,
    pub outline_color: Color,
}

impl Rectangle {
    pub fn new(position: Vec2, size: Vec2, color: Color) -> Self {
        Self {
            transform: Transform::at(position),
            size,
            color,

            filled: true,
            outline_thickness: 0.0,
            outline_color: color,
        }
    }

    /// Pygame-like outline rectangle (non-filled) with a line thickness.
    pub fn new_outline(position: Vec2, size: Vec2, color: Color, thickness: f32) -> Self {
        let mut rect = Self::new(position, size, color);
        rect.filled = false;
        rect.outline_thickness = thickness;
        rect.outline_color = color;
        rect
    }

    pub fn set_filled(&mut self, filled: bool) {
        self.filled = filled;
    }

    pub fn set_outline(&mut self, thickness: f32, color: Color) {
        self.outline_thickness = thickness;
        self.outline_color = color;
    }

    fn world_corners(&self) -> [Vec2; 4] {
        [
            self.transform_point(Vec2::ZERO),
            self.transform_point(Vec2::new(self.size.x, 0.0)),
            self.transform_point(Vec2::new(0.0, self.size.y)),
            self.transform_point(Vec2::new(self.size.x, self.size.y)),
        ]
    }

    fn transform_point(&self, local: Vec2) -> Vec2 {
        self.transform.transform_point(local, self.size)
    }

    pub fn set_origin_keep_position(&mut self, origin: Vec2) {
        self.transform.set_origin_keep_position(origin, self.size);
    }

    pub fn set_origin_center_keep_position(&mut self) {
        self.transform.set_origin_center_keep_position(self.size);
    }

    pub fn world_outline(&self) -> Vec<Vec2> {
        let corners = self.world_corners();
        vec![corners[0], corners[1], corners[3], corners[2]]
    }

    // Corner getters in world space
    pub fn topleft(&self) -> Vec2 {
        self.transform_point(Vec2::ZERO)
    }

    pub fn topright(&self) -> Vec2 {
        self.transform_point(Vec2::new(self.size.x, 0.0))
    }

    pub fn bottomleft(&self) -> Vec2 {
        self.transform_point(Vec2::new(0.0, self.size.y))
    }

    pub fn bottomright(&self) -> Vec2 {
        self.transform_point(Vec2::new(self.size.x, self.size.y))
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
        self.transform_point(Vec2::new(self.size.x * 0.5, self.size.y * 0.5))
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
}

impl Transform2d for Rectangle {
    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}

impl Drawable for Rectangle {
    fn draw(&self, ctx: &mut RenderContext) {
        let mut push_quad = |local_min: Vec2, local_max: Vec2, color: Color| {
            let tl = self.transform_point(local_min);
            let tr = self.transform_point(Vec2::new(local_max.x, local_min.y));
            let bl = self.transform_point(Vec2::new(local_min.x, local_max.y));
            let br = self.transform_point(local_max);

            // Convert pixel space → NDC
            let tl = ctx.to_ndc(tl);
            let tr = ctx.to_ndc(tr);
            let bl = ctx.to_ndc(bl);
            let br = ctx.to_ndc(br);

            let color = color.to_linear_rgba();

            // Two triangles (CCW)
            let vertices = [
                Vertex {
                    pos: tl.to_array(),
                    color,
                },
                Vertex {
                    pos: tr.to_array(),
                    color,
                },
                Vertex {
                    pos: bl.to_array(),
                    color,
                },
                Vertex {
                    pos: tr.to_array(),
                    color,
                },
                Vertex {
                    pos: br.to_array(),
                    color,
                },
                Vertex {
                    pos: bl.to_array(),
                    color,
                },
            ];

            ctx.extend(&vertices);
        };

        // Fill
        if self.filled {
            push_quad(Vec2::ZERO, self.size, self.color);
        }

        // Outline (pygame-style thickness)
        if self.outline_thickness > 0.0 {
            let w = self.size.x.max(0.0);
            let h = self.size.y.max(0.0);
            if w > 0.0 && h > 0.0 {
                let t = self.outline_thickness.max(0.5).min(w * 0.5).min(h * 0.5);

                // Top
                push_quad(Vec2::new(0.0, 0.0), Vec2::new(w, t), self.outline_color);
                // Bottom
                push_quad(Vec2::new(0.0, h - t), Vec2::new(w, h), self.outline_color);
                // Left
                push_quad(Vec2::new(0.0, t), Vec2::new(t, h - t), self.outline_color);
                // Right
                push_quad(Vec2::new(w - t, t), Vec2::new(w, h - t), self.outline_color);
            }
        }
    }
}

impl Collider for Rectangle {
    fn contains_point(&self, point: Vec2) -> bool {
        if let Some(local) = self.transform.to_local(point, self.size) {
            local.x >= 0.0 && local.x <= self.size.x && local.y >= 0.0 && local.y <= self.size.y
        } else {
            false
        }
    }

    fn as_shape(&self) -> ShapeRef<'_> {
        ShapeRef::Rectangle(self)
    }
}
