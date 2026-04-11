use glam::{IVec3, Vec3};
use itertools::Itertools as _;

use crate::constants::{CHUNK_SIZEI, VIEW_DISTANCE, VIEW_DISTANCE_Y};

const RENDER_RANGE_X: std::ops::Range<i32> = -VIEW_DISTANCE..VIEW_DISTANCE;
const RENDER_RANGE_Y: std::ops::Range<i32> = -VIEW_DISTANCE_Y..VIEW_DISTANCE_Y;
const RENDER_RANGE_Z: std::ops::Range<i32> = -VIEW_DISTANCE..VIEW_DISTANCE;

const RENDER_VOLUME: usize =
    (VIEW_DISTANCE * 2 * VIEW_DISTANCE_Y * 2 * VIEW_DISTANCE * 2) as usize;

pub struct ChunkLoader {
    loaded: Vec<IVec3>,
    last_chunk_pos: IVec3,
}

impl ChunkLoader {
    pub fn loaded_chunks(&self) -> &[IVec3] {
        &self.loaded
    }

    pub fn update(&mut self, player_pos: Vec3) {
        let player_chunk_pos =
            player_pos.floor().as_ivec3().div_euclid(IVec3::splat(CHUNK_SIZEI));

        if !self.loaded.is_empty() && player_chunk_pos == self.last_chunk_pos {
            return;
        }

        self.last_chunk_pos = player_chunk_pos;
        let mut queued = Vec::with_capacity(RENDER_VOLUME);
        let IVec3 { x, y, z } = player_chunk_pos;

        // index, x offset, y offset, z offset
        for (xo, yo, zo) in RENDER_RANGE_X
            .cartesian_product(RENDER_RANGE_Y)
            .cartesian_product(RENDER_RANGE_Z)
            .map(|((xo, yo), zo)| (xo, yo, zo))
        {
            queued.push(IVec3 {
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
            loaded: Vec::with_capacity(RENDER_VOLUME),
            last_chunk_pos: IVec3::splat(i32::MAX),
        }
    }
}
