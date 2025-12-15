use crate::core::color::Color;
use crate::core::render_context::RenderContext;
use crate::core::vertex::Vertex;
use crate::math::vec2::Vec2;
use crate::render::Drawable;

use super::Collider;

pub struct Polygon {
    pub points: Vec<Vec2>,
    pub color: Color,
}

impl Polygon {
    pub fn new(points: Vec<Vec2>, color: Color) -> Self {
        Self { points, color }
    }
}

impl Drawable for Polygon {
    fn draw(&self, ctx: &mut RenderContext) {
        let point_count = self.points.len();
        if point_count < 3 {
            return;
        }

        let color = self.color.to_rgba();
        let ndc_points: Vec<Vec2> = self.points.iter().map(|p| ctx.to_ndc(*p)).collect();

        // Fan triangulation around the first point to cover the polygon area.
        let mut vertices: Vec<Vertex> = Vec::with_capacity((point_count - 2) * 3);
        let anchor = ndc_points[0];
        for i in 1..(point_count - 1) {
            let v1 = ndc_points[i];
            let v2 = ndc_points[i + 1];

            vertices.push(Vertex {
                pos: anchor.to_array(),
                color,
            });
            vertices.push(Vertex {
                pos: v1.to_array(),
                color,
            });
            vertices.push(Vertex {
                pos: v2.to_array(),
                color,
            });
        }

        ctx.extend(&vertices);
    }
}

impl Collider for Polygon {
    fn contains_point(&self, point: Vec2) -> bool {
        let count = self.points.len();
        if count < 3 {
            return false;
        }

        let mut inside = false;
        let mut prev = self.points[count - 1];
        for &curr in &self.points {
            let edge = curr - prev;
            let to_point = point - prev;
            let cross = edge.x * to_point.y - edge.y * to_point.x;
            if cross.abs() <= f32::EPSILON {
                let dot = to_point.x * edge.x + to_point.y * edge.y;
                let edge_len_sq = edge.x * edge.x + edge.y * edge.y;
                if dot >= 0.0 && dot <= edge_len_sq {
                    return true;
                }
            }

            let intersects = ((curr.y > point.y) != (prev.y > point.y))
                && (prev.y - curr.y).abs() > f32::EPSILON
                && {
                    let x_int = prev.x + (point.y - prev.y) * (curr.x - prev.x) / (curr.y - prev.y);
                    point.x <= x_int
                };

            if intersects {
                inside = !inside;
            }

            prev = curr;
        }

        inside
    }

    fn intersects(&self, _other: &Self) -> bool {
        todo!();
    }
}
