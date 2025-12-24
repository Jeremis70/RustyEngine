use crate::math::color::Color;
use crate::math::Transform;
use crate::render::context::RenderContext;
use crate::render::Vertex;
use crate::math::vec2::Vec2;

use super::{Collider, Drawable, ShapeRef, Transform2d};

pub struct Line {
    pub transform: Transform,
    pub size: Vec2,
    pub color: Color,
    pub thickness: f32,
}

impl Line {
    pub fn new(start: Vec2, end: Vec2, color: Color, thickness: f32) -> Self {
        let thickness = thickness.max(0.5);
        let delta = end - start;
        let length = delta.length().max(1e-3);
        let angle = delta.y.atan2(delta.x);
        let size = Vec2::new(length, thickness);
        let pivot = Vec2::new(0.0, thickness * 0.5);
        let position = start - pivot;

        Self {
            transform: Transform::at(position)
                .with_rotation(angle)
                .with_origin(Vec2::new(0.0, 0.5)),
            size,
            color,
            thickness,
        }
    }

    fn local_corners(&self) -> [Vec2; 4] {
        [
            Vec2::ZERO,
            Vec2::new(self.size.x, 0.0),
            Vec2::new(self.size.x, self.size.y),
            Vec2::new(0.0, self.size.y),
        ]
    }

    fn world_corners(&self) -> [Vec2; 4] {
        self.local_corners()
            .map(|corner| self.transform.transform_point(corner, self.size))
    }

    pub fn set_origin_keep_position(&mut self, origin: Vec2) {
        self.transform.set_origin_keep_position(origin, self.size);
    }

    pub fn set_origin_center_keep_position(&mut self) {
        self.transform.set_origin_center_keep_position(self.size);
    }

    pub fn world_outline(&self) -> Vec<Vec2> {
        self.world_corners().to_vec()
    }
}

impl Drawable for Line {
    fn draw(&self, ctx: &mut RenderContext) {
        let [tl, tr, br, bl] = {
            let corners = self.world_corners();
            [corners[0], corners[1], corners[2], corners[3]]
        };

        let color = self.color.to_rgba();
        let vertices = [
            Vertex {
                pos: ctx.to_ndc(tl).to_array(),
                color,
            },
            Vertex {
                pos: ctx.to_ndc(tr).to_array(),
                color,
            },
            Vertex {
                pos: ctx.to_ndc(br).to_array(),
                color,
            },
            Vertex {
                pos: ctx.to_ndc(br).to_array(),
                color,
            },
            Vertex {
                pos: ctx.to_ndc(bl).to_array(),
                color,
            },
            Vertex {
                pos: ctx.to_ndc(tl).to_array(),
                color,
            },
        ];

        ctx.extend(&vertices);
    }
}

impl Transform2d for Line {
    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}

impl Collider for Line {
    fn contains_point(&self, point: Vec2) -> bool {
        if let Some(local) = self.transform.to_local(point, self.size) {
            local.x >= 0.0 && local.x <= self.size.x && local.y >= 0.0 && local.y <= self.size.y
        } else {
            false
        }
    }

    fn as_shape(&self) -> ShapeRef<'_> {
        ShapeRef::Line(self)
    }
}
