use crate::math::color::Color;
use crate::render::context::RenderContext;
use crate::render::Vertex;
use crate::math::vec2::Vec2;

use super::{Collider, Drawable, ShapeRef, Transform2d};

pub struct Polygon {
    pub position: Vec2,
    pub local_points: Vec<Vec2>,
    pub color: Color,
    pub rotation: f32,
    pub scale: Vec2,
    pub origin: Vec2,
    pub size: Vec2,
}

impl Polygon {
    pub fn new(points: Vec<Vec2>, color: Color) -> Self {
        if points.is_empty() {
            return Self {
                position: Vec2::ZERO,
                local_points: Vec::new(),
                color,
                rotation: 0.0,
                scale: Vec2::new(1.0, 1.0),
                origin: Vec2::ZERO,
                size: Vec2::ZERO,
            };
        }

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for p in &points {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        let position = Vec2::new(min_x, min_y);
        let size = Vec2::new(max_x - min_x, max_y - min_y);
        let local_points = points.into_iter().map(|p| p - position).collect();

        Self {
            position,
            local_points,
            color,
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
            origin: Vec2::ZERO,
            size,
        }
    }

    fn transform_point(&self, local: Vec2) -> Vec2 {
        <Self as Transform2d>::transform_point(self, local, self.size)
    }

    fn world_points(&self) -> Vec<Vec2> {
        self.local_points
            .iter()
            .map(|p| self.transform_point(*p))
            .collect()
    }

    pub fn set_origin_keep_position(&mut self, origin: Vec2) {
        <Self as Transform2d>::set_origin_keep_position(self, origin, self.size);
    }

    pub fn set_origin_center_keep_position(&mut self) {
        <Self as Transform2d>::set_origin_center_keep_position(self, self.size);
    }

    pub fn world_outline(&self) -> Vec<Vec2> {
        self.world_points()
    }
}

impl Drawable for Polygon {
    fn draw(&self, ctx: &mut RenderContext) {
        let point_count = self.local_points.len();
        if point_count < 3 {
            return;
        }

        let color = self.color.to_linear_rgba();
        let world_points = self.world_points();
        let ndc_points: Vec<Vec2> = world_points.iter().map(|p| ctx.to_ndc(*p)).collect();

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
        let count = self.local_points.len();
        if count < 3 {
            return false;
        }

        if let Some(local_point) = <Self as Transform2d>::to_local(self, point, self.size) {
            let mut inside = false;
            let mut prev = self.local_points[count - 1];
            for &curr in &self.local_points {
                let edge = curr - prev;
                let to_point = local_point - prev;
                let cross = edge.x * to_point.y - edge.y * to_point.x;
                if cross.abs() <= f32::EPSILON {
                    let dot = to_point.x * edge.x + to_point.y * edge.y;
                    let edge_len_sq = edge.x * edge.x + edge.y * edge.y;
                    if dot >= 0.0 && dot <= edge_len_sq {
                        return true;
                    }
                }

                let intersects = ((curr.y > local_point.y) != (prev.y > local_point.y))
                    && (prev.y - curr.y).abs() > f32::EPSILON
                    && {
                        let x_int = prev.x
                            + (local_point.y - prev.y) * (curr.x - prev.x) / (curr.y - prev.y);
                        local_point.x <= x_int
                    };

                if intersects {
                    inside = !inside;
                }

                prev = curr;
            }

            inside
        } else {
            false
        }
    }

    fn as_shape(&self) -> ShapeRef<'_> {
        ShapeRef::Polygon(self)
    }
}

impl Transform2d for Polygon {
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
