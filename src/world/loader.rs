use glam::{IVec3, Vec3};

use crate::{
    HashSet,
    constants::{CHUNK_SIZEI, VIEW_DISTANCE, VIEW_DISTANCE_Y},
};

/// Chunks in the half-open view region around the player.
pub struct ChunkLoader {
    loaded: HashSet<IVec3>,
    last_chunk_pos: IVec3,
    horizontal: i32,
    vertical: i32,
}

impl ChunkLoader {
    pub fn new(horizontal: i32, vertical: i32) -> Self {
        assert!(horizontal > 0 && vertical > 0);
        Self {
            loaded: HashSet::default(),
            last_chunk_pos: IVec3::splat(i32::MAX),
            horizontal,
            vertical,
        }
    }

    pub fn contains(&self, pos: &IVec3) -> bool {
        self.loaded.contains(pos)
    }
    pub fn loaded_chunks(&self) -> impl Iterator<Item = &IVec3> {
        self.loaded.iter()
    }

    /// Rebuild only when crossing a chunk boundary.
    pub fn update(&mut self, player_pos: Vec3) {
        let centre = player_pos
            .floor()
            .as_ivec3()
            .div_euclid(IVec3::splat(CHUNK_SIZEI));
        if !self.loaded.is_empty() && centre == self.last_chunk_pos {
            return;
        }
        self.last_chunk_pos = centre;
        self.loaded.clear();
        for z in -self.horizontal..self.horizontal {
            for y in -self.vertical..self.vertical {
                for x in -self.horizontal..self.horizontal {
                    let pos = centre + IVec3::new(x, y, z);
                    self.loaded.insert(pos);
                }
            }
        }
    }
}

impl Default for ChunkLoader {
    fn default() -> Self {
        Self::new(VIEW_DISTANCE, VIEW_DISTANCE_Y)
    }
}
