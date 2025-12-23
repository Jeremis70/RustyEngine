use super::circle::Circle;
use super::ellipse::Ellipse;
use super::line::Line;
use super::polygon::Polygon;
use super::polyline::Polyline;
use super::rectangle::Rectangle;
use super::triangle::Triangle;
use crate::math::vec2::Vec2;

#[derive(Clone, Copy)]
pub enum ShapeRef<'a> {
    Circle(&'a Circle),
    Ellipse(&'a Ellipse),
    Line(&'a Line),
    Polygon(&'a Polygon),
    Polyline(&'a Polyline),
    Rectangle(&'a Rectangle),
    Triangle(&'a Triangle),
}

impl<'a> ShapeRef<'a> {
    pub fn outline(self) -> Vec<Vec2> {
        match self {
            ShapeRef::Circle(circle) => circle.world_outline(),
            ShapeRef::Ellipse(ellipse) => ellipse.world_outline(),
            ShapeRef::Line(line) => line.world_outline(),
            ShapeRef::Polygon(polygon) => polygon.world_outline(),
            ShapeRef::Polyline(polyline) => polyline.world_outline().unwrap_or_default(),
            ShapeRef::Rectangle(rectangle) => rectangle.world_outline(),
            ShapeRef::Triangle(triangle) => triangle.world_outline(),
        }
    }
}

pub fn shapes_intersect(a: ShapeRef<'_>, b: ShapeRef<'_>) -> bool {
    let outline_a = a.outline();
    let outline_b = b.outline();
    polygon_intersects_outline(&outline_a, &outline_b)
}

fn polygon_intersects_outline(a: &[Vec2], b: &[Vec2]) -> bool {
    if a.len() < 3 || b.len() < 3 {
        return false;
    }

    if !bounding_boxes_overlap(a, b) {
        return false;
    }

    if a.iter().any(|p| point_in_polygon(*p, b)) {
        return true;
    }

    if b.iter().any(|p| point_in_polygon(*p, a)) {
        return true;
    }

    for (a_start, a_end) in polygon_segments(a) {
        for (b_start, b_end) in polygon_segments(b) {
            if segments_intersect(a_start, a_end, b_start, b_end) {
                return true;
            }
        }
    }

    false
}

fn bounding_boxes_overlap(a: &[Vec2], b: &[Vec2]) -> bool {
    let (min_a, max_a) = compute_bounds(a);
    let (min_b, max_b) = compute_bounds(b);

    !(max_a.x < min_b.x || min_a.x > max_b.x || max_a.y < min_b.y || min_a.y > max_b.y)
}

fn compute_bounds(points: &[Vec2]) -> (Vec2, Vec2) {
    let mut min = Vec2::new(f32::INFINITY, f32::INFINITY);
    let mut max = Vec2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);

    for p in points {
        min.x = min.x.min(p.x);
        min.y = min.y.min(p.y);
        max.x = max.x.max(p.x);
        max.y = max.y.max(p.y);
    }

    (min, max)
}

fn polygon_segments(points: &[Vec2]) -> impl Iterator<Item = (Vec2, Vec2)> + '_ {
    points
        .iter()
        .enumerate()
        .map(move |(i, &start)| (start, points[(i + 1) % points.len()]))
}

fn point_in_polygon(point: Vec2, polygon: &[Vec2]) -> bool {
    let mut inside = false;
    let mut j = polygon.len() - 1;

    for (i, current) in polygon.iter().enumerate() {
        let prev = polygon[j];
        if point_on_segment(prev, *current, point) {
            return true;
        }

        let crosses = (current.y > point.y) != (prev.y > point.y);
        if crosses {
            let denom = prev.y - current.y;
            if denom.abs() <= f32::EPSILON {
                j = i;
                continue;
            }
            let x_intersect = (prev.x - current.x) * (point.y - current.y) / denom + current.x;
            if point.x < x_intersect {
                inside = !inside;
            }
        }

        j = i;
    }

    inside
}

fn point_on_segment(a: Vec2, b: Vec2, p: Vec2) -> bool {
    let ap = p - a;
    let ab = b - a;
    let cross = ap.x * ab.y - ap.y * ab.x;
    if cross.abs() > 1e-4 {
        return false;
    }

    let dot = ap.x * ab.x + ap.y * ab.y;
    let ab_len_sq = ab.x * ab.x + ab.y * ab.y;
    dot >= 0.0 && dot <= ab_len_sq
}

fn segments_intersect(a_start: Vec2, a_end: Vec2, b_start: Vec2, b_end: Vec2) -> bool {
    let o1 = orientation(a_start, a_end, b_start);
    let o2 = orientation(a_start, a_end, b_end);
    let o3 = orientation(b_start, b_end, a_start);
    let o4 = orientation(b_start, b_end, a_end);

    let crosses = (o1 * o2) < 0.0 && (o3 * o4) < 0.0;
    if crosses {
        return true;
    }

    point_on_segment(a_start, a_end, b_start)
        || point_on_segment(a_start, a_end, b_end)
        || point_on_segment(b_start, b_end, a_start)
        || point_on_segment(b_start, b_end, a_end)
}

fn orientation(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    cross(b - a, c - a)
}

fn cross(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}
