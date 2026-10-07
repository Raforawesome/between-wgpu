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

    /// Rebuild only when crossing a chunk boundary. Returns whether the region changed.
    pub fn update(&mut self, player_pos: Vec3) -> bool {
        let centre = player_pos
            .floor()
            .as_ivec3()
            .div_euclid(IVec3::splat(CHUNK_SIZEI));
        if !self.loaded.is_empty() && centre == self.last_chunk_pos {
            return false;
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
        true
    }
}

impl Default for ChunkLoader {
    fn default() -> Self {
        Self::new(VIEW_DISTANCE, VIEW_DISTANCE_Y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_changes_only_on_initial_load_and_chunk_crossings() {
        let mut loader = ChunkLoader::new(2, 1);
        assert!(loader.update(Vec3::ZERO));
        assert_eq!(loader.loaded_chunks().count(), 32);
        assert!(!loader.update(Vec3::splat(15.9)));

        assert!(loader.update(Vec3::new(16.0, 0.0, 0.0)));
        assert!(!loader.contains(&IVec3::new(-2, 0, 0)));
        assert!(loader.contains(&IVec3::new(2, 0, 0)));
        assert!(!loader.update(Vec3::new(16.0, 0.0, 0.0)));
    }

    #[test]
    fn negative_positions_cross_boundaries_using_floor_division() {
        let mut loader = ChunkLoader::new(1, 1);
        assert!(loader.update(Vec3::ZERO));
        assert!(loader.update(Vec3::new(-0.1, 0.0, 0.0)));
        assert!(loader.contains(&IVec3::new(-2, 0, 0)));
        assert!(!loader.contains(&IVec3::ZERO));
        assert!(!loader.update(Vec3::new(-16.0, 0.0, 0.0)));
        assert!(loader.update(Vec3::new(-16.1, 0.0, 0.0)));
        assert!(loader.contains(&IVec3::new(-3, 0, 0)));
    }
}
