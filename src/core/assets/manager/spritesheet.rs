use std::path::Path;

use super::super::error::{AssetError, AssetResult};
use super::super::image::ImageId;
use super::super::spritesheet::{
    SpriteRegion, SpritesheetAtlas, SpritesheetConfig, calculate_sprite_positions,
};
use super::AssetManager;

fn extract_sprite_from_rgba_buffer(
    src: &[u8],
    src_width: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Vec<u8> {
    let bytes_per_pixel = 4usize;
    let row_bytes = width as usize * bytes_per_pixel;
    let mut out = Vec::with_capacity(height as usize * row_bytes);

    for row in 0..height {
        let src_x = x as usize;
        let src_y = (y + row) as usize;
        let start = (src_y * src_width as usize + src_x) * bytes_per_pixel;
        let end = start + row_bytes;
        out.extend_from_slice(&src[start..end]);
    }

    out
}

impl AssetManager {
    /// Load a spritesheet as a single texture plus per-sprite UV regions.
    ///
    /// This is the GPU-friendly representation.
    pub fn load_spritesheet_atlas<P: AsRef<Path>>(
        &mut self,
        path: P,
        config: SpritesheetConfig,
    ) -> AssetResult<SpritesheetAtlas> {
        let info = self.compute_path_info(path.as_ref());
        self.enforce_path_policy(path.as_ref(), &info)?;
        let path_buf = info.io_path;

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

        // Load or reuse the full spritesheet image.
        let sheet_id = self.load_image(&path_buf)?;
        let sheet = self
            .get_image(sheet_id)
            .ok_or_else(|| AssetError::InvalidSpritesheet {
                path: path_buf.clone(),
                reason: "Spritesheet image was not found after loading".to_string(),
            })?;

        let sheet_width = sheet.width;
        let sheet_height = sheet.height;

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

        let positions = calculate_sprite_positions(&config);
        let mut regions = Vec::with_capacity(positions.len());

        for (col, row) in positions {
            let x = config.margin + col * (config.sprite_width + config.spacing);
            let y = config.margin + row * (config.sprite_height + config.spacing);

            let uv_min = crate::math::Vec2::new(
                x as f32 / sheet_width as f32,
                y as f32 / sheet_height as f32,
            );
            let uv_max = crate::math::Vec2::new(
                (x + config.sprite_width) as f32 / sheet_width as f32,
                (y + config.sprite_height) as f32 / sheet_height as f32,
            );

            regions.push(SpriteRegion {
                x,
                y,
                width: config.sprite_width,
                height: config.sprite_height,
                uv_min,
                uv_max,
            });
        }

        Ok(SpritesheetAtlas {
            image: sheet_id,
            regions,
        })
    }

    pub fn load_spritesheet<P: AsRef<Path>>(
        &mut self,
        path: P,
        config: SpritesheetConfig,
    ) -> AssetResult<Vec<ImageId>> {
        let atlas = self.load_spritesheet_atlas(path, config)?;
        atlas.as_image_vec(self)
    }
}

impl SpritesheetAtlas {
    /// Split the atlas into individual `ImageId`s.
    ///
    /// This is CPU/memory heavier than using UVs, but can be handy for simpler workflows.
    pub fn as_image_vec(&self, assets: &mut AssetManager) -> AssetResult<Vec<ImageId>> {
        let (sheet_width, sheet_height, sheet_data) = {
            let sheet =
                assets
                    .get_image(self.image)
                    .ok_or_else(|| AssetError::InvalidSpritesheet {
                        path: assets.asset_root.clone(),
                        reason: "Spritesheet atlas source image is not loaded".to_string(),
                    })?;
            (sheet.width, sheet.height, sheet.data.clone())
        };

        let mut out = Vec::with_capacity(self.regions.len());
        for region in &self.regions {
            if region.x + region.width > sheet_width || region.y + region.height > sheet_height {
                return Err(AssetError::InvalidSpritesheet {
                    path: assets.asset_root.clone(),
                    reason: "Sprite region is outside spritesheet bounds".to_string(),
                });
            }

            let sprite_data = extract_sprite_from_rgba_buffer(
                &sheet_data,
                sheet_width,
                region.x,
                region.y,
                region.width,
                region.height,
            );

            let sprite = super::super::image::ImageAsset {
                width: region.width,
                height: region.height,
                data: sprite_data,
            };

            let id = assets.load_image_from_asset(sprite)?;
            out.push(id);
        }

        Ok(out)
    }
}
