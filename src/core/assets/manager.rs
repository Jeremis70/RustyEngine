use std::path::Path;

use crate::audio::{AudioError, AudioResult, AudioSystem, LoadStrategy, SoundId};

use super::cache::{AssetStore, FontKey, ImageKey, normalize_path};
use super::error::{AssetError, AssetResult};
use super::font::{FontAsset, FontId, Glyph};
use super::image::{ImageAsset, ImageId};
use super::sound_tracking::{SoundAsset, SoundKey};
use super::spritesheet::{SpritesheetConfig, calculate_sprite_positions, extract_sprite_data};

/// Simple asset manager capable of loading and caching images.
/// Tracks memory usage and supports unloading.
pub struct AssetManager {
    images: AssetStore<ImageId, ImageKey, ImageAsset>,
    fonts: AssetStore<FontId, FontKey, FontAsset>,
    sounds: AssetStore<SoundId, SoundKey, SoundAsset>,
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
            images: AssetStore::new(),
            fonts: AssetStore::new(),
            sounds: AssetStore::new(),
            max_memory_bytes: max_bytes,
            current_memory_bytes: 0,
        }
    }

    /// Load an image from disk and cache it under a newly generated identifier.
    /// Returns the ImageId that can be used to retrieve the image later.
    pub fn load_image<P: AsRef<Path>>(&mut self, path: P) -> AssetResult<ImageId> {
        let path_buf = normalize_path(path.as_ref());
        let key = ImageKey {
            path: path_buf.clone(),
        };

        if let Some(existing) = self.images.get_existing_id(&key) {
            return Ok(existing);
        }

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
        self.images.insert_keyed(id, key, image);
        self.current_memory_bytes += image_size;
        Ok(id)
    }

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
        let path_buf = normalize_path(path.as_ref());
        let key = SoundKey {
            path: path_buf.clone(),
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
        let positions = calculate_sprite_positions(&config);

        let mut sprite_ids = Vec::with_capacity(positions.len());

        // Extract each sprite
        for (col, row) in positions {
            let x = config.margin + col * (config.sprite_width + config.spacing);
            let y = config.margin + row * (config.sprite_height + config.spacing);

            // Extract the sub-image
            let sprite_data = extract_sprite_data(
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
            self.images.insert_unkeyed(id, sprite);
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

    pub fn load_font<P: AsRef<Path>>(&mut self, path: P, font_size: f32) -> AssetResult<FontId> {
        use crate::math::Vec2;
        use fontdue::Font;
        use std::collections::HashMap;

        let path_buf = normalize_path(path.as_ref());
        let key = FontKey::new(path_buf.clone(), font_size);

        if let Some(existing) = self.fonts.get_existing_id(&key) {
            return Ok(existing);
        }

        // Read font data from disk
        let data = std::fs::read(&path_buf).map_err(|source| AssetError::Io {
            source,
            path: path_buf.clone(),
        })?;

        let font_data_size = data.len();
        self.ensure_capacity_for(font_data_size)?;

        // Load the font using fontdue
        let font = Font::from_bytes(data.clone(), fontdue::FontSettings::default())
            .map_err(|_| AssetError::InvalidFont)?;

        // Prepare to rasterize glyphs into an atlas
        const ATLAS_SIZE: u32 = 1024;
        let mut atlas_pixels = vec![0u8; (ATLAS_SIZE * ATLAS_SIZE) as usize];
        let mut glyphs = HashMap::new();

        let mut pen_x = 0u32;
        let mut pen_y = 0u32;
        let mut row_height = 0u32;

        // ASCII printable
        for ch in 32u8..=126 {
            let ch = ch as char;

            let (metrics, bitmap) = font.rasterize(ch, font_size);

            if metrics.width == 0 || metrics.height == 0 {
                glyphs.insert(
                    ch,
                    Glyph {
                        uv_min: Vec2::ZERO,
                        uv_max: Vec2::ZERO,
                        size: Vec2::ZERO,
                        bearing: Vec2::new(metrics.xmin as f32, metrics.ymin as f32),
                        advance: metrics.advance_width,
                    },
                );

                pen_x += metrics.advance_width.ceil() as u32;
                continue;
            }

            if pen_x + metrics.width as u32 >= ATLAS_SIZE {
                pen_x = 0;
                pen_y += row_height + 1;
                row_height = 0;
            }

            if pen_y + metrics.height as u32 >= ATLAS_SIZE {
                return Err(AssetError::OutOfMemory);
            }

            // Copy bitmap into atlas
            for y in 0..metrics.height {
                for x in 0..metrics.width {
                    let src = x + y * metrics.width;
                    let dst = (pen_x + x as u32) + (pen_y + y as u32) * ATLAS_SIZE;

                    atlas_pixels[dst as usize] = bitmap[src];
                }
            }

            let uv_min = Vec2::new(
                pen_x as f32 / ATLAS_SIZE as f32,
                pen_y as f32 / ATLAS_SIZE as f32,
            );

            let uv_max = Vec2::new(
                (pen_x + metrics.width as u32) as f32 / ATLAS_SIZE as f32,
                (pen_y + metrics.height as u32) as f32 / ATLAS_SIZE as f32,
            );

            glyphs.insert(
                ch,
                Glyph {
                    uv_min,
                    uv_max,
                    size: Vec2::new(metrics.width as f32, metrics.height as f32),
                    bearing: Vec2::new(metrics.xmin as f32, metrics.ymin as f32),
                    advance: metrics.advance_width,
                },
            );

            pen_x += metrics.width as u32 + 1;
            row_height = row_height.max(metrics.height as u32);
        }

        // Convert grayscale atlas to RGBA
        let mut atlas_rgba = Vec::with_capacity((ATLAS_SIZE * ATLAS_SIZE * 4) as usize);
        for &gray in &atlas_pixels {
            atlas_rgba.push(255); // R
            atlas_rgba.push(255); // G
            atlas_rgba.push(255); // B
            atlas_rgba.push(gray); // A (alpha = grayscale value)
        }

        let atlas_asset = ImageAsset {
            width: ATLAS_SIZE,
            height: ATLAS_SIZE,
            data: atlas_rgba,
        };
        let atlas_image = self.load_image_from_asset(atlas_asset)?;

        // Create FontAsset
        let font_asset = FontAsset {
            data,
            atlas: atlas_image,
            glyphs,
            line_height: font
                .horizontal_line_metrics(font_size)
                .map(|m| m.new_line_size)
                .unwrap_or(font_size),
            font_size,
        };

        let id = FontId::new();
        self.fonts.insert_keyed(id, key, font_asset);
        self.current_memory_bytes += font_data_size;

        log::info!("Loaded font {:?} ({}px)", path_buf, font_size);

        Ok(id)
    }

    /// Load an image from an existing ImageAsset
    pub fn load_image_from_asset(&mut self, asset: ImageAsset) -> AssetResult<ImageId> {
        let image_size = asset.data.len(); // bytes
        self.ensure_capacity_for(image_size)?;

        let id = ImageId::new();
        self.images.insert_unkeyed(id, asset);
        self.current_memory_bytes += image_size;
        Ok(id)
    }

    /// Check if an image with the given ID exists
    pub fn image_exists(&self, id: ImageId) -> bool {
        self.images.contains_id(id)
    }
    /// Get the total number of loaded images
    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    /// Retrieve a previously loaded image by its identifier.
    pub fn get_image(&self, id: ImageId) -> Option<&ImageAsset> {
        self.images.by_id.get(&id).map(|entry| &entry.asset)
    }

    /// Check if a font with the given ID exists
    pub fn font_exists(&self, id: FontId) -> bool {
        self.fonts.contains_id(id)
    }

    /// Get the total number of loaded fonts
    pub fn font_count(&self) -> usize {
        self.fonts.len()
    }

    /// Retrieve a previously loaded font by its identifier.
    pub fn get_font(&self, id: FontId) -> Option<&FontAsset> {
        self.fonts.by_id.get(&id).map(|entry| &entry.asset)
    }

    /// Unload and remove an image from memory
    /// Returns true if the image was found and unloaded, false otherwise
    pub fn unload_image(&mut self, id: ImageId) -> bool {
        if let Some(entry) = self.images.remove(id) {
            self.current_memory_bytes -= entry.asset.data.len();
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
        if let Some(entry) = self.fonts.remove(id) {
            self.current_memory_bytes -= entry.asset.data.len();
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
        let freed: usize = self
            .fonts
            .by_id
            .values()
            .map(|entry| entry.asset.data.len())
            .sum();
        self.fonts.clear();
        self.current_memory_bytes = self.current_memory_bytes.saturating_sub(freed);
        log::debug!(
            "Unloaded all fonts, memory now: {}",
            self.current_memory_bytes
        );
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

    /// Get current memory usage in bytes
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
impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}
