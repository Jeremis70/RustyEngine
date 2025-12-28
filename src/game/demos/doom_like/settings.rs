use std::sync::OnceLock;

use crate::math::Color;
use crate::math::vec2::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct Settings {
    /// Size of one map tile in world units (pixels).
    pub tile_size: f32,
    /// Player start position in tile units (e.g. x=1.5 tiles).
    pub player_start_tiles: Vec2,
    pub player_angle_start: f32,
    pub player_speed: f32,
    pub player_rotation_speed: f32,

    // Raycasting settings (ported from the python reference)
    pub fov: f32,
    pub num_rays: usize,
    /// Maximum ray depth in tile units.
    pub max_depth: f32,

    pub floor_color: Color,
    pub sky_color: Color,
    pub wall_color: Color,
    pub wall_shade_strength: f32,

    pub player_max_health: i32,
    pub health_recovery_delay_ms: f32,
    pub player_size_scale: f32,
    pub mouse_max_rel: f32,
    /// Wall outline color (pygame "darkgray").
    pub wall_outline_color: Color,
    /// Wall outline thickness in pixels.
    pub wall_outline_thickness: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            tile_size: 100.0,
            player_start_tiles: Vec2::new(1.5, 5.0),
            wall_outline_color: Color::from((169, 169, 169)),
            wall_outline_thickness: 2.0,
            player_angle_start: 0.0,
            player_speed: 0.004,
            player_rotation_speed: 0.002,

            fov: std::f32::consts::FRAC_PI_3,
            num_rays: 300,
            max_depth: 20.0,

            floor_color: Color::from((30, 30, 30)),
            sky_color: Color::from((10, 10, 30)),
            wall_color: Color::from((220, 220, 220)),
            wall_shade_strength: 0.85,

            player_max_health: 100,
            health_recovery_delay_ms: 700.0,
            player_size_scale: 60.0,
            mouse_max_rel: 40.0,
        }
    }
}

static SETTINGS: OnceLock<Settings> = OnceLock::new();

/// Initialize settings once. If called multiple times, the first call wins.
pub fn init(value: Settings) -> &'static Settings {
    let _ = SETTINGS.set(value);
    settings()
}

/// Read-only access to settings from anywhere.
/// If not initialized yet, defaults are used.
pub fn settings() -> &'static Settings {
    SETTINGS.get_or_init(Settings::default)
}
