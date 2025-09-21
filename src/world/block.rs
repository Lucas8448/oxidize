#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Block {
    Air,
    Grass,
    Dirt,
    Stone,
}

pub type BlockId = Block;

impl Block {
    pub fn is_air(self) -> bool { matches!(self, Block::Air) }
}

#[derive(Clone, Copy, Debug)]
pub struct BlockTextureSet {
    pub top: (u32,u32),
    pub side: (u32,u32),
    pub bottom: (u32,u32),
}

pub struct BlockRegistry {
    entries: std::collections::HashMap<BlockId, BlockTextureSet>,
}

impl BlockRegistry {
    pub fn basic() -> Self {
        use Block::*;
        let mut map = std::collections::HashMap::new();
        map.insert(Grass, BlockTextureSet { top: (0,0), side: (3,0), bottom: (1,0) });
        map.insert(Dirt,  BlockTextureSet { top: (1,0), side: (1,0), bottom: (1,0) });
        map.insert(Stone, BlockTextureSet { top: (2,0), side: (2,0), bottom: (2,0) });
        Self { entries: map }
    }
    pub fn get(&self, b: BlockId) -> Option<&BlockTextureSet> { self.entries.get(&b) }
}
