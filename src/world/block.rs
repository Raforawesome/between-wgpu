#[derive(Copy, Clone, Eq, Hash, PartialEq, Debug, bytemuck::NoUninit)]
#[repr(u8)]
pub enum BlockType {
    // GPU IDs: keep these values in sync with fragment.slang's atlas_layer.
    Grass = 0,
    Dirt = 1,
    Stone = 2,
    Water = 3,
    Air = 4,
    Sand = 5,
    Log = 6,
    Leaf = 7,
}

impl BlockType {
    pub const fn is_clear(&self) -> bool {
        matches!(self, BlockType::Air | BlockType::Water)
    }
}
