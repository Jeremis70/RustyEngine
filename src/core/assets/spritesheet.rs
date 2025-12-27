use crate::math::Vec2;

use super::image::ImageId;

/// Configuration for extracting sprites from a spritesheet
#[derive(Debug, Clone, Copy)]
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

/// A single sprite's region in a spritesheet.
///
/// - `x`, `y`, `width`, `height` are pixel coordinates in the source image.
/// - `uv_min`, `uv_max` are normalized 0..1 texture coordinates.
#[derive(Debug, Clone, Copy)]
pub struct SpriteRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub uv_min: Vec2,
    pub uv_max: Vec2,
}

/// Spritesheet represented as a single texture plus per-sprite UV regions.
///
/// This is the GPU-friendly representation; you can still split it into
/// per-sprite `ImageId`s via `AssetManager` when needed.
#[derive(Debug, Clone)]
pub struct SpritesheetAtlas {
    pub image: ImageId,
    pub regions: Vec<SpriteRegion>,
}

/// Calculate the (column, row) positions for each sprite based on the order.
pub(crate) fn calculate_sprite_positions(config: &SpritesheetConfig) -> Vec<(u32, u32)> {
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

/// Extract pixel data for a single sprite from the spritesheet.
pub(crate) fn extract_sprite_data(
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
