use crate::math::color::Color;
use crate::render::context::RenderContext;
use crate::render::Vertex;
use crate::math::vec2::Vec2;

use super::{Collider, Drawable, ShapeRef, Transform2d};

pub struct Triangle {
    pub position: Vec2,
    pub local_points: [Vec2; 3],
    pub color: Color,
    pub rotation: f32,
    pub scale: Vec2,
    pub origin: Vec2,
    pub size: Vec2,
}

impl Triangle {
    pub fn new(p1: Vec2, p2: Vec2, p3: Vec2, color: Color) -> Self {
        let min_x = p1.x.min(p2.x).min(p3.x);
        let min_y = p1.y.min(p2.y).min(p3.y);
        let max_x = p1.x.max(p2.x).max(p3.x);
        let max_y = p1.y.max(p2.y).max(p3.y);
        let position = Vec2::new(min_x, min_y);
        let size = Vec2::new(max_x - min_x, max_y - min_y);

        let local_points = [p1 - position, p2 - position, p3 - position];

        Self {
            position,
            local_points,
            color,
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
            origin: Vec2::ZERO,
            size,
        }
    }

    fn transform_point(&self, local: Vec2) -> Vec2 {
        <Self as Transform2d>::transform_point(self, local, self.size)
    }

    fn world_points(&self) -> [Vec2; 3] {
        [
            self.transform_point(self.local_points[0]),
            self.transform_point(self.local_points[1]),
            self.transform_point(self.local_points[2]),
        ]
    }

    pub fn set_origin_keep_position(&mut self, origin: Vec2) {
        <Self as Transform2d>::set_origin_keep_position(self, origin, self.size);
    }

    pub fn set_origin_center_keep_position(&mut self) {
        <Self as Transform2d>::set_origin_center_keep_position(self, self.size);
    }

    pub fn world_outline(&self) -> Vec<Vec2> {
        self.world_points().to_vec()
    }
}

impl Drawable for Triangle {
    fn draw(&self, ctx: &mut RenderContext) {
        let color = self.color.to_linear_rgba();
        let [v1, v2, v3] = self.world_points();

        let vertices = [
            Vertex {
                pos: ctx.to_ndc(v1).to_array(),
                color,
            },
            Vertex {
                pos: ctx.to_ndc(v2).to_array(),
                color,
            },
            Vertex {
                pos: ctx.to_ndc(v3).to_array(),
                color,
            },
        ];

        ctx.extend(&vertices);
    }
}

impl Transform2d for Triangle {
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

impl Collider for Triangle {
    fn contains_point(&self, p: Vec2) -> bool {
        if let Some(local) = <Self as Transform2d>::to_local(self, p, self.size) {
            let a = self.local_points[0];
            let b = self.local_points[1];
            let c = self.local_points[2];

            let u = c - a;
            let v = b - a;
            let w = local - a;

            let dot_uu = u * u;
            let dot_uv = u * v;
            let dot_uw = u * w;
            let dot_vv = v * v;
            let dot_vw = v * w;

            let denom = dot_uu * dot_vv - dot_uv * dot_uv;
            if denom.abs() <= f32::EPSILON {
                return false;
            }
            let inv_denom = 1.0 / denom;
            let s = (dot_vv * dot_uw - dot_uv * dot_vw) * inv_denom;
            let t = (dot_uu * dot_vw - dot_uv * dot_uw) * inv_denom;

            s >= 0.0 && t >= 0.0 && (s + t) <= 1.0
        } else {
            false
        }
    }

    fn as_shape(&self) -> ShapeRef<'_> {
        ShapeRef::Triangle(self)
    }
}
