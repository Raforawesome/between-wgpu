use glam::IVec3;

use crate::meshing::faces::Face;

/// Marker component for chunk entities, stores chunk coordinates
#[derive(Copy, Clone, Eq, Hash, PartialEq, Debug)]
pub struct ChunkCoords(pub IVec3);

#[derive(Copy, Clone, Eq, Hash, PartialEq, Debug, bytemuck::NoUninit)]
#[repr(u8)]
pub enum BlockType {
    Grass,
    Dirt,
    Stone,
    Water,
    Air,
    Sand,
    Log,
    Leaf,
}

impl BlockType {
    pub const fn texture_index(&self, face: Face) -> usize {
        match self {
            BlockType::Air => usize::MAX,
            BlockType::Grass => match face {
                Face::Top => 0,
                Face::Bottom => 2,
                Face::Left | Face::Right | Face::Front | Face::Back => 1,
            },
            BlockType::Dirt => 2,
            BlockType::Stone => 3,
            BlockType::Sand => 4,
            BlockType::Log => 5,
            BlockType::Leaf => 6,
            BlockType::Water => 7,
        }
    }

    pub const fn rgba(&self) -> [u8; 4] {
        match self {
            BlockType::Air => [0, 0, 0, 0],
            BlockType::Grass => [76, 127, 76, 255],
            BlockType::Dirt => [139, 115, 76, 255],
            BlockType::Stone => [128, 128, 128, 255],
            BlockType::Water => [0, 127, 255, 170],
            BlockType::Sand => [255, 223, 128, 255],
            BlockType::Log => [139, 69, 19, 255],
            BlockType::Leaf => [34, 139, 34, 255],
        }
    }

    pub const fn is_clear(&self) -> bool {
        matches!(self, BlockType::Air | BlockType::Water)
    }
}
