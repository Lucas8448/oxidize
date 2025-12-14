use crate::engine::block::Block;
use crate::engine::mesh::Mesh;
use crate::engine::constants::blocks;
use crate::engine::texture::{get_tile_uvs, block_textures};

pub const CHUNK_SIZE: usize = 32;

#[inline(always)]
fn index(x: usize, y: usize, z: usize) -> usize { (y * CHUNK_SIZE * CHUNK_SIZE) + (z * CHUNK_SIZE) + x }

/// Returns true if the block is transparent (like water)
#[inline(always)]
fn is_transparent(block: Block) -> bool {
    matches!(block, Block::Solid(blocks::WATER))
}

/// Get the texture tile index for a block face
/// Returns (tile_index, alpha) for the given block and face direction
fn get_block_texture(block_id: u8, dx: i32, dy: i32, _dz: i32) -> (u32, f32) {
    match block_id {
        blocks::GRASS => {
            if dy > 0 {
                (block_textures::GRASS_TOP, 1.0)  // Top face
            } else if dy < 0 {
                (block_textures::DIRT, 1.0)       // Bottom face
            } else {
                (block_textures::GRASS_SIDE, 1.0) // Side faces
            }
        }
        blocks::DIRT => (block_textures::DIRT, 1.0),
        blocks::STONE => (block_textures::STONE, 1.0),
        blocks::BEDROCK => (block_textures::BEDROCK, 1.0),
        blocks::WATER => (block_textures::WATER, 0.7),
        blocks::SAND => (block_textures::SAND, 1.0),
        blocks::GRAVEL => (block_textures::GRAVEL, 1.0),
        _ => (block_textures::MISSING, 1.0),
    }
}

/// Position of a chunk in chunk coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos { pub x: i32, pub y: i32, pub z: i32 }

/// A cubic section of the world containing blocks.
pub struct Chunk {
    pub pos: ChunkPos,
    pub blocks: Vec<Block>,
    pub mesh: Option<Mesh>,
    pub transparent_mesh: Option<Mesh>,
    pub dirty: bool,
}

impl Chunk {
    pub fn new(pos: ChunkPos) -> Self {
        Self { pos, blocks: vec![Block::Air; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE], mesh: None, transparent_mesh: None, dirty: true }
    }

    #[allow(dead_code)]
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, b: Block) {
        self.blocks[index(x, y, z)] = b; self.dirty = true;
    }
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Block { self.blocks[index(x, y, z)] }

    pub fn rebuild_mesh<F: Fn(i32, i32, i32) -> Block>(&mut self, neighbor_block: F) {
        if !self.dirty { return; }
        // Estimate: worst case ~6 faces per block, 2 triangles per face, 12 floats per vertex
        let estimated_verts = CHUNK_SIZE * CHUNK_SIZE * 6 * 2 * 3 * 12;
        let mut opaque_vertices: Vec<f32> = Vec::with_capacity(estimated_verts);
        let mut transparent_vertices: Vec<f32> = Vec::with_capacity(estimated_verts / 4);
        
        // Direction data: (dx, dy, dz, shade_factor, normal)
        let directions = [
            (-1, 0, 0, 0.7,  [ -1.0,  0.0,  0.0 ]),  // Left
            ( 1, 0, 0, 0.7,  [  1.0,  0.0,  0.0 ]),  // Right
            ( 0,-1, 0, 0.5,  [  0.0, -1.0,  0.0 ]),  // Bottom
            ( 0, 1, 0, 1.0,  [  0.0,  1.0,  0.0 ]),  // Top
            ( 0, 0,-1, 0.8,  [  0.0,  0.0, -1.0 ]),  // Back
            ( 0, 0, 1, 0.8,  [  0.0,  0.0,  1.0 ])   // Front
        ];
        
        // Iterate through all blocks
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let current = self.get_block(x, y, z);
                    if current.is_air() { continue; }
                    
                    let block_id = match current {
                        Block::Solid(id) => id,
                        Block::Air => continue,
                    };
                    
                    let current_transparent = is_transparent(current);
                    
                    // Check each face
                    for &(dx, dy, dz, shade_factor, normal_vec) in &directions {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        let nz = z as i32 + dz;
                        
                        let neighbor = neighbor_block(nx, ny, nz);
                        let neighbor_transparent = is_transparent(neighbor);
                        
                        // Determine if we should show this face
                        let should_show = if current_transparent {
                            neighbor.is_air()
                        } else {
                            neighbor.is_air() || neighbor_transparent
                        };
                        
                        if !should_show { continue; }
                        
                        // Get texture and alpha for this face
                        let (tile_idx, alpha) = get_block_texture(block_id, dx, dy, dz);
                        let (u_min, v_min, u_max, v_max) = get_tile_uvs(tile_idx);
                        
                        // Calculate vertex positions for this face
                        let fx = x as f32;
                        let fy = y as f32;
                        let fz = z as f32;
                        
                        // Define face vertices based on direction
                        // For CCW front-face winding with back-face culling:
                        // When looking at face from outside, vertices go counter-clockwise
                        let (corners, uvs) = match (dx, dy, dz) {
                            (-1, 0, 0) => {
                                // Left face (-X): looking from -X toward +X, CCW is: bottom-back, bottom-front, top-front, top-back
                                ([
                                    [fx, fy, fz],           // bottom-back
                                    [fx, fy, fz + 1.0],     // bottom-front
                                    [fx, fy + 1.0, fz + 1.0], // top-front
                                    [fx, fy + 1.0, fz],     // top-back
                                ], [
                                    [u_max, v_max], [u_min, v_max], [u_min, v_min], [u_max, v_min],
                                ])
                            }
                            (1, 0, 0) => {
                                // Right face (+X): looking from +X toward -X, CCW is: bottom-front, bottom-back, top-back, top-front
                                ([
                                    [fx + 1.0, fy, fz + 1.0],     // bottom-front
                                    [fx + 1.0, fy, fz],           // bottom-back
                                    [fx + 1.0, fy + 1.0, fz],     // top-back
                                    [fx + 1.0, fy + 1.0, fz + 1.0], // top-front
                                ], [
                                    [u_max, v_max], [u_min, v_max], [u_min, v_min], [u_max, v_min],
                                ])
                            }
                            (0, -1, 0) => {
                                // Bottom face (-Y): looking from -Y toward +Y, CCW is: back-left, back-right, front-right, front-left
                                ([
                                    [fx, fy, fz],             // back-left
                                    [fx + 1.0, fy, fz],       // back-right
                                    [fx + 1.0, fy, fz + 1.0], // front-right
                                    [fx, fy, fz + 1.0],       // front-left
                                ], [
                                    [u_min, v_min], [u_max, v_min], [u_max, v_max], [u_min, v_max],
                                ])
                            }
                            (0, 1, 0) => {
                                // Top face (+Y): looking from +Y toward -Y, CCW is: front-left, front-right, back-right, back-left
                                ([
                                    [fx, fy + 1.0, fz + 1.0],       // front-left
                                    [fx + 1.0, fy + 1.0, fz + 1.0], // front-right
                                    [fx + 1.0, fy + 1.0, fz],       // back-right
                                    [fx, fy + 1.0, fz],             // back-left
                                ], [
                                    [u_min, v_max], [u_max, v_max], [u_max, v_min], [u_min, v_min],
                                ])
                            }
                            (0, 0, -1) => {
                                // Back face (-Z): looking from -Z toward +Z, CCW is: bottom-right, bottom-left, top-left, top-right
                                ([
                                    [fx + 1.0, fy, fz],       // bottom-right
                                    [fx, fy, fz],             // bottom-left
                                    [fx, fy + 1.0, fz],       // top-left
                                    [fx + 1.0, fy + 1.0, fz], // top-right
                                ], [
                                    [u_max, v_max], [u_min, v_max], [u_min, v_min], [u_max, v_min],
                                ])
                            }
                            (0, 0, 1) => {
                                // Front face (+Z): looking from +Z toward -Z, CCW is: bottom-left, bottom-right, top-right, top-left
                                ([
                                    [fx, fy, fz + 1.0],             // bottom-left
                                    [fx + 1.0, fy, fz + 1.0],       // bottom-right
                                    [fx + 1.0, fy + 1.0, fz + 1.0], // top-right
                                    [fx, fy + 1.0, fz + 1.0],       // top-left
                                ], [
                                    [u_min, v_max], [u_max, v_max], [u_max, v_min], [u_min, v_min],
                                ])
                            }
                            _ => continue,
                        };
                        
                        let (nx, ny, nz) = (normal_vec[0], normal_vec[1], normal_vec[2]);
                        let shade = shade_factor as f32;
                        
                        // Choose which vertex buffer
                        let vertices = if current_transparent { &mut transparent_vertices } else { &mut opaque_vertices };
                        
                        // Emit vertex: pos(3) + normal(3) + uv(2) + color(4) = 12 floats
                        let emit_vertex = |v: &mut Vec<f32>, pos: [f32; 3], uv: [f32; 2]| {
                            v.extend_from_slice(&[pos[0], pos[1], pos[2]]);
                            v.extend_from_slice(&[nx, ny, nz]);
                            v.extend_from_slice(&[uv[0], uv[1]]);
                            v.extend_from_slice(&[shade, shade, shade, alpha]);
                        };
                        
                        // Two triangles: 0-1-2 and 0-2-3 (CCW winding)
                        emit_vertex(vertices, corners[0], uvs[0]);
                        emit_vertex(vertices, corners[1], uvs[1]);
                        emit_vertex(vertices, corners[2], uvs[2]);
                        
                        emit_vertex(vertices, corners[0], uvs[0]);
                        emit_vertex(vertices, corners[2], uvs[2]);
                        emit_vertex(vertices, corners[3], uvs[3]);
                    }
                }
            }
        }
        
        self.mesh = if opaque_vertices.is_empty() { None } else { Some(Mesh::from_vertices(&opaque_vertices)) };
        self.transparent_mesh = if transparent_vertices.is_empty() { None } else { Some(Mesh::from_vertices(&transparent_vertices)) };
        self.dirty = false;
    }
}
