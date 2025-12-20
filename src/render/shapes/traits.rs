use crate::core::render_context::RenderContext;
use crate::math::vec2::Vec2;

use super::{ShapeRef, shapes_intersect};

pub trait Collider {
    fn contains_point(&self, point: Vec2) -> bool;
    fn as_shape(&self) -> ShapeRef<'_>;

    fn intersects_shape(&self, other: ShapeRef<'_>) -> bool {
        shapes_intersect(self.as_shape(), other)
    }

    fn intersects(&self, other: &dyn Collider) -> bool {
        shapes_intersect(self.as_shape(), other.as_shape())
    }
}

pub trait Drawable {
    fn draw(&self, ctx: &mut RenderContext);
}
