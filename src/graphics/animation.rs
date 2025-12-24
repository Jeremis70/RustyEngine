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

impl Animation {
    /// Create a new animation from a list of image IDs with uniform frame duration.
    pub fn from_frames(image_ids: &[ImageId], frame_duration: Duration, looped: bool) -> Self {
        let frames = image_ids
            .iter()
            .map(|&id| AnimationFrame {
                image_id: id,
                duration: frame_duration,
            })
            .collect();

        Self { frames, looped }
    }

    /// Create a looping animation with uniform frame duration (convenience method).
    pub fn looping(image_ids: &[ImageId], frame_duration: Duration) -> Self {
        Self::from_frames(image_ids, frame_duration, true)
    }

    /// Create a one-shot animation with uniform frame duration (convenience method).
    pub fn once(image_ids: &[ImageId], frame_duration: Duration) -> Self {
        Self::from_frames(image_ids, frame_duration, false)
    }

    /// Set the total duration of the animation, distributing time evenly across all frames.
    /// Returns the updated animation for method chaining.
    pub fn with_total_duration(mut self, total_duration: Duration) -> Self {
        if !self.frames.is_empty() {
            let frame_duration = total_duration / self.frames.len() as u32;
            for frame in &mut self.frames {
                frame.duration = frame_duration;
            }
        }
        self
    }

    /// Set the duration for a specific frame by index.
    /// Panics if index is out of bounds.
    pub fn set_frame_duration(&mut self, frame_index: usize, duration: Duration) {
        self.frames[frame_index].duration = duration;
    }

    /// Set the duration for a specific frame by index (builder pattern).
    /// Panics if index is out of bounds.
    pub fn with_frame_duration(mut self, frame_index: usize, duration: Duration) -> Self {
        self.frames[frame_index].duration = duration;
        self
    }

    /// Set custom durations for all frames.
    /// Panics if the length doesn't match the number of frames.
    pub fn with_frame_durations(mut self, durations: &[Duration]) -> Self {
        assert_eq!(
            self.frames.len(),
            durations.len(),
            "Number of durations must match number of frames"
        );
        for (frame, &duration) in self.frames.iter_mut().zip(durations.iter()) {
            frame.duration = duration;
        }
        self
    }

    /// Get the total duration of the animation (one playthrough).
    pub fn total_duration(&self) -> Duration {
        self.frames.iter().map(|f| f.duration).sum()
    }
}
