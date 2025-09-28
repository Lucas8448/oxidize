use crate::engine::block::Block;
use crate::engine::mesh::Mesh;

pub const CHUNK_SIZE: usize = 32;

#[inline(always)]
fn index(x: usize, y: usize, z: usize) -> usize { (y * CHUNK_SIZE * CHUNK_SIZE) + (z * CHUNK_SIZE) + x }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos { pub x: i32, pub y: i32, pub z: i32 }

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
        
    for &(dx, dy, dz, shade_factor, normal_vec) in &directions {
            let (u_axis, v_axis) = if dx != 0 { (1, 2) } else if dy != 0 { (0, 2) } else { (0, 1) };
            let w_axis = 3 - u_axis - v_axis;
            for w in 0..(CHUNK_SIZE as i32) {
                let mut mask = vec![None::<Block>; CHUNK_SIZE * CHUNK_SIZE];
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
                let mut visited = vec![false; CHUNK_SIZE * CHUNK_SIZE];
                for v in 0..CHUNK_SIZE {
                    for u in 0..CHUNK_SIZE {
                        let idx = v * CHUNK_SIZE + u;
                        if visited[idx] || mask[idx].is_none() { continue; }
                        let block_type = mask[idx].unwrap();
                        let mut sample = [0i32;3];
                        sample[u_axis] = u as i32;
                        sample[v_axis] = v as i32;
                        sample[w_axis] = w;
                        let local_y = sample[1] as f32;
                        let world_y = self.pos.y as f32 * CHUNK_SIZE as f32 + local_y;
                        let t = (world_y / 48.0).clamp(0.0, 1.0);
                        let grass = [0.12, 0.55, 0.18];
                        let rock  = [0.55, 0.53, 0.50];
                        let base_color = [
                            grass[0] * (1.0 - t) + rock[0] * t,
                            grass[1] * (1.0 - t) + rock[1] * t,
                            grass[2] * (1.0 - t) + rock[2] * t,
                        ];
                        let color = [base_color[0] * shade_factor, base_color[1] * shade_factor, base_color[2] * shade_factor];
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
                        for i in 0..4 {
                            let uu = if i == 1 || i == 2 { width } else { 0 };
                            let vv = if i == 2 || i == 3 { height } else { 0 };
                            corners[i][u_axis] = (u + uu) as f32;
                            corners[i][v_axis] = (v + vv) as f32;
                            corners[i][w_axis] = w as f32;
                            if dx > 0 { corners[i][0] += 1.0; }
                            if dy > 0 { corners[i][1] += 1.0; }
                            if dz > 0 { corners[i][2] += 1.0; }
                        }
                        let (c1, c2, c3) = (color[0], color[1], color[2]);
                        let (nx, ny, nz) = (normal_vec[0], normal_vec[1], normal_vec[2]);
                        let emit_triangle = |v: &mut Vec<f32>, a: [f32;3], b: [f32;3], c: [f32;3], nx: f32, ny: f32, nz: f32| {
                            v.extend_from_slice(&[a[0],a[1],a[2], nx,ny,nz, c1,c2,c3]);
                            v.extend_from_slice(&[b[0],b[1],b[2], nx,ny,nz, c1,c2,c3]);
                            v.extend_from_slice(&[c[0],c[1],c[2], nx,ny,nz, c1,c2,c3]);
                        };
                        if dx < 0 || dy < 0 || dz < 0 {
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
