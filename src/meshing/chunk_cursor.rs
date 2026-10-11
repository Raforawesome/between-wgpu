/*
 * Note that this file works in the chunk data space, NOT the real world
 * where +z means forward and we start at the top left. In the data space
 * we start at the bottom left. However, the methods that check blocks
 * check their real world equivalent (i.e., block_front checks in the +z direction)
 */

use crate::constants::{CHUNK_IDX, CHUNK_SIZE, CHUNK_VOL, Y_INC, Z_INC_IDX};
use crate::world::{block::BlockType, chunks};
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

    pub fn at(pos: U8Vec3, blocks: &'a [BlockType; CHUNK_VOL]) -> Self {
        Self {
            blocks,
            pos,
            index: chunks::linearize(pos),
        }
    }

    pub fn block_at_pos(&self) -> BlockType {
        self.blocks[self.index]
    }
}

// peek methods
impl<'a> ChunkCursor<'a> {
    pub fn peek(&self, dx: i32, dy: i32, dz: i32) -> Option<BlockType> {
        let x = self.pos.x as i32 + dx;
        let y = self.pos.y as i32 + dy;
        let z = self.pos.z as i32 + dz;

        if x < 0
            || x > CHUNK_IDX as i32
            || y < 0
            || y > CHUNK_IDX as i32
            || z < 0
            || z > CHUNK_IDX as i32
        {
            None
        } else {
            let index =
                x as usize + (y as usize) * CHUNK_SIZE + (z as usize) * CHUNK_SIZE * CHUNK_SIZE;
            Some(self.blocks[index])
        }
    }

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

// move methods
impl<'a> ChunkCursor<'a> {
    pub fn move_above(&mut self) -> bool {
        if self.pos.y < CHUNK_IDX {
            self.pos.y += 1;
            self.index += CHUNK_SIZE;
            true
        } else {
            false
        }
    }

    pub fn move_below(&mut self) -> bool {
        if self.pos.y > 0 {
            self.pos.y -= 1;
            self.index -= CHUNK_SIZE;
            true
        } else {
            false
        }
    }

    pub fn move_left(&mut self) -> bool {
        if self.pos.x > 0 {
            self.pos.x -= 1;
            self.index -= 1;
            true
        } else {
            false
        }
    }

    pub fn move_right(&mut self) -> bool {
        if self.pos.x < CHUNK_IDX {
            self.pos.x += 1;
            self.index += 1;
            true
        } else {
            false
        }
    }

    pub fn move_forward(&mut self) -> bool {
        if self.pos.z < CHUNK_IDX {
            self.pos.z += 1;
            self.index += Y_INC;
            true
        } else {
            false
        }
    }

    pub fn move_back(&mut self) -> bool {
        if self.pos.z > 0 {
            self.pos.z -= 1;
            self.index -= Y_INC;
            true
        } else {
            false
        }
    }

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
