use dashmap::mapref::one::Ref as MapRef;
use glam::{IVec3, U8Vec3};

use crate::{
    Dashmap,
    constants::{CHUNK_SIZE, CHUNK_SIZEI, CHUNK_VOL},
    world::{block::BlockType, worldgen},
};

pub type ChunkRef<'a> = MapRef<'a, IVec3, Chunk>;

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

#[derive(Default)]
pub struct ChunkManager {
    chunk_map: Dashmap<IVec3, Chunk>,
}

pub struct Chunk {
    coords: IVec3,
    blocks: [BlockType; CHUNK_VOL],
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
}

impl ChunkManager {
    // private functions
    pub fn generate_chunk(coords: IVec3) -> Chunk {
        let IVec3 { x, y, z } = coords;
        let mut blocks = [BlockType::Air; CHUNK_VOL];

        // initial block {x, y, z}
        let (ibx, iby, ibz) = (x * CHUNK_SIZEI, y * CHUNK_SIZEI, z * CHUNK_SIZEI);
        for bx in ibx..(ibx + CHUNK_SIZEI) {
            for by in iby..(iby + CHUNK_SIZEI) {
                for bz in ibz..(ibz + CHUNK_SIZEI) {
                    let (lx, ly, lz) = (bx - ibx, by - iby, bz - ibz);
                    let idx = linearize(U8Vec3::new(lx as u8, ly as u8, lz as u8));

                    blocks[idx] = worldgen::get_block_type(bx as f32, by as f32, bz as f32);
                }
            }
        }

        Chunk { coords, blocks }
    }

    // public functions
    pub fn world_to_chunk_coords(world_pos: IVec3) -> (IVec3, U8Vec3) {
        let chunk_coords = world_pos.div_euclid(IVec3::splat(CHUNK_SIZEI));
        let local_pos = world_pos.rem_euclid(IVec3::splat(CHUNK_SIZEI)).as_u8vec3();
        (chunk_coords, local_pos)
    }

    pub fn set_block(&self, world_pos: IVec3, block_type: BlockType) -> Vec<IVec3> {
        let (chunk_coords, local_pos) = Self::world_to_chunk_coords(world_pos);

        let mut affected_chunks = vec![chunk_coords];

        if let Some(mut chunk) = self.chunk_map.get_mut(&chunk_coords) {
            let idx = linearize(local_pos);
            chunk.blocks[idx] = block_type;

            // Check boundaries for neighbor updates
            if local_pos.x == 0 {
                affected_chunks.push(chunk_coords - IVec3::X);
            } else if local_pos.x == CHUNK_SIZE as u8 - 1 {
                affected_chunks.push(chunk_coords + IVec3::X);
            }

            if local_pos.y == 0 {
                affected_chunks.push(chunk_coords - IVec3::Y);
            } else if local_pos.y == CHUNK_SIZE as u8 - 1 {
                affected_chunks.push(chunk_coords + IVec3::Y);
            }

            if local_pos.z == 0 {
                affected_chunks.push(chunk_coords - IVec3::Z);
            } else if local_pos.z == CHUNK_SIZE as u8 - 1 {
                affected_chunks.push(chunk_coords + IVec3::Z);
            }
        }

        affected_chunks
    }

    pub fn chunk_map(&self) -> &Dashmap<IVec3, Chunk> {
        &self.chunk_map
    }

    pub fn get_chunk(&self, chunk_coords: &IVec3) -> Option<ChunkRef<'_>> {
        self.chunk_map.get(chunk_coords)
    }

    pub fn get_or_generate_chunk(&self, chunk_coords: IVec3) -> ChunkRef<'_> {
        self.chunk_map
            .entry(chunk_coords)
            .or_insert_with(|| Self::generate_chunk(chunk_coords))
            .downgrade()
    }

    // pub fn chunk_iter(&self) -> impl Iterator<Item = (&IVec3, &Chunk)> {
    //     self.chunk_map.iter()
    // }
}
