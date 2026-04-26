use glam::{IVec3, U8Vec3};

use crate::{
    constants::{CHUNK_SIZE, CHUNK_SIZEI, CHUNK_VOL, ChunkMask},
    world::block::BlockType,
};

/// Takes 3 positive numbers, the local xyz coordinates within a chunk, and
/// returns an index number that represents the index of that block in a 1D array.
#[inline]
pub const fn linearize(pos: U8Vec3) -> usize {
    (pos.x as usize) + (pos.y as usize) * CHUNK_SIZE + (pos.z as usize) * CHUNK_SIZE * CHUNK_SIZE
}

#[inline]
pub const fn delinearize(idx: usize) -> U8Vec3 {
    U8Vec3::new(
        (idx % CHUNK_SIZE) as u8,
        ((idx / CHUNK_SIZE) % CHUNK_SIZE) as u8,
        ((idx / (CHUNK_SIZE * CHUNK_SIZE)) % CHUNK_SIZE) as u8,
    )
}

pub fn chunk_to_world_coords(chunk_coords: IVec3) -> IVec3 {
    chunk_coords * CHUNK_SIZEI
}

pub struct Chunk {
    pub coords: IVec3,
    pub blocks: [BlockType; CHUNK_VOL],
    pub dirty: bool,
    pub solid_mask: ChunkMask,
}

impl Chunk {
    pub fn get_block(&self, pos: U8Vec3) -> BlockType {
        self.blocks[linearize(pos)]
    }

    pub fn blocks(&self) -> &[BlockType; CHUNK_VOL] {
        &self.blocks
    }

    pub fn coords(&self) -> IVec3 {
        self.coords
    }

    pub fn chunk_to_world_space(&self, chunk_coords: U8Vec3) -> IVec3 {
        IVec3 {
            x: self.coords.x * CHUNK_SIZEI + chunk_coords.x as i32,
            y: self.coords.y * CHUNK_SIZEI + chunk_coords.y as i32,
            z: self.coords.z * CHUNK_SIZEI + chunk_coords.z as i32,
        }
    }

    pub fn is_empty(&self) -> bool {
        let rows = self.solid_mask.iter().flatten().copied();
        rows.reduce(|acc, row| acc | row).unwrap() == 0
    }

    pub fn neighbours_pos_list(&self) -> [IVec3; 6] {
        [
            IVec3::new(self.coords.x - 1, self.coords.y, self.coords.z),
            IVec3::new(self.coords.x + 1, self.coords.y, self.coords.z),
            IVec3::new(self.coords.x, self.coords.y - 1, self.coords.z),
            IVec3::new(self.coords.x, self.coords.y + 1, self.coords.z),
            IVec3::new(self.coords.x, self.coords.y, self.coords.z - 1),
            IVec3::new(self.coords.x, self.coords.y, self.coords.z + 1),
        ]
    }

    pub fn neighbours_pos_list_at(coords: IVec3) -> [IVec3; 6] {
        [
            IVec3::new(coords.x - 1, coords.y, coords.z),
            IVec3::new(coords.x + 1, coords.y, coords.z),
            IVec3::new(coords.x, coords.y - 1, coords.z),
            IVec3::new(coords.x, coords.y + 1, coords.z),
            IVec3::new(coords.x, coords.y, coords.z - 1),
            IVec3::new(coords.x, coords.y, coords.z + 1),
        ]
    }
}
