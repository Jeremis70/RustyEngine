use crate::core::color::Color;
use crate::core::render_context::RenderContext;
use crate::core::vertex::Vertex;
use crate::math::vec2::Vec2;

use super::{Collider, Drawable, ShapeRef, Transform2d};

pub struct Rectangle {
    pub position: Vec2, // top-left in pixels
    pub size: Vec2,     // width / height in pixels
    pub color: Color,
    pub rotation: f32, // radians
    pub scale: Vec2,   // non-uniform scale factors
    pub origin: Vec2,  // normalized pivot (0..1)
}

impl Rectangle {
    pub fn new(position: Vec2, size: Vec2, color: Color) -> Self {
        Self {
            position,
            size,
            color,
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
            origin: Vec2::ZERO,
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
        <Self as Transform2d>::transform_point(self, local, self.size)
    }

    pub fn set_origin_keep_position(&mut self, origin: Vec2) {
        <Self as Transform2d>::set_origin_keep_position(self, origin, self.size);
    }

    pub fn set_origin_center_keep_position(&mut self) {
        <Self as Transform2d>::set_origin_center_keep_position(self, self.size);
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
        self.translate(delta);
    }

    pub fn set_topright(&mut self, p: Vec2) {
        let delta = p - self.topright();
        self.translate(delta);
    }

    pub fn set_bottomleft(&mut self, p: Vec2) {
        let delta = p - self.bottomleft();
        self.translate(delta);
    }

    pub fn set_bottomright(&mut self, p: Vec2) {
        let delta = p - self.bottomright();
        self.translate(delta);
    }

    // Center getter/setter
    pub fn center(&self) -> Vec2 {
        self.transform_point(Vec2::new(self.size.x * 0.5, self.size.y * 0.5))
    }

    pub fn set_center(&mut self, c: Vec2) {
        let delta = c - self.center();
        self.translate(delta);
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
        self.translate(Vec2::new(0.0, delta));
    }

    pub fn set_bottom(&mut self, b: f32) {
        let delta = b - self.bottom();
        self.translate(Vec2::new(0.0, delta));
    }

    pub fn set_left(&mut self, l: f32) {
        let delta = l - self.left();
        self.translate(Vec2::new(delta, 0.0));
    }

    pub fn set_right(&mut self, r: f32) {
        let delta = r - self.right();
        self.translate(Vec2::new(delta, 0.0));
    }
}

impl Transform2d for Rectangle {
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
        if let Some(local) = <Self as Transform2d>::to_local(self, point, self.size) {
            local.x >= 0.0 && local.x <= self.size.x && local.y >= 0.0 && local.y <= self.size.y
        } else {
            false
        }
    }

    fn as_shape(&self) -> ShapeRef<'_> {
        ShapeRef::Rectangle(self)
    }
}
