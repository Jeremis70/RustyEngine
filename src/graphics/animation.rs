use crate::core::assets::ImageId;
use std::time::Duration;

#[derive(Clone)]
pub struct AnimationFrame {
    pub image_id: ImageId,
    pub duration: Duration,
}

#[derive(Clone)]
pub struct Animation {
    pub frames: Vec<AnimationFrame>,
    pub looped: bool,
}
