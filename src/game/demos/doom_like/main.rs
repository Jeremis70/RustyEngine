use log::info;
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::window::WindowConfig;
use crate::core::engine::Engine;
use crate::core::engine_state::EngineState;
use crate::graphics::Text;
use crate::math::Color;
use crate::math::Vec2;
use crate::render::Drawable;
use crate::render::context::RenderContext;

use super::map::Map;
use super::object_renderer::ObjectRenderer;
use super::player::Player;
use super::raycasting::RayCasting;
use super::settings;

pub fn install(engine: &mut Engine) {
    info!("Doom-like demo loaded");

    // Doom-like wants FPS mouse look by default.
    // Other demos can omit this and they'll get a normal visible cursor.
    engine.set_window_config(
        WindowConfig::builder()
            .width(700)
            .height(400)
            .resizable(true)
            .fullscreen(false)
            .continuous(true)
            .cursor_grab(true)
            .cursor_visible(false)
            .build(),
    );

    // FPS overlay font
    let font_id = engine
        .assets
        .load_font("src/game/assets/LEMONMILK-Regular.otf", 48.0)
        .expect("Failed to load LEMONMILK-Regular.otf");
    let font_asset = engine
        .assets
        .get_font(font_id)
        .cloned()
        .expect("Font should be available after load");

    let fps_value = Rc::new(RefCell::new(0.0f64));
    let last_fps_int = Rc::new(RefCell::new(u32::MAX));
    let fps_text = Rc::new(RefCell::new({
        let mut t = Text::new(font_id, "FPS: 0", 18, Color::WHITE);
        t.transform.position = Vec2::new(10.0, 10.0);
        t.layout_with_font_asset(&font_asset);
        t
    }));

    let settings = settings::init(settings::Settings::default());

    let map = Rc::new(Map::demo(settings.tile_size));
    let player = Rc::new(RefCell::new(Player::new_from_settings()));
    let raycasting = Rc::new(RefCell::new(RayCasting::new()));
    let renderer = Rc::new(ObjectRenderer::new());

    {
        let map = Rc::clone(&map);
        let player = Rc::clone(&player);
        let fps_value = Rc::clone(&fps_value);
        engine
            .events
            .on_update_with_input(move |state: &EngineState, input| {
                *fps_value.borrow_mut() = state.fps;
                player.borrow_mut().update(state, input, map.as_ref());
            });
    }

    {
        let map = Rc::clone(&map);
        let player = Rc::clone(&player);
        let raycasting = Rc::clone(&raycasting);
        let renderer = Rc::clone(&renderer);
        let fps_value = Rc::clone(&fps_value);
        let last_fps_int = Rc::clone(&last_fps_int);
        let fps_text = Rc::clone(&fps_text);
        engine.events.on_render(move |ctx: &mut RenderContext| {
            ctx.clear(Color::BLACK);

            // 3D view
            raycasting
                .borrow_mut()
                .update(map.as_ref(), &player.borrow(), ctx.size);
            renderer.draw(ctx, raycasting.borrow().rays());

            // FPS overlay (top-left)
            let fps_int = fps_value.borrow().round().max(0.0) as u32;
            let mut last = last_fps_int.borrow_mut();
            if *last != fps_int {
                *last = fps_int;
                let mut t = fps_text.borrow_mut();
                t.content = format!("FPS: {fps_int}");
                t.layout_with_font_asset(&font_asset);
            }
            fps_text.borrow().draw(ctx);
        });
    }
}
