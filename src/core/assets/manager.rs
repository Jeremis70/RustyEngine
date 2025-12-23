use std::collections::HashMap;
use std::path::Path;

use super::error::{AssetError, AssetResult};
use super::image::{ImageAsset, ImageId};

/// Simple asset manager capable of loading and caching images.
/// Tracks memory usage and supports unloading.
pub struct AssetManager {
    images: HashMap<ImageId, ImageAsset>,
    max_memory_bytes: usize,
    current_memory_bytes: usize,
}

impl AssetManager {
    /// Create a new asset manager with unlimited memory
    pub fn new() -> Self {
        Self::with_limit(usize::MAX)
    }

    /// Create a new asset manager with a memory limit
    ///
    /// # Arguments
    /// * `max_bytes` - Maximum memory in bytes (e.g., 512 * 1024 * 1024 for 512MB)
    pub fn with_limit(max_bytes: usize) -> Self {
        Self {
            images: HashMap::new(),
            max_memory_bytes: max_bytes,
            current_memory_bytes: 0,
        }
    }

    /// Load an image from disk and cache it under a newly generated identifier.
    /// Returns the ImageId that can be used to retrieve the image later.
    pub fn load_image<P: AsRef<Path>>(&mut self, path: P) -> AssetResult<ImageId> {
        let path_buf = path.as_ref().to_path_buf();
        let dyn_img = image::open(&path_buf).map_err(|source| AssetError::Image {
            source,
            path: path_buf.clone(),
        })?;
        let rgba = dyn_img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let data = rgba.into_raw();

        let image_size = data.len(); // bytes
        if self.current_memory_bytes + image_size > self.max_memory_bytes {
            return Err(AssetError::MemoryExceeded {
                current: self.current_memory_bytes,
                limit: self.max_memory_bytes,
            });
        }

        let image = ImageAsset {
            width,
            height,
            data,
        };

        let id = ImageId::new();
        self.images.insert(id, image);
        self.current_memory_bytes += image_size;
        Ok(id)
    }

    /// Retrieve a previously loaded image by its identifier.
    pub fn get_image(&self, id: ImageId) -> Option<&ImageAsset> {
        self.images.get(&id)
    }

    /// Unload and remove an image from memory
    /// Returns true if the image was found and unloaded, false otherwise
    pub fn unload_image(&mut self, id: ImageId) -> bool {
        if let Some(image) = self.images.remove(&id) {
            self.current_memory_bytes -= image.data.len();
            log::debug!(
                "Unloaded image {:?}, memory now: {}",
                id,
                self.current_memory_bytes
            );
            true
        } else {
            false
        }
    }

    /// Get current memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        self.current_memory_bytes
    }

    /// Get memory limit in bytes
    pub fn memory_limit(&self) -> usize {
        self.max_memory_bytes
    }

    /// Get memory usage as percentage (0.0-1.0)
    pub fn memory_usage_percent(&self) -> f32 {
        if self.max_memory_bytes == 0 {
            0.0
        } else {
            self.current_memory_bytes as f32 / self.max_memory_bytes as f32
        }
    }

    /// Iterate over all loaded images.
    pub fn iter_images(&self) -> impl Iterator<Item = (ImageId, &ImageAsset)> {
        self.images.iter().map(|(&id, asset)| (id, asset))
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}
