use crate::core::color::Color;
use crate::core::render_context::RenderContext;
use crate::core::vertex::Vertex;
use crate::math::vec2::Vec2;

use super::{Collider, Drawable};

pub struct Triangle {
    pub p1: Vec2,
    pub p2: Vec2,
    pub p3: Vec2,
    pub color: Color,
}

impl Triangle {
    pub fn new(p1: Vec2, p2: Vec2, p3: Vec2, color: Color) -> Self {
        Self { p1, p2, p3, color }
    }
}

impl Drawable for Triangle {
    fn draw(&self, ctx: &mut RenderContext) {
        let color = self.color.to_rgba();

        // Convert pixel space → NDC
        let v1 = ctx.to_ndc(self.p1);
        let v2 = ctx.to_ndc(self.p2);
        let v3 = ctx.to_ndc(self.p3);

        let vertices = [
            Vertex {
                pos: v1.to_array(),
                color,
            },
            Vertex {
                pos: v2.to_array(),
                color,
            },
            Vertex {
                pos: v3.to_array(),
                color,
            },
        ];

        ctx.extend(&vertices);
    }
}

impl Collider for Triangle {
    fn contains_point(&self, p: Vec2) -> bool {
        let a = self.p1;
        let b = self.p2;
        let c = self.p3;

        let u = c - a;
        let v = b - a;
        let w = p - a;

        let dot_uu = u * u;
        let dot_uv = u * v;
        let dot_uw = u * w;
        let dot_vv = v * v;
        let dot_vw = v * w;

        let denom = dot_uu * dot_vv - dot_uv * dot_uv;
        if denom == 0.0 {
            return false; // Degenerate triangle
        }
        let inv_denom = 1.0 / denom;
        let s = (dot_vv * dot_uw - dot_uv * dot_vw) * inv_denom;
        let t = (dot_uu * dot_vw - dot_uv * dot_uw) * inv_denom;

        s >= 0.0 && t >= 0.0 && (s + t) <= 1.0
    }

    fn intersects(&self, _other: &Self) -> bool {
        todo!();
    }
}
