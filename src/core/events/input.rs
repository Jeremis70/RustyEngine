use super::input_events::{Key, Modifiers, MouseButton, Position};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Input {
    // Keyboard
    pressed_keys: HashSet<Key>,
    just_pressed_keys: HashSet<Key>,
    just_released_keys: HashSet<Key>,
    modifiers: Modifiers,

    // Mouse
    pressed_buttons: HashSet<MouseButton>,
    just_pressed_buttons: HashSet<MouseButton>,
    just_released_buttons: HashSet<MouseButton>,
    mouse_position: Position,
    mouse_delta: (f32, f32),
}

impl Input {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            just_pressed_keys: HashSet::new(),
            just_released_keys: HashSet::new(),
            modifiers: Modifiers::default(),
            pressed_buttons: HashSet::new(),
            just_pressed_buttons: HashSet::new(),
            just_released_buttons: HashSet::new(),
            mouse_position: Position { x: 0.0, y: 0.0 },
            mouse_delta: (0.0, 0.0),
        }
    }

    // === FRAME STATE MANAGEMENT ===

    /// Called at START of each frame - clears one-frame states
    pub fn frame_reset(&mut self) {
        self.clear_frame_state();
    }

    /// Check if key is held DOWN (including this frame)
    pub fn is_key_held(&self, key: Key) -> bool {
        self.pressed_keys.contains(&key)
    }

    /// Check if key was PRESSED this frame only
    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.just_pressed_keys.contains(&key)
    }

    /// Check if key was RELEASED this frame only
    pub fn is_key_released(&self, key: Key) -> bool {
        self.just_released_keys.contains(&key)
    }

    /// Check if ANY key pressed this frame
    pub fn any_key_pressed(&self) -> bool {
        !self.just_pressed_keys.is_empty()
    }

    // === KEYBOARD POLLING ===

    /// Key currently held down
    pub fn key(&self, key: Key) -> bool {
        self.pressed_keys.contains(&key)
    }

    /// Key pressed THIS frame
    pub fn key_just_pressed(&self, key: Key) -> bool {
        self.just_pressed_keys.contains(&key)
    }

    /// Key released THIS frame
    pub fn key_just_released(&self, key: Key) -> bool {
        self.just_released_keys.contains(&key)
    }

    /// All keys in `keys` are currently down
    pub fn keys(&self, keys: &[Key]) -> bool {
        keys.iter().all(|k| self.pressed_keys.contains(k))
    }

    /// Key with specific shift/ctrl modifiers
    pub fn key_with_mods(&self, key: Key, shift: bool, ctrl: bool) -> bool {
        self.key(key) && self.modifiers.shift == shift && self.modifiers.ctrl == ctrl
    }

    // === MOUSE POLLING ===

    pub fn mouse_button(&self, button: MouseButton) -> bool {
        self.pressed_buttons.contains(&button)
    }

    pub fn mouse_position(&self) -> Position {
        self.mouse_position
    }

    pub fn mouse_delta(&self) -> (f32, f32) {
        self.mouse_delta
    }

    // === MODIFIERS ===

    pub fn shift(&self) -> bool {
        self.modifiers.shift
    }
    pub fn ctrl(&self) -> bool {
        self.modifiers.ctrl
    }
    pub fn alt(&self) -> bool {
        self.modifiers.alt
    }
    pub fn logo(&self) -> bool {
        self.modifiers.logo
    }

    // === SNAPSHOTS ===
    /// Deterministic (sorted) list of keys currently pressed
    pub fn pressed_keys_list(&self) -> Vec<Key> {
        let mut v: Vec<Key> = self.pressed_keys.iter().cloned().collect();
        v.sort_by_key(|k| *k as u32);
        v
    }

    /// Deterministic list of keys pressed THIS frame
    pub fn just_pressed_keys_list(&self) -> Vec<Key> {
        let mut v: Vec<Key> = self.just_pressed_keys.iter().cloned().collect();
        v.sort_by_key(|k| *k as u32);
        v
    }

    /// Deterministic list of keys released THIS frame
    pub fn just_released_keys_list(&self) -> Vec<Key> {
        let mut v: Vec<Key> = self.just_released_keys.iter().cloned().collect();
        v.sort_by_key(|k| *k as u32);
        v
    }

    /// List of mouse buttons currently pressed (unsorted)
    pub fn pressed_buttons_list(&self) -> Vec<MouseButton> {
        let v: Vec<MouseButton> = self.pressed_buttons.iter().cloned().collect();
        v
    }

    /// Buttons pressed THIS frame
    pub fn just_pressed_buttons_list(&self) -> Vec<MouseButton> {
        self.just_pressed_buttons.iter().cloned().collect()
    }

    /// Buttons released THIS frame
    pub fn just_released_buttons_list(&self) -> Vec<MouseButton> {
        self.just_released_buttons.iter().cloned().collect()
    }

    // === INTERNAL (called by EventHandler) ===

    pub(crate) fn clear_frame_state(&mut self) {
        self.just_pressed_keys.clear();
        self.just_released_keys.clear();
        self.just_pressed_buttons.clear();
        self.just_released_buttons.clear();
        self.mouse_delta = (0.0, 0.0);
    }

    pub(crate) fn on_key_pressed(&mut self, key: Key) {
        if self.pressed_keys.insert(key) {
            self.just_pressed_keys.insert(key);
        }
    }

    pub(crate) fn on_key_released(&mut self, key: Key) {
        self.pressed_keys.remove(&key);
        self.just_released_keys.insert(key);
    }

    pub(crate) fn on_modifiers_changed(&mut self, mods: Modifiers) {
        self.modifiers = mods;
    }

    pub(crate) fn on_mouse_button_pressed(&mut self, button: MouseButton) {
        if self.pressed_buttons.insert(button) {
            self.just_pressed_buttons.insert(button);
        }
    }

    pub(crate) fn on_mouse_button_released(&mut self, button: MouseButton) {
        self.pressed_buttons.remove(&button);
        self.just_released_buttons.insert(button);
    }

    pub(crate) fn on_mouse_move(&mut self, pos: Position, last_pos: Position) {
        self.mouse_delta = (pos.x - last_pos.x, pos.y - last_pos.y);
        self.mouse_position = pos;
    }
}
