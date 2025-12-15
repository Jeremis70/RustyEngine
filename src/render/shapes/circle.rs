use crate::core::color::Color;
use crate::core::render_context::RenderContext;
use crate::core::vertex::Vertex;
use crate::math::vec2::Vec2;

use super::{Collider, Drawable};

pub struct Circle {
    pub center: Vec2,
    pub radius: f32,
    pub color: Color,
    pub segments: u32, // Segment count controls tessellation quality
}

impl Circle {
    pub fn new(center: Vec2, radius: f32, color: Color) -> Self {
        Self {
            center,
            radius,
            color,
            segments: 32, // Reasonable default for a smooth circle
        }
    }
}

impl Drawable for Circle {
    fn draw(&self, ctx: &mut RenderContext) {
        let center = ctx.to_ndc(self.center);
        let color = self.color.to_rgba();

        let mut verts = Vec::with_capacity((self.segments * 3) as usize);

        // Triangulate the circle as a fan emitted from the center point
        for i in 0..self.segments {
            let a0 = (i as f32 / self.segments as f32) * std::f32::consts::TAU;
            let a1 = ((i + 1) as f32 / self.segments as f32) * std::f32::consts::TAU;

            let p0 = self.center + Vec2::new(a0.cos(), a0.sin()) * self.radius;
            let p1 = self.center + Vec2::new(a1.cos(), a1.sin()) * self.radius;

            verts.push(Vertex {
                pos: center.to_array(),
                color,
            });
            verts.push(Vertex {
                pos: ctx.to_ndc(p0).to_array(),
                color,
            });
            verts.push(Vertex {
                pos: ctx.to_ndc(p1).to_array(),
                color,
            });
        }

        ctx.extend(&verts);
    }
}

impl Collider for Circle {
    fn contains_point(&self, point: Vec2) -> bool {
        let dist_sq = (point - self.center).length().powi(2);
        dist_sq <= self.radius * self.radius
    }

    fn intersects(&self, _other: &Self) -> bool {
        todo!();
    }
}
