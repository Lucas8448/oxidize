mod engine;

use engine::game::Game;
use engine::core::Engine;
use engine::shader::ShaderProgram;
use engine::world::World;
use engine::shader_sources::{BLOCK_WORLD_VERT, BLOCK_WORLD_FRAG};
use engine::constants::{DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT};

pub struct DemoGame {
    shader: Option<ShaderProgram>,
    world: World,
}

impl DemoGame {
    pub fn new() -> Self {
        Self { shader: None, world: World::new() }
    }
}

impl Game for DemoGame {
    fn on_start(&mut self, _engine: &mut Engine) {
        unsafe {
            self.shader = Some(ShaderProgram::from_source(BLOCK_WORLD_VERT, BLOCK_WORLD_FRAG)
                .expect("shader compile"));
        }
    }
    fn update(&mut self, engine: &mut Engine, _dt: f32) { 
        self.world.update_chunks(engine.camera.position);
        self.world.rebuild_dirty();
    }
    fn render(&mut self, engine: &mut Engine) {
        unsafe {
            let cam_uni = engine.camera.projection_matrix() * engine.camera.view_matrix();
            if let Some(shader) = &self.shader {
                shader.use_program();
                shader.set_mat4("uViewProj", &cam_uni);
                self.world.render_chunks(&engine.camera, shader);
            }            
        }
    }
}

fn main() {
    let mut engine = Engine::new(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT, "Oxidize");
    let mut game = DemoGame::new();
    engine.run(&mut game);
}