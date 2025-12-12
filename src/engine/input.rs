use std::collections::HashMap;
use glfw::{Action, Key};

#[derive(Default)]
pub struct InputState {
    keys: HashMap<Key, KeyState>,
}

#[derive(Copy, Clone, Default)]
pub struct KeyState {
    pub down: bool,
    pub pressed: bool,
    pub released: bool,
}

impl InputState {
    pub fn begin_frame(&mut self) {
        for state in self.keys.values_mut() { state.pressed = false; state.released = false; }
    }
    pub fn key_event(&mut self, key: Key, action: Action) {
        let entry = self.keys.entry(key).or_default();
        match action {
            Action::Press => {
                if !entry.down { entry.pressed = true; }
                entry.down = true;
            }
            Action::Release => {
                if entry.down { entry.released = true; }
                entry.down = false;
            }
            Action::Repeat => {}
        }
    }
    pub fn is_key_down(&self, key: Key) -> bool { self.keys.get(&key).map(|s| s.down).unwrap_or(false) }
    pub fn was_key_pressed(&self, key: Key) -> bool { self.keys.get(&key).map(|s| s.pressed).unwrap_or(false) }

    #[allow(dead_code)]
    pub fn was_key_released(&self, key: Key) -> bool { self.keys.get(&key).map(|s| s.released).unwrap_or(false) }
}
