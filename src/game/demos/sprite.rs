use log::info;

use crate::math::color::Color;
use crate::core::engine::Engine;
use crate::core::engine_state::EngineState;
use crate::render::context::RenderContext;
use crate::graphics::Sprite;
use crate::math::vec2::Vec2;

/// Simple demo that loads an image via the asset manager and displays it as a sprite.
pub fn install(engine: &mut Engine) {
    info!("Sprite demo loaded: image + sprite test.");

    // Load an image from disk (PNG/JPEG/BMP are supported by the image crate).
    // If the file is missing or invalid, the demo will just show a black screen.
    match engine.assets.load_image(
        r#"D:\Code\Rust\RustyEngine\src\game\assets\Player Idle 48x48.png"#,
    ) {
        Ok(image_id) => {
            match engine.assets.get_image(image_id) {
                Some(image) => {
                    let mut sprite = Sprite::from_image(image_id, image);
                    // Place the sprite roughly at the center of the default window (700x400 in main.rs).
                    sprite.position = Vec2::new(350.0, 200.0);

                    engine.events.on_update(|_state: &EngineState| {
                        // Game logic could update the sprite here (animation, movement, etc.).
                    });

                    engine.events.on_render(move |ctx: &mut RenderContext| {
                        ctx.clear(Color::BLACK);
                        ctx.draw_sprite(&sprite);
                    });
                }
                None => {
                    log::error!("Failed to retrieve loaded sprite image");
                    engine.events.on_render(|ctx: &mut RenderContext| {
                        ctx.clear(Color::BLACK);
                    });
                }
            }
        }
        Err(e) => {
            log::error!("Failed to load sprite image: {}", e);
            engine.events.on_render(|ctx: &mut RenderContext| {
                ctx.clear(Color::BLACK);
            });
        }
    }
}
