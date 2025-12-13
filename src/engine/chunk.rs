use crate::engine::block::Block;
use crate::engine::mesh::Mesh;
use crate::engine::constants::blocks;

pub const CHUNK_SIZE: usize = 32;

#[inline(always)]
fn index(x: usize, y: usize, z: usize) -> usize { (y * CHUNK_SIZE * CHUNK_SIZE) + (z * CHUNK_SIZE) + x }

/// Position of a chunk in chunk coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos { pub x: i32, pub y: i32, pub z: i32 }

/// A cubic section of the world containing blocks.
pub struct Chunk {
    pub pos: ChunkPos,
    pub blocks: Vec<Block>,
    pub mesh: Option<Mesh>,
    pub dirty: bool,
}

impl Chunk {
    pub fn new(pos: ChunkPos) -> Self {
        Self { pos, blocks: vec![Block::Air; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE], mesh: None, dirty: true }
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, b: Block) {
        self.blocks[index(x, y, z)] = b; self.dirty = true;
    }
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Block { self.blocks[index(x, y, z)] }

    pub fn rebuild_mesh<F: Fn(i32, i32, i32) -> Block>(&mut self, neighbor_block: F) {
        if !self.dirty { return; }
        let mut vertices: Vec<f32> = Vec::with_capacity(16 * 1024);
        let directions = [
            (-1, 0, 0, 1.0,  [ -1.0,  0.0,  0.0 ]),
            ( 1, 0, 0, 1.0,  [  1.0,  0.0,  0.0 ]),
            ( 0,-1, 0, 0.96, [  0.0, -1.0,  0.0 ]),
            ( 0, 1, 0, 1.04, [  0.0,  1.0,  0.0 ]),
            ( 0, 0,-1, 1.0,  [  0.0,  0.0, -1.0 ]),
            ( 0, 0, 1, 1.0,  [  0.0,  0.0,  1.0 ])
        ];
        
        // Pre-allocate buffers outside loops to reduce allocations
        let mut mask = vec![None::<Block>; CHUNK_SIZE * CHUNK_SIZE];
        let mut visited = vec![false; CHUNK_SIZE * CHUNK_SIZE];
        
        for &(dx, dy, dz, shade_factor, normal_vec) in &directions {
            let (u_axis, v_axis) = if dx != 0 { (1, 2) } else if dy != 0 { (0, 2) } else { (0, 1) };
            let w_axis = 3 - u_axis - v_axis;
            for w in 0..(CHUNK_SIZE as i32) {
                // Clear buffers instead of reallocating
                mask.fill(None);
                for v in 0..CHUNK_SIZE {
                    for u in 0..CHUNK_SIZE {
                        let mut pos = [0i32; 3];
                        pos[u_axis] = u as i32;
                        pos[v_axis] = v as i32; 
                        pos[w_axis] = w;
                        let (x, y, z) = (pos[0], pos[1], pos[2]);
                        let current = if x >= 0 && x < CHUNK_SIZE as i32 && y >= 0 && y < CHUNK_SIZE as i32 && z >= 0 && z < CHUNK_SIZE as i32 {
                            self.get_block(x as usize, y as usize, z as usize)
                        } else { Block::Air };
                        let neighbor = neighbor_block(x + dx, y + dy, z + dz);
                        if !current.is_air() && neighbor.is_air() {
                            mask[v * CHUNK_SIZE + u] = Some(current);
                        }
                    }
                }
                visited.fill(false);
                for v in 0..CHUNK_SIZE {
                    for u in 0..CHUNK_SIZE {
                        let idx = v * CHUNK_SIZE + u;
                        if visited[idx] || mask[idx].is_none() { continue; }
                        let block_type = mask[idx].unwrap();
                        let mut sample = [0i32;3];
                        sample[u_axis] = u as i32;
                        sample[v_axis] = v as i32;
                        sample[w_axis] = w;
                        // Color based on block id and face direction (simple palette)
                        let color = match block_type {
                            Block::Solid(id) => {
                                let (r,g,b) = match id {
                                    blocks::GRASS => {
                                        // Top face bright green, sides earthy
                                        if dy > 0 { (0.35, 0.75, 0.30) } else { (0.40, 0.55, 0.25) }
                                    }
                                    blocks::DIRT => (0.55, 0.38, 0.25),
                                    blocks::STONE => (0.62, 0.60, 0.58),
                                    blocks::BEDROCK => (0.15, 0.15, 0.18),
                                    _ => (1.0, 0.0, 1.0), // debug magenta
                                }; [r as f32 * shade_factor, g as f32 * shade_factor, b as f32 * shade_factor]
                            }
                            Block::Air => [0.0, 0.0, 0.0],
                        };
                        let (c1, c2, c3) = (color[0], color[1], color[2]);
                        let mut width = 1;
                        while u + width < CHUNK_SIZE {
                            let next_idx = v * CHUNK_SIZE + (u + width);
                            if visited[next_idx] || mask[next_idx] != Some(block_type) { break; }
                            width += 1;
                        }
                        let mut height = 1;
                        'height_loop: while v + height < CHUNK_SIZE {
                            for u_offset in 0..width {
                                let check_idx = (v + height) * CHUNK_SIZE + (u + u_offset);
                                if visited[check_idx] || mask[check_idx] != Some(block_type) {
                                    break 'height_loop;
                                }
                            }
                            height += 1;
                        }
                        for v_offset in 0..height {
                            for u_offset in 0..width {
                                visited[(v + v_offset) * CHUNK_SIZE + (u + u_offset)] = true;
                            }
                        }
                        let mut corners = [[0.0f32; 3]; 4];
                        let corner_offsets = [(0, 0), (1, 0), (1, 1), (0, 1)];
                        for (i, corner) in corners.iter_mut().enumerate() {
                            let (u_off, v_off) = corner_offsets[i];
                            let uu = if u_off == 1 { width } else { 0 };
                            let vv = if v_off == 1 { height } else { 0 };
                            corner[u_axis] = (u + uu) as f32;
                            corner[v_axis] = (v + vv) as f32;
                            corner[w_axis] = w as f32;
                            if dx > 0 { corner[0] += 1.0; }
                            if dy > 0 { corner[1] += 1.0; }
                            if dz > 0 { corner[2] += 1.0; }
                        }
                        // c1,c2,c3 already defined
                        let (nx, ny, nz) = (normal_vec[0], normal_vec[1], normal_vec[2]);
                        let emit_triangle = |v: &mut Vec<f32>, a: [f32;3], b: [f32;3], c: [f32;3], nx: f32, ny: f32, nz: f32| {
                            v.extend_from_slice(&[a[0],a[1],a[2], nx,ny,nz, c1,c2,c3]);
                            v.extend_from_slice(&[b[0],b[1],b[2], nx,ny,nz, c1,c2,c3]);
                            v.extend_from_slice(&[c[0],c[1],c[2], nx,ny,nz, c1,c2,c3]);
                        };
                        // Determine winding order based on face direction
                        // For CCW front-facing triangles with GL_BACK culling:
                        // Positive X, negative Y, positive Z need one winding
                        // Negative X, positive Y, negative Z need the opposite
                        let flip = dx < 0 || dy > 0 || dz < 0;
                        if flip {
                            emit_triangle(&mut vertices, corners[0], corners[2], corners[1], nx,ny,nz);
                            emit_triangle(&mut vertices, corners[0], corners[3], corners[2], nx,ny,nz);
                        } else {
                            emit_triangle(&mut vertices, corners[0], corners[1], corners[2], nx,ny,nz);
                            emit_triangle(&mut vertices, corners[0], corners[2], corners[3], nx,ny,nz);
                        }
                    }
                }
            }
        }
        self.mesh = if vertices.is_empty() { None } else { Some(Mesh::from_vertices(&vertices)) };
        self.dirty = false;
    }
}
