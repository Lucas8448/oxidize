use std::collections::HashMap;
use super::{chunk::{Chunk}, generation::generate_chunk};

pub type ChunkPos = (i32,i32);

pub struct World {
    pub seed: u32,
    chunks: HashMap<ChunkPos, Chunk>,
}

impl World {
    pub fn new(seed: u32) -> Self { Self { seed, chunks: HashMap::new() } }

    pub fn ensure_chunk(&mut self, cx: i32, cz: i32) {
        self.chunks.entry((cx,cz)).or_insert_with(|| generate_chunk(self.seed, cx, cz));
    }

    pub fn get_chunk(&self, cx: i32, cz: i32) -> Option<&Chunk> { self.chunks.get(&(cx,cz)) }
}
