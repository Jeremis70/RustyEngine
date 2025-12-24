use crate::math::Transform;

/// Trait for entities that have a 2D transformation.
/// This allows generic systems to work with any transformable entity
/// without knowing the concrete type (Sprite, Circle, Rectangle, etc.)
///
/// # Examples
///
/// ```ignore
/// fn move_all<T: Transform2d>(entities: &mut [T], offset: Vec2) {
///     for entity in entities {
///         entity.transform_mut().translate(offset);
///     }
/// }
/// ```
pub trait Transform2d {
    /// Get an immutable reference to the transform
    fn transform(&self) -> &Transform;

    /// Get a mutable reference to the transform
    fn transform_mut(&mut self) -> &mut Transform;
}
