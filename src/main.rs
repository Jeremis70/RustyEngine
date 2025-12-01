mod backend;
mod core;

use crate::backend::winit_backend::WinitBackend;
use crate::core::engine::Engine;
use crate::core::engine_state::EngineState;
use crate::core::events::Key;
use log::error;
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    env_logger::init();

    let backend = match WinitBackend::try_new() {
        Ok(b) => b,
        Err(e) => {
            error!("Failed to initialize backend: {}", e);
            return;
        }
    };

    let mut engine = Engine::new(Box::new(backend));

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

    // Shared player position and pressed-keys snapshot.
    let player = Rc::new(RefCell::new((0.0f32, 0.0f32)));
    let keys_snapshot = Rc::new(RefCell::new(Vec::<Key>::new()));

    // Keep an up-to-date snapshot of currently pressed keys.
    {
        let keys_snapshot = keys_snapshot.clone();
        engine.events.on_keys_state_changed(move |keys: &Vec<Key>| {
            *keys_snapshot.borrow_mut() = keys.clone();
        });
    }

    // Per-frame update: read the snapshot and update player position.
    {
        let player = player.clone();
        let keys_snapshot = keys_snapshot.clone();
        engine.events.on_update(move |state: &EngineState| {
            let input_keys = keys_snapshot.borrow();
            let dt = state.delta_seconds();

            let mut velocity = (0.0f32, 0.0f32);
            if input_keys.contains(&Key::W) {
                velocity.1 -= 1.0;
            }
            if input_keys.contains(&Key::S) {
                velocity.1 += 1.0;
            }
            if input_keys.contains(&Key::A) {
                velocity.0 -= 1.0;
            }
            if input_keys.contains(&Key::D) {
                velocity.0 += 1.0;
            }

            if velocity != (0.0, 0.0) {
                let speed =
                    if input_keys.contains(&Key::LShift) || input_keys.contains(&Key::RShift) {
                        200.0
                    } else {
                        100.0
                    };
                let mut p = player.borrow_mut();
                p.0 += velocity.0 * speed * dt;
                p.1 += velocity.1 * speed * dt;

                println!("Player: ({:.1}, {:.1}) | Speed: {}", p.0, p.1, speed);
            }
        });
    }

    if let Err(e) = engine.run() {
        error!("Engine run failed: {}", e);
    }
}
