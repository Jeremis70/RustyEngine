use std::cell::RefCell;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use log::info;

use crate::backend::window::WindowConfig;
use crate::core::assets::ImageAsset;
use crate::core::engine::Engine;
use crate::core::engine_state::EngineState;
use crate::core::events::{MouseButton, MouseWheelDelta};
use crate::math::color::Color;
use crate::math::vec2::Vec2;
use crate::render::Drawable;
use crate::render::context::RenderContext;
use crate::render::shapes::Circle;

use super::grass::{GrassAssets, GrassManager, SimpleRng};

const WINDOW_SIZE: (u32, u32) = (600, 600);
const LOGICAL_SIZE: (u32, u32) = (300, 300);
const SCALE: f32 = 2.0;

pub fn install(engine: &mut Engine) {
    info!("Grass demo loaded");

    engine.set_window_config(
        WindowConfig::builder()
            .width(WINDOW_SIZE.0)
            .height(WINDOW_SIZE.1)
            .resizable(false)
            .fullscreen(false)
            .continuous(true)
            .cursor_visible(true)
            .cursor_grab(false)
            .build(),
    );

    // Load blade images (match pygame demo IDs 0..5).
    let mut blades: Vec<ImageAsset> = Vec::new();
    for i in 0..=5 {
        let path = format!("src/game/assets/grass/grass_{i}.png");
        let id = engine
            .assets
            .load_image(&path)
            .unwrap_or_else(|e| panic!("Failed to load {path}: {e}"));
        let asset = engine
            .assets
            .get_image(id)
            .cloned()
            .expect("image should be available after load");
        blades.push(asset);
    }

    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;

    let scene = Rc::new(RefCell::new(Scene::new(blades, seed)));

    // Mouse wheel -> brush size.
    {
        let scene = Rc::clone(&scene);
        engine
            .events
            .on_mouse_wheel(move |delta: &MouseWheelDelta| {
                let mut s = scene.borrow_mut();
                let dir = match *delta {
                    MouseWheelDelta::Lines(v) => v,
                    MouseWheelDelta::Pixels(v) => v / 50.0,
                };
                if dir > 0.0 {
                    s.brush_size = (s.brush_size + 0.1).min(1.0);
                } else if dir < 0.0 {
                    s.brush_size = (s.brush_size - 0.1).max(0.1);
                }
            });
    }

    // Update: camera + forces + placement + time.
    {
        let scene = Rc::clone(&scene);
        engine
            .events
            .on_update_with_input(move |state: &EngineState, input| {
                let dt = state.delta_seconds();
                let mut s = scene.borrow_mut();
                s.dt = dt;

                let mp = input.mouse_position();
                let mx = mp.x / SCALE;
                let my = mp.y / SCALE;
                s.mouse_logical = Vec2::new(mx, my);

                // Move camera based on mouse position.
                if (mx / LOGICAL_SIZE.0 as f32) < 0.2 {
                    s.scroll.x -= s.camera_speed * dt;
                }
                if (mx / LOGICAL_SIZE.0 as f32) > 0.8 {
                    s.scroll.x += s.camera_speed * dt;
                }
                if (my / LOGICAL_SIZE.1 as f32) < 0.2 {
                    s.scroll.y -= s.camera_speed * dt;
                }
                if (my / LOGICAL_SIZE.1 as f32) > 0.8 {
                    s.scroll.y += s.camera_speed * dt;
                }

                s.clicking = input.mouse_button(MouseButton::Left);

                // Apply force at mouse position (world coords).
                let scroll = s.scroll;
                let brush_size = s.brush_size;
                s.gm.apply_force(
                    (mx + scroll.x, my + scroll.y),
                    10.0 * brush_size,
                    25.0 * brush_size,
                );

                // Increment master time.
                s.t += dt * 100.0;

                // Place new tiles while clicking.
                if s.clicking {
                    let scroll = s.scroll;
                    let tile_size = s.gm.tile_size as f32;
                    let tx = ((mx + scroll.x) / tile_size).floor() as i32;
                    let ty = ((my + scroll.y) / tile_size).floor() as i32;

                    let brush_size = s.brush_size;
                    let density = (s.rng.gen_f32() * 12.0 * brush_size + 1.0) as u32;
                    {
                        let Scene { gm, rng, .. } = &mut *s;
                        gm.place_tile((tx, ty), density, vec![0, 1, 2, 3, 5], rng);
                    }

                    if (brush_size - 1.0).abs() < f32::EPSILON {
                        let offsets = [
                            (-1, 0),
                            (-1, -1),
                            (0, -1),
                            (1, -1),
                            (1, 0),
                            (1, 1),
                            (0, 1),
                            (-1, 1),
                        ];
                        for (ox, oy) in offsets {
                            let density = (s.rng.gen_f32() * 14.0 + 3.0) as u32;
                            let Scene { gm, rng, .. } = &mut *s;
                            gm.place_tile((tx + ox, ty + oy), density, vec![0, 1, 2, 3, 5], rng);
                        }
                    }
                }
            });
    }

    // Render: generate tile textures as needed and draw.
    {
        let scene = Rc::clone(&scene);
        engine
            .events
            .on_render_with_assets(move |ctx: &mut RenderContext, assets| {
                let mut s = scene.borrow_mut();
                ctx.clear(Color::rgb(27, 66, 52));

                let offset = (s.scroll.x, s.scroll.y);
                let t = s.t;
                let dt = s.dt;

                let rot_fn = |x: f32, _y: f32| -> i32 {
                    // Match Python: int(sin(t / 60 + x / 100) * 15)
                    ((t / 60.0 + x / 100.0).sin() * 15.0) as i32
                };

                s.gm.update_render(ctx, assets, dt, offset, LOGICAL_SIZE, SCALE, Some(&rot_fn));

                // Cursor circles (match pygame demo look using 2 circles).
                let mx = s.mouse_logical.x;
                let my = s.mouse_logical.y;
                let r = 10.0 * s.brush_size;

                // Convert to screen coords.
                let center = Vec2::new(mx * SCALE, my * SCALE);

                if !s.clicking {
                    // Ring thickness ~2
                    let mut outer = Circle::new(center, r * SCALE, Color::WHITE);
                    outer.segments = 64;
                    outer.draw(ctx);

                    let mut inner =
                        Circle::new(center, (r - 2.0).max(0.0) * SCALE, Color::rgb(27, 66, 52));
                    inner.segments = 64;
                    inner.draw(ctx);
                } else {
                    // Filled inner (r-2)
                    let mut fill = Circle::new(center, (r - 2.0).max(0.0) * SCALE, Color::WHITE);
                    fill.segments = 64;
                    fill.draw(ctx);

                    // Thin-ish outer ring (approx) at radius r
                    let mut outer = Circle::new(center, r * SCALE, Color::WHITE);
                    outer.segments = 64;
                    outer.draw(ctx);

                    let mut hole =
                        Circle::new(center, (r - 1.0).max(0.0) * SCALE, Color::rgb(27, 66, 52));
                    hole.segments = 64;
                    hole.draw(ctx);
                }
            });
    }
}

struct Scene {
    gm: GrassManager,
    rng: SimpleRng,

    scroll: Vec2,
    camera_speed: f32,

    clicking: bool,
    brush_size: f32,

    t: f32,
    dt: f32,

    mouse_logical: Vec2,
}

impl Scene {
    fn new(blades: Vec<ImageAsset>, seed: u64) -> Self {
        let grass_assets = GrassAssets::new(blades);
        let mut gm = GrassManager::new(grass_assets, 10, 100, 600.0, 5, (0.0, 1.0), 13);
        gm.enable_ground_shadows(40, 4, (0, 0, 1), (1, 2));

        let mut rng = SimpleRng::new(seed);

        // Fill base square (match pygame demo).
        for y in 0..20 {
            let y = y + 5;
            for x in 0..20 {
                let x = x + 5;
                let v = rng.gen_f32();
                if v > 0.1 {
                    gm.place_tile((x, y), (v * 12.0) as u32, vec![0, 1, 2, 3, 4], &mut rng);
                }
            }
        }

        Self {
            gm,
            rng,
            scroll: Vec2::new(0.0, 0.0),
            camera_speed: 170.0,
            clicking: false,
            brush_size: 1.0,
            t: 0.0,
            dt: 0.0,
            mouse_logical: Vec2::new(0.0, 0.0),
        }
    }
}
