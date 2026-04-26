use std::ops::Range;

use glam::{IVec3, Vec3};
use itertools::Itertools as _;

use crate::{
    HashSet,
    constants::{CHUNK_SIZEI, VIEW_DISTANCE, VIEW_DISTANCE_Y},
};

const RENDER_RANGE_X: Range<i32> = -VIEW_DISTANCE..VIEW_DISTANCE;
const RENDER_RANGE_Y: Range<i32> = -VIEW_DISTANCE_Y..VIEW_DISTANCE_Y;
const RENDER_RANGE_Z: Range<i32> = -VIEW_DISTANCE..VIEW_DISTANCE;

pub struct ChunkLoader {
    loaded: HashSet<IVec3>,
    last_chunk_pos: IVec3,
}

impl ChunkLoader {
    pub fn contains(&self, pos: &IVec3) -> bool {
        self.loaded.contains(pos)
    }

    pub fn loaded_chunks(&self) -> impl Iterator<Item = &IVec3> {
        self.loaded.iter()
    }

    pub fn update(&mut self, player_pos: Vec3) {
        let player_chunk_pos = player_pos
            .floor()
            .as_ivec3()
            .div_euclid(IVec3::splat(CHUNK_SIZEI));

        if !self.loaded.is_empty() && player_chunk_pos == self.last_chunk_pos {
            return;
        }

        self.last_chunk_pos = player_chunk_pos;
        let mut queued = HashSet::default();
        let IVec3 { x, y, z } = player_chunk_pos;

        // index, x offset, y offset, z offset
        for (xo, yo, zo) in RENDER_RANGE_X
            .cartesian_product(RENDER_RANGE_Y)
            .cartesian_product(RENDER_RANGE_Z)
            .map(|((xo, yo), zo)| (xo, yo, zo))
        {
            queued.insert(IVec3 {
                x: x + xo,
                y: y + yo,
                z: z + zo,
            });
        }

        self.loaded = queued;
    }
}

impl Default for ChunkLoader {
    fn default() -> Self {
        ChunkLoader {
            loaded: HashSet::default(),
            last_chunk_pos: IVec3::splat(i32::MAX),
        }
    }
}
