pub mod chunk_cursor;
pub mod faces;

use glam::{IVec3, U8Vec3, Vec3};

use crate::constants::{CHUNK_IDX, CHUNK_VOL};
use crate::meshing::{chunk_cursor::ChunkCursor, faces::Face};
use crate::render::vertex::Vertex;
use crate::world::{
    block::BlockType,
    chunks::{Chunk, ChunkManager, ChunkRef},
};

use faces::{
    add_back_face, add_bottom_face, add_front_face, add_left_face, add_right_face, add_top_face,
};

/// Stores (in order): vertices, normals, uvs, indices
pub struct ChunkMeshData(Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>);

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
    pub fn mesh_chunk(&self, chunk: &Chunk) -> (Vec<Vertex>, Vec<u32>) {
        let chunk_neighbours = ChunkNeighbours {
            up: self.chunk_map().get(&(chunk.coords() + IVec3::Y)),
            down: self.chunk_map().get(&(chunk.coords() - IVec3::Y)),
            front: self.chunk_map().get(&(chunk.coords() + IVec3::Z)),
            back: self.chunk_map().get(&(chunk.coords() - IVec3::Z)),
            left: self.chunk_map().get(&(chunk.coords() - IVec3::X)),
            right: self.chunk_map().get(&(chunk.coords() + IVec3::X)),
        };

        let ChunkMeshData(positions, _normals, uvs, indices) =
            chunk_mesh_data(chunk, &chunk_neighbours);

        let mut vertices = Vec::with_capacity(positions.len());
        for (pos, uv) in positions.into_iter().zip(uvs) {
            // convert vertices to world space
            let [vx, vy, vz] = pos;
            let vert_pos = U8Vec3::new(vx as u8, vy as u8, vz as u8);
            let IVec3 { x, y, z } = chunk.chunk_to_world_space(vert_pos);

            vertices.push(Vertex::from_arrays([x as f32, y as f32, z as f32], uv));
        }

        (vertices, indices)
    }
}

fn chunk_mesh_data(chunk: &Chunk, neighbours: &ChunkNeighbours) -> ChunkMeshData {
    let vertices: Vec<[f32; 3]> = Vec::new();
    let normals: Vec<[f32; 3]> = Vec::new();
    let uvs: Vec<[f32; 2]> = Vec::new();
    let indices: Vec<u32> = Vec::new();
    let mut mesh_data = ChunkMeshData(vertices, normals, uvs, indices);

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
                        cursor.block_at_pos().texture_index(Face::Right),
                    );
                }
            }
        }

        cursor.advance();
    }

    mesh_data
}
