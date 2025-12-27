use std::path::Path;
use std::time::Duration;

use super::error::AudioResult;
use super::sound::SoundId;
use super::sound_group::SoundGroup;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum LoadStrategy {
    Auto,
    Buffered,
    Streaming,
}

/// Audio backend trait for abstraction over different audio implementations
///
/// This trait defines the interface for audio playback, allowing implementations
/// such as Rodio, FMOD, or Wwise to be swapped transparently.
pub trait AudioBackend {
    fn load(&mut self, path: &Path, strategy: LoadStrategy) -> AudioResult<SoundId>;
    fn play(&mut self, sound: SoundId) -> AudioResult<()>;
    fn stop(&mut self, sound: SoundId) -> AudioResult<()>;
    fn pause(&mut self, sound: SoundId) -> AudioResult<()>;
    fn resume(&mut self, sound: SoundId) -> AudioResult<()>;
    fn set_volume(&mut self, sound: SoundId, volume: f32) -> AudioResult<()>;
    fn stop_all(&mut self);
    fn set_master_volume(&mut self, volume: f32);
    fn is_playing(&self, sound: SoundId) -> bool;
    fn duration(&self, sound: SoundId) -> Option<Duration>;
    fn unload(&mut self, sound: SoundId) -> AudioResult<()>;
    fn unload_all(&mut self);

    /// Set the pan (left/right stereo positioning) for a sound
    ///
    /// # Arguments
    /// * `sound` - The sound to adjust
    /// * `pan` - Pan value from -1.0 (full left) to 1.0 (full right), 0.0 is center
    ///
    /// # Returns
    /// `Ok(())` if successful, `AudioError` otherwise
    fn set_pan(&mut self, sound: SoundId, pan: f32) -> AudioResult<()> {
        let _ = (sound, pan);
        // Default no-op implementation for backends that don't support panning
        Ok(())
    }

    /// Set the pitch/playback speed for a sound
    ///
    /// # Arguments
    /// * `sound` - The sound to adjust
    /// * `pitch` - Pitch multiplier (0.5 = half speed, 2.0 = double speed)
    ///
    /// # Returns
    /// `Ok(())` if successful, `AudioError` otherwise
    fn set_pitch(&mut self, sound: SoundId, pitch: f32) -> AudioResult<()> {
        let _ = (sound, pitch);
        // Default no-op implementation for backends that don't support pitch shifting
        Ok(())
    }

    /// Assign a sound to a specific group
    ///
    /// # Arguments
    /// * `sound` - The sound to assign
    /// * `group` - The group to assign to
    ///
    /// # Returns
    /// `Ok(())` if successful, `AudioError` otherwise
    fn set_group(&mut self, sound: SoundId, group: SoundGroup) -> AudioResult<()> {
        let _ = (sound, group);
        // Default no-op implementation
        Ok(())
    }

    /// Set the volume for all sounds in a specific group
    ///
    /// # Arguments
    /// * `group` - The group to adjust
    /// * `volume` - Volume level from 0.0 (silent) to 1.0 (full)
    ///
    /// # Returns
    /// `Ok(())` if successful, `AudioError` otherwise
    fn set_group_volume(&mut self, group: SoundGroup, volume: f32) -> AudioResult<()> {
        let _ = (group, volume);
        // Default no-op implementation
        Ok(())
    }

    /// Get the current pan value for a sound
    fn get_pan(&self, sound: SoundId) -> Option<f32> {
        let _ = sound;
        None
    }

    /// Get the current pitch value for a sound
    fn get_pitch(&self, sound: SoundId) -> Option<f32> {
        let _ = sound;
        None
    }

    /// Get the group a sound is assigned to
    fn get_group(&self, sound: SoundId) -> Option<SoundGroup> {
        let _ = sound;
        None
    }

    /// Get the volume for a specific group
    fn get_group_volume(&self, group: SoundGroup) -> Option<f32> {
        let _ = group;
        None
    }
}
