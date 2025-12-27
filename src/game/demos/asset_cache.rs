use log::{error, info};

use crate::core::engine::Engine;
use crate::core::engine_state::EngineState;
use crate::math::Color;
use crate::render::context::RenderContext;

/// Demo that validates asset cache deduplication across images, fonts and sounds.
pub fn install(engine: &mut Engine) {
    info!("AssetCache demo loaded: testing image/font/sound dedup.");

    // Use relative paths (cwd = repo root) and also a ./ variant to test normalization.
    let image_path = "src/game/assets/Player Idle 48x48.png";
    let image_path_dup = "./src/game/assets/Player Idle 48x48.png";

    let font_path = "src/game/assets/LEMONMILK-Regular.otf";
    let font_path_dup = "./src/game/assets/LEMONMILK-Regular.otf";

    let sound_path = "src/game/audio/toby fox - UNDERTALE Soundtrack - 17 Snowy.flac";
    let sound_path_dup = "./src/game/audio/toby fox - UNDERTALE Soundtrack - 17 Snowy.flac";

    // --- Image ---
    match (
        engine.assets.load_image(image_path),
        engine.assets.load_image(image_path_dup),
    ) {
        (Ok(id1), Ok(id2)) => {
            info!(
                "Image load #1 -> id={} (raw={}); load #2 -> id={} (raw={}); same? {}",
                id1,
                id1.as_usize(),
                id2,
                id2.as_usize(),
                id1 == id2
            );
        }
        (Err(e), _) | (_, Err(e)) => {
            error!("Image load failed: {}", e);
        }
    }

    // --- Font ---
    let font_size = 48.0;
    match (
        engine.assets.load_font(font_path, font_size),
        engine.assets.load_font(font_path_dup, font_size),
    ) {
        (Ok(id1), Ok(id2)) => {
            info!(
                "Font load #1 -> id={} (raw={}); load #2 -> id={} (raw={}); same? {}",
                id1,
                id1.as_usize(),
                id2,
                id2.as_usize(),
                id1 == id2
            );
        }
        (Err(e), _) | (_, Err(e)) => {
            error!("Font load failed: {}", e);
        }
    }

    // --- Sound ---
    match (
        engine.assets.load_sound(&mut engine.audio, sound_path),
        engine.assets.load_sound(&mut engine.audio, sound_path_dup),
    ) {
        (Ok(id1), Ok(id2)) => {
            info!(
                "Sound load #1 -> id={} (raw={}); load #2 -> id={} (raw={}); same? {}",
                id1,
                id1.as_usize(),
                id2,
                id2.as_usize(),
                id1 == id2
            );
        }
        (Err(e), _) | (_, Err(e)) => {
            error!("Sound load failed: {}", e);
        }
    }

    engine.events.on_update(|_state: &EngineState| {
        // No-op: this demo is about asset caching.
    });

    engine.events.on_render(|ctx: &mut RenderContext| {
        ctx.clear(Color::BLACK);
    });
}
