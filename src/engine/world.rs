use std::collections::HashMap;
use crate::engine::block::Block;
use crate::engine::chunk::{Chunk, ChunkPos, CHUNK_SIZE};
use crate::engine::camera::Camera;
use crate::engine::constants::{MAX_NEW_CHUNKS_PER_FRAME, DEFAULT_RENDER_DISTANCE};
use crate::engine::constants::{noise, blocks};

/// Manages the voxel world, including chunk loading and terrain generation.
pub struct World {
    pub chunks: HashMap<(i32, i32, i32), Chunk>,
    pub render_distance: i32,
    pub last_player_chunk: (i32, i32, i32),
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            render_distance: DEFAULT_RENDER_DISTANCE,
            last_player_chunk: (0, 0, 0),
        }
    }

    #[allow(dead_code)]
    pub fn set_render_distance(&mut self, radius: i32) {
        self.render_distance = radius.max(1);
    }

    #[allow(dead_code)]
    pub fn get_render_distance(&self) -> i32 {
        self.render_distance
    }

    /// Returns the number of loaded chunks.
    #[allow(dead_code)]
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
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
                        // Mark neighbors dirty so they rebuild without border faces
                        let neighbor_offsets = [(-1,0,0),(1,0,0),(0,0,-1),(0,0,1),(0,-1,0),(0,1,0)];
                        for (dx,dy,dz) in neighbor_offsets.iter() {
                            if let Some(n) = self.chunks.get_mut(&(cx+dx, cy+dy, cz+dz)) { n.dirty = true; }
                        }
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
    use ::noise::{Perlin, NoiseFn};
        let perlin = Perlin::new(noise::SEED);

        let base_global_y = chunk.pos.y * CHUNK_SIZE as i32;

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let global_x = chunk.pos.x * CHUNK_SIZE as i32 + x as i32;
                let global_z = chunk.pos.z * CHUNK_SIZE as i32 + z as i32;

                let mut frequency = noise::BASE_SCALE;
                let mut amplitude = 1.0f32;
                let mut total = 0.0f32;
                let mut amp_norm = 0.0f32;
                for _oct in 0..noise::OCTAVES {
                    let n = perlin.get([
                        global_x as f64 * frequency,
                        global_z as f64 * frequency,
                    ]) as f32; // n in [-1,1]
                    total += n * amplitude;
                    amp_norm += amplitude;
                    amplitude *= noise::PERSISTENCE;
                    frequency *= noise::LACUNARITY as f64;
                }
                let normalized = (total / amp_norm * 0.5 + 0.5).clamp(0.0, 1.0);
                let shaped = normalized.powf(noise::HEIGHT_EXPONENT);
                let height_world = (shaped * noise::MAX_HEIGHT) as i32;

                if height_world >= base_global_y {
                    let local_surface = (height_world - base_global_y).clamp(0, CHUNK_SIZE as i32 - 1) as usize;
                    for y in 0..=local_surface {
                        let world_y = base_global_y + y as i32;
                        let block_id = if y == local_surface {
                            blocks::GRASS
                        } else if local_surface - y <= noise::DIRT_DEPTH {
                            blocks::DIRT
                        } else if world_y <= 2 {
                            blocks::BEDROCK
                        } else {
                            blocks::STONE
                        };
                        chunk.set_block(x, y, z, Block::Solid(block_id));
                    }
                }
            }
        }
    }

    pub fn render_chunks(&mut self, camera: &Camera, shader: &crate::engine::shader::ShaderProgram) {
        let frustum = camera.frustum();
        let chunk_size_f = CHUNK_SIZE as f32;
        let cam_pos = camera.position;
        
        // Collect visible chunks with distances for front-to-back sorting
        let mut visible: Vec<_> = self.chunks.iter()
            .filter_map(|((cx, cy, cz), chunk)| {
                chunk.mesh.as_ref().map(|mesh| {
                    let chunk_center = glam::vec3(
                        *cx as f32 * chunk_size_f + chunk_size_f * 0.5,
                        *cy as f32 * chunk_size_f + chunk_size_f * 0.5,
                        *cz as f32 * chunk_size_f + chunk_size_f * 0.5,
                    );
                    let dist_sq = (chunk_center - cam_pos).length_squared();
                    (*cx, *cy, *cz, mesh, dist_sq)
                })
            })
            .collect();
        
        // Sort front-to-back to improve depth test efficiency
        visible.sort_by(|a, b| a.4.partial_cmp(&b.4).unwrap_or(std::cmp::Ordering::Equal));
        
        unsafe {
            for (cx, cy, cz, mesh, _) in visible {
                let chunk_world_pos = glam::vec3(cx as f32 * chunk_size_f, cy as f32 * chunk_size_f, cz as f32 * chunk_size_f);
                let min = chunk_world_pos;
                let max = chunk_world_pos + glam::vec3(chunk_size_f, chunk_size_f, chunk_size_f);
                if !frustum.contains_aabb(min, max) { continue; }
                
                let model = glam::Mat4::from_translation(chunk_world_pos);
                shader.set_mat4("uModel", &model);
                mesh.draw();
            }
        }
    }

    /// Rebuilds meshes for all chunks marked as dirty.
    /// Uses a two-pass approach to avoid unsafe aliasing: first collect neighbor data, then rebuild.
    pub fn rebuild_dirty(&mut self) {
        let dirty_keys: Vec<(i32, i32, i32)> = self.chunks
            .iter()
            .filter_map(|(k, c)| if c.dirty { Some(*k) } else { None })
            .collect();

        for key in dirty_keys {
            // Pre-fetch neighbor chunk block data to avoid unsafe pointer aliasing
            let neighbor_blocks = self.collect_neighbor_blocks(key);

            // Also collect the current chunk's blocks into a temporary copy for the closure
            let current_chunk_blocks: Option<Vec<Block>> = self.chunks
                .get(&key)
                .map(|c| c.blocks.clone());

            if let (Some(chunk), Some(chunk_blocks)) = (self.chunks.get_mut(&key), current_chunk_blocks) {
                chunk.rebuild_mesh(|lx, ly, lz| {
                    Self::get_block_from_neighbors_with_blocks(key, lx, ly, lz, &neighbor_blocks, &chunk_blocks)
                });
            }
        }
    }

    /// Collects blocks from neighboring chunks that might be needed for mesh generation.
    /// Only collects the border blocks to minimize memory usage.
    fn collect_neighbor_blocks(&self, key: (i32, i32, i32)) -> HashMap<(i32, i32, i32), Block> {
        let (cx, cy, cz) = key;
        let mut border_blocks = HashMap::new();
        let offsets = [(-1, 0, 0), (1, 0, 0), (0, -1, 0), (0, 1, 0), (0, 0, -1), (0, 0, 1)];

        for (dx, dy, dz) in offsets {
            let neighbor_key = (cx + dx, cy + dy, cz + dz);
            if let Some(neighbor) = self.chunks.get(&neighbor_key) {
                // Collect only the face of the neighbor that borders our chunk
                Self::collect_border_face(neighbor, dx, dy, dz, neighbor_key, &mut border_blocks);
            }
        }

        border_blocks
    }

    /// Collects blocks from a single face of a neighboring chunk.
    fn collect_border_face(
        neighbor: &Chunk,
        dx: i32,
        dy: i32,
        dz: i32,
        neighbor_key: (i32, i32, i32),
        border_blocks: &mut HashMap<(i32, i32, i32), Block>,
    ) {
        let size = CHUNK_SIZE;
        let (nx, ny, nz) = neighbor_key;

        match (dx, dy, dz) {
            (-1, 0, 0) => {
                // Left neighbor: get right face (x = CHUNK_SIZE - 1)
                for y in 0..size {
                    for z in 0..size {
                        let block = neighbor.get_block(size - 1, y, z);
                        // Store as global block position
                        border_blocks.insert((nx * size as i32 + (size - 1) as i32, ny * size as i32 + y as i32, nz * size as i32 + z as i32), block);
                    }
                }
            }
            (1, 0, 0) => {
                // Right neighbor: get left face (x = 0)
                for y in 0..size {
                    for z in 0..size {
                        let block = neighbor.get_block(0, y, z);
                        border_blocks.insert((nx * size as i32, ny * size as i32 + y as i32, nz * size as i32 + z as i32), block);
                    }
                }
            }
            (0, -1, 0) => {
                // Bottom neighbor: get top face (y = CHUNK_SIZE - 1)
                for x in 0..size {
                    for z in 0..size {
                        let block = neighbor.get_block(x, size - 1, z);
                        border_blocks.insert((nx * size as i32 + x as i32, ny * size as i32 + (size - 1) as i32, nz * size as i32 + z as i32), block);
                    }
                }
            }
            (0, 1, 0) => {
                // Top neighbor: get bottom face (y = 0)
                for x in 0..size {
                    for z in 0..size {
                        let block = neighbor.get_block(x, 0, z);
                        border_blocks.insert((nx * size as i32 + x as i32, ny * size as i32, nz * size as i32 + z as i32), block);
                    }
                }
            }
            (0, 0, -1) => {
                // Back neighbor: get front face (z = CHUNK_SIZE - 1)
                for x in 0..size {
                    for y in 0..size {
                        let block = neighbor.get_block(x, y, size - 1);
                        border_blocks.insert((nx * size as i32 + x as i32, ny * size as i32 + y as i32, nz * size as i32 + (size - 1) as i32), block);
                    }
                }
            }
            (0, 0, 1) => {
                // Front neighbor: get back face (z = 0)
                for x in 0..size {
                    for y in 0..size {
                        let block = neighbor.get_block(x, y, 0);
                        border_blocks.insert((nx * size as i32 + x as i32, ny * size as i32 + y as i32, nz * size as i32), block);
                    }
                }
            }
            _ => {}
        }
    }

    /// Gets a block at the given local coordinates using pre-fetched block data.
    fn get_block_from_neighbors_with_blocks(
        chunk_key: (i32, i32, i32),
        lx: i32,
        ly: i32,
        lz: i32,
        neighbor_blocks: &HashMap<(i32, i32, i32), Block>,
        current_blocks: &[Block],
    ) -> Block {
        let (cx, cy, cz) = chunk_key;
        let size = CHUNK_SIZE as i32;

        // Check if coordinates are within the current chunk
        if lx >= 0 && lx < size && ly >= 0 && ly < size && lz >= 0 && lz < size {
            let idx = (ly as usize * CHUNK_SIZE * CHUNK_SIZE) + (lz as usize * CHUNK_SIZE) + lx as usize;
            return current_blocks[idx];
        }

        // Calculate global position and look up in neighbor blocks
        let global_x = cx * size + lx;
        let global_y = cy * size + ly;
        let global_z = cz * size + lz;

        neighbor_blocks
            .get(&(global_x, global_y, global_z))
            .copied()
            .unwrap_or(Block::Air)
    }
}
