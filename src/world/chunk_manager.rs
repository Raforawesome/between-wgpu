use std::sync::Arc;

use glam::{IVec3, U8Vec3};

use crate::{HashMap, constants::CHUNK_SIZEI, world::chunks::Chunk};

#[derive(Default)]
pub struct ChunkManager {
    chunk_map: HashMap<IVec3, Arc<Chunk>>,
}

pub struct NeighbourSnapshot {
    pub top: Arc<Chunk>,
    pub bottom: Arc<Chunk>,
    pub left: Arc<Chunk>,
    pub right: Arc<Chunk>,
    pub front: Arc<Chunk>,
    pub back: Arc<Chunk>,
}

impl ChunkManager {
    pub fn len(&self) -> usize {
        self.chunk_map.len()
    }
    pub fn is_empty(&self) -> bool {
        self.chunk_map.is_empty()
    }
    // public functions
    pub fn new() -> Self {
        Self::default()
    }

    pub fn world_to_chunk_coords(world_pos: IVec3) -> (IVec3, U8Vec3) {
        let chunk_coords = world_pos.div_euclid(IVec3::splat(CHUNK_SIZEI));
        let local_pos = world_pos.rem_euclid(IVec3::splat(CHUNK_SIZEI)).as_u8vec3();
        (chunk_coords, local_pos)
    }

    pub fn insert_chunk(&mut self, chunk: Chunk) {
        self.chunk_map.insert(chunk.coords(), Arc::new(chunk));
    }

    pub fn get_chunk(&self, chunk_coords: &IVec3) -> Option<&Arc<Chunk>> {
        self.chunk_map.get(chunk_coords)
    }

    pub fn get_chunk_mut(&mut self, chunk_coords: &IVec3) -> Option<&mut Arc<Chunk>> {
        self.chunk_map.get_mut(chunk_coords)
    }

    pub fn chunk_snapshot(&self, chunk_coords: &IVec3) -> Option<Arc<Chunk>> {
        self.chunk_map.get(chunk_coords).cloned()
    }

    pub fn neighbour_snapshot(&self, chunk_coords: &IVec3) -> Option<NeighbourSnapshot> {
        // Check availability before cloning, so incomplete neighbourhoods do no refcount work.
        let top = self.chunk_map.get(&(chunk_coords + IVec3::Y))?;
        let bottom = self.chunk_map.get(&(chunk_coords + IVec3::NEG_Y))?;
        let left = self.chunk_map.get(&(chunk_coords + IVec3::NEG_X))?;
        let right = self.chunk_map.get(&(chunk_coords + IVec3::X))?;
        let front = self.chunk_map.get(&(chunk_coords + IVec3::Z))?;
        let back = self.chunk_map.get(&(chunk_coords + IVec3::NEG_Z))?;

        Some(NeighbourSnapshot {
            top: Arc::clone(top),
            bottom: Arc::clone(bottom),
            left: Arc::clone(left),
            right: Arc::clone(right),
            front: Arc::clone(front),
            back: Arc::clone(back),
        })
    }

    pub fn chunk_iter(&self) -> impl Iterator<Item = (&IVec3, &Arc<Chunk>)> {
        self.chunk_map.iter()
    }
}
