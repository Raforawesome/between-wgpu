/*
 * Note that this file works in the chunk data space, NOT the real world
 * where +z means forward and we start at the top left. In the data space
 * we start at the bottom left. However, the methods that check blocks
 * check their real world equivalent (i.e., block_front checks in the +z direction)
 */

use crate::constants::{CHUNK_IDX, CHUNK_SIZE, CHUNK_VOL, Y_STRIDE, Z_STRIDE};
use crate::world::block::BlockType;
use glam::U8Vec3;

/// Contains a reference to a chunk's block array and provides
/// convenience methods to traverse the chunk one block at a time.
pub struct ChunkCursor<'a> {
    blocks: &'a [BlockType; CHUNK_VOL],
    pub pos: U8Vec3,
    pub index: usize,
}

// basic usage
impl<'a> ChunkCursor<'a> {
    pub fn at_origin(blocks: &'a [BlockType; CHUNK_VOL]) -> Self {
        Self {
            blocks,
            pos: U8Vec3::ZERO,
            index: 0,
        }
    }
}

// peek methods
impl<'a> ChunkCursor<'a> {
    pub fn block_above(&self) -> Option<BlockType> {
        if self.pos.y < CHUNK_IDX {
            Some(self.blocks[self.index + Y_STRIDE])
        } else {
            None
        }
    }

    pub fn block_below(&self) -> Option<BlockType> {
        if self.pos.y > 0 {
            Some(self.blocks[self.index - Y_STRIDE])
        } else {
            None
        }
    }

    pub fn block_left(&self) -> Option<BlockType> {
        if self.pos.x > 0 {
            Some(self.blocks[self.index - 1])
        } else {
            None
        }
    }

    pub fn block_right(&self) -> Option<BlockType> {
        if self.pos.x < CHUNK_IDX {
            Some(self.blocks[self.index + 1])
        } else {
            None
        }
    }

    pub fn block_front(&self) -> Option<BlockType> {
        if self.pos.z < CHUNK_IDX {
            Some(self.blocks[self.index + Z_STRIDE])
        } else {
            None
        }
    }

    pub fn block_back(&self) -> Option<BlockType> {
        if self.pos.z > 0 {
            Some(self.blocks[self.index - Z_STRIDE])
        } else {
            None
        }
    }
}

// traversal
impl<'a> ChunkCursor<'a> {
    pub fn advance(&mut self) -> bool {
        if self.index >= CHUNK_VOL - 1 {
            return false;
        }

        self.index += 1;
        self.pos.x += 1;

        if self.pos.x >= CHUNK_SIZE as u8 {
            self.pos.x = 0;
            self.pos.y += 1;
            if self.pos.y >= CHUNK_SIZE as u8 {
                self.pos.y = 0;
                self.pos.z += 1;
            }
        }

        true
    }
}
