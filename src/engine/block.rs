/// Represents a block in the voxel world.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Block {
    /// Empty space
    Air,
    /// A solid block with a type ID
    Solid(u8),
}

impl Block {
    pub fn is_air(&self) -> bool { matches!(self, Block::Air) }
    
    #[allow(dead_code)]
    pub fn is_solid(&self) -> bool { matches!(self, Block::Solid(_)) }
    
    #[allow(dead_code)]
    pub fn block_id(&self) -> Option<u8> {
        match self {
            Block::Solid(id) => Some(*id),
            Block::Air => None,
        }
    }
}
