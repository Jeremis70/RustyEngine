use super::super::font::FontAsset;
use super::super::font::FontId;
use super::super::image::ImageAsset;
use super::super::image::ImageId;
use super::AssetManager;

impl AssetManager {
    /// Get current memory usage in bytes.
    pub fn memory_usage(&self) -> usize {
        self.current_memory_bytes
    }

    /// Get memory usage for images only (sum of raw pixel buffers).
    pub fn images_memory_usage_bytes(&self) -> usize {
        self.images
            .by_id
            .values()
            .map(|entry| entry.asset.data.len())
            .sum()
    }

    /// Get memory usage for fonts only (sum of raw font buffers).
    pub fn fonts_memory_usage_bytes(&self) -> usize {
        self.fonts
            .by_id
            .values()
            .map(|entry| entry.asset.data.len())
            .sum()
    }

    /// Get memory usage for sounds (best-effort estimate; currently based on file size).
    pub fn sounds_memory_usage_bytes(&self) -> usize {
        self.sounds
            .by_id
            .values()
            .map(|entry| entry.asset.estimated_bytes)
            .sum()
    }

    /// Get memory usage for a specific image.
    pub fn image_memory_usage_bytes(&self, id: ImageId) -> Option<usize> {
        self.images
            .by_id
            .get(&id)
            .map(|entry| entry.asset.data.len())
    }

    /// Get memory usage for a specific font.
    pub fn font_memory_usage_bytes(&self, id: FontId) -> Option<usize> {
        self.fonts
            .by_id
            .get(&id)
            .map(|entry| entry.asset.data.len())
    }

    /// Get memory limit in bytes.
    pub fn memory_limit(&self) -> usize {
        self.max_memory_bytes
    }

    /// Get memory usage as percentage (0.0-1.0).
    pub fn memory_usage_percent(&self) -> f32 {
        if self.max_memory_bytes == 0 {
            0.0
        } else {
            self.current_memory_bytes as f32 / self.max_memory_bytes as f32
        }
    }

    /// Iterate over all loaded images.
    pub fn iter_images(&self) -> impl Iterator<Item = (ImageId, &ImageAsset)> {
        self.images
            .by_id
            .iter()
            .map(|(&id, entry)| (id, &entry.asset))
    }

    /// Iterate over all loaded fonts.
    pub fn iter_fonts(&self) -> impl Iterator<Item = (FontId, &FontAsset)> {
        self.fonts
            .by_id
            .iter()
            .map(|(&id, entry)| (id, &entry.asset))
    }
}
