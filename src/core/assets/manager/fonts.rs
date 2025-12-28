use std::path::Path;

use super::super::cache::FontKey;
use super::super::error::{AssetError, AssetResult};
use super::super::font::{FontAsset, FontCharset, FontId, Glyph};
use super::super::image::ImageAsset;
use super::AssetManager;

impl AssetManager {
    pub fn load_font<P: AsRef<Path>>(&mut self, path: P, font_size: f32) -> AssetResult<FontId> {
        // Preserve the original behavior by default (ASCII printable only).
        self.load_font_with_charset(path, font_size, FontCharset::Ascii)
    }

    /// Convenience: rasterize ASCII printable (U+0020..=U+007E).
    pub fn load_font_ascii<P: AsRef<Path>>(
        &mut self,
        path: P,
        font_size: f32,
    ) -> AssetResult<FontId> {
        self.load_font_with_charset(path, font_size, FontCharset::Ascii)
    }

    /// Convenience: rasterize Latin-1 (U+0020..=U+00FF). Useful for accents like éàèùç.
    pub fn load_font_latin1<P: AsRef<Path>>(
        &mut self,
        path: P,
        font_size: f32,
    ) -> AssetResult<FontId> {
        self.load_font_with_charset(path, font_size, FontCharset::Latin1)
    }

    /// Load a font and rasterize a specific set of characters into the atlas.
    ///
    /// Note: the charset is part of the cache key, so loading the same path/size with a
    /// different charset will produce a different `FontId`.
    pub fn load_font_with_charset<P: AsRef<Path>>(
        &mut self,
        path: P,
        font_size: f32,
        charset: FontCharset,
    ) -> AssetResult<FontId> {
        use crate::math::Vec2;
        use fontdue::Font;
        use std::collections::HashMap;
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        if !font_size.is_finite() || font_size <= 0.0 {
            return Err(AssetError::InvalidFontSize { font_size });
        }

        let charset_hash: u64 = match &charset {
            FontCharset::Ascii => 1,
            FontCharset::Latin1 => 2,
            FontCharset::Custom(chars) => {
                // Stable hash: sort + dedup before hashing.
                let mut v = chars.clone();
                v.sort_unstable();
                v.dedup();
                let mut hasher = DefaultHasher::new();
                v.hash(&mut hasher);
                hasher.finish()
            }
        };

        let info = self.compute_path_info(path.as_ref());
        self.enforce_path_policy(path.as_ref(), &info)?;
        let key_path = info.key.clone();
        let path_buf = info.io_path.clone();
        let key = FontKey::new(key_path, font_size, charset_hash);

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

        let mut chars: Vec<char> = match charset {
            FontCharset::Ascii => (0x20u32..=0x7Eu32).filter_map(char::from_u32).collect(),
            FontCharset::Latin1 => (0x20u32..=0xFFu32).filter_map(char::from_u32).collect(),
            FontCharset::Custom(mut v) => {
                // Ensure we have sane defaults for spacing + fallback.
                if !v.contains(&' ') {
                    v.push(' ');
                }
                if !v.contains(&'?') {
                    v.push('?');
                }
                v
            }
        };

        chars.sort_unstable();
        chars.dedup();

        for ch in chars {
            let (metrics, bitmap) = font.rasterize(ch, font_size);

            if metrics.width == 0 || metrics.height == 0 {
                glyphs.insert(
                    ch,
                    Glyph {
                        uv_min: Vec2::ZERO,
                        uv_max: Vec2::ZERO,
                        size: Vec2::ZERO,
                        // fontdue metrics:
                        // - xmin: offset of the left-most bitmap edge from the origin.
                        // - ymin: offset of the bottom-most bitmap edge from the baseline (Y-up).
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
                    // fontdue metrics:
                    // - xmin: offset of the left-most bitmap edge from the origin.
                    // - ymin: offset of the bottom-most bitmap edge from the baseline (Y-up).
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

    /// Check if a font with the given ID exists.
    pub fn font_exists(&self, id: FontId) -> bool {
        self.fonts.contains_id(id)
    }

    /// Get the total number of loaded fonts.
    pub fn font_count(&self) -> usize {
        self.fonts.len()
    }

    /// Retrieve a previously loaded font by its identifier.
    pub fn get_font(&self, id: FontId) -> Option<&FontAsset> {
        self.fonts.by_id.get(&id).map(|entry| &entry.asset)
    }

    /// Unload and remove a font from memory.
    /// Returns true if the font was found and unloaded, false otherwise.
    pub fn unload_font(&mut self, id: FontId) -> bool {
        if let Some(entry) = self.fonts.remove(id) {
            self.current_memory_bytes = self
                .current_memory_bytes
                .saturating_sub(entry.asset.data.len());
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
}
