pub mod chunk_cursor;
pub mod faces;

use glam::IVec3;

use crate::constants::{CHUNK_IDX, CHUNK_SIZEI, CHUNK_VOL};
use crate::meshing::{chunk_cursor::ChunkCursor, faces::Face};
use crate::render::vertex::Vertex;
use crate::world::{
    block::BlockType,
    chunk_manager::{ChunkManager, ChunkRef},
    chunks::Chunk,
};

use faces::{
    add_back_face, add_bottom_face, add_front_face, add_left_face, add_right_face, add_top_face,
};

/// Stores vertices and indices
#[derive(Default)]
pub struct ChunkMeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl From<ChunkMeshData> for (Vec<Vertex>, Vec<u32>) {
    fn from(val: ChunkMeshData) -> Self {
        (val.vertices, val.indices)
    }
}

/// Stores neighbouring chunks, if they exist
struct ChunkNeighbours<'a> {
    up: Option<ChunkRef<'a>>,
    down: Option<ChunkRef<'a>>,
    front: Option<ChunkRef<'a>>,
    back: Option<ChunkRef<'a>>,
    left: Option<ChunkRef<'a>>,
    right: Option<ChunkRef<'a>>,
}

impl ChunkManager {
    pub fn mesh_chunk(&self, chunk: &Chunk) -> ChunkMeshData {
        let chunk_neighbours = ChunkNeighbours {
            up: self.chunk_map().get(&(chunk.coords() + IVec3::Y)),
            down: self.chunk_map().get(&(chunk.coords() - IVec3::Y)),
            front: self.chunk_map().get(&(chunk.coords() + IVec3::Z)),
            back: self.chunk_map().get(&(chunk.coords() - IVec3::Z)),
            left: self.chunk_map().get(&(chunk.coords() - IVec3::X)),
            right: self.chunk_map().get(&(chunk.coords() + IVec3::X)),
        };

        if chunk.is_empty() {
            ChunkMeshData::default()
        } else {
            chunk_mesh_data(chunk, &chunk_neighbours)
        }
    }
}

fn chunk_mesh_data(chunk: &Chunk, neighbours: &ChunkNeighbours) -> ChunkMeshData {
    let mut mesh_data = ChunkMeshData {
        vertices: Vec::with_capacity(512),
        indices: Vec::with_capacity(768),
    };

    let base_pos = chunk.coords() * CHUNK_SIZEI;

    let blocks: &[BlockType; CHUNK_VOL] = chunk.blocks();
    let mut cursor: ChunkCursor = ChunkCursor::at_origin(blocks);

    for &block in blocks {
        match block {
            BlockType::Air => (),
            // the two branches below are almost identical.
            // the only difference is for water we add faces where air is,
            // and for everything else we add faces where any clear block is.
            BlockType::Water => {
                let pos = cursor.pos;

                if match cursor.block_above() {
                    Some(b) => b == BlockType::Air,
                    None => neighbours
                        .up
                        .as_deref()
                        .is_none_or(|c| c.get_block(pos.with_y(0)) == BlockType::Air),
                } {
                    add_top_face(
                        &mut mesh_data,
                        pos,
                        base_pos,
                        cursor.block_at_pos().texture_index(Face::Top),
                    );
                }

                if match cursor.block_below() {
                    Some(b) => b == BlockType::Air,
                    None => neighbours
                        .down
                        .as_deref()
                        .is_none_or(|c| c.get_block(pos.with_y(CHUNK_IDX)) == BlockType::Air),
                } {
                    add_bottom_face(
                        &mut mesh_data,
                        pos,
                        base_pos,
                        cursor.block_at_pos().texture_index(Face::Bottom),
                    );
                }

                if match cursor.block_front() {
                    Some(b) => b == BlockType::Air,
                    None => neighbours
                        .front
                        .as_deref()
                        .is_none_or(|c| c.get_block(pos.with_z(0)) == BlockType::Air),
                } {
                    add_front_face(
                        &mut mesh_data,
                        pos,
                        base_pos,
                        cursor.block_at_pos().texture_index(Face::Front),
                    );
                }

                if match cursor.block_back() {
                    Some(b) => b == BlockType::Air,
                    None => neighbours
                        .back
                        .as_deref()
                        .is_none_or(|c| c.get_block(pos.with_z(CHUNK_IDX)) == BlockType::Air),
                } {
                    add_back_face(
                        &mut mesh_data,
                        pos,
                        base_pos,
                        cursor.block_at_pos().texture_index(Face::Back),
                    );
                }

                if match cursor.block_left() {
                    Some(b) => b == BlockType::Air,
                    None => neighbours
                        .left
                        .as_deref()
                        .is_none_or(|c| c.get_block(pos.with_x(CHUNK_IDX)) == BlockType::Air),
                } {
                    add_left_face(
                        &mut mesh_data,
                        pos,
                        base_pos,
                        cursor.block_at_pos().texture_index(Face::Left),
                    );
                }

                if match cursor.block_right() {
                    Some(b) => b == BlockType::Air,
                    None => neighbours
                        .right
                        .as_deref()
                        .is_none_or(|c| c.get_block(pos.with_x(0)) == BlockType::Air),
                } {
                    add_right_face(
                        &mut mesh_data,
                        pos,
                        base_pos,
                        cursor.block_at_pos().texture_index(Face::Right),
                    );
                }
            }
            _ => {
                let pos = cursor.pos;

                if match cursor.block_above() {
                    Some(b) => b.is_clear(),
                    None => neighbours
                        .up
                        .as_deref()
                        .is_none_or(|c| c.get_block(pos.with_y(0)).is_clear()),
                } {
                    add_top_face(
                        &mut mesh_data,
                        pos,
                        base_pos,
                        cursor.block_at_pos().texture_index(Face::Top),
                    );
                }

                if match cursor.block_below() {
                    Some(b) => b.is_clear(),
                    None => neighbours
                        .down
                        .as_deref()
                        .is_none_or(|c| c.get_block(pos.with_y(CHUNK_IDX)).is_clear()),
                } {
                    add_bottom_face(
                        &mut mesh_data,
                        pos,
                        base_pos,
                        cursor.block_at_pos().texture_index(Face::Bottom),
                    );
                }

                if match cursor.block_front() {
                    Some(b) => b.is_clear(),
                    None => neighbours
                        .front
                        .as_deref()
                        .is_none_or(|c| c.get_block(pos.with_z(0)).is_clear()),
                } {
                    add_front_face(
                        &mut mesh_data,
                        pos,
                        base_pos,
                        cursor.block_at_pos().texture_index(Face::Front),
                    );
                }

                if match cursor.block_back() {
                    Some(b) => b.is_clear(),
                    None => neighbours
                        .back
                        .as_deref()
                        .is_none_or(|c| c.get_block(pos.with_z(CHUNK_IDX)).is_clear()),
                } {
                    add_back_face(
                        &mut mesh_data,
                        pos,
                        base_pos,
                        cursor.block_at_pos().texture_index(Face::Back),
                    );
                }

                if match cursor.block_left() {
                    Some(b) => b.is_clear(),
                    None => neighbours
                        .left
                        .as_deref()
                        .is_none_or(|c| c.get_block(pos.with_x(CHUNK_IDX)).is_clear()),
                } {
                    add_left_face(
                        &mut mesh_data,
                        pos,
                        base_pos,
                        cursor.block_at_pos().texture_index(Face::Left),
                    );
                }

                if match cursor.block_right() {
                    Some(b) => b.is_clear(),
                    None => neighbours
                        .right
                        .as_deref()
                        .is_none_or(|c| c.get_block(pos.with_x(0)).is_clear()),
                } {
                    add_right_face(
                        &mut mesh_data,
                        pos,
                        base_pos,
                        cursor.block_at_pos().texture_index(Face::Right),
                    );
                }
            }
        }

        cursor.advance();
    }

    mesh_data
}
