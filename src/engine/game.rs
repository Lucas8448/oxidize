use crate::engine::core::Engine;

pub trait Game {
    fn on_start(&mut self, _engine: &mut Engine) {}
    fn update(&mut self, _engine: &mut Engine, _dt: f32) {}
    fn render(&mut self, _engine: &mut Engine) {}
    fn on_shutdown(&mut self, _engine: &mut Engine) {}
}
