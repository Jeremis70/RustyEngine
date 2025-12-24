use log::info;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::core::assets::{SpriteOrder, SpritesheetConfig};
use crate::core::engine::Engine;
use crate::core::engine_state::EngineState;
use crate::graphics::{AnimatedSprite, Animation, Sprite};
use crate::math::color::Color;
use crate::math::vec2::Vec2;
use crate::render::Drawable;
use crate::render::context::RenderContext;

/// Simple demo that loads a spritesheet and animates it.
pub fn install(engine: &mut Engine) {
    info!("Sprite demo loaded: animated spritesheet.");

    // --- Spritesheet config ---
    let config = SpritesheetConfig {
        columns: 10,
        rows: 1,
        sprite_width: 48,
        sprite_height: 48,
        order: SpriteOrder::LeftToRightTopToBottom,
        spacing: 0,
        margin: 0,
    };

    let sprite_ids = engine
        .assets
        .load_spritesheet(
            r#"D:\Code\Rust\RustyEngine\src\game\assets\Player Idle 48x48.png"#,
            config,
        )
        .unwrap();

    // --- Create manual animated sprite (old method) ---
    let image = engine.assets.get_image(sprite_ids[0]).unwrap();
    let mut sprite = Sprite::from_image(sprite_ids[0], image);
    sprite.position = Vec2::new(250.0, 200.0);

    // --- Shared sprite state ---
    let sprite = Rc::new(RefCell::new(sprite));

    // --- Create AnimatedSprite (new method) ---
    let animation = Animation::looping(&sprite_ids, Duration::from_millis(100));

    let mut animated_sprite = AnimatedSprite::new(animation, 48, 48);
    animated_sprite.position = Vec2::new(450.0, 200.0);
    let animated_sprite = Rc::new(RefCell::new(animated_sprite));

    // --- Animation state ---
    let mut current_frame: usize = 0;
    let mut last_switch = Instant::now();
    let frame_duration = Duration::from_millis(100);

    // --- Update manual sprite ---
    {
        let sprite = Rc::clone(&sprite);
        let sprite_ids = sprite_ids.clone();

        engine.events.on_update(move |_state: &EngineState| {
            if last_switch.elapsed() >= frame_duration {
                last_switch = Instant::now();
                current_frame = (current_frame + 1) % sprite_ids.len();

                sprite.borrow_mut().image_id = sprite_ids[current_frame];
            }
        });
    }

    // --- Update AnimatedSprite ---
    {
        let animated_sprite = Rc::clone(&animated_sprite);

        engine.events.on_update(move |state: &EngineState| {
            animated_sprite.borrow_mut().update(state.delta_time);
        });
    }

    // --- Render ---
    {
        let sprite = Rc::clone(&sprite);
        let animated_sprite = Rc::clone(&animated_sprite);

        engine.events.on_render(move |ctx: &mut RenderContext| {
            ctx.clear(Color::BLACK);

            // Manual animation (left)
            sprite.borrow().draw(ctx);

            // AnimatedSprite (right)
            animated_sprite.borrow().draw(ctx);
        });
    }
}
