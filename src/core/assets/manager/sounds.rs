use std::path::Path;

use super::super::sound_tracking::{SoundAsset, SoundKey};
use super::AssetManager;
use crate::audio::{AudioError, AudioResult, AudioSystem, LoadStrategy, SoundId};

impl AssetManager {
    /// Load a sound via the engine audio system.
    ///
    /// This is a thin orchestration helper and does not store or own audio state.
    pub fn load_sound<P>(&mut self, audio: &mut AudioSystem, path: P) -> AudioResult<SoundId>
    where
        P: AsRef<Path>,
    {
        self.load_sound_with_strategy(audio, path, LoadStrategy::Auto)
    }

    /// Load a sound via the engine audio system, forcing a buffered strategy.
    pub fn load_sound_buffered<P>(
        &mut self,
        audio: &mut AudioSystem,
        path: P,
    ) -> AudioResult<SoundId>
    where
        P: AsRef<Path>,
    {
        self.load_sound_with_strategy(audio, path, LoadStrategy::Buffered)
    }

    /// Load a sound via the engine audio system, forcing a streaming strategy.
    pub fn load_sound_streaming<P>(
        &mut self,
        audio: &mut AudioSystem,
        path: P,
    ) -> AudioResult<SoundId>
    where
        P: AsRef<Path>,
    {
        self.load_sound_with_strategy(audio, path, LoadStrategy::Streaming)
    }

    fn load_sound_with_strategy<P>(
        &mut self,
        audio: &mut AudioSystem,
        path: P,
        strategy: LoadStrategy,
    ) -> AudioResult<SoundId>
    where
        P: AsRef<Path>,
    {
        let info = self.compute_path_info(path.as_ref());
        if let Err(err) = self.enforce_path_policy(path.as_ref(), &info) {
            return Err(AudioError::Backend(err.to_string()));
        }
        let key_path = info.key.clone();
        let path_buf = info.io_path.clone();
        let key = SoundKey {
            path: key_path,
            strategy,
        };

        if let Some(existing) = self.sounds.get_existing_id(&key) {
            return Ok(existing);
        }

        let estimated_bytes = std::fs::metadata(&path_buf)
            .map(|m| m.len() as usize)
            .unwrap_or(0);

        if let Err(err) = self.ensure_capacity_for(estimated_bytes) {
            return Err(AudioError::Backend(err.to_string()));
        }

        let sound_id = audio.load_with_strategy(&path_buf, strategy)?;

        self.sounds
            .insert_keyed(sound_id, key, SoundAsset { estimated_bytes });
        self.current_memory_bytes += estimated_bytes;
        Ok(sound_id)
    }

    /// Check if a sound with the given ID exists.
    pub fn sound_exists(&self, id: SoundId) -> bool {
        self.sounds.contains_id(id)
    }

    /// Get the total number of loaded sounds tracked by the asset manager.
    pub fn sound_count(&self) -> usize {
        self.sounds.len()
    }

    /// Unload and remove a sound from the audio system and asset tracking.
    pub fn unload_sound(&mut self, audio: &mut AudioSystem, id: SoundId) -> AudioResult<bool> {
        let Some(entry) = self.sounds.remove(id) else {
            return Ok(false);
        };

        self.current_memory_bytes = self
            .current_memory_bytes
            .saturating_sub(entry.asset.estimated_bytes);
        audio.unload(id)?;
        Ok(true)
    }

    /// Unload all sounds tracked by the asset manager.
    pub fn unload_all_sounds(&mut self, audio: &mut AudioSystem) {
        let ids: Vec<SoundId> = self.sounds.by_id.keys().copied().collect();
        for id in ids {
            let _ = self.unload_sound(audio, id);
        }
    }
}
