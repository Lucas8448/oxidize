use super::block::Block;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_HEIGHT: usize = 64;

pub struct Chunk {
    pub blocks: [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_HEIGHT],
}

impl Chunk {
    pub fn empty() -> Self { Self { blocks: [[[Block::Air; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_HEIGHT] } }

    pub fn get(&self, x: usize, y: usize, z: usize) -> Block {
        if x < CHUNK_SIZE && z < CHUNK_SIZE && y < CHUNK_HEIGHT { self.blocks[y][z][x] } else { Block::Air }
    }
}
