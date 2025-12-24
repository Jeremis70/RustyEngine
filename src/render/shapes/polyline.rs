use crate::math::Transform;
use crate::math::color::Color;
use crate::math::vec2::Vec2;
use crate::render::Vertex;
use crate::render::context::RenderContext;

use super::{Collider, Drawable, ShapeRef, Transform2d};

pub struct Polyline {
    pub transform: Transform,
    pub local_points: Vec<Vec2>,
    pub color: Color,
    pub thickness: f32,
    pub size: Vec2,
}

impl Polyline {
    pub fn new(points: Vec<Vec2>, color: Color, thickness: f32) -> Self {
        if points.is_empty() {
            return Self {
                transform: Transform::new(),
                local_points: Vec::new(),
                color,
                thickness,
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

        let half_t = thickness * 0.5;
        let position = Vec2::new(min_x - half_t, min_y - half_t);
        let size = Vec2::new((max_x - min_x) + thickness, (max_y - min_y) + thickness);
        let local_points = points.into_iter().map(|p| p - position).collect();

        Self {
            transform: Transform::at(position),
            local_points,
            color,
            thickness,
            size,
        }
    }

    pub fn set_origin_keep_position(&mut self, origin: Vec2) {
        self.transform.set_origin_keep_position(origin, self.size);
    }

    pub fn set_origin_center_keep_position(&mut self) {
        self.transform.set_origin_center_keep_position(self.size);
    }

    fn offset_geometry(&self) -> Option<(Vec<Vec2>, Vec<Vec2>, Vec<Vec2>)> {
        if self.local_points.len() < 2 {
            return None;
        }

        let mut points: Vec<Vec2> = Vec::with_capacity(self.local_points.len());
        if let Some(first) = self.local_points.first() {
            points.push(*first);
            for p in self.local_points.iter().skip(1) {
                if let Some(last) = points.last() {
                    let delta = *p - *last;
                    if (delta * delta) > 1e-6 {
                        points.push(*p);
                    }
                }
            }
        }

        if points.len() < 2 {
            return None;
        }

        let mut directions: Vec<Vec2> = Vec::with_capacity(points.len() - 1);
        for segment in points.windows(2) {
            let dir = segment[1] - segment[0];
            let length = dir.length();
            if length <= f32::EPSILON {
                directions.push(Vec2::ZERO);
            } else {
                directions.push(dir / length);
            }
        }

        let normals: Vec<Vec2> = directions
            .iter()
            .map(|dir| {
                if ((*dir) * (*dir)) <= 1e-6 {
                    Vec2::ZERO
                } else {
                    Vec2::new(-dir.y, dir.x)
                }
            })
            .collect();

        let half_thickness = self.thickness * 0.5;
        let mut left_offsets = vec![Vec2::ZERO; points.len()];
        let mut right_offsets = vec![Vec2::ZERO; points.len()];

        let compute_miter = |n1: Vec2, n2: Vec2| -> Vec2 {
            let n1_len_sq = n1 * n1;
            let n2_len_sq = n2 * n2;

            match (n1_len_sq > 1e-6, n2_len_sq > 1e-6) {
                (false, false) => Vec2::ZERO,
                (true, false) => n1 * half_thickness,
                (false, true) => n2 * half_thickness,
                (true, true) => {
                    let sum = n1 + n2;
                    let sum_len_sq = sum * sum;
                    if sum_len_sq <= 1e-6 {
                        return n2 * half_thickness;
                    }

                    let miter = sum / sum_len_sq.sqrt();
                    let denom = miter * n2;
                    if denom.abs() <= 1e-6 {
                        return n2 * half_thickness;
                    }

                    let offset = miter * (half_thickness / denom);
                    if offset.length() > half_thickness * 4.0 {
                        n2 * half_thickness
                    } else {
                        offset
                    }
                }
            }
        };

        for i in 0..points.len() {
            if i == 0 {
                let normal = normals[0];
                left_offsets[i] = normal * half_thickness;
                right_offsets[i] = -normal * half_thickness;
            } else if i == points.len() - 1 {
                let normal = normals[normals.len() - 1];
                left_offsets[i] = normal * half_thickness;
                right_offsets[i] = -normal * half_thickness;
            } else {
                let prev_normal = normals[i - 1];
                let next_normal = normals[i];
                left_offsets[i] = compute_miter(prev_normal, next_normal);
                right_offsets[i] = compute_miter(-prev_normal, -next_normal);
            }
        }

        Some((points, left_offsets, right_offsets))
    }

    pub fn world_outline(&self) -> Option<Vec<Vec2>> {
        self.offset_geometry().map(|(points, left, right)| {
            let mut outline = Vec::with_capacity(points.len() * 2);

            for (p, offset) in points.iter().zip(&left) {
                outline.push(self.transform.transform_point(*p + *offset, self.size));
            }

            for (p, offset) in points.iter().zip(&right).rev() {
                outline.push(self.transform.transform_point(*p + *offset, self.size));
            }

            outline
        })
    }
}

impl Drawable for Polyline {
    fn draw(&self, ctx: &mut RenderContext) {
        let Some((points, left_offsets, right_offsets)) = self.offset_geometry() else {
            return;
        };

        let color = self.color.to_rgba();
        let mut vertices: Vec<Vertex> = Vec::with_capacity((points.len() - 1) * 6);

        for i in 0..points.len() - 1 {
            let p0 = points[i];
            let p1 = points[i + 1];

            let v0 = self
                .transform
                .transform_point(p0 + left_offsets[i], self.size);
            let v1 = self
                .transform
                .transform_point(p1 + left_offsets[i + 1], self.size);
            let v2 = self
                .transform
                .transform_point(p1 + right_offsets[i + 1], self.size);
            let v3 = self
                .transform
                .transform_point(p0 + right_offsets[i], self.size);

            vertices.push(Vertex {
                pos: ctx.to_ndc(v0).to_array(),
                color,
            });
            vertices.push(Vertex {
                pos: ctx.to_ndc(v1).to_array(),
                color,
            });
            vertices.push(Vertex {
                pos: ctx.to_ndc(v2).to_array(),
                color,
            });
            vertices.push(Vertex {
                pos: ctx.to_ndc(v2).to_array(),
                color,
            });
            vertices.push(Vertex {
                pos: ctx.to_ndc(v3).to_array(),
                color,
            });
            vertices.push(Vertex {
                pos: ctx.to_ndc(v0).to_array(),
                color,
            });
        }

        ctx.extend(&vertices);
    }
}

impl Collider for Polyline {
    fn contains_point(&self, point: Vec2) -> bool {
        if self.local_points.len() < 2 {
            return false;
        }

        if let Some(local_point) = self.transform.to_local(point, self.size) {
            let radius = self.thickness * 0.5;
            let radius_sq = radius * radius;

            for segment in self.local_points.windows(2) {
                let a = segment[0];
                let b = segment[1];
                let ab = b - a;
                let len_sq = ab * ab;

                let distance_sq = if len_sq <= f32::EPSILON {
                    let delta = local_point - a;
                    delta * delta
                } else {
                    let t: f32 = ((local_point - a) * ab) / len_sq;
                    let t_clamped = t.clamp(0.0, 1.0);
                    let closest = a + ab * t_clamped;
                    let delta = local_point - closest;
                    delta * delta
                };

                if distance_sq <= radius_sq {
                    return true;
                }
            }
        }

        false
    }

    fn as_shape(&self) -> ShapeRef<'_> {
        ShapeRef::Polyline(self)
    }
}

impl Transform2d for Polyline {
    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}
