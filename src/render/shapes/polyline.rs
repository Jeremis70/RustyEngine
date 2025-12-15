use crate::core::color::Color;
use crate::math::vec2::Vec2;

use super::Collider;

pub(super) struct Polyline {
    pub points: Vec<Vec2>,
    pub color: Color,
    pub thickness: f32,
}

impl Polyline {
    pub fn new(points: Vec<Vec2>, color: Color, thickness: f32) -> Self {
        Self {
            points,
            color,
            thickness,
        }
    }
}

impl Collider for Polyline {
    fn contains_point(&self, _point: Vec2) -> bool {
        todo!();
    }

    fn intersects(&self, _other: &Self) -> bool {
        todo!();
    }
}
