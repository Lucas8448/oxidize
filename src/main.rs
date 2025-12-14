mod engine;

use engine::game::Game;
use engine::core::Engine;
use engine::shader::ShaderProgram;
use engine::world::World;
use engine::texture::{Texture, generate_block_atlas};
use engine::shader_sources::{BLOCK_WORLD_VERT, BLOCK_WORLD_FRAG};
use engine::constants::{DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT};

pub struct DemoGame {
    shader: Option<ShaderProgram>,
    block_atlas: Option<Texture>,
    world: World,
}

impl Default for DemoGame {
    fn default() -> Self {
        Self::new()
    }
}

impl DemoGame {
    pub fn new() -> Self {
        Self { shader: None, block_atlas: None, world: World::new() }
    }
}

impl Game for DemoGame {
    fn on_start(&mut self, _engine: &mut Engine) {
        unsafe {
            self.shader = Some(ShaderProgram::from_source(BLOCK_WORLD_VERT, BLOCK_WORLD_FRAG)
                .expect("shader compile"));
            
            // Generate and load the block texture atlas
            self.block_atlas = Some(generate_block_atlas());
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
                shader.set_vec3("uCameraPos", &engine.camera.position);
                
                // Bind texture atlas
                if let Some(atlas) = &self.block_atlas {
                    atlas.bind(0);
                    shader.set_int("uTexture", 0);
                }
                
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