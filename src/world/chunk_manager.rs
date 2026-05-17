use glam::{IVec3, U8Vec3};

use crate::{
    HashMap,
    constants::{CHUNK_SIZE, CHUNK_SIZEI, CHUNK_VOL},
    world::{
        WorldSampler,
        block::BlockType,
        chunks::{Chunk, linearize},
    },
    worldgen::DensitySampler,
};

#[derive(Default)]
pub struct ChunkManager {
    chunk_map: HashMap<IVec3, Chunk>,
    density_sampler: WorldSampler,
}

pub struct NeighbourSnapshot {
    pub top: Chunk,
    pub bottom: Chunk,
    pub left: Chunk,
    pub right: Chunk,
    pub front: Chunk,
    pub back: Chunk,
}

impl ChunkManager {
    // private functions
    pub fn fill_chunk_with_sampler<S: DensitySampler>(
        density_sampler: &S,
        coords: &IVec3,
    ) -> Chunk {
        let IVec3 { x, y, z } = coords;
        let mut blocks = [BlockType::Air; CHUNK_VOL];
        let mut solid_mask = [[0; CHUNK_SIZE]; CHUNK_SIZE];

        // initial block {x, y, z}
        let (ibx, iby, ibz) = (x * CHUNK_SIZEI, y * CHUNK_SIZEI, z * CHUNK_SIZEI);
        let densities = density_sampler.sample_chunk_densities(*coords);

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

        Chunk {
            coords: *coords,
            blocks,
            solid_mask,
            dirty: false,
        }
    }

    // public functions
    pub fn fill_chunk(&self, coords: IVec3) -> Chunk {
        Self::fill_chunk_with_sampler(&self.density_sampler, &coords)
    }

    pub fn world_to_chunk_coords(world_pos: IVec3) -> (IVec3, U8Vec3) {
        let chunk_coords = world_pos.div_euclid(IVec3::splat(CHUNK_SIZEI));
        let local_pos = world_pos.rem_euclid(IVec3::splat(CHUNK_SIZEI)).as_u8vec3();
        (chunk_coords, local_pos)
    }

    // chunk data editing
    pub fn generate_chunk(&mut self, coords: IVec3) {
        let chunk = Self::fill_chunk_with_sampler(&self.density_sampler, &coords);
        self.chunk_map.insert(coords, chunk);
    }

    pub fn insert_chunk(&mut self, chunk: Chunk) {
        self.chunk_map.insert(chunk.coords(), chunk);
    }

    pub fn get_chunk(&self, chunk_coords: &IVec3) -> Option<&Chunk> {
        self.chunk_map.get(chunk_coords)
    }

    pub fn get_chunk_mut(&mut self, chunk_coords: &IVec3) -> Option<&mut Chunk> {
        self.chunk_map.get_mut(chunk_coords)
    }

    pub fn chunk_snapshot(&self, chunk_coords: &IVec3) -> Option<Chunk> {
        self.chunk_map.get(chunk_coords).cloned()
    }

    pub fn has_neighbours(&self, chunk_coords: &IVec3) -> bool {
        let top = chunk_coords + IVec3::Y;
        let bottom = chunk_coords + IVec3::NEG_Y;
        let left = chunk_coords + IVec3::NEG_X;
        let right = chunk_coords + IVec3::X;
        let front = chunk_coords + IVec3::Z;
        let back = chunk_coords + IVec3::NEG_Z;

        self.chunk_map.contains_key(&top)
            && self.chunk_map.contains_key(&bottom)
            && self.chunk_map.contains_key(&left)
            && self.chunk_map.contains_key(&right)
            && self.chunk_map.contains_key(&front)
            && self.chunk_map.contains_key(&back)
    }

    pub fn neighbour_snapshot(&self, chunk_coords: &IVec3) -> Option<NeighbourSnapshot> {
        let top = chunk_coords + IVec3::Y;
        let bottom = chunk_coords + IVec3::NEG_Y;
        let left = chunk_coords + IVec3::NEG_X;
        let right = chunk_coords + IVec3::X;
        let front = chunk_coords + IVec3::Z;
        let back = chunk_coords + IVec3::NEG_Z;

        let snapshot = NeighbourSnapshot {
            top: self.chunk_map.get(&top).cloned()?,
            bottom: self.chunk_map.get(&bottom).cloned()?,
            left: self.chunk_map.get(&left).cloned()?,
            right: self.chunk_map.get(&right).cloned()?,
            front: self.chunk_map.get(&front).cloned()?,
            back: self.chunk_map.get(&back).cloned()?,
        };

        Some(snapshot)
    }

    pub fn chunk_iter(&self) -> impl Iterator<Item = (&IVec3, &Chunk)> {
        self.chunk_map.iter()
    }

    // density sampler related methods
    pub fn new_with_sampler(sampler: WorldSampler) -> Self {
        Self {
            chunk_map: HashMap::default(),
            density_sampler: sampler,
        }
    }
}
