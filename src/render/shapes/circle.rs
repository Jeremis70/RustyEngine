use crate::core::color::Color;
use crate::core::render_context::RenderContext;
use crate::core::vertex::Vertex;
use crate::math::vec2::Vec2;

use super::{Collider, Drawable, ShapeRef, Transform2d};

pub struct Circle {
    pub position: Vec2, // top-left of bounding box
    pub radius: f32,
    pub color: Color,
    pub segments: u32, // Segment count controls tessellation quality
    pub rotation: f32,
    pub scale: Vec2,
    pub origin: Vec2,
}

impl Circle {
    pub fn new(center: Vec2, radius: f32, color: Color) -> Self {
        let position = Vec2::new(center.x - radius, center.y - radius);
        Self {
            position,
            radius,
            color,
            segments: 32,
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
            origin: Vec2::ZERO,
        }
    }

    fn size(&self) -> Vec2 {
        Vec2::new(self.radius * 2.0, self.radius * 2.0)
    }

    fn local_center(&self) -> Vec2 {
        Vec2::new(self.radius, self.radius)
    }

    fn transform_point(&self, local: Vec2) -> Vec2 {
        <Self as Transform2d>::transform_point(self, local, self.size())
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
        <Self as Transform2d>::set_origin_keep_position(self, origin, self.size());
    }

    pub fn set_origin_center_keep_position(&mut self) {
        <Self as Transform2d>::set_origin_center_keep_position(self, self.size());
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

impl Collider for Circle {
    fn contains_point(&self, point: Vec2) -> bool {
        if let Some(local) = <Self as Transform2d>::to_local(self, point, self.size()) {
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
