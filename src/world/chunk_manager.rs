use std::sync::Arc;

use glam::{IVec3, U8Vec3};

use crate::{
    HashMap,
    constants::CHUNK_SIZEI,
    world::chunks::Chunk,
};

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
            top: Arc::clone(self.chunk_map.get(&top)?),
            bottom: Arc::clone(self.chunk_map.get(&bottom)?),
            left: Arc::clone(self.chunk_map.get(&left)?),
            right: Arc::clone(self.chunk_map.get(&right)?),
            front: Arc::clone(self.chunk_map.get(&front)?),
            back: Arc::clone(self.chunk_map.get(&back)?),
        };

        Some(snapshot)
    }

    pub fn chunk_iter(&self) -> impl Iterator<Item = (&IVec3, &Arc<Chunk>)> {
        self.chunk_map.iter()
    }
}
