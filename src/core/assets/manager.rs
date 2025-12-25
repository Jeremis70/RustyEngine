use std::collections::HashMap;
use std::path::Path;

use super::error::{AssetError, AssetResult};
use super::font::{FontAsset, FontId};
use super::image::{ImageAsset, ImageId};

/// Configuration for extracting sprites from a spritesheet
pub struct SpritesheetConfig {
    /// Number of columns in the spritesheet
    pub columns: u32,
    /// Number of rows in the spritesheet
    pub rows: u32,
    /// Width of each sprite in pixels
    pub sprite_width: u32,
    /// Height of each sprite in pixels
    pub sprite_height: u32,
    /// Order in which to extract sprites
    pub order: SpriteOrder,
    /// Optional spacing between sprites in pixels
    pub spacing: u32,
    /// Optional margin around the entire spritesheet in pixels
    pub margin: u32,
}

/// Defines the order in which sprites are extracted from the sheet
#[derive(Debug, Clone, Copy)]
pub enum SpriteOrder {
    /// Left to right, top to bottom (most common)
    /// ```text
    /// 1 2 3
    /// 4 5 6
    /// 7 8 9
    /// ```
    LeftToRightTopToBottom,

    /// Right to left, top to bottom
    /// ```text
    /// 3 2 1
    /// 6 5 4
    /// 9 8 7
    /// ```
    RightToLeftTopToBottom,

    /// Left to right, bottom to top
    /// ```text
    /// 7 8 9
    /// 4 5 6
    /// 1 2 3
    /// ```
    LeftToRightBottomToTop,

    /// Zigzag: alternating left-to-right and right-to-left
    /// ```text
    /// 1 2 3
    /// 6 5 4
    /// 7 8 9
    /// ```
    Zigzag,

    /// Top to bottom, left to right
    /// ```text
    /// 1 4 7
    /// 2 5 8
    /// 3 6 9
    /// ```
    TopToBottomLeftToRight,
}

/// Simple asset manager capable of loading and caching images.
/// Tracks memory usage and supports unloading.
pub struct AssetManager {
    images: HashMap<ImageId, ImageAsset>,
    fonts: HashMap<FontId, FontAsset>,
    max_memory_bytes: usize,
    current_memory_bytes: usize,
}

impl AssetManager {
    fn ensure_capacity_for(&self, additional_bytes: usize) -> AssetResult<()> {
        let new_total = self
            .current_memory_bytes
            .checked_add(additional_bytes)
            .ok_or(AssetError::MemoryExceeded {
                current: self.current_memory_bytes,
                limit: self.max_memory_bytes,
            })?;

        if new_total > self.max_memory_bytes {
            return Err(AssetError::MemoryExceeded {
                current: self.current_memory_bytes,
                limit: self.max_memory_bytes,
            });
        }

        Ok(())
    }

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
            fonts: HashMap::new(),
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
        self.ensure_capacity_for(image_size)?;

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

    pub fn load_spritesheet<P: AsRef<Path>>(
        &mut self,
        path: P,
        config: SpritesheetConfig,
    ) -> AssetResult<Vec<ImageId>> {
        let path_buf = path.as_ref().to_path_buf();

        if config.columns == 0
            || config.rows == 0
            || config.sprite_width == 0
            || config.sprite_height == 0
        {
            return Err(AssetError::InvalidSpritesheet {
                path: path_buf,
                reason: "Spritesheet config must have non-zero columns/rows/sprite dimensions"
                    .to_string(),
            });
        }

        // Load the full spritesheet
        let dyn_img = image::open(&path_buf).map_err(|source| AssetError::Image {
            source,
            path: path_buf.clone(),
        })?;
        let spritesheet = dyn_img.to_rgba8();

        // Validate dimensions
        let sheet_width = spritesheet.width();
        let sheet_height = spritesheet.height();

        let expected_width = (config.margin)
            .checked_mul(2)
            .and_then(|v| v.checked_add(config.columns.checked_mul(config.sprite_width)?))
            .and_then(|v| {
                v.checked_add(
                    config
                        .columns
                        .saturating_sub(1)
                        .checked_mul(config.spacing)?,
                )
            })
            .ok_or_else(|| AssetError::InvalidSpritesheet {
                path: path_buf.clone(),
                reason: "Spritesheet expected width overflow".to_string(),
            })?;

        let expected_height = (config.margin)
            .checked_mul(2)
            .and_then(|v| v.checked_add(config.rows.checked_mul(config.sprite_height)?))
            .and_then(|v| v.checked_add(config.rows.saturating_sub(1).checked_mul(config.spacing)?))
            .ok_or_else(|| AssetError::InvalidSpritesheet {
                path: path_buf.clone(),
                reason: "Spritesheet expected height overflow".to_string(),
            })?;

        if sheet_width < expected_width || sheet_height < expected_height {
            return Err(AssetError::InvalidSpritesheet {
                path: path_buf,
                reason: format!(
                    "Spritesheet dimensions {}x{} are smaller than expected {}x{}",
                    sheet_width, sheet_height, expected_width, expected_height
                ),
            });
        }

        // Calculate positions for each sprite based on order
        let positions = Self::calculate_sprite_positions(&config);

        let mut sprite_ids = Vec::with_capacity(positions.len());

        // Extract each sprite
        for (col, row) in positions {
            let x = config.margin + col * (config.sprite_width + config.spacing);
            let y = config.margin + row * (config.sprite_height + config.spacing);

            // Extract the sub-image
            let sprite_data = Self::extract_sprite_data(
                &spritesheet,
                x,
                y,
                config.sprite_width,
                config.sprite_height,
            );

            let image_size = sprite_data.len();

            // Check memory limit
            self.ensure_capacity_for(image_size)?;

            // Create and store the sprite
            let sprite = ImageAsset {
                width: config.sprite_width,
                height: config.sprite_height,
                data: sprite_data,
            };

            let id = ImageId::new();
            self.images.insert(id, sprite);
            self.current_memory_bytes += image_size;
            sprite_ids.push(id);
        }

        log::info!(
            "Loaded {} sprites from {:?}, memory usage: {}",
            sprite_ids.len(),
            path_buf,
            self.current_memory_bytes
        );

        Ok(sprite_ids)
    }

    /// Calculate the (column, row) positions for each sprite based on the order
    fn calculate_sprite_positions(config: &SpritesheetConfig) -> Vec<(u32, u32)> {
        let total_sprites = (config.columns * config.rows) as usize;
        let mut positions = Vec::with_capacity(total_sprites);

        match config.order {
            SpriteOrder::LeftToRightTopToBottom => {
                for row in 0..config.rows {
                    for col in 0..config.columns {
                        positions.push((col, row));
                    }
                }
            }

            SpriteOrder::RightToLeftTopToBottom => {
                for row in 0..config.rows {
                    for col in (0..config.columns).rev() {
                        positions.push((col, row));
                    }
                }
            }

            SpriteOrder::LeftToRightBottomToTop => {
                for row in (0..config.rows).rev() {
                    for col in 0..config.columns {
                        positions.push((col, row));
                    }
                }
            }

            SpriteOrder::Zigzag => {
                for row in 0..config.rows {
                    if row % 2 == 0 {
                        // Even rows: left to right
                        for col in 0..config.columns {
                            positions.push((col, row));
                        }
                    } else {
                        // Odd rows: right to left
                        for col in (0..config.columns).rev() {
                            positions.push((col, row));
                        }
                    }
                }
            }

            SpriteOrder::TopToBottomLeftToRight => {
                for col in 0..config.columns {
                    for row in 0..config.rows {
                        positions.push((col, row));
                    }
                }
            }
        }

        positions
    }

    /// Extract pixel data for a single sprite from the spritesheet
    fn extract_sprite_data(
        spritesheet: &image::RgbaImage,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Vec<u8> {
        let mut data = Vec::with_capacity((width * height * 4) as usize);

        for py in y..(y + height) {
            for px in x..(x + width) {
                let pixel = spritesheet.get_pixel(px, py);
                data.extend_from_slice(&pixel.0);
            }
        }

        data
    }

    pub fn load_font<P: AsRef<Path>>(&mut self, path: P) -> AssetResult<FontId> {
        let path_buf = path.as_ref().to_path_buf();

        // Read the font file as raw bytes
        let data = std::fs::read(&path_buf).map_err(|source| AssetError::Io {
            source,
            path: path_buf.clone(),
        })?;

        let font_size = data.len();

        // Check memory limit
        self.ensure_capacity_for(font_size)?;

        let font = FontAsset { data };
        let id = FontId::new();

        self.fonts.insert(id, font);
        self.current_memory_bytes += font_size;

        log::info!(
            "Loaded font {:?}, memory usage: {}",
            path_buf,
            self.current_memory_bytes
        );

        Ok(id)
    }

    pub fn load_font_from_asset(&mut self, asset: FontAsset) -> AssetResult<FontId> {
        let font_size = asset.data.len(); // bytes
        self.ensure_capacity_for(font_size)?;

        let id = FontId::new();
        self.fonts.insert(id, asset);
        self.current_memory_bytes += font_size;
        Ok(id)
    }

    /// Load an image from an existing ImageAsset
    pub fn load_image_from_asset(&mut self, asset: ImageAsset) -> AssetResult<ImageId> {
        let image_size = asset.data.len(); // bytes
        self.ensure_capacity_for(image_size)?;

        let id = ImageId::new();
        self.images.insert(id, asset);
        self.current_memory_bytes += image_size;
        Ok(id)
    }

    /// Check if an image with the given ID exists
    pub fn image_exists(&self, id: ImageId) -> bool {
        self.images.contains_key(&id)
    }
    /// Get the total number of loaded images
    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    /// Retrieve a previously loaded image by its identifier.
    pub fn get_image(&self, id: ImageId) -> Option<&ImageAsset> {
        self.images.get(&id)
    }

    /// Check if a font with the given ID exists
    pub fn font_exists(&self, id: FontId) -> bool {
        self.fonts.contains_key(&id)
    }

    /// Get the total number of loaded fonts
    pub fn font_count(&self) -> usize {
        self.fonts.len()
    }

    /// Retrieve a previously loaded font by its identifier.
    pub fn get_font(&self, id: FontId) -> Option<&FontAsset> {
        self.fonts.get(&id)
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

    /// Unload and remove a font from memory
    /// Returns true if the font was found and unloaded, false otherwise
    pub fn unload_font(&mut self, id: FontId) -> bool {
        if let Some(font) = self.fonts.remove(&id) {
            self.current_memory_bytes -= font.data.len();
            log::debug!(
                "Unloaded font {:?}, memory now: {}",
                id,
                self.current_memory_bytes
            );
            true
        } else {
            false
        }
    }
    pub fn load_fonts<P: AsRef<Path>>(&mut self, paths: &[P]) -> AssetResult<Vec<FontId>> {
        let mut ids = Vec::with_capacity(paths.len());
        for path in paths {
            let id = self.load_font(path)?;
            ids.push(id);
        }
        Ok(ids)
    }

    /// Load multiple images from disk
    pub fn load_images<P: AsRef<Path>>(&mut self, paths: &[P]) -> AssetResult<Vec<ImageId>> {
        let mut ids = Vec::with_capacity(paths.len());
        for path in paths {
            let id = self.load_image(path)?;
            ids.push(id);
        }
        Ok(ids)
    }

    /// Unload multiple images from memory
    pub fn unload_images(&mut self, ids: &[ImageId]) {
        for &id in ids {
            self.unload_image(id);
        }
    }

    /// Unload multiple fonts from memory
    pub fn unload_fonts(&mut self, ids: &[FontId]) {
        for &id in ids {
            self.unload_font(id);
        }
    }

    pub fn unload_all_fonts(&mut self) {
        let freed: usize = self.fonts.values().map(|font| font.data.len()).sum();
        self.fonts.clear();
        self.current_memory_bytes = self.current_memory_bytes.saturating_sub(freed);
        log::debug!(
            "Unloaded all fonts, memory now: {}",
            self.current_memory_bytes
        );
    }

    pub fn unload_all_images(&mut self) {
        let freed: usize = self.images.values().map(|image| image.data.len()).sum();
        self.images.clear();
        self.current_memory_bytes = self.current_memory_bytes.saturating_sub(freed);
        log::debug!(
            "Unloaded all images, memory now: {}",
            self.current_memory_bytes
        );
    }

    /// Unload all assets from memory
    pub fn unload_all(&mut self) {
        self.images.clear();
        self.fonts.clear();
        self.current_memory_bytes = 0;
        log::debug!(
            "Unloaded all assets, memory now: {}",
            self.current_memory_bytes
        );
    }

    /// Get current memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        self.current_memory_bytes
    }

    /// Get memory usage for images only (sum of raw pixel buffers).
    pub fn images_memory_usage_bytes(&self) -> usize {
        self.images.values().map(|image| image.data.len()).sum()
    }

    /// Get memory usage for fonts only (sum of raw font buffers).
    pub fn fonts_memory_usage_bytes(&self) -> usize {
        self.fonts.values().map(|font| font.data.len()).sum()
    }

    /// Get memory usage for a specific image.
    pub fn image_memory_usage_bytes(&self, id: ImageId) -> Option<usize> {
        self.images.get(&id).map(|image| image.data.len())
    }

    /// Get memory usage for a specific font.
    pub fn font_memory_usage_bytes(&self, id: FontId) -> Option<usize> {
        self.fonts.get(&id).map(|font| font.data.len())
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

    /// Iterate over all loaded fonts.
    pub fn iter_fonts(&self) -> impl Iterator<Item = (FontId, &FontAsset)> {
        self.fonts.iter().map(|(&id, asset)| (id, asset))
    }
}
impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}
