use crate::math::vec2::Vec2;

/// A 2D transformation component containing position, rotation, scale, and origin.
/// This is the core transform component used by all drawable entities in RustyEngine.
#[derive(Clone, Debug, PartialEq)]
pub struct Transform {
    /// Position in world space (pixels)
    pub position: Vec2,
    /// Rotation in radians
    pub rotation: f32,
    /// Non-uniform scale factors (1.0 = normal size)
    pub scale: Vec2,
    /// Normalized pivot point (0..1, where 0.5 = center)
    pub origin: Vec2,
}

impl Transform {
    /// Create a new transform with default values:
    /// - position: (0, 0)
    /// - rotation: 0
    /// - scale: (1, 1)
    /// - origin: (0.5, 0.5) - centered
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
            origin: Vec2::new(0.5, 0.5),
        }
    }

    /// Create a transform at a specific position
    pub fn at(position: Vec2) -> Self {
        Self {
            position,
            ..Self::new()
        }
    }

    /// Builder: Set position
    pub fn with_position(mut self, position: Vec2) -> Self {
        self.position = position;
        self
    }

    /// Builder: Set rotation
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    /// Builder: Set scale
    pub fn with_scale(mut self, scale: Vec2) -> Self {
        self.scale = scale;
        self
    }

    /// Builder: Set uniform scale
    pub fn with_uniform_scale(mut self, scale: f32) -> Self {
        self.scale = Vec2::new(scale, scale);
        self
    }

    /// Builder: Set origin
    pub fn with_origin(mut self, origin: Vec2) -> Self {
        self.origin = origin;
        self
    }

    /// Translate by a delta vector
    pub fn translate(&mut self, delta: Vec2) {
        self.position = self.position + delta;
    }

    /// Rotate by a delta angle (radians)
    pub fn rotate(&mut self, delta: f32) {
        self.rotation += delta;
    }

    /// Scale uniformly by a factor
    pub fn scale_uniform(&mut self, factor: f32) {
        self.scale = self.scale.scale(factor);
    }

    /// Set origin to center (0.5, 0.5)
    pub fn set_origin_center(&mut self) {
        self.origin = Vec2::new(0.5, 0.5);
    }

    /// Get the pivot point in pixels for a given size
    pub fn origin_pivot(&self, size: Vec2) -> Vec2 {
        self.scaled_size(size).hadamard(self.origin)
    }

    /// Get the size after applying scale
    pub fn scaled_size(&self, size: Vec2) -> Vec2 {
        size.hadamard(self.scale)
    }

    /// Transform a local point to world space
    ///
    /// # Arguments
    /// * `local` - Point in local space (pre-scale, pre-rotation)
    /// * `size` - The size of the entity being transformed
    pub fn transform_point(&self, local: Vec2, size: Vec2) -> Vec2 {
        let scaled = local.hadamard(self.scale);
        let pivot = self.origin_pivot(size);
        (scaled - pivot).rotated(self.rotation) + pivot + self.position
    }

    /// Transform a world point back to local space
    /// Returns None if the scale is near zero (degenerate transform)
    pub fn to_local(&self, point: Vec2, size: Vec2) -> Option<Vec2> {
        let scaled_size = self.scaled_size(size);
        if scaled_size.x.abs() <= f32::EPSILON || scaled_size.y.abs() <= f32::EPSILON {
            return None;
        }

        let pivot = self.origin_pivot(size);
        let shifted = point - self.position - pivot;
        let inv_rotated = shifted.rotated(-self.rotation);
        let scaled = inv_rotated + pivot;

        Some(Vec2::new(scaled.x / self.scale.x, scaled.y / self.scale.y))
    }

    /// Set origin while keeping the visual position constant
    /// This adjusts the position to compensate for the origin change
    pub fn set_origin_keep_position(&mut self, origin: Vec2, size: Vec2) {
        let reference = self.transform_point(Vec2::ZERO, size);
        self.origin = origin;
        let new_reference = self.transform_point(Vec2::ZERO, size);
        let delta = reference - new_reference;
        self.translate(delta);
    }

    /// Set origin to center while keeping the visual position constant
    pub fn set_origin_center_keep_position(&mut self, size: Vec2) {
        self.set_origin_keep_position(Vec2::new(0.5, 0.5), size);
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

// Support for trait-based access (compatibility with existing code)
impl Transform {
    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn position_mut(&mut self) -> &mut Vec2 {
        &mut self.position
    }

    pub fn rotation(&self) -> f32 {
        self.rotation
    }

    pub fn rotation_mut(&mut self) -> &mut f32 {
        &mut self.rotation
    }

    pub fn scale(&self) -> Vec2 {
        self.scale
    }

    pub fn scale_mut(&mut self) -> &mut Vec2 {
        &mut self.scale
    }

    pub fn origin(&self) -> Vec2 {
        self.origin
    }

    pub fn origin_mut(&mut self) -> &mut Vec2 {
        &mut self.origin
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    pub fn set_rotation(&mut self, radians: f32) {
        self.rotation = radians;
    }

    pub fn set_scale(&mut self, scale: Vec2) {
        self.scale = scale;
    }

    pub fn set_origin(&mut self, origin: Vec2) {
        self.origin = origin;
    }
}
