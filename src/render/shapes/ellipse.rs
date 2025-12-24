use crate::math::Transform;
use crate::math::color::Color;
use crate::math::vec2::Vec2;
use crate::render::Vertex;
use crate::render::context::RenderContext;

use super::{Collider, Drawable, ShapeRef, Transform2d};

pub struct Ellipse {
    pub transform: Transform,
    pub radii: Vec2,
    pub color: Color,
    pub segments: u32,
}

impl Ellipse {
    pub fn new(center: Vec2, radius_x: f32, radius_y: f32, color: Color) -> Self {
        let radii = Vec2::new(radius_x, radius_y);
        let position = center - radii;

        Self {
            transform: Transform::at(position),
            radii,
            color,
            segments: 32,
        }
    }

    fn size(&self) -> Vec2 {
        self.radii * 2.0
    }

    fn local_center(&self) -> Vec2 {
        self.radii
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
            let local_offset = Vec2::new(angle.cos() * self.radii.x, angle.sin() * self.radii.y);
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

impl Drawable for Ellipse {
    fn draw(&self, ctx: &mut RenderContext) {
        let segments = self.segments.max(3);
        let color = self.color.to_rgba();
        let center_world = self.transform_point(self.local_center());
        let center_ndc = ctx.to_ndc(center_world);

        let mut verts = Vec::with_capacity((segments * 3) as usize);

        for i in 0..segments {
            let a0 = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let a1 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;

            let local_offset0 = Vec2::new(a0.cos() * self.radii.x, a0.sin() * self.radii.y);
            let local_offset1 = Vec2::new(a1.cos() * self.radii.x, a1.sin() * self.radii.y);

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

impl Transform2d for Ellipse {
    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}

impl Collider for Ellipse {
    fn contains_point(&self, p: Vec2) -> bool {
        if let Some(local) = self.transform.to_local(p, self.size()) {
            let centered = local - self.local_center();
            let x = centered.x / self.radii.x;
            let y = centered.y / self.radii.y;
            x * x + y * y <= 1.0
        } else {
            false
        }
    }

    fn as_shape(&self) -> ShapeRef<'_> {
        ShapeRef::Ellipse(self)
    }
}
