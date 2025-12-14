use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use crate::engine::block::Block;
use crate::engine::chunk::{Chunk, ChunkPos, CHUNK_SIZE};
use crate::engine::camera::Camera;
use crate::engine::constants::{MAX_NEW_CHUNKS_PER_FRAME, DEFAULT_RENDER_DISTANCE, MAX_MESH_REBUILDS_PER_FRAME, MAX_CHUNK_RECEIVES_PER_FRAME};
use crate::engine::constants::{noise, blocks};

/// Message sent to worker threads for chunk generation
struct ChunkGenRequest {
    pos: ChunkPos,
}

/// Result from worker threads with generated chunk data
struct ChunkGenResult {
    pos: ChunkPos,
    blocks: Vec<Block>,
}

/// Manages the voxel world, including chunk loading and terrain generation.
pub struct World {
    pub chunks: HashMap<(i32, i32, i32), Chunk>,
    pub render_distance: i32,
    pub last_player_chunk: (i32, i32, i32),
    
    // Threading for chunk generation
    chunk_request_tx: Sender<ChunkGenRequest>,
    chunk_result_rx: Receiver<ChunkGenResult>,
    pending_chunks: HashSet<(i32, i32, i32)>,
    _worker_handles: Vec<thread::JoinHandle<()>>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        // Create channels for chunk generation
        let (request_tx, request_rx) = mpsc::channel::<ChunkGenRequest>();
        let (result_tx, result_rx) = mpsc::channel::<ChunkGenResult>();
        
        // Use crossbeam for multi-consumer request queue
        let request_rx = std::sync::Arc::new(std::sync::Mutex::new(request_rx));
        
        // Spawn worker threads (use available parallelism minus 1 for main thread)
        let num_workers = std::thread::available_parallelism()
            .map(|p| p.get().saturating_sub(1).max(1))
            .unwrap_or(2);
        
        let mut handles = Vec::with_capacity(num_workers);
        
        for _ in 0..num_workers {
            let rx = request_rx.clone();
            let tx = result_tx.clone();
            
            let handle = thread::spawn(move || {
                loop {
                    // Try to get a request
                    let request = {
                        let guard = rx.lock().unwrap();
                        guard.recv()
                    };
                    
                    match request {
                        Ok(req) => {
                            // Generate chunk terrain on worker thread
                            let blocks = Self::generate_terrain_data(req.pos);
                            let _ = tx.send(ChunkGenResult {
                                pos: req.pos,
                                blocks,
                            });
                        }
                        Err(_) => break, // Channel closed, exit thread
                    }
                }
            });
            handles.push(handle);
        }
        
        Self {
            chunks: HashMap::new(),
            render_distance: DEFAULT_RENDER_DISTANCE,
            last_player_chunk: (i32::MAX, i32::MAX, i32::MAX), // Force initial load
            chunk_request_tx: request_tx,
            chunk_result_rx: result_rx,
            pending_chunks: HashSet::new(),
            _worker_handles: handles,
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

        // Receive completed chunks from worker threads (limit per frame)
        let mut received_this_frame = 0;
        while let Ok(result) = self.chunk_result_rx.try_recv() {
            let key = (result.pos.x, result.pos.y, result.pos.z);
            self.pending_chunks.remove(&key);
            
            // Only add chunk if it's still in render distance
            let dx = (result.pos.x - player_chunk_x).abs();
            let dz = (result.pos.z - player_chunk_z).abs();
            if dx <= self.render_distance && dz <= self.render_distance {
                let mut chunk = Chunk::new(result.pos);
                chunk.blocks = result.blocks;
                chunk.dirty = true;
                self.chunks.insert(key, chunk);
                
                // Mark neighbors dirty
                let neighbor_offsets = [(-1,0,0),(1,0,0),(0,0,-1),(0,0,1),(0,-1,0),(0,1,0)];
                for (dx,dy,dz) in neighbor_offsets.iter() {
                    if let Some(n) = self.chunks.get_mut(&(key.0+dx, key.1+dy, key.2+dz)) { 
                        n.dirty = true; 
                    }
                }
            }
            
            received_this_frame += 1;
            if received_this_frame >= MAX_CHUNK_RECEIVES_PER_FRAME { break; }
        }

        // Request new chunks (prioritize by distance to player)
        let mut chunks_to_request: Vec<(i32, i32, i32, i32)> = Vec::new(); // (cx, cy, cz, dist_sq)
        
        for cy in 0..1 {
            for cz in (player_chunk_z - self.render_distance)..=(player_chunk_z + self.render_distance) {
                for cx in (player_chunk_x - self.render_distance)..=(player_chunk_x + self.render_distance) {
                    let key = (cx, cy, cz);
                    if !self.chunks.contains_key(&key) && !self.pending_chunks.contains(&key) {
                        let dist_sq = (cx - player_chunk_x).pow(2) + (cz - player_chunk_z).pow(2);
                        chunks_to_request.push((cx, cy, cz, dist_sq));
                    }
                }
            }
        }
        
        // Sort by distance (closest first)
        chunks_to_request.sort_by_key(|&(_, _, _, dist)| dist);
        
        // Send requests (limit per frame)
        for (i, (cx, cy, cz, _)) in chunks_to_request.into_iter().enumerate() {
            if i >= MAX_NEW_CHUNKS_PER_FRAME { break; }
            
            let key = (cx, cy, cz);
            self.pending_chunks.insert(key);
            let _ = self.chunk_request_tx.send(ChunkGenRequest {
                pos: ChunkPos { x: cx, y: cy, z: cz },
            });
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
            
            // Also cancel pending chunks that are now out of range
            self.pending_chunks.retain(|(cx, _cy, cz)| {
                let dx = (cx - player_chunk_x).abs();
                let dz = (cz - player_chunk_z).abs();
                dx <= unload_distance && dz <= unload_distance
            });
        }
    }

    /// Generate terrain data on a worker thread (no Chunk creation, just block data)
    fn generate_terrain_data(pos: ChunkPos) -> Vec<Block> {
        use ::noise::{Perlin, NoiseFn};
        let perlin = Perlin::new(noise::SEED);
        
        let mut block_data = vec![Block::Air; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
        let base_global_y = pos.y * CHUNK_SIZE as i32;

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let global_x = pos.x * CHUNK_SIZE as i32 + x as i32;
                let global_z = pos.z * CHUNK_SIZE as i32 + z as i32;

                let mut frequency = noise::BASE_SCALE;
                let mut amplitude = 1.0f32;
                let mut total = 0.0f32;
                let mut amp_norm = 0.0f32;
                for _oct in 0..noise::OCTAVES {
                    let n = perlin.get([
                        global_x as f64 * frequency,
                        global_z as f64 * frequency,
                    ]) as f32;
                    total += n * amplitude;
                    amp_norm += amplitude;
                    amplitude *= noise::PERSISTENCE;
                    frequency *= noise::LACUNARITY as f64;
                }
                let normalized = (total / amp_norm * 0.5 + 0.5).clamp(0.0, 1.0);
                let shaped = normalized.powf(noise::HEIGHT_EXPONENT);
                let height_world = (shaped * noise::MAX_HEIGHT) as i32;

                // Determine if this is a beach/water area based on height
                let is_near_water = height_world <= noise::WATER_LEVEL + 2;
                let is_underwater = height_world < noise::WATER_LEVEL;

                if height_world >= base_global_y {
                    let local_surface = (height_world - base_global_y).clamp(0, CHUNK_SIZE as i32 - 1) as usize;
                    for y in 0..=local_surface {
                        let world_y = base_global_y + y as i32;
                        let block_id = if y == local_surface {
                            if is_underwater {
                                blocks::SAND
                            } else if is_near_water {
                                blocks::SAND
                            } else {
                                blocks::GRASS
                            }
                        } else if local_surface - y <= noise::DIRT_DEPTH {
                            if is_near_water {
                                blocks::SAND
                            } else {
                                blocks::DIRT
                            }
                        } else if world_y <= 2 {
                            blocks::BEDROCK
                        } else {
                            blocks::STONE
                        };
                        let idx = (y * CHUNK_SIZE * CHUNK_SIZE) + (z * CHUNK_SIZE) + x;
                        block_data[idx] = Block::Solid(block_id);
                    }
                }

                // Fill water above terrain up to water level
                let water_start = if height_world >= base_global_y {
                    (height_world - base_global_y + 1).max(0) as usize
                } else {
                    0
                };
                let water_end = (noise::WATER_LEVEL - base_global_y).clamp(0, CHUNK_SIZE as i32 - 1) as usize;
                
                if water_end >= water_start {
                    for y in water_start..=water_end {
                        let idx = (y * CHUNK_SIZE * CHUNK_SIZE) + (z * CHUNK_SIZE) + x;
                        block_data[idx] = Block::Solid(blocks::WATER);
                    }
                }
            }
        }
        
        block_data
    }

    pub fn render_chunks(&mut self, camera: &Camera, shader: &crate::engine::shader::ShaderProgram) {
        let frustum = camera.frustum();
        let chunk_size_f = CHUNK_SIZE as f32;
        let cam_pos = camera.position;
        
        // Collect visible chunks with distances for sorting
        // We need both opaque (front-to-back) and transparent (back-to-front) meshes
        let mut opaque_visible: Vec<_> = Vec::new();
        let mut transparent_visible: Vec<_> = Vec::new();
        
        for ((cx, cy, cz), chunk) in self.chunks.iter() {
            let chunk_center = glam::vec3(
                *cx as f32 * chunk_size_f + chunk_size_f * 0.5,
                *cy as f32 * chunk_size_f + chunk_size_f * 0.5,
                *cz as f32 * chunk_size_f + chunk_size_f * 0.5,
            );
            let dist_sq = (chunk_center - cam_pos).length_squared();
            
            if let Some(mesh) = chunk.mesh.as_ref() {
                opaque_visible.push((*cx, *cy, *cz, mesh, dist_sq));
            }
            if let Some(mesh) = chunk.transparent_mesh.as_ref() {
                transparent_visible.push((*cx, *cy, *cz, mesh, dist_sq));
            }
        }
        
        // Sort opaque front-to-back to improve depth test efficiency
        opaque_visible.sort_by(|a, b| a.4.partial_cmp(&b.4).unwrap_or(std::cmp::Ordering::Equal));
        
        // Sort transparent back-to-front for correct alpha blending
        transparent_visible.sort_by(|a, b| b.4.partial_cmp(&a.4).unwrap_or(std::cmp::Ordering::Equal));
        
        unsafe {
            // Pass 1: Render opaque geometry
            for (cx, cy, cz, mesh, _) in opaque_visible {
                let chunk_world_pos = glam::vec3(cx as f32 * chunk_size_f, cy as f32 * chunk_size_f, cz as f32 * chunk_size_f);
                let min = chunk_world_pos;
                let max = chunk_world_pos + glam::vec3(chunk_size_f, chunk_size_f, chunk_size_f);
                if !frustum.contains_aabb(min, max) { continue; }
                
                let model = glam::Mat4::from_translation(chunk_world_pos);
                shader.set_mat4("uModel", &model);
                mesh.draw();
            }
            
            // Pass 2: Render transparent geometry with blending
            if !transparent_visible.is_empty() {
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                gl::DepthMask(gl::FALSE); // Don't write to depth buffer for transparent objects
                
                for (cx, cy, cz, mesh, _) in transparent_visible {
                    let chunk_world_pos = glam::vec3(cx as f32 * chunk_size_f, cy as f32 * chunk_size_f, cz as f32 * chunk_size_f);
                    let min = chunk_world_pos;
                    let max = chunk_world_pos + glam::vec3(chunk_size_f, chunk_size_f, chunk_size_f);
                    if !frustum.contains_aabb(min, max) { continue; }
                    
                    let model = glam::Mat4::from_translation(chunk_world_pos);
                    shader.set_mat4("uModel", &model);
                    mesh.draw();
                }
                
                gl::DepthMask(gl::TRUE);
                gl::Disable(gl::BLEND);
            }
        }
    }

    /// Rebuilds meshes for all chunks marked as dirty.
    /// Uses a two-pass approach to avoid unsafe aliasing: first collect neighbor data, then rebuild.
    /// Limits rebuilds per frame to prevent lag spikes.
    pub fn rebuild_dirty(&mut self) {
        let cam_chunk = self.last_player_chunk;
        
        // Collect dirty chunks and sort by distance to camera (closest first)
        let mut dirty_keys: Vec<(i32, i32, i32)> = self.chunks
            .iter()
            .filter_map(|(k, c)| if c.dirty { Some(*k) } else { None })
            .collect();
        
        // Sort by distance to player chunk (prioritize visible chunks)
        dirty_keys.sort_by_key(|&(cx, cy, cz)| {
            let dx = cx - cam_chunk.0;
            let dy = cy - cam_chunk.1;
            let dz = cz - cam_chunk.2;
            dx * dx + dy * dy + dz * dz
        });

        let mut rebuilt_count = 0;
        for key in dirty_keys {
            if rebuilt_count >= MAX_MESH_REBUILDS_PER_FRAME {
                break;
            }
            
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
                rebuilt_count += 1;
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
