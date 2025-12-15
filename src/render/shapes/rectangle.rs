use crate::core::color::Color;
use crate::core::render_context::RenderContext;
use crate::core::vertex::Vertex;
use crate::math::vec2::Vec2;

use super::{Collider, Drawable};

pub struct Rectangle {
    pub position: Vec2, // top-left in pixels
    pub size: Vec2,     // width / height in pixels
    pub color: Color,
}

impl Rectangle {
    pub fn new(position: Vec2, size: Vec2, color: Color) -> Self {
        Self {
            position,
            size,
            color,
        }
    }

    // Corner getters
    pub fn topleft(&self) -> Vec2 {
        self.position
    }

    pub fn topright(&self) -> Vec2 {
        Vec2 {
            x: self.position.x + self.size.x,
            y: self.position.y,
        }
    }

    pub fn bottomleft(&self) -> Vec2 {
        Vec2 {
            x: self.position.x,
            y: self.position.y + self.size.y,
        }
    }

    pub fn bottomright(&self) -> Vec2 {
        Vec2 {
            x: self.position.x + self.size.x,
            y: self.position.y + self.size.y,
        }
    }

    // Corner setters (keep size constant, adjust position accordingly)
    pub fn set_topleft(&mut self, p: Vec2) {
        self.position = p;
    }

    pub fn set_topright(&mut self, p: Vec2) {
        self.position.x = p.x - self.size.x;
        self.position.y = p.y;
    }

    pub fn set_bottomleft(&mut self, p: Vec2) {
        self.position.x = p.x;
        self.position.y = p.y - self.size.y;
    }

    pub fn set_bottomright(&mut self, p: Vec2) {
        self.position.x = p.x - self.size.x;
        self.position.y = p.y - self.size.y;
    }

    // Center getter/setter
    pub fn center(&self) -> Vec2 {
        Vec2 {
            x: self.position.x + self.size.x * 0.5,
            y: self.position.y + self.size.y * 0.5,
        }
    }

    pub fn top(&self) -> f32 {
        self.position.y
    }

    pub fn bottom(&self) -> f32 {
        self.position.y + self.size.y
    }

    pub fn left(&self) -> f32 {
        self.position.x
    }

    pub fn right(&self) -> f32 {
        self.position.x + self.size.x
    }

    pub fn set_top(&mut self, t: f32) {
        self.position.y = t;
    }

    pub fn set_bottom(&mut self, b: f32) {
        self.position.y = b - self.size.y;
    }

    pub fn set_left(&mut self, l: f32) {
        self.position.x = l;
    }

    pub fn set_right(&mut self, r: f32) {
        self.position.x = r - self.size.x;
    }

    pub fn set_center(&mut self, c: Vec2) {
        self.position.x = c.x - self.size.x * 0.5;
        self.position.y = c.y - self.size.y * 0.5;
    }
}

impl Drawable for Rectangle {
    fn draw(&self, ctx: &mut RenderContext) {
        let x = self.position.x;
        let y = self.position.y;
        let w = self.size.x;
        let h = self.size.y;

        // Convert pixel space → NDC
        let tl = ctx.to_ndc(Vec2 { x, y });
        let tr = ctx.to_ndc(Vec2 { x: x + w, y });
        let bl = ctx.to_ndc(Vec2 { x, y: y + h });
        let br = ctx.to_ndc(Vec2 { x: x + w, y: y + h });

        let color = self.color.to_rgba();

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
        point.x >= self.left()
            && point.x <= self.right()
            && point.y >= self.top()
            && point.y <= self.bottom()
    }

    fn intersects(&self, _other: &Self) -> bool {
        todo!();
    }
}
