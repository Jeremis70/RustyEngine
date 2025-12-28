use super::callbacks::{Callbacks, Mut, Ref2};
use super::input::Input;
use super::input_events::{
    AxisMotionEvent, GestureEvent, ImeEvent, Key, KeyEvent, Modifiers, MouseButtonEvent,
    MouseMotionEvent, MouseWheelDelta, PanEvent, Position, Size, Theme, Touch,
    TouchpadPressureEvent,
};
use crate::backend::surface_provider::SurfaceProvider;
use crate::core::engine_state::EngineState;
use crate::render::context::RenderContext;
use std::path::{Path, PathBuf};

/// Trait used by the backend to invoke events.
pub trait EventHandlerApi {
    /// Called once a native window/display handle is ready for rendering.
    fn on_surface_ready(&mut self, _surface: &dyn SurfaceProvider) {}
    /// Called by the backend to indicate a frame tick. The engine
    /// layer should update its `EngineState` and then invoke `on_update`.
    fn on_tick(&mut self) {}
    fn on_resize(&mut self, _size: &Size) {}
    fn on_move(&mut self, _pos: &(i32, i32)) {}
    fn on_close(&mut self) {}
    fn on_destroy(&mut self) {}
    fn on_focus(&mut self, _focused: &bool) {}
    fn on_scale_factor_changed(&mut self, _scale: &f64) {}
    fn on_theme_changed(&mut self, _theme: &Theme) {}
    fn on_occluded(&mut self, _occluded: &bool) {}

    fn on_key_pressed(&mut self, _ev: &KeyEvent) {}
    fn on_key_released(&mut self, _ev: &KeyEvent) {}
    fn on_modifiers_changed(&mut self, _mods: &Modifiers) {}
    fn on_ime(&mut self, _ime: &ImeEvent) {}

    fn on_mouse_button_pressed(&mut self, _ev: &MouseButtonEvent) {}
    fn on_mouse_button_released(&mut self, _ev: &MouseButtonEvent) {}
    fn on_mouse_move(&mut self, _pos: &Position) {}
    fn on_mouse_motion(&mut self, _ev: &MouseMotionEvent) {}
    fn on_mouse_wheel(&mut self, _delta: &MouseWheelDelta) {}
    fn on_mouse_enter(&mut self) {}
    fn on_mouse_leave(&mut self) {}

    fn on_touch(&mut self, _touch: &Touch) {}

    fn on_pinch(&mut self, _gesture: &GestureEvent) {}
    fn on_pan(&mut self, _pan: &PanEvent) {}
    fn on_rotate(&mut self, _gesture: &GestureEvent) {}
    fn on_double_tap(&mut self) {}
    fn on_touchpad_pressure(&mut self, _ev: &TouchpadPressureEvent) {}

    fn on_file_dropped(&mut self, _path: &Path) {}
    fn on_file_hovered(&mut self, _path: &Path) {}
    fn on_file_hover_cancelled(&mut self) {}

    fn on_axis_motion(&mut self, _ev: &AxisMotionEvent) {}
    fn on_activation_token(&mut self, _token: &str) {}

    fn on_redraw(&mut self) {}
    fn on_update(&mut self, _state: &EngineState) {}
}

/// Orchestrates user callbacks and input state.
///
/// Public contract:
/// - Gameplay reads input via polling during `on_update` (pygame-like).
/// - Event callbacks (on_key_pressed/on_mouse_move/...) remain available, but are
///   optional and should not be required for typical gameplay logic.
///
/// Per-frame ordering (conceptual):
/// - One-frame input state (just_pressed/just_released/mouse_delta) is cleared at
///   the end of the frame.
/// - The next frame collects OS events into one-frame state.
/// - `on_update` runs and gameplay polls input.
/// - `on_redraw` runs after `on_update` (rendering only).
pub struct EventHandler {
    // === WINDOW ===
    on_resize: Callbacks<Size>,
    on_move: Callbacks<(i32, i32)>,
    on_close: Callbacks<()>,
    on_destroy: Callbacks<()>,
    on_focus: Callbacks<bool>,
    on_scale_factor_changed: Callbacks<f64>,
    on_theme_changed: Callbacks<Theme>,
    on_occluded: Callbacks<bool>, // Window fully hidden

    // === KEYBOARD ===
    on_key_pressed: Callbacks<KeyEvent>,
    on_key_released: Callbacks<KeyEvent>,
    pub on_modifiers_changed: Callbacks<Modifiers>,
    pub on_ime: Callbacks<ImeEvent>, // Input Method Editor

    // === MOUSE ===
    on_mouse_button_pressed: Callbacks<MouseButtonEvent>,
    on_mouse_button_released: Callbacks<MouseButtonEvent>,
    pub on_mouse_move: Callbacks<Position>,
    pub on_mouse_motion: Callbacks<MouseMotionEvent>,
    pub on_mouse_wheel: Callbacks<MouseWheelDelta>,
    pub on_mouse_enter: Callbacks<()>,
    pub on_mouse_leave: Callbacks<()>,

    // === TOUCH ===
    pub on_touch: Callbacks<Touch>,

    // === GESTURES ===
    pub on_pinch: Callbacks<GestureEvent>,  // Pinch zoom
    pub on_pan: Callbacks<PanEvent>,        // Swipe/scroll
    pub on_rotate: Callbacks<GestureEvent>, // Rotation
    pub on_double_tap: Callbacks<()>,       // Double tap
    on_touchpad_pressure: Callbacks<TouchpadPressureEvent>, // Pressure + stage

    // === FILE DROP ===
    pub on_file_dropped: Callbacks<PathBuf>,
    pub on_file_hovered: Callbacks<PathBuf>,
    pub on_file_hover_cancelled: Callbacks<()>,

    // === GAMEPAD/JOYSTICK ===
    on_axis_motion: Callbacks<AxisMotionEvent>, // axis_id, value

    // === SPECIAL ===
    on_activation_token: Callbacks<String>, // Wayland activation token

    // === GAME LOOP ===
    pub on_redraw: Callbacks<()>,
    on_update: Callbacks<(EngineState, Input), Ref2>,

    // === RENDER CONTEXT CALLBACKS ===
    pub on_render: Callbacks<RenderContext, Mut>,

    // === INPUT SNAPSHOT ===
    pub on_keys_state_changed: Callbacks<Vec<Key>>,

    // === INTERNAL STATE ===
    current_modifiers: Modifiers,
    pub input: Input,
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            on_resize: Callbacks::new(),
            on_move: Callbacks::new(),
            on_close: Callbacks::new(),
            on_destroy: Callbacks::new(),
            on_focus: Callbacks::new(),
            on_scale_factor_changed: Callbacks::new(),
            on_theme_changed: Callbacks::new(),
            on_occluded: Callbacks::new(),
            on_key_pressed: Callbacks::new(),
            on_key_released: Callbacks::new(),
            on_modifiers_changed: Callbacks::new(),
            on_ime: Callbacks::new(),
            on_mouse_button_pressed: Callbacks::new(),
            on_mouse_button_released: Callbacks::new(),
            on_mouse_move: Callbacks::new(),
            on_mouse_motion: Callbacks::new(),
            on_mouse_wheel: Callbacks::new(),
            on_mouse_enter: Callbacks::new(),
            on_mouse_leave: Callbacks::new(),
            on_touch: Callbacks::new(),
            on_pinch: Callbacks::new(),
            on_pan: Callbacks::new(),
            on_rotate: Callbacks::new(),
            on_double_tap: Callbacks::new(),
            on_touchpad_pressure: Callbacks::new(),
            on_file_dropped: Callbacks::new(),
            on_file_hovered: Callbacks::new(),
            on_file_hover_cancelled: Callbacks::new(),
            on_axis_motion: Callbacks::new(),
            on_activation_token: Callbacks::new(),
            on_redraw: Callbacks::new(),
            on_update: Callbacks::new(),
            on_render: Callbacks::new(),
            on_keys_state_changed: Callbacks::new(),
            current_modifiers: Modifiers::default(),
            input: Input::new(),
        }
    }

    // === STATE QUERIES ===
    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.input.key(key)
    }

    pub fn pressed_keys(&self) -> impl Iterator<Item = &Key> {
        self.input.pressed_keys_iter()
    }

    pub fn modifiers(&self) -> Modifiers {
        self.current_modifiers
    }

    pub fn input(&self) -> &Input {
        &self.input
    }

    // === Registration helper methods ===
    pub fn on_close<F: FnMut(&()) + 'static>(&mut self, f: F) -> usize {
        self.on_close.add(f)
    }
    pub fn on_key_pressed<F: FnMut(&KeyEvent) + 'static>(&mut self, f: F) -> usize {
        self.on_key_pressed.add(f)
    }
    pub fn on_key_released<F: FnMut(&KeyEvent) + 'static>(&mut self, f: F) -> usize {
        self.on_key_released.add(f)
    }
    pub fn on_mouse_button_pressed<F: FnMut(&MouseButtonEvent) + 'static>(
        &mut self,
        f: F,
    ) -> usize {
        self.on_mouse_button_pressed.add(f)
    }
    pub fn on_mouse_button_released<F: FnMut(&MouseButtonEvent) + 'static>(
        &mut self,
        f: F,
    ) -> usize {
        self.on_mouse_button_released.add(f)
    }
    pub fn on_mouse_move<F: FnMut(&Position) + 'static>(&mut self, f: F) -> usize {
        self.on_mouse_move.add(f)
    }

    pub fn on_mouse_motion<F: FnMut(&MouseMotionEvent) + 'static>(&mut self, f: F) -> usize {
        self.on_mouse_motion.add(f)
    }
    pub fn on_mouse_wheel<F: FnMut(&MouseWheelDelta) + 'static>(&mut self, f: F) -> usize {
        self.on_mouse_wheel.add(f)
    }
    pub fn on_redraw<F: FnMut(&()) + 'static>(&mut self, f: F) -> usize {
        self.on_redraw.add(f)
    }
    pub fn on_render<F: FnMut(&mut RenderContext) + 'static>(&mut self, f: F) -> usize {
        self.on_render.add(f)
    }
    /// Primary gameplay hook: update logic (poll input here).
    pub fn on_update<F: FnMut(&EngineState) + 'static>(&mut self, mut f: F) -> usize {
        self.on_update.add(move |state, _input| f(state))
    }

    /// Variant that also provides read-only access to Input for polling.
    pub fn on_update_with_input<F: FnMut(&EngineState, &Input) + 'static>(
        &mut self,
        f: F,
    ) -> usize {
        self.on_update.add(f)
    }
    pub fn on_keys_state_changed<F: FnMut(&Vec<Key>) + 'static>(&mut self, f: F) -> usize {
        self.on_keys_state_changed.add(f)
    }
    pub fn on_modifiers_changed<F: FnMut(&Modifiers) + 'static>(&mut self, f: F) -> usize {
        self.on_modifiers_changed.add(f)
    }
    pub fn on_ime<F: FnMut(&ImeEvent) + 'static>(&mut self, f: F) -> usize {
        self.on_ime.add(f)
    }
    pub fn on_mouse_enter<F: FnMut(&()) + 'static>(&mut self, f: F) -> usize {
        self.on_mouse_enter.add(f)
    }
    pub fn on_mouse_leave<F: FnMut(&()) + 'static>(&mut self, f: F) -> usize {
        self.on_mouse_leave.add(f)
    }
    pub fn on_resize<F: FnMut(&Size) + 'static>(&mut self, f: F) -> usize {
        self.on_resize.add(f)
    }
    pub fn on_move<F: FnMut(&(i32, i32)) + 'static>(&mut self, f: F) -> usize {
        self.on_move.add(f)
    }
    pub fn on_destroy<F: FnMut(&()) + 'static>(&mut self, f: F) -> usize {
        self.on_destroy.add(f)
    }
    pub fn on_focus<F: FnMut(&bool) + 'static>(&mut self, f: F) -> usize {
        self.on_focus.add(f)
    }
    pub fn on_scale_factor_changed<F: FnMut(&f64) + 'static>(&mut self, f: F) -> usize {
        self.on_scale_factor_changed.add(f)
    }
    pub fn on_theme_changed<F: FnMut(&Theme) + 'static>(&mut self, f: F) -> usize {
        self.on_theme_changed.add(f)
    }
    pub fn on_occluded<F: FnMut(&bool) + 'static>(&mut self, f: F) -> usize {
        self.on_occluded.add(f)
    }
    pub fn on_touch<F: FnMut(&Touch) + 'static>(&mut self, f: F) -> usize {
        self.on_touch.add(f)
    }
    pub fn on_pinch<F: FnMut(&GestureEvent) + 'static>(&mut self, f: F) -> usize {
        self.on_pinch.add(f)
    }
    pub fn on_pan<F: FnMut(&PanEvent) + 'static>(&mut self, f: F) -> usize {
        self.on_pan.add(f)
    }
    pub fn on_rotate<F: FnMut(&GestureEvent) + 'static>(&mut self, f: F) -> usize {
        self.on_rotate.add(f)
    }
    pub fn on_double_tap<F: FnMut(&()) + 'static>(&mut self, f: F) -> usize {
        self.on_double_tap.add(f)
    }
    pub fn on_touchpad_pressure<F: FnMut(&TouchpadPressureEvent) + 'static>(
        &mut self,
        f: F,
    ) -> usize {
        self.on_touchpad_pressure.add(f)
    }
    pub fn on_file_dropped<F: FnMut(&PathBuf) + 'static>(&mut self, f: F) -> usize {
        self.on_file_dropped.add(f)
    }
    pub fn on_file_hovered<F: FnMut(&PathBuf) + 'static>(&mut self, f: F) -> usize {
        self.on_file_hovered.add(f)
    }
    pub fn on_file_hover_cancelled<F: FnMut(&()) + 'static>(&mut self, f: F) -> usize {
        self.on_file_hover_cancelled.add(f)
    }
    pub fn on_axis_motion<F: FnMut(&AxisMotionEvent) + 'static>(&mut self, f: F) -> usize {
        self.on_axis_motion.add(f)
    }
    pub fn on_activation_token<F: FnMut(&String) + 'static>(&mut self, f: F) -> usize {
        self.on_activation_token.add(f)
    }
}

/// Implementation of EventHandlerApi for EventHandler
impl EventHandlerApi for EventHandler {
    fn on_resize(&mut self, size: &Size) {
        self.on_resize.invoke(size);
    }

    fn on_move(&mut self, pos: &(i32, i32)) {
        self.on_move.invoke(pos);
    }

    fn on_close(&mut self) {
        self.on_close.invoke(&());
    }

    fn on_destroy(&mut self) {
        self.on_destroy.invoke(&());
    }

    fn on_focus(&mut self, focused: &bool) {
        self.on_focus.invoke(focused);
    }

    fn on_scale_factor_changed(&mut self, scale: &f64) {
        self.on_scale_factor_changed.invoke(scale);
    }

    fn on_theme_changed(&mut self, theme: &Theme) {
        self.on_theme_changed.invoke(theme);
    }

    fn on_occluded(&mut self, occluded: &bool) {
        self.on_occluded.invoke(occluded);
    }

    fn on_key_pressed(&mut self, ev: &KeyEvent) {
        self.input.on_key_pressed(ev.key);
        self.on_key_pressed.invoke(ev);
        self.on_keys_state_changed
            .invoke(&self.input.pressed_keys_list());
    }

    fn on_key_released(&mut self, ev: &KeyEvent) {
        self.input.on_key_released(ev.key);
        self.on_key_released.invoke(ev);
        self.on_keys_state_changed
            .invoke(&self.input.pressed_keys_list());
    }

    fn on_modifiers_changed(&mut self, mods: &Modifiers) {
        self.current_modifiers = *mods;
        self.input.on_modifiers_changed(*mods);
        self.on_modifiers_changed.invoke(mods);
    }

    fn on_ime(&mut self, ime: &ImeEvent) {
        self.on_ime.invoke(ime);
    }

    fn on_mouse_button_pressed(&mut self, ev: &MouseButtonEvent) {
        self.input.on_mouse_button_pressed(ev.button);
        self.on_mouse_button_pressed.invoke(ev);
    }

    fn on_mouse_button_released(&mut self, ev: &MouseButtonEvent) {
        self.input.on_mouse_button_released(ev.button);
        self.on_mouse_button_released.invoke(ev);
    }

    fn on_mouse_move(&mut self, pos: &Position) {
        let last = self.input.mouse_position();
        self.input.on_mouse_move(*pos, last);
        self.on_mouse_move.invoke(pos);
    }

    fn on_mouse_motion(&mut self, ev: &MouseMotionEvent) {
        self.input.on_mouse_motion(ev.delta_x, ev.delta_y);
        self.on_mouse_motion.invoke(ev);
    }

    fn on_mouse_wheel(&mut self, delta: &MouseWheelDelta) {
        self.on_mouse_wheel.invoke(delta);
    }

    fn on_mouse_enter(&mut self) {
        self.on_mouse_enter.invoke(&());
    }

    fn on_mouse_leave(&mut self) {
        self.on_mouse_leave.invoke(&());
    }

    fn on_touch(&mut self, touch: &Touch) {
        self.on_touch.invoke(touch);
    }

    fn on_pinch(&mut self, gesture: &GestureEvent) {
        self.on_pinch.invoke(gesture);
    }

    fn on_pan(&mut self, pan: &PanEvent) {
        self.on_pan.invoke(pan);
    }

    fn on_rotate(&mut self, gesture: &GestureEvent) {
        self.on_rotate.invoke(gesture);
    }

    fn on_double_tap(&mut self) {
        self.on_double_tap.invoke(&());
    }

    fn on_touchpad_pressure(&mut self, ev: &TouchpadPressureEvent) {
        self.on_touchpad_pressure.invoke(ev);
    }

    fn on_file_dropped(&mut self, path: &Path) {
        self.on_file_dropped.invoke(&path.to_path_buf());
    }

    fn on_file_hovered(&mut self, path: &Path) {
        self.on_file_hovered.invoke(&path.to_path_buf());
    }

    fn on_file_hover_cancelled(&mut self) {
        self.on_file_hover_cancelled.invoke(&());
    }

    fn on_axis_motion(&mut self, ev: &AxisMotionEvent) {
        self.on_axis_motion.invoke(ev);
    }

    fn on_activation_token(&mut self, token: &str) {
        self.on_activation_token.invoke(&token.to_string());
    }

    fn on_redraw(&mut self) {
        log::trace!("render: begin");
        self.on_redraw.invoke(&());
        log::trace!("render: end");
    }
    fn on_update(&mut self, state: &EngineState) {
        // Note: per-frame input state (just_pressed/just_released/mouse_delta) is
        // cleared after rendering (end of frame) so polling in on_update sees the
        // events collected since the last frame.
        log::trace!("update: begin");
        // Derive action states from current raw input before gameplay polls.
        self.input.update_actions();
        self.on_update.invoke(state, &self.input);
        log::trace!("update: end");
    }
}
