use crate::math::color::Color;
use crate::math::Transform;
use crate::render::context::RenderContext;
use crate::render::Vertex;
use crate::math::vec2::Vec2;

use super::{Collider, Drawable, ShapeRef, Transform2d};

pub struct Circle {
    pub transform: Transform,
    pub radius: f32,
    pub color: Color,
    pub segments: u32,
}

impl Circle {
    pub fn new(center: Vec2, radius: f32, color: Color) -> Self {
        let position = Vec2::new(center.x - radius, center.y - radius);
        Self {
            transform: Transform::at(position),
            radius,
            color,
            segments: 32,
        }
    }

    fn size(&self) -> Vec2 {
        Vec2::new(self.radius * 2.0, self.radius * 2.0)
    }

    fn local_center(&self) -> Vec2 {
        Vec2::new(self.radius, self.radius)
    }

    fn transform_point(&self, local: Vec2) -> Vec2 {
        self.transform.transform_point(local, self.size())
    }

    pub fn world_center(&self) -> Vec2 {
        self.transform_point(self.local_center())
    }

    pub fn world_outline(&self) -> Vec<Vec2> {
        let segments = self.segments.max(12);
        let mut points = Vec::with_capacity(segments as usize);

        for i in 0..segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let local_offset = Vec2::new(angle.cos() * self.radius, angle.sin() * self.radius);
            points.push(self.transform_point(self.local_center() + local_offset));
        }

        points
    }

    pub fn set_origin_keep_position(&mut self, origin: Vec2) {
        self.transform.set_origin_keep_position(origin, self.size());
    }

    pub fn set_origin_center_keep_position(&mut self) {
        self.transform.set_origin_center_keep_position(self.size());
    }
}

impl Drawable for Circle {
    fn draw(&self, ctx: &mut RenderContext) {
        let center = self.transform_point(self.local_center());
        let center_ndc = ctx.to_ndc(center);
        let color = self.color.to_linear_rgba();

        let mut verts = Vec::with_capacity((self.segments * 3) as usize);

        // Triangulate the circle as a fan emitted from the center point
        for i in 0..self.segments {
            let a0 = (i as f32 / self.segments as f32) * std::f32::consts::TAU;
            let a1 = ((i + 1) as f32 / self.segments as f32) * std::f32::consts::TAU;

            let local_offset0 = Vec2::new(a0.cos(), a0.sin()) * self.radius;
            let local_offset1 = Vec2::new(a1.cos(), a1.sin()) * self.radius;
            let p0 = self.transform_point(self.local_center() + local_offset0);
            let p1 = self.transform_point(self.local_center() + local_offset1);

            verts.push(Vertex {
                pos: center_ndc.to_array(),
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

impl Transform2d for Circle {
    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}

impl Collider for Circle {
    fn contains_point(&self, point: Vec2) -> bool {
        if let Some(local) = self.transform.to_local(point, self.size()) {
            let center = self.local_center();
            (local - center).length() <= self.radius
        } else {
            false
        }
    }

    fn as_shape(&self) -> ShapeRef<'_> {
        ShapeRef::Circle(self)
    }
}
