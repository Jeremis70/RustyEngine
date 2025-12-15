#![allow(unused)] // Keep forward-declared APIs without warnings during engine build-out

mod audio;
mod backend;
mod core;
mod math;
mod render;

use crate::backend::winit_backend::WinitBackend;
use crate::core::color::Color;
use crate::core::engine::Engine;
use crate::core::events::Position;
use crate::core::render_context::RenderContext;
use crate::math::vec2::Vec2;
use crate::render::Collider;
use crate::render::Drawable;
use crate::render::Rectangle;
use crate::render::Triangle;
use crate::render::WgpuRenderer;
use crate::render::shapes::Circle;
use crate::render::shapes::Polygon;
use log::error;
use std::sync::Arc;

fn main() {
    env_logger::init();

    let backend = match WinitBackend::try_new() {
        Ok(b) => b,
        Err(e) => {
            error!("Failed to initialize backend: {}", e);
            return;
        }
    };

    // Create a stub renderer and pass it to the engine
    let renderer = WgpuRenderer::new();
    let mut engine = Engine::new(Box::new(backend), Box::new(renderer));

    let config = core::window_config::WindowConfig::builder()
        .width(700)
        .height(400)
        .resizable(true)
        .fullscreen(false)
        .continuous(true) // disabled when target_fps is used
        .build();

    if let Err(e) = engine.create_window(config) {
        error!("Failed to create window: {}", e);
        return;
    }

    let rect = Arc::new(Rectangle::new(
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 50.0),
        Color::RED,
    ));

    let trg = Arc::new(Triangle::new(
        Vec2::new(200.0, 200.0),
        Vec2::new(981.0, 345.0),
        Vec2::new(832.0, 125.0),
        Color::GREEN,
    ));
    let circ = Arc::new(Circle::new(Vec2::new(400.0, 200.0), 75.0, Color::YELLOW));
    let poly = Arc::new(Polygon::new(
        vec![
            Vec2::new(520.0, 60.0),
            Vec2::new(640.0, 140.0),
            Vec2::new(600.0, 210.0),
            Vec2::new(690.0, 260.0),
            Vec2::new(510.0, 320.0),
            Vec2::new(470.0, 170.0),
        ],
        Color::new(0.9, 0.3, 0.8, 1.0),
    ));

    // Simple API: draw using RenderContext in a single callback
    let render_rect = Arc::clone(&rect);
    let render_trg = Arc::clone(&trg);
    let render_circ = Arc::clone(&circ);
    let render_poly = Arc::clone(&poly);
    engine.events.on_render(move |ctx: &mut RenderContext| {
        // Clear the screen each frame
        ctx.clear(Color::BLUE);

        render_rect.draw(ctx);
        render_trg.draw(ctx);
        render_circ.draw(ctx);
        render_poly.draw(ctx);
    });

    let collision_rect = Arc::clone(&rect);
    let collision_trg = Arc::clone(&trg);
    let collision_circ = Arc::clone(&circ);
    let collision_poly = Arc::clone(&poly);
    engine.events.on_mouse_move(move |pos: &Position| {
        if collision_rect.contains_point(Vec2::new(pos.x, pos.y)) {
            println!("La souris est dans le rectangle !");
        }
        if collision_trg.contains_point(Vec2::new(pos.x, pos.y)) {
            println!("La souris est dans le triangle !");
        }
        if collision_circ.contains_point(Vec2::new(pos.x, pos.y)) {
            println!("La souris est dans le cercle !");
        }
        if collision_poly.contains_point(Vec2::new(pos.x, pos.y)) {
            println!("La souris est dans le polygone !");
        }
    });

    let sound = engine
        .audio
        .load(r#"D:\Code\Rust\RustyEngine\toby fox - UNDERTALE Soundtrack - 17 Snowy.flac"#);
    engine.audio.play(sound);

    if let Err(e) = engine.run() {
        error!("Engine run failed: {}", e);
    }
}
