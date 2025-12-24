use std::collections::HashMap;
use std::path::Path;

use super::error::{AssetError, AssetResult};
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

    pub fn load_spritesheet<P: AsRef<Path>>(
        &mut self,
        path: P,
        config: SpritesheetConfig,
    ) -> AssetResult<Vec<ImageId>> {
        let path_buf = path.as_ref().to_path_buf();

        // Load the full spritesheet
        let dyn_img = image::open(&path_buf).map_err(|source| AssetError::Image {
            source,
            path: path_buf.clone(),
        })?;
        let spritesheet = dyn_img.to_rgba8();

        // Validate dimensions
        let sheet_width = spritesheet.width();
        let sheet_height = spritesheet.height();

        let expected_width = config.margin * 2
            + config.columns * config.sprite_width
            + (config.columns - 1) * config.spacing;
        let expected_height = config.margin * 2
            + config.rows * config.sprite_height
            + (config.rows - 1) * config.spacing;

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
            if self.current_memory_bytes + image_size > self.max_memory_bytes {
                return Err(AssetError::MemoryExceeded {
                    current: self.current_memory_bytes,
                    limit: self.max_memory_bytes,
                });
            }

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
