use glam::{IVec3, U8Vec3};

use crate::{
    constants::{CHUNK_SIZE, CHUNK_SIZEI, CHUNK_VOL, ChunkMask},
    world::block::BlockType,
    worldgen::DensitySampler,
};

/// Takes 3 positive numbers, the local xyz coordinates within a chunk, and
/// returns an index number that represents the index of that block in a 1D array.
#[inline]
pub const fn linearize_xyz(x: usize, y: usize, z: usize) -> usize {
    x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE
}

#[inline]
pub const fn linearize(pos: U8Vec3) -> usize {
    linearize_xyz(pos.x as usize, pos.y as usize, pos.z as usize)
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

#[derive(Clone)]
pub struct Chunk {
    pub coords: IVec3,
    pub blocks: [BlockType; CHUNK_VOL],
    pub dirty: bool,
    pub solid_mask: ChunkMask,
}

impl Chunk {
    pub fn from_sampler<S: DensitySampler>(coords: IVec3, density_sampler: &S) -> Self {
        let IVec3 { x, y, z } = coords;
        let mut blocks = [BlockType::Air; CHUNK_VOL];
        let mut solid_mask = [[0; CHUNK_SIZE]; CHUNK_SIZE];

        // initial block {x, y, z}
        let (ibx, iby, ibz) = (x * CHUNK_SIZEI, y * CHUNK_SIZEI, z * CHUNK_SIZEI);
        let densities = density_sampler.sample_chunk_densities(coords);

        for bz in ibz..(ibz + CHUNK_SIZEI) {
            let lz = bz - ibz;

            for by in iby..(iby + CHUNK_SIZEI) {
                let ly = by - iby;

                for bx in ibx..(ibx + CHUNK_SIZEI) {
                    let lx = bx - ibx;
                    let idx = linearize(U8Vec3::new(lx as u8, ly as u8, lz as u8));

                    let block_type =
                        density_sampler.density_to_block_type(densities[idx], by as f32);
                    blocks[idx] = block_type;
                    if block_type != BlockType::Air {
                        solid_mask[lz as usize][ly as usize] |= 1 << lx;
                    }
                }
            }
        }

        Self {
            coords,
            blocks,
            solid_mask,
            dirty: false,
        }
    }

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
