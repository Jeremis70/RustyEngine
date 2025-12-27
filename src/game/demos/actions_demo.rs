use log::info;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::core::engine::Engine;
use crate::core::engine_state::EngineState;
use crate::core::events::{Binding, Key, MouseButton, Trigger};
use crate::math::Color;
use crate::math::vec2::Vec2;
use crate::render::Drawable;
use crate::render::context::RenderContext;
use crate::render::shapes::Rectangle;

/// Demo: minimal "Actions" (AnyOf/AllOf + buffering) driven by polling.
pub fn install(engine: &mut Engine) {
    info!("Actions demo loaded: bindings + chords + buffering.");

    // --- Configure actions ---
    let (move_group, left, right, run_left, run_right, jump) = {
        let actions = engine.events.input.actions_mut();

        let move_group = actions.group("move");

        let left = actions.action("left");
        actions.bind(
            left,
            Binding::AnyOf(vec![
                Binding::Trigger(Trigger::Key(Key::A)),
                Binding::Trigger(Trigger::Key(Key::Left)),
            ]),
        );
        actions.set_group_priority(move_group, left, 0);

        let right = actions.action("right");
        actions.bind(
            right,
            Binding::AnyOf(vec![
                Binding::Trigger(Trigger::Key(Key::D)),
                Binding::Trigger(Trigger::Key(Key::Right)),
            ]),
        );
        actions.set_group_priority(move_group, right, 0);

        let run_left = actions.action("run_left");
        actions.bind(
            run_left,
            Binding::AnyOf(vec![
                Binding::AllOf(vec![
                    Binding::Trigger(Trigger::Key(Key::LShift)),
                    Binding::Trigger(Trigger::Key(Key::A)),
                ]),
                Binding::AllOf(vec![
                    Binding::Trigger(Trigger::Key(Key::LShift)),
                    Binding::Trigger(Trigger::Key(Key::Left)),
                ]),
            ]),
        );
        actions.set_group_priority(move_group, run_left, 10);

        let run_right = actions.action("run_right");
        actions.bind(
            run_right,
            Binding::AnyOf(vec![
                Binding::AllOf(vec![
                    Binding::Trigger(Trigger::Key(Key::LShift)),
                    Binding::Trigger(Trigger::Key(Key::D)),
                ]),
                Binding::AllOf(vec![
                    Binding::Trigger(Trigger::Key(Key::LShift)),
                    Binding::Trigger(Trigger::Key(Key::Right)),
                ]),
            ]),
        );
        actions.set_group_priority(move_group, run_right, 10);

        let jump = actions.action("jump");
        actions.bind(
            jump,
            Binding::AnyOf(vec![
                Binding::Trigger(Trigger::Key(Key::Space)),
                Binding::Trigger(Trigger::MouseButton(MouseButton::Left)),
            ]),
        );

        (move_group, left, right, run_left, run_right, jump)
    };

    // --- Simple moving rectangle ---
    let player = Rc::new(RefCell::new(Rectangle::new(
        Vec2::new(320.0, 180.0),
        Vec2::new(40.0, 40.0),
        Color::WHITE,
    )));

    // Track buffered jump window for debug visualization/logging.
    let jump_buffer_active = Rc::new(RefCell::new(false));
    let jump_flash_until = Rc::new(RefCell::new(Instant::now()));

    // --- Update (poll actions here) ---
    {
        let player = Rc::clone(&player);
        let jump_buffer_active = Rc::clone(&jump_buffer_active);
        let jump_flash_until = Rc::clone(&jump_flash_until);

        engine
            .events
            .on_update_with_input(move |state: &EngineState, input| {
                let dt = state.delta_seconds();

                let walk_speed = 220.0;
                let run_speed = 360.0;

                let active = input.actions().active_in_group(move_group);
                let mut vx = 0.0;

                if active.contains(&run_left) {
                    vx -= run_speed;
                } else if active.contains(&left) {
                    vx -= walk_speed;
                }

                if active.contains(&run_right) {
                    vx += run_speed;
                } else if active.contains(&right) {
                    vx += walk_speed;
                }

                if vx != 0.0 {
                    player.borrow_mut().transform.position.x += vx * dt;
                }

                // Visual feedback + buffering primitive.
                // "Buffered" means: pressed in the last 120ms even if not down now.
                let buffered = input.action_was_pressed_within(jump, Duration::from_millis(120));

                {
                    let mut was_active = jump_buffer_active.borrow_mut();
                    if buffered && !*was_active {
                        info!("jump buffered (<=120ms)");
                        *was_active = true;
                    } else if !buffered && *was_active {
                        *was_active = false;
                    }
                }

                if input.action_just_pressed(jump) {
                    *jump_flash_until.borrow_mut() = Instant::now() + Duration::from_millis(120);
                    info!("jump just_pressed");
                }
            });
    }

    // --- Render ---
    {
        let player = Rc::clone(&player);
        let jump_flash_until = Rc::clone(&jump_flash_until);

        engine.events.on_render(move |ctx: &mut RenderContext| {
            ctx.clear(Color::BLACK);

            let mut p = player.borrow_mut();
            if Instant::now() < *jump_flash_until.borrow() {
                p.color = Color::rgb(255, 200, 0);
            } else {
                p.color = Color::WHITE;
            }
            p.draw(ctx);
        });
    }
}
