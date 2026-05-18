pub mod chunk_cursor;
pub mod faces;

use crate::constants::{CHUNK_IDX, CHUNK_VOL};
use crate::meshing::chunk_cursor::ChunkCursor;
use crate::render::vertex::Vertex;
use crate::world::chunk_manager::NeighbourSnapshot;
use crate::world::{block::BlockType, chunks::Chunk};

use faces::{
    add_back_face, add_bottom_face, add_front_face, add_left_face, add_right_face, add_top_face,
};

/// Stores vertices and indices
#[derive(Default)]
pub struct ChunkMeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl<'a> From<&'a ChunkMeshData> for (&'a [Vertex], &'a [u32]) {
    fn from(val: &'a ChunkMeshData) -> Self {
        let ChunkMeshData { vertices, indices } = val;
        (vertices.as_slice(), indices.as_slice())
    }
}

pub fn mesh_chunk(chunk: &Chunk, neighbours: &NeighbourSnapshot) -> ChunkMeshData {
    if chunk.is_empty() {
        ChunkMeshData::default()
    } else {
        chunk_mesh_data(chunk, neighbours)
    }
}

fn chunk_mesh_data(chunk: &Chunk, neighbours: &NeighbourSnapshot) -> ChunkMeshData {
    let mut mesh_data = ChunkMeshData {
        vertices: Vec::with_capacity(512),
        indices: Vec::with_capacity(768),
    };

    let blocks: &[BlockType; CHUNK_VOL] = chunk.blocks();
    let mut cursor: ChunkCursor = ChunkCursor::at_origin(blocks);

    for &block in blocks {
        match block {
            BlockType::Air => (),
            // the two branches below are almost identical.
            // the only difference is for water we add faces where air is,
            // and for everything else we add faces where any clear block is.
            // I couldn't find a way to genericize this that wasn't ridiculously complex.
            BlockType::Water => {
                let pos = cursor.pos;

                if match cursor.block_above() {
                    Some(b) => b == BlockType::Air,
                    None => neighbours.top.get_block(pos.with_y(0)) == BlockType::Air,
                } {
                    add_top_face(&mut mesh_data, pos);
                }

                if match cursor.block_below() {
                    Some(b) => b == BlockType::Air,
                    None => neighbours.bottom.get_block(pos.with_y(CHUNK_IDX)) == BlockType::Air,
                } {
                    add_bottom_face(&mut mesh_data, pos);
                }

                if match cursor.block_front() {
                    Some(b) => b == BlockType::Air,
                    None => neighbours.front.get_block(pos.with_z(0)) == BlockType::Air,
                } {
                    add_front_face(&mut mesh_data, pos);
                }

                if match cursor.block_back() {
                    Some(b) => b == BlockType::Air,
                    None => neighbours.back.get_block(pos.with_z(CHUNK_IDX)) == BlockType::Air,
                } {
                    add_back_face(&mut mesh_data, pos);
                }

                if match cursor.block_left() {
                    Some(b) => b == BlockType::Air,
                    None => neighbours.left.get_block(pos.with_x(CHUNK_IDX)) == BlockType::Air,
                } {
                    add_left_face(&mut mesh_data, pos);
                }

                if match cursor.block_right() {
                    Some(b) => b == BlockType::Air,
                    None => neighbours.right.get_block(pos.with_x(0)) == BlockType::Air,
                } {
                    add_right_face(&mut mesh_data, pos);
                }
            }
            _ => {
                let pos = cursor.pos;

                if match cursor.block_above() {
                    Some(b) => b.is_clear(),
                    None => neighbours.top.get_block(pos.with_y(0)).is_clear(),
                } {
                    add_top_face(&mut mesh_data, pos);
                }

                if match cursor.block_below() {
                    Some(b) => b.is_clear(),
                    None => neighbours
                        .bottom
                        .get_block(pos.with_y(CHUNK_IDX))
                        .is_clear(),
                } {
                    add_bottom_face(&mut mesh_data, pos);
                }

                if match cursor.block_front() {
                    Some(b) => b.is_clear(),
                    None => neighbours.front.get_block(pos.with_z(0)).is_clear(),
                } {
                    add_front_face(&mut mesh_data, pos);
                }

                if match cursor.block_back() {
                    Some(b) => b.is_clear(),
                    None => neighbours.back.get_block(pos.with_z(CHUNK_IDX)).is_clear(),
                } {
                    add_back_face(&mut mesh_data, pos);
                }

                if match cursor.block_left() {
                    Some(b) => b.is_clear(),
                    None => neighbours.left.get_block(pos.with_x(CHUNK_IDX)).is_clear(),
                } {
                    add_left_face(&mut mesh_data, pos);
                }

                if match cursor.block_right() {
                    Some(b) => b.is_clear(),
                    None => neighbours.right.get_block(pos.with_x(0)).is_clear(),
                } {
                    add_right_face(&mut mesh_data, pos);
                }
            }
        }

        cursor.advance();
    }

    mesh_data
}
