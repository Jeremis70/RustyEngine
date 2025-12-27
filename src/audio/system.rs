use std::path::Path;
use std::time::Duration;

use super::backend::{AudioBackend, LoadStrategy};
use super::error::AudioResult;
use super::sound::SoundId;
use super::sound_group::SoundGroup;

/// High-level audio system API
///
/// Provides convenient methods for loading, playing, and managing sounds.
/// This wraps the underlying `AudioBackend` trait implementation.
pub struct AudioSystem {
    backend: Box<dyn AudioBackend>,
}

impl AudioSystem {
    pub fn new(backend: Box<dyn AudioBackend>) -> Self {
        Self { backend }
    }

    pub(crate) fn load<P>(&mut self, path: P) -> AudioResult<SoundId>
    where
        P: AsRef<Path>,
    {
        self.load_with_strategy(path, LoadStrategy::Auto)
    }

    pub(crate) fn load_with_strategy<P>(
        &mut self,
        path: P,
        strategy: LoadStrategy,
    ) -> AudioResult<SoundId>
    where
        P: AsRef<Path>,
    {
        self.backend.load(path.as_ref(), strategy)
    }

    pub(crate) fn load_buffered<P>(&mut self, path: P) -> AudioResult<SoundId>
    where
        P: AsRef<Path>,
    {
        self.load_with_strategy(path, LoadStrategy::Buffered)
    }

    pub(crate) fn load_streaming<P>(&mut self, path: P) -> AudioResult<SoundId>
    where
        P: AsRef<Path>,
    {
        self.load_with_strategy(path, LoadStrategy::Streaming)
    }

    pub fn play(&mut self, sound: SoundId) -> AudioResult<()> {
        self.backend.play(sound)
    }

    pub fn stop(&mut self, sound: SoundId) -> AudioResult<()> {
        self.backend.stop(sound)
    }

    pub fn pause(&mut self, sound: SoundId) -> AudioResult<()> {
        self.backend.pause(sound)
    }

    pub fn resume(&mut self, sound: SoundId) -> AudioResult<()> {
        self.backend.resume(sound)
    }

    pub fn set_volume(&mut self, sound: SoundId, volume: f32) -> AudioResult<()> {
        self.backend.set_volume(sound, volume)
    }

    pub fn stop_all(&mut self) {
        self.backend.stop_all()
    }

    pub fn set_master_volume(&mut self, volume: f32) {
        self.backend.set_master_volume(volume)
    }

    pub fn is_playing(&self, sound: SoundId) -> bool {
        self.backend.is_playing(sound)
    }

    pub fn duration(&self, sound: SoundId) -> Option<Duration> {
        self.backend.duration(sound)
    }

    pub fn unload(&mut self, sound: SoundId) -> AudioResult<()> {
        self.backend.unload(sound)
    }

    pub fn unload_all(&mut self) {
        self.backend.unload_all()
    }

    /// Set the pan (left/right stereo positioning) for a sound
    ///
    /// # Arguments
    /// * `sound` - The sound to adjust
    /// * `pan` - Pan value from -1.0 (full left) to 1.0 (full right), 0.0 is center
    pub fn set_pan(&mut self, sound: SoundId, pan: f32) -> AudioResult<()> {
        self.backend.set_pan(sound, pan)
    }

    /// Set the pitch/playback speed for a sound
    ///
    /// # Arguments
    /// * `sound` - The sound to adjust
    /// * `pitch` - Pitch multiplier (0.5 = half speed, 2.0 = double speed)
    pub fn set_pitch(&mut self, sound: SoundId, pitch: f32) -> AudioResult<()> {
        self.backend.set_pitch(sound, pitch)
    }

    /// Assign a sound to a specific group
    pub fn set_group(&mut self, sound: SoundId, group: SoundGroup) -> AudioResult<()> {
        self.backend.set_group(sound, group)
    }

    /// Set the volume for all sounds in a specific group
    pub fn set_group_volume(&mut self, group: SoundGroup, volume: f32) -> AudioResult<()> {
        self.backend.set_group_volume(group, volume)
    }

    /// Get the current pan value for a sound
    pub fn get_pan(&self, sound: SoundId) -> Option<f32> {
        self.backend.get_pan(sound)
    }

    /// Get the current pitch value for a sound
    pub fn get_pitch(&self, sound: SoundId) -> Option<f32> {
        self.backend.get_pitch(sound)
    }

    /// Get the group a sound is assigned to
    pub fn get_group(&self, sound: SoundId) -> Option<SoundGroup> {
        self.backend.get_group(sound)
    }

    /// Get the volume for a specific group
    pub fn get_group_volume(&self, group: SoundGroup) -> Option<f32> {
        self.backend.get_group_volume(group)
    }
}
