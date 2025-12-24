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
}

impl Rectangle {
    pub fn new(position: Vec2, size: Vec2, color: Color) -> Self {
        Self {
            transform: Transform::at(position),
            size,
            color,
        }
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
        let [tl, tr, bl, br] = self.world_corners();

        // Convert pixel space → NDC
        let tl = ctx.to_ndc(tl);
        let tr = ctx.to_ndc(tr);
        let bl = ctx.to_ndc(bl);
        let br = ctx.to_ndc(br);

        let color = self.color.to_linear_rgba();

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
