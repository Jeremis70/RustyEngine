use std::path::Path;

use super::super::cache::AssetPathInfo;
use super::super::cache::ImageKey;
use super::super::error::{AssetError, AssetResult};
use super::super::image::{ImageAsset, ImageId};
use super::AssetManager;

impl AssetManager {
    /// Load an image from disk and cache it under a newly generated identifier.
    /// Returns the ImageId that can be used to retrieve the image later.
    pub fn load_image<P: AsRef<Path>>(&mut self, path: P) -> AssetResult<ImageId> {
        let info = self.compute_path_info(path.as_ref());
        self.enforce_path_policy(path.as_ref(), &info)?;
        self.load_image_from_path_info(&info)
    }

    pub(crate) fn load_image_from_path_info(
        &mut self,
        info: &AssetPathInfo,
    ) -> AssetResult<ImageId> {
        let key = ImageKey {
            path: info.key.clone(),
        };

        if let Some(existing) = self.images.get_existing_id(&key) {
            return Ok(existing);
        }

        let path_buf = info.io_path.clone();
        let dyn_img = image::open(&path_buf).map_err(|source| AssetError::Image {
            source,
            path: path_buf.clone(),
        })?;
        let rgba = dyn_img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let data = rgba.into_raw();

        let image_size = data.len();
        self.ensure_capacity_for(image_size)?;

        let image = ImageAsset {
            width,
            height,
            data,
        };

        let id = ImageId::new();
        self.images.insert_keyed(id, key, image);
        self.current_memory_bytes += image_size;
        Ok(id)
    }

    /// Load an image from an existing ImageAsset.
    pub fn load_image_from_asset(&mut self, asset: ImageAsset) -> AssetResult<ImageId> {
        let image_size = asset.data.len();
        self.ensure_capacity_for(image_size)?;

        let id = ImageId::new();
        self.images.insert_unkeyed(id, asset);
        self.current_memory_bytes += image_size;
        Ok(id)
    }

    /// Check if an image with the given ID exists.
    pub fn image_exists(&self, id: ImageId) -> bool {
        self.images.contains_id(id)
    }

    /// Get the total number of loaded images.
    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    /// Retrieve a previously loaded image by its identifier.
    pub fn get_image(&self, id: ImageId) -> Option<&ImageAsset> {
        self.images.by_id.get(&id).map(|entry| &entry.asset)
    }

    /// Unload and remove an image from memory.
    /// Returns true if the image was found and unloaded, false otherwise.
    pub fn unload_image(&mut self, id: ImageId) -> bool {
        if let Some(entry) = self.images.remove(id) {
            self.current_memory_bytes = self
                .current_memory_bytes
                .saturating_sub(entry.asset.data.len());
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

    pub fn unload_all_images(&mut self) {
        let freed: usize = self
            .images
            .by_id
            .values()
            .map(|entry| entry.asset.data.len())
            .sum();
        self.images.clear();
        self.current_memory_bytes = self.current_memory_bytes.saturating_sub(freed);
        log::debug!(
            "Unloaded all images, memory now: {}",
            self.current_memory_bytes
        );
    }
}
