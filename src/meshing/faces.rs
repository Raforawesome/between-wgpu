use crate::meshing::ChunkMeshData;
use crate::render::vertex::Vertex;
use glam::U8Vec3;

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Face {
    // GPU IDs used for face bias and lighting in fragment.slang.
    Top = 0,
    Bottom = 1,
    Left = 2,
    Right = 3,
    Back = 4,
    Front = 5,
}

impl Face {
    pub const fn to_norm(self) -> [f32; 3] {
        match self {
            Face::Top => [0.0, 1.0, 0.0],
            Face::Bottom => [0.0, -1.0, 0.0],
            Face::Left => [-1.0, 0.0, 0.0],
            Face::Right => [1.0, 0.0, 0.0],
            Face::Back => [0.0, 0.0, -1.0],
            Face::Front => [0.0, 0.0, 1.0],
        }
    }

    pub const fn shader_id(self) -> u32 {
        self as u32
    }
}

const WINDING_A: [u32; 6] = [3, 2, 1, 3, 1, 0];
const WINDING_B: [u32; 6] = [3, 1, 2, 3, 0, 1];

fn push_quad(
    chunk_mesh_data: &mut ChunkMeshData,
    face: Face,
    vertices: [U8Vec3; 4],
    indices: [u32; 6],
) {
    let base_id = chunk_mesh_data.vertices.len() as u32;
    let face_id = face.shader_id();

    for (uv_corner, vertex_pos) in vertices.into_iter().enumerate() {
        chunk_mesh_data
            .vertices
            .push(Vertex::new(vertex_pos, uv_corner as u8, face_id));
    }

    chunk_mesh_data
        .indices
        .extend(indices.into_iter().map(|idx| base_id + idx));
}

/*
 * Helper functions
 *
 * The functions below provide methods that add a new face in the specified direction
 * to the mesh data lists passed to them. The 4 vertices each face function generates
 * is relative from the bottom left vertex.
 */

pub fn add_top_face(chunk_mesh_data: &mut ChunkMeshData, pos: U8Vec3) {
    push_quad(
        chunk_mesh_data,
        Face::Top,
        [
            pos + U8Vec3::new(0, 1, 1),
            pos + U8Vec3::new(1, 1, 1),
            pos + U8Vec3::new(1, 1, 0),
            pos + U8Vec3::new(0, 1, 0),
        ],
        WINDING_A,
    );
}

pub fn add_bottom_face(chunk_mesh_data: &mut ChunkMeshData, pos: U8Vec3) {
    push_quad(
        chunk_mesh_data,
        Face::Bottom,
        [
            pos + U8Vec3::new(0, 0, 1),
            pos + U8Vec3::new(1, 0, 1),
            pos + U8Vec3::new(1, 0, 0),
            pos + U8Vec3::new(0, 0, 0),
        ],
        WINDING_B,
    );
}

pub fn add_front_face(chunk_mesh_data: &mut ChunkMeshData, pos: U8Vec3) {
    push_quad(
        chunk_mesh_data,
        Face::Front,
        [
            pos + U8Vec3::new(0, 0, 1),
            pos + U8Vec3::new(1, 0, 1),
            pos + U8Vec3::new(1, 1, 1),
            pos + U8Vec3::new(0, 1, 1),
        ],
        WINDING_A,
    );
}

pub fn add_back_face(chunk_mesh_data: &mut ChunkMeshData, pos: U8Vec3) {
    push_quad(
        chunk_mesh_data,
        Face::Back,
        [
            pos + U8Vec3::new(0, 0, 0),
            pos + U8Vec3::new(1, 0, 0),
            pos + U8Vec3::new(1, 1, 0),
            pos + U8Vec3::new(0, 1, 0),
        ],
        WINDING_B,
    );
}

pub fn add_left_face(chunk_mesh_data: &mut ChunkMeshData, pos: U8Vec3) {
    push_quad(
        chunk_mesh_data,
        Face::Left,
        [
            pos + U8Vec3::new(0, 0, 0),
            pos + U8Vec3::new(0, 0, 1),
            pos + U8Vec3::new(0, 1, 1),
            pos + U8Vec3::new(0, 1, 0),
        ],
        WINDING_A,
    );
}

pub fn add_right_face(chunk_mesh_data: &mut ChunkMeshData, pos: U8Vec3) {
    push_quad(
        chunk_mesh_data,
        Face::Right,
        [
            pos + U8Vec3::new(1, 0, 0),
            pos + U8Vec3::new(1, 0, 1),
            pos + U8Vec3::new(1, 1, 1),
            pos + U8Vec3::new(1, 1, 0),
        ],
        WINDING_B,
    );
}
