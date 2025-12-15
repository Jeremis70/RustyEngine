use crate::core::color::Color;
use crate::math::vec2::Vec2;

use super::Collider;

pub(super) struct Ellipse {
    pub center: Vec2,
    pub radius_x: f32,
    pub radius_y: f32,
    pub color: Color,
    pub segments: u32,
}

impl Ellipse {
    pub fn new(center: Vec2, radius_x: f32, radius_y: f32, color: Color) -> Self {
        Self {
            center,
            radius_x,
            radius_y,
            color,
            segments: 32,
        }
    }
}

impl Collider for Ellipse {
    fn contains_point(&self, p: Vec2) -> bool {
        let dx = p.x - self.center.x;
        let dy = p.y - self.center.y;

        (dx * dx) / (self.radius_x * self.radius_x) + (dy * dy) / (self.radius_y * self.radius_y)
            <= 1.0
    }

    fn intersects(&self, _other: &Self) -> bool {
        todo!();
    }
}
