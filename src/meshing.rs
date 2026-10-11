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
        // water only shows faces against air, everything else against any clear block.
        // each arm monomorphizes its own copy of add_visible_faces, so the test is inlined.
        match block {
            BlockType::Air => (),
            BlockType::Water => {
                add_visible_faces(&mut mesh_data, &cursor, neighbours, |b| b == BlockType::Air)
            }
            _ => add_visible_faces(&mut mesh_data, &cursor, neighbours, |b| b.is_clear()),
        }

        cursor.advance();
    }

    mesh_data
}

/// Adds each face of the block under the cursor whose neighbouring block passes `shows_face`.
/// Neighbours across the chunk border are read from the neighbour snapshot.
#[inline(always)]
fn add_visible_faces(
    mesh_data: &mut ChunkMeshData,
    cursor: &ChunkCursor,
    neighbours: &NeighbourSnapshot,
    shows_face: impl Fn(BlockType) -> bool,
) {
    let pos = cursor.pos;

    let above = cursor
        .block_above()
        .unwrap_or_else(|| neighbours.top.get_block(pos.with_y(0)));
    if shows_face(above) {
        add_top_face(mesh_data, pos);
    }

    let below = cursor
        .block_below()
        .unwrap_or_else(|| neighbours.bottom.get_block(pos.with_y(CHUNK_IDX)));
    if shows_face(below) {
        add_bottom_face(mesh_data, pos);
    }

    let front = cursor
        .block_front()
        .unwrap_or_else(|| neighbours.front.get_block(pos.with_z(0)));
    if shows_face(front) {
        add_front_face(mesh_data, pos);
    }

    let back = cursor
        .block_back()
        .unwrap_or_else(|| neighbours.back.get_block(pos.with_z(CHUNK_IDX)));
    if shows_face(back) {
        add_back_face(mesh_data, pos);
    }

    let left = cursor
        .block_left()
        .unwrap_or_else(|| neighbours.left.get_block(pos.with_x(CHUNK_IDX)));
    if shows_face(left) {
        add_left_face(mesh_data, pos);
    }

    let right = cursor
        .block_right()
        .unwrap_or_else(|| neighbours.right.get_block(pos.with_x(0)));
    if shows_face(right) {
        add_right_face(mesh_data, pos);
    }
}
