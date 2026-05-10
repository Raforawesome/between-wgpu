use glam::{IVec3, U8Vec3};

use crate::{
    HashMap,
    constants::{CHUNK_SIZE, CHUNK_SIZEI, CHUNK_VOL},
    world::{
        block::BlockType,
        chunks::{Chunk, linearize},
    },
    worldgen::DensitySampler,
};

#[derive(Default)]
pub struct ChunkManager<S: DensitySampler> {
    chunk_map: HashMap<IVec3, Chunk>,
    density_sampler: S,
}

impl<S: DensitySampler> ChunkManager<S> {
    // private functions
    pub fn generate_chunk_with_sampler(density_sampler: &S, coords: &IVec3) -> Chunk {
        let IVec3 { x, y, z } = coords;
        let mut blocks = [BlockType::Air; CHUNK_VOL];
        let mut solid_mask = [[0; CHUNK_SIZE]; CHUNK_SIZE];

        // initial block {x, y, z}
        let (ibx, iby, ibz) = (x * CHUNK_SIZEI, y * CHUNK_SIZEI, z * CHUNK_SIZEI);
        for bx in ibx..(ibx + CHUNK_SIZEI) {
            for by in iby..(iby + CHUNK_SIZEI) {
                for bz in ibz..(ibz + CHUNK_SIZEI) {
                    let (lx, ly, lz) = (bx - ibx, by - iby, bz - ibz);
                    let idx = linearize(U8Vec3::new(lx as u8, ly as u8, lz as u8));

                    let block_type =
                        density_sampler.get_block_type(bx as f32, by as f32, bz as f32);
                    blocks[idx] = block_type;
                    if block_type != BlockType::Air {
                        solid_mask[lz as usize][ly as usize] |= 1 << lx;
                    }
                }
            }
        }

        Chunk {
            coords: *coords,
            blocks,
            solid_mask,
            dirty: false,
        }
    }

    pub fn generate_chunk(&self, coords: &IVec3) -> Chunk {
        Self::generate_chunk_with_sampler(&self.density_sampler, coords)
    }

    // public functions
    pub fn world_to_chunk_coords(world_pos: IVec3) -> (IVec3, U8Vec3) {
        let chunk_coords = world_pos.div_euclid(IVec3::splat(CHUNK_SIZEI));
        let local_pos = world_pos.rem_euclid(IVec3::splat(CHUNK_SIZEI)).as_u8vec3();
        (chunk_coords, local_pos)
    }

    pub fn set_block(&mut self, world_pos: IVec3, block_type: BlockType) -> Vec<IVec3> {
        let (chunk_coords, local_pos) = Self::world_to_chunk_coords(world_pos);

        let mut affected_chunks = vec![chunk_coords];

        if let Some(chunk) = self.chunk_map.get_mut(&chunk_coords) {
            let idx = linearize(local_pos);
            chunk.blocks[idx] = block_type;

            // check boundaries for neighbor updates
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

    pub fn get_chunk(&self, chunk_coords: &IVec3) -> Option<&Chunk> {
        self.chunk_map.get(chunk_coords)
    }

    pub fn insert_chunk(&mut self, chunk: Chunk) {
        self.chunk_map.insert(chunk.coords(), chunk);
    }

    pub fn chunk_iter(&self) -> impl Iterator<Item = (&IVec3, &Chunk)> {
        self.chunk_map.iter()
    }

    // density sampler related methods
    pub fn new_with_seed(seed: i32) -> Self {
        Self {
            chunk_map: HashMap::default(),
            density_sampler: S::with_seed(seed),
        }
    }
}
