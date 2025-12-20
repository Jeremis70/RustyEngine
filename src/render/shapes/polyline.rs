use crate::core::color::Color;
use crate::core::render_context::RenderContext;
use crate::core::vertex::Vertex;
use crate::math::vec2::Vec2;

use super::{Collider, Drawable, ShapeRef, Transform2d};

pub struct Polyline {
    pub position: Vec2,
    pub local_points: Vec<Vec2>,
    pub color: Color,
    pub thickness: f32,
    pub rotation: f32,
    pub scale: Vec2,
    pub origin: Vec2,
    pub size: Vec2,
}

impl Polyline {
    pub fn new(points: Vec<Vec2>, color: Color, thickness: f32) -> Self {
        if points.is_empty() {
            return Self {
                position: Vec2::ZERO,
                local_points: Vec::new(),
                color,
                thickness,
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

        let half_t = thickness * 0.5;
        let position = Vec2::new(min_x - half_t, min_y - half_t);
        let size = Vec2::new((max_x - min_x) + thickness, (max_y - min_y) + thickness);
        let local_points = points.into_iter().map(|p| p - position).collect();

        Self {
            position,
            local_points,
            color,
            thickness,
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
            origin: Vec2::ZERO,
            size,
        }
    }

    pub fn set_origin_keep_position(&mut self, origin: Vec2) {
        <Self as Transform2d>::set_origin_keep_position(self, origin, self.size);
    }

    pub fn set_origin_center_keep_position(&mut self) {
        <Self as Transform2d>::set_origin_center_keep_position(self, self.size);
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
                outline.push(<Self as Transform2d>::transform_point(
                    self,
                    *p + *offset,
                    self.size,
                ));
            }

            for (p, offset) in points.iter().zip(&right).rev() {
                outline.push(<Self as Transform2d>::transform_point(
                    self,
                    *p + *offset,
                    self.size,
                ));
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

            let v0 = <Self as Transform2d>::transform_point(self, p0 + left_offsets[i], self.size);
            let v1 =
                <Self as Transform2d>::transform_point(self, p1 + left_offsets[i + 1], self.size);
            let v2 =
                <Self as Transform2d>::transform_point(self, p1 + right_offsets[i + 1], self.size);
            let v3 = <Self as Transform2d>::transform_point(self, p0 + right_offsets[i], self.size);

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

        if let Some(local_point) = <Self as Transform2d>::to_local(self, point, self.size) {
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
                    let t = ((local_point - a) * ab) / len_sq;
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
