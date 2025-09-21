use super::{chunk::{Chunk, CHUNK_SIZE, CHUNK_HEIGHT}, block::{Block, BlockRegistry}};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

#[allow(dead_code)]
pub struct ChunkMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

fn emit_face(vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>, next: &mut u32,
             p: [[f32;3];4], tile: (u32,u32), tile_size: f32, flip_v: bool) {
    let (tx, ty) = tile;
    let u0 = tx as f32 * tile_size; let v0 = ty as f32 * tile_size;
    let u1 = u0 + tile_size;        let v1 = v0 + tile_size;
    let (t00, t10, t01, t11) = if flip_v { (v1, v1, v0, v0) } else { (v0, v0, v1, v1) };
    let a = glam::Vec3::from(p[0]);
    let b = glam::Vec3::from(p[1]);
    let n = (glam::Vec3::from(p[2]) - a).cross(b - a).normalize_or_zero();
    let normal = [n.x, n.y, n.z];
    let quad = [
        Vertex { position: p[0], normal, uv: [u0, t00] },
        Vertex { position: p[1], normal, uv: [u1, t10] },
        Vertex { position: p[2], normal, uv: [u0, t01] },
        Vertex { position: p[3], normal, uv: [u1, t11] },
    ];
    vertices.extend_from_slice(&quad);
    indices.extend_from_slice(&[*next, *next + 1, *next + 2, *next + 2, *next + 1, *next + 3]);
    *next += 4;
}

#[allow(dead_code)]
pub fn build_chunk_mesh<'a, F>(get_chunk: &F, cx: i32, cz: i32) -> ChunkMesh
where F: Fn(i32,i32) -> Option<&'a Chunk> {
    let reg = BlockRegistry::basic();
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut next: u32 = 0;
    let tile_size = 0.25f32;

    let block_at = |wx: i32, wy: i32, wz: i32| -> Block {
        if wy < 0 || wy >= CHUNK_HEIGHT as i32 { return Block::Air; }
        let mut ccx = cx; let mut ccz = cz;
        let mut lx = wx - cx * CHUNK_SIZE as i32;
        let mut lz = wz - cz * CHUNK_SIZE as i32;
        while lx < 0 { ccx -= 1; lx += CHUNK_SIZE as i32; }
        while lz < 0 { ccz -= 1; lz += CHUNK_SIZE as i32; }
        while lx >= CHUNK_SIZE as i32 { ccx += 1; lx -= CHUNK_SIZE as i32; }
        while lz >= CHUNK_SIZE as i32 { ccz += 1; lz -= CHUNK_SIZE as i32; }
        if let Some(chunk) = get_chunk(ccx, ccz) { chunk.get(lx as usize, wy as usize, lz as usize) } else { Block::Air }
    };

    let chunk = get_chunk(cx, cz).expect("chunk must exist before building mesh");
    for y in 0..CHUNK_HEIGHT {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let block = chunk.get(x,y,z);
                if block == Block::Air { continue; }
                let world_x = cx * CHUNK_SIZE as i32 + x as i32;
                let world_z = cz * CHUNK_SIZE as i32 + z as i32;
                let surface = y + 1 == CHUNK_HEIGHT || block_at(world_x, y as i32 + 1, world_z) == Block::Air;
                let tex = reg.get(block).unwrap();
                let top_tile = tex.top;
                let side_tile = if let Block::Grass = block { if surface { tex.side } else { reg.get(Block::Dirt).unwrap().side } } else { tex.side };
                let fx = world_x as f32; let fy = y as f32; let fz = world_z as f32;
                if surface { emit_face(&mut vertices,&mut indices,&mut next,
                    [ [fx,fy+1.0,fz], [fx+1.0,fy+1.0,fz], [fx,fy+1.0,fz+1.0], [fx+1.0,fy+1.0,fz+1.0] ], top_tile, tile_size, false); }
                if block_at(world_x - 1, y as i32, world_z) == Block::Air { emit_face(&mut vertices,&mut indices,&mut next,
                    [ [fx,fy+1.0,fz], [fx,fy+1.0,fz+1.0], [fx,fy,fz], [fx,fy,fz+1.0] ], side_tile, tile_size, true); }
                if block_at(world_x + 1, y as i32, world_z) == Block::Air { emit_face(&mut vertices,&mut indices,&mut next,
                    [ [fx+1.0,fy+1.0,fz+1.0], [fx+1.0,fy+1.0,fz], [fx+1.0,fy,fz+1.0], [fx+1.0,fy,fz] ], side_tile, tile_size, true); }
                if block_at(world_x, y as i32, world_z - 1) == Block::Air { emit_face(&mut vertices,&mut indices,&mut next,
                    [ [fx+1.0,fy+1.0,fz], [fx,fy+1.0,fz], [fx+1.0,fy,fz], [fx,fy,fz] ], side_tile, tile_size, true); }
                if block_at(world_x, y as i32, world_z + 1) == Block::Air { emit_face(&mut vertices,&mut indices,&mut next,
                    [ [fx,fy+1.0,fz+1.0], [fx+1.0,fy+1.0,fz+1.0], [fx,fy,fz+1.0], [fx+1.0,fy,fz+1.0] ], side_tile, tile_size, true); }
            }
        }
    }

    ChunkMesh { vertices, indices }
}

impl Vertex {
    pub fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 0, shader_location: 0 },
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 12, shader_location: 1 },
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 24, shader_location: 2 },
            ],
        }
    }
}
