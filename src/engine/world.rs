use std::collections::HashMap;
use crate::engine::block::Block;
use crate::engine::chunk::{Chunk, ChunkPos, CHUNK_SIZE};
use crate::engine::camera::Camera;
use crate::engine::constants::MAX_NEW_CHUNKS_PER_FRAME;

pub struct World {
    pub chunks: HashMap<(i32,i32,i32), Chunk>,
    pub render_distance: i32,
    pub last_player_chunk: (i32, i32, i32),
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            render_distance: 4,
            last_player_chunk: (0, 0, 0),
        }
    }

    pub fn set_render_distance(&mut self, radius: i32) {
        self.render_distance = radius.max(1);
    }

    pub fn update_chunks(&mut self, player_pos: glam::Vec3) {
        let player_chunk_x = (player_pos.x / (CHUNK_SIZE as f32)).floor() as i32;
        let player_chunk_z = (player_pos.z / (CHUNK_SIZE as f32)).floor() as i32;
        let player_chunk_y = 0;

        let current_chunk = (player_chunk_x, player_chunk_y, player_chunk_z);
        let moved = current_chunk != self.last_player_chunk;
        if moved { self.last_player_chunk = current_chunk; }

        let mut generated_this_frame = 0usize;
        'gen: for cy in 0..1 {
            for cz in (player_chunk_z - self.render_distance)..=(player_chunk_z + self.render_distance) {
                for cx in (player_chunk_x - self.render_distance)..=(player_chunk_x + self.render_distance) {
                    let key = (cx, cy, cz);
                    if !self.chunks.contains_key(&key) {
                        let pos = ChunkPos { x: cx, y: cy, z: cz };
                        let mut chunk = Chunk::new(pos);
                        self.generate_chunk_terrain(&mut chunk);
                        self.chunks.insert(key, chunk);
                        generated_this_frame += 1;
                        if generated_this_frame >= MAX_NEW_CHUNKS_PER_FRAME { break 'gen; }
                    }
                }
            }
        }

        if moved {
            let unload_distance = self.render_distance + 2;
            let to_remove: Vec<_> = self.chunks.keys()
                .filter(|(cx, _cy, cz)| {
                    let dx = (cx - player_chunk_x).abs();
                    let dz = (cz - player_chunk_z).abs();
                    dx > unload_distance || dz > unload_distance
                })
                .cloned()
                .collect();
            for key in to_remove { self.chunks.remove(&key); }
        }
    }

    fn generate_chunk_terrain(&self, chunk: &mut Chunk) {
        use noise::{Perlin, NoiseFn};
        let perlin = Perlin::new(0);
        let base_scale = 0.015;
        let octaves = 4;
        let persistence = 0.55;
        let lacunarity = 2.1;
        let max_height = 38.0f32;
        
        let base_global_y = chunk.pos.y * CHUNK_SIZE as i32;
        
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let global_x = chunk.pos.x * CHUNK_SIZE as i32 + x as i32;
                let global_z = chunk.pos.z * CHUNK_SIZE as i32 + z as i32;
                
                let mut freq = base_scale;
                let mut amp = 1.0f32;
                let mut sum = 0.0f32;
                let mut norm = 0.0f32;
                for _ in 0..octaves {
                    let n = perlin.get([global_x as f64 * freq as f64, global_z as f64 * freq as f64]) as f32;
                    sum += n * amp;
                    norm += amp;
                    amp *= persistence;
                    freq *= lacunarity;
                }
                let noise_val = (sum / norm * 0.5 + 0.5).clamp(0.0, 1.0);
                let h = (noise_val.powf(1.2) * max_height) as i32;
                
                if h >= base_global_y {
                    let local_max = (h - base_global_y).clamp(0, CHUNK_SIZE as i32 - 1) as usize;
                    for y in 0..=local_max {
                        chunk.set_block(x, y, z, Block::Solid(1));
                    }
                }
            }
        }
    }

    pub fn render_chunks(&mut self, camera: &Camera, shader: &crate::engine::shader::ShaderProgram) {
        let frustum = camera.frustum();
        let chunk_size_f = CHUNK_SIZE as f32;
        
        unsafe {
            for ((cx, cy, cz), chunk) in &self.chunks {
                if let Some(mesh) = &chunk.mesh {
                    let chunk_world_pos = glam::vec3(*cx as f32 * chunk_size_f, *cy as f32 * chunk_size_f, *cz as f32 * chunk_size_f);
                    let min = chunk_world_pos;
                    let max = chunk_world_pos + glam::vec3(chunk_size_f, chunk_size_f, chunk_size_f);
                    if !frustum.contains_aabb(min, max) { continue; }
                    
                    let model = glam::Mat4::from_translation(chunk_world_pos);
                    shader.set_mat4("uModel", &model);
                    mesh.draw();
                }
            }
        }
    }

    pub fn rebuild_dirty(&mut self) {
        let dirty_keys: Vec<(i32,i32,i32)> = self.chunks.iter()
            .filter_map(|(k,c)| if c.dirty { Some(*k) } else { None })
            .collect();
        for key in dirty_keys {
            let ptr: *const std::collections::HashMap<(i32,i32,i32), Chunk> = &self.chunks;
            if let Some(chunk_mut) = self.chunks.get_mut(&key) {
                chunk_mut.rebuild_mesh(|lx,ly,lz| unsafe {
                    let map = &*ptr;
                    let (cx,cy,cz) = key;
                    let mut nx = cx; let mut ny = cy; let mut nz = cz;
                    let mut lx2 = lx; let mut ly2 = ly; let mut lz2 = lz;
                    if lx2 < 0 { nx -=1; lx2 += CHUNK_SIZE as i32; }
                    if lx2 >= CHUNK_SIZE as i32 { nx +=1; lx2 -= CHUNK_SIZE as i32; }
                    if ly2 < 0 { ny -=1; ly2 += CHUNK_SIZE as i32; }
                    if ly2 >= CHUNK_SIZE as i32 { ny +=1; ly2 -= CHUNK_SIZE as i32; }
                    if lz2 < 0 { nz -=1; lz2 += CHUNK_SIZE as i32; }
                    if lz2 >= CHUNK_SIZE as i32 { nz +=1; lz2 -= CHUNK_SIZE as i32; }
                    map.get(&(nx,ny,nz)).map(|c| c.get_block(lx2 as usize, ly2 as usize, lz2 as usize)).unwrap_or(Block::Air)
                });
            }
        }
    }
}
