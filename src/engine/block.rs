#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Block {
    Air,
    Solid(u8),
}

impl Block {
    pub fn is_air(&self) -> bool { matches!(self, Block::Air) }
}
