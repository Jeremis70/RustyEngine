use crate::math::vec2::Vec2;
use crate::render::Drawable;
use crate::render::context::RenderContext;
use crate::render::shapes::Rectangle;

use super::raycasting::RayHit;
use super::settings;

#[derive(Debug, Default)]
pub struct ObjectRenderer {}

impl ObjectRenderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn draw(&self, ctx: &mut RenderContext, rays: &[RayHit]) {
        let s = settings::settings();

        let w = ctx.size.0.max(1) as f32;
        let h = ctx.size.1.max(1) as f32;

        // Ceiling / floor background (python fills sky then floor)
        Rectangle::new(Vec2::new(0.0, 0.0), Vec2::new(w, h * 0.5), s.sky_color).draw(ctx);
        Rectangle::new(
            Vec2::new(0.0, h * 0.5),
            Vec2::new(w, h * 0.5),
            s.floor_color,
        )
        .draw(ctx);

        if rays.is_empty() {
            return;
        }

        let scale = w / rays.len() as f32;
        let half_h = h * 0.5;

        for (i, ray) in rays.iter().enumerate() {
            let column_h = ray.proj_height;
            let x = i as f32 * scale;
            let y = half_h - column_h * 0.5;

            let shade = (s.max_depth / ray.depth).clamp(0.0, 1.0);
            let shade = shade * s.wall_shade_strength + (1.0 - s.wall_shade_strength);
            let shaded = crate::math::Color::new(
                s.wall_color.r * shade,
                s.wall_color.g * shade,
                s.wall_color.b * shade,
                s.wall_color.a,
            );

            Rectangle::new(
                Vec2::new(x, y.max(0.0)),
                Vec2::new(scale + 1.0, column_h.min(h * 2.0)),
                shaded,
            )
            .draw(ctx);
        }
    }
}
