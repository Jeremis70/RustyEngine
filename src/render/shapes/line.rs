use crate::core::color::Color;
use crate::core::render_context::RenderContext;
use crate::core::vertex::Vertex;
use crate::math::vec2::Vec2;

use super::{Collider, Drawable, ShapeRef, Transform2d};

pub struct Line {
    pub position: Vec2,
    pub size: Vec2,
    pub color: Color,
    pub thickness: f32,
    pub rotation: f32,
    pub scale: Vec2,
    pub origin: Vec2,
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
            position,
            size,
            color,
            thickness,
            rotation: angle,
            scale: Vec2::new(1.0, 1.0),
            origin: Vec2::new(0.0, 0.5),
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
            .map(|corner| <Self as Transform2d>::transform_point(self, corner, self.size))
    }

    pub fn set_origin_keep_position(&mut self, origin: Vec2) {
        <Self as Transform2d>::set_origin_keep_position(self, origin, self.size);
    }

    pub fn set_origin_center_keep_position(&mut self) {
        <Self as Transform2d>::set_origin_center_keep_position(self, self.size);
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

impl Collider for Line {
    fn contains_point(&self, point: Vec2) -> bool {
        if let Some(local) = <Self as Transform2d>::to_local(self, point, self.size) {
            local.x >= 0.0 && local.x <= self.size.x && local.y >= 0.0 && local.y <= self.size.y
        } else {
            false
        }
    }

    fn as_shape(&self) -> ShapeRef<'_> {
        ShapeRef::Line(self)
    }
}
