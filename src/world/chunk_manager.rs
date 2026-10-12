use std::sync::Arc;

use glam::IVec3;

use crate::{HashMap, world::chunks::Chunk};

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

    pub fn insert_chunk(&mut self, chunk: Chunk) {
        self.chunk_map.insert(chunk.coords(), Arc::new(chunk));
    }

    pub fn get_chunk(&self, chunk_coords: &IVec3) -> Option<&Arc<Chunk>> {
        self.chunk_map.get(chunk_coords)
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
}
