use crate::core::engine_state::EngineState;
use crate::core::events::input::Input;
use crate::core::events::{Key, MouseButton};
use crate::math::Color;
use crate::math::vec2::Vec2;
use crate::render::Drawable;
use crate::render::context::RenderContext;
use crate::render::shapes::{Circle, Line};

use super::map::Map;
use super::settings;

#[derive(Debug, Clone)]
pub struct Player {
    /// Position in tile units (same as python: self.x, self.y).
    pub pos_tiles: Vec2,
    pub angle: f32,
    pub shot: bool,
    pub health: i32,

    pub rel: f32,
    diag_move_corr: f32,

    health_recovery_timer_ms: f32,
}

impl Player {
    pub fn new(pos_tiles: Vec2, angle: f32) -> Self {
        let s = settings::settings();
        Self {
            pos_tiles,
            angle,
            shot: false,
            health: s.player_max_health,
            rel: 0.0,
            diag_move_corr: 1.0 / 2.0_f32.sqrt(),
            health_recovery_timer_ms: 0.0,
        }
    }

    pub fn new_from_settings() -> Self {
        let s = settings::settings();
        Self::new(s.player_start_tiles, s.player_angle_start)
    }

    pub fn update(&mut self, state: &EngineState, input: &Input, map: &Map) {
        self.single_fire_event(input);
        self.movement(state, input, map);
        self.mouse_control(state, input);
        self.recover_health(state);
        self.angle = self.angle.rem_euclid(std::f32::consts::TAU);
    }

    pub fn recover_health(&mut self, state: &EngineState) {
        let s = settings::settings();
        if self.health >= s.player_max_health {
            return;
        }

        self.health_recovery_timer_ms += state.delta_seconds() * 1000.0;
        if self.health_recovery_timer_ms >= s.health_recovery_delay_ms {
            self.health_recovery_timer_ms = 0.0;
            self.health += 1;
        }
    }

    pub fn get_damage(&mut self, damage: i32) {
        self.health -= damage;
    }

    pub fn is_dead(&self) -> bool {
        self.health < 1
    }

    pub fn single_fire_event(&mut self, input: &Input) {
        // Equivalent to: if MOUSEBUTTONDOWN left and not shot
        if self.shot {
            return;
        }
        let just_pressed = input
            .just_pressed_buttons_list()
            .contains(&MouseButton::Left);
        if just_pressed {
            self.shot = true;
        }
    }

    pub fn consume_shot(&mut self) -> bool {
        if self.shot {
            self.shot = false;
            true
        } else {
            false
        }
    }

    fn movement(&mut self, state: &EngineState, input: &Input, map: &Map) {
        let s = settings::settings();

        let sin_a = self.angle.sin();
        let cos_a = self.angle.cos();

        let dt_ms = state.delta_seconds() * 1000.0;
        let speed = s.player_speed * dt_ms;
        let speed_sin = speed * sin_a;
        let speed_cos = speed * cos_a;

        let mut dx = 0.0;
        let mut dy = 0.0;
        let mut pressed = 0;

        if input.key(Key::W) {
            pressed += 1;
            dx += speed_cos;
            dy += speed_sin;
        }
        if input.key(Key::S) {
            pressed += 1;
            dx -= speed_cos;
            dy -= speed_sin;
        }
        if input.key(Key::A) {
            pressed += 1;
            dx += speed_sin;
            dy -= speed_cos;
        }
        if input.key(Key::D) {
            pressed += 1;
            dx -= speed_sin;
            dy += speed_cos;
        }

        if pressed > 1 {
            dx *= self.diag_move_corr;
            dy *= self.diag_move_corr;
        }

        self.check_wall_collision(dx, dy, state, map);
    }

    fn check_wall_collision(&mut self, dx: f32, dy: f32, state: &EngineState, map: &Map) {
        let s = settings::settings();

        // Keep the same behavior as the python reference: scale = PLAYER_SIZE_SCALE / delta_time
        // Here we operate in ms (see movement), so use dt_ms.
        let dt_ms = (state.delta_seconds() * 1000.0).max(1e-3);
        let scale = s.player_size_scale / dt_ms;

        let next_x = self.pos_tiles.x + dx * scale;
        let next_y = self.pos_tiles.y + dy * scale;

        if !map.is_wall_i32(next_x as i32, self.pos_tiles.y as i32) {
            self.pos_tiles.x += dx;
        }
        if !map.is_wall_i32(self.pos_tiles.x as i32, next_y as i32) {
            self.pos_tiles.y += dy;
        }
    }

    fn mouse_control(&mut self, state: &EngineState, input: &Input) {
        let s = settings::settings();

        let (mx, _my) = input.mouse_delta();
        let mut rel = mx;
        rel = rel.clamp(-s.mouse_max_rel, s.mouse_max_rel);
        self.rel = rel;

        let dt_ms = state.delta_seconds() * 1000.0;
        self.angle += self.rel * s.player_rotation_speed * dt_ms;
    }

    /// Debug top-down rendering (like the python reference draw()).
    pub fn draw_debug(&self, ctx: &mut RenderContext) {
        let s = settings::settings();

        let pos_px = Vec2::new(
            self.pos_tiles.x * s.tile_size,
            self.pos_tiles.y * s.tile_size,
        );

        let dir_end = pos_px
            + Vec2::new(
                ctx.size.0 as f32 * self.angle.cos(),
                ctx.size.0 as f32 * self.angle.sin(),
            );
        Line::new(pos_px, dir_end, Color::from((255, 255, 0)), 2.0).draw(ctx);

        Circle::new(pos_px, 15.0, Color::from((0, 255, 0))).draw(ctx);
    }

    pub fn map_pos(&self) -> (i32, i32) {
        (self.pos_tiles.x as i32, self.pos_tiles.y as i32)
    }
}
