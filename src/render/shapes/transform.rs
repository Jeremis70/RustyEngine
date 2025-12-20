use crate::math::vec2::Vec2;

pub trait Transform2d {
    fn position(&self) -> Vec2;
    fn position_mut(&mut self) -> &mut Vec2;
    fn rotation(&self) -> f32;
    fn rotation_mut(&mut self) -> &mut f32;
    fn scale(&self) -> Vec2;
    fn scale_mut(&mut self) -> &mut Vec2;
    fn origin(&self) -> Vec2;
    fn origin_mut(&mut self) -> &mut Vec2;

    fn set_position(&mut self, position: Vec2) {
        *self.position_mut() = position;
    }

    fn translate(&mut self, delta: Vec2) {
        let next = self.position() + delta;
        *self.position_mut() = next;
    }

    fn set_rotation(&mut self, radians: f32) {
        *self.rotation_mut() = radians;
    }

    fn rotate(&mut self, delta: f32) {
        *self.rotation_mut() = self.rotation() + delta;
    }

    fn set_scale(&mut self, scale: Vec2) {
        *self.scale_mut() = scale;
    }

    fn scale_uniform(&mut self, factor: f32) {
        *self.scale_mut() = self.scale() * factor;
    }

    fn set_origin(&mut self, origin: Vec2) {
        *self.origin_mut() = origin;
    }

    fn set_origin_center(&mut self) {
        self.set_origin(Vec2::new(0.5, 0.5));
    }

    fn origin_pivot(&self, size: Vec2) -> Vec2 {
        self.scaled_size(size).hadamard(self.origin())
    }

    fn scaled_size(&self, size: Vec2) -> Vec2 {
        size.hadamard(self.scale())
    }

    fn transform_point(&self, local: Vec2, size: Vec2) -> Vec2 {
        let scaled = local.hadamard(self.scale());
        let pivot = self.origin_pivot(size);
        (scaled - pivot).rotated(self.rotation()) + pivot + self.position()
    }

    fn to_local(&self, point: Vec2, size: Vec2) -> Option<Vec2> {
        let scaled_size = self.scaled_size(size);
        if scaled_size.x.abs() <= f32::EPSILON || scaled_size.y.abs() <= f32::EPSILON {
            return None;
        }

        let pivot = self.origin_pivot(size);
        let shifted = point - self.position() - pivot;
        let inv_rotated = shifted.rotated(-self.rotation());
        let scaled = inv_rotated + pivot;

        Some(Vec2::new(
            scaled.x / self.scale().x,
            scaled.y / self.scale().y,
        ))
    }

    fn set_origin_keep_position(&mut self, origin: Vec2, size: Vec2) {
        let reference = self.transform_point(Vec2::ZERO, size);
        self.set_origin(origin);
        let new_reference = self.transform_point(Vec2::ZERO, size);
        let delta = reference - new_reference;
        self.translate(delta);
    }

    fn set_origin_center_keep_position(&mut self, size: Vec2) {
        self.set_origin_keep_position(Vec2::new(0.5, 0.5), size);
    }
}
