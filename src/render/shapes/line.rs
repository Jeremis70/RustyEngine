use crate::core::color::Color;
use crate::math::vec2::Vec2;

use super::Collider;

pub(super) struct Line {
    pub start: Vec2,
    pub end: Vec2,
    pub color: Color,
    pub thickness: f32,
}

impl Line {
    pub fn new(start: Vec2, end: Vec2, color: Color, thickness: f32) -> Self {
        Self {
            start,
            end,
            color,
            thickness,
        }
    }
}

impl Collider for Line {
    fn contains_point(&self, _point: Vec2) -> bool {
        todo!();
    }

    fn intersects(&self, _other: &Self) -> bool {
        todo!();
    }
}
