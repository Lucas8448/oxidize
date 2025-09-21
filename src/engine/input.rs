use std::collections::HashSet;

pub struct InputState {
    pressed: HashSet<String>,
}
impl InputState {
    pub fn new() -> Self { Self { pressed: HashSet::new() } }
    pub fn press(&mut self, key: String) { self.pressed.insert(key); }
    pub fn release(&mut self, key: &str) { self.pressed.remove(key); }
    pub fn is(&self, key: &str) -> bool { self.pressed.contains(key) }
}
