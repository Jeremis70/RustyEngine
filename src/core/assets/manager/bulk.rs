use std::path::Path;

use super::super::error::AssetResult;
use super::super::font::FontId;
use super::super::image::ImageId;
use super::AssetManager;
use crate::audio::AudioSystem;

impl AssetManager {
    pub fn load_fonts<P: AsRef<Path>>(
        &mut self,
        paths: &[P],
        font_size: f32,
    ) -> AssetResult<Vec<FontId>> {
        let mut ids = Vec::with_capacity(paths.len());
        for path in paths {
            let id = self.load_font(path, font_size)?;
            ids.push(id);
        }
        Ok(ids)
    }

    /// Load multiple images from disk.
    pub fn load_images<P: AsRef<Path>>(&mut self, paths: &[P]) -> AssetResult<Vec<ImageId>> {
        let mut ids = Vec::with_capacity(paths.len());
        for path in paths {
            let id = self.load_image(path)?;
            ids.push(id);
        }
        Ok(ids)
    }

    /// Unload multiple images from memory.
    pub fn unload_images(&mut self, ids: &[ImageId]) {
        for &id in ids {
            self.unload_image(id);
        }
    }

    /// Unload multiple fonts from memory.
    pub fn unload_fonts(&mut self, ids: &[FontId]) {
        for &id in ids {
            self.unload_font(id);
        }
    }

    /// Unload all assets from memory.
    ///
    /// Sounds require access to the audio system for unloading.
    pub fn unload_all(&mut self, audio: &mut AudioSystem) {
        self.unload_all_sounds(audio);
        self.unload_all_images();
        self.unload_all_fonts();
        self.current_memory_bytes = 0;
        log::debug!(
            "Unloaded all assets, memory now: {}",
            self.current_memory_bytes
        );
    }
}
