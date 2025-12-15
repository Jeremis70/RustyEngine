mod circle;
mod ellipse;
mod line;
mod polygon;
mod polyline;
mod rectangle;
mod triangle;

pub use circle::Circle;
pub use polygon::Polygon;
pub use rectangle::Rectangle;
pub use triangle::Triangle;

use crate::core::render_context::RenderContext;
use crate::math::vec2::Vec2;

pub trait Collider {
    fn contains_point(&self, point: Vec2) -> bool;
    fn intersects(&self, other: &Self) -> bool;
}

pub trait Drawable {
    fn draw(&self, ctx: &mut RenderContext);
}
