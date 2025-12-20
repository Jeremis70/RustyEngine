#![allow(dead_code)] // Engine API still expanding; silence stub warnings for now.

mod audio;
mod backend;
mod core;
mod game;
mod math;
mod render;

use crate::backend::winit_backend::WinitBackend;
use crate::core::engine::Engine;
use crate::core::window_config::WindowConfig;
use crate::render::WgpuRenderer;
use log::error;

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
    let mut engine = match Engine::new(Box::new(backend), Box::new(renderer)) {
        Ok(engine) => engine,
        Err(e) => {
            error!("Failed to initialize audio system: {}", e);
            return;
        }
    };

    let config = WindowConfig::builder()
        .width(700)
        .height(400)
        .resizable(true)
        .fullscreen(false)
        .continuous(true)
        .build();

    if let Err(e) = engine.create_window(config) {
        error!("Failed to create window: {}", e);
        return;
    }

    game::install_active_demo(&mut engine);

    if let Err(e) = engine.run() {
        error!("Engine run failed: {}", e);
    }
}
