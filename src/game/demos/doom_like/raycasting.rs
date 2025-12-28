use super::map::Map;
use super::player::Player;
use super::settings;

#[derive(Debug, Clone, Copy)]
pub struct RayHit {
    pub depth: f32,
    pub proj_height: f32,
}

#[derive(Debug, Default)]
pub struct RayCasting {
    rays: Vec<RayHit>,
}

impl RayCasting {
    pub fn new() -> Self {
        Self { rays: Vec::new() }
    }

    pub fn rays(&self) -> &[RayHit] {
        &self.rays
    }

    pub fn update(&mut self, map: &Map, player: &Player, screen_size: (u32, u32)) {
        let s = settings::settings();
        let (screen_w, screen_h) = (screen_size.0.max(1) as f32, screen_size.1.max(1) as f32);

        let half_fov = s.fov * 0.5;
        let num_rays = s.num_rays.max(1);
        let delta_angle = s.fov / num_rays as f32;

        // SCREEN_DIST equivalent (computed dynamically from window size)
        let screen_dist = (screen_w * 0.5) / half_fov.tan();

        self.rays.clear();
        self.rays.reserve(num_rays);

        let px = player.pos_tiles.x;
        let py = player.pos_tiles.y;

        let mut ray_angle = player.angle - half_fov + 0.0001;
        for _ in 0..num_rays {
            ray_angle = ray_angle.rem_euclid(std::f32::consts::TAU);

            let sin_a = ray_angle.sin();
            let cos_a = ray_angle.cos();

            // --- horizontal intersections ---
            let mut depth_h = f32::INFINITY;
            if sin_a.abs() > 1e-6 {
                let (mut y_h, dy) = if sin_a > 0.0 {
                    (py.floor() + 1.0, 1.0)
                } else {
                    (py.floor() - 1e-6, -1.0)
                };

                let mut depth = (y_h - py) / sin_a;
                let mut x_h = px + depth * cos_a;

                let delta_depth = dy / sin_a;
                let dx = delta_depth * cos_a;

                // step grid lines
                let mut steps = 0;
                while steps < 1024 && depth < s.max_depth {
                    let tile_x = x_h as i32;
                    // Note: when looking up we start at (floor(py) - 1e-6), so casting to i32
                    // already yields the correct tile. Don't subtract again.
                    let tile_y = y_h as i32;
                    if map.is_wall_i32(tile_x, tile_y) {
                        depth_h = depth;
                        break;
                    }
                    x_h += dx;
                    y_h += dy;
                    depth += delta_depth;
                    steps += 1;
                }
            }

            // --- vertical intersections ---
            let mut depth_v = f32::INFINITY;
            if cos_a.abs() > 1e-6 {
                let (mut x_v, dx_step) = if cos_a > 0.0 {
                    (px.floor() + 1.0, 1.0)
                } else {
                    (px.floor() - 1e-6, -1.0)
                };

                let mut depth = (x_v - px) / cos_a;
                let mut y_v = py + depth * sin_a;

                let delta_depth = dx_step / cos_a;
                let dy = delta_depth * sin_a;

                let mut steps = 0;
                while steps < 1024 && depth < s.max_depth {
                    // Note: when looking left we start at (floor(px) - 1e-6), so casting to i32
                    // already yields the correct tile. Don't subtract again.
                    let tile_x = x_v as i32;
                    let tile_y = y_v as i32;
                    if map.is_wall_i32(tile_x, tile_y) {
                        depth_v = depth;
                        break;
                    }
                    x_v += dx_step;
                    y_v += dy;
                    depth += delta_depth;
                    steps += 1;
                }
            }

            let mut depth = depth_h.min(depth_v);
            if !depth.is_finite() {
                depth = s.max_depth;
            }

            // fishbowl correction (python-style)
            depth *= (player.angle - ray_angle).cos().max(1e-6);
            depth = depth.max(1e-4);

            let proj_height = (screen_dist / depth).min(screen_h * 2.0);
            self.rays.push(RayHit { depth, proj_height });

            ray_angle += delta_angle;
        }
    }
}
