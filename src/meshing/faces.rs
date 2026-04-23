#![allow(clippy::too_many_arguments)]
use std::ops::Add;

use crate::meshing::ChunkMeshData;
use crate::render::vertex::Vertex;
use glam::{IVec3, U8Vec3, Vec3};

#[derive(Copy, Clone)]
pub enum Face {
    Top,
    Bottom,
    Left,
    Right,
    Back,
    Front,
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
}

/*
 * Helper functions
 *
 * The functions below provide methods that add a new face in the specified direction
 * to the mesh data lists passed to them
 */

pub fn add_top_face(chunk_mesh_data: &mut ChunkMeshData, pos: U8Vec3, chunk_pos: IVec3) {
    let wp = world_pos(chunk_pos, pos);
    let lp = pos.as_vec3();

    let v1_id = chunk_mesh_data.vertices.len() as u32;
    let v2_id = v1_id + 1;
    let v3_id = v1_id + 2;
    let v4_id = v1_id + 3;

    let f_id = Face::Top.shader_id();

    chunk_mesh_data.vertices.push(Vertex::new(
        wp + Vec3::new(0.0, 1.0, 1.0),
        lp + Vec3::new(0.0, 1.0, 1.0),
        [0.0, 1.0],
        f_id,
    ));
    chunk_mesh_data.vertices.push(Vertex::new(
        wp + Vec3::new(1.0, 1.0, 1.0),
        lp + Vec3::new(1.0, 1.0, 1.0),
        [1.0, 1.0],
        f_id,
    ));
    chunk_mesh_data.vertices.push(Vertex::new(
        wp + Vec3::new(1.0, 1.0, 0.0),
        lp + Vec3::new(1.0, 1.0, 0.0),
        [1.0, 0.0],
        f_id,
    ));
    chunk_mesh_data.vertices.push(Vertex::new(
        wp + Vec3::new(0.0, 1.0, 0.0),
        lp + Vec3::new(0.0, 1.0, 0.0),
        [0.0, 0.0],
        f_id,
    ));

    chunk_mesh_data.indices.push(v4_id);
    chunk_mesh_data.indices.push(v3_id);
    chunk_mesh_data.indices.push(v2_id);
    chunk_mesh_data.indices.push(v4_id);
    chunk_mesh_data.indices.push(v2_id);
    chunk_mesh_data.indices.push(v1_id);
}

pub fn add_bottom_face(chunk_mesh_data: &mut ChunkMeshData, pos: U8Vec3, chunk_pos: IVec3) {
    let wp = world_pos(chunk_pos, pos);
    let lp = pos.as_vec3();

    let v1_id = chunk_mesh_data.vertices.len() as u32;
    let v2_id = v1_id + 1;
    let v3_id = v1_id + 2;
    let v4_id = v1_id + 3;

    let f_id = Face::Bottom.shader_id();

    chunk_mesh_data.vertices.push(Vertex::new(
        wp + Vec3::new(0.0, 0.0, 1.0),
        lp + Vec3::new(0.0, 0.0, 1.0),
        [0.0, 1.0],
        f_id,
    ));
    chunk_mesh_data.vertices.push(Vertex::new(
        wp + Vec3::new(1.0, 0.0, 1.0),
        lp + Vec3::new(1.0, 0.0, 1.0),
        [1.0, 1.0],
        f_id,
    ));
    chunk_mesh_data.vertices.push(Vertex::new(
        wp + Vec3::new(1.0, 0.0, 0.0),
        lp + Vec3::new(1.0, 0.0, 0.0),
        [1.0, 0.0],
        f_id,
    ));
    chunk_mesh_data.vertices.push(Vertex::new(
        wp + Vec3::new(0.0, 0.0, 0.0),
        lp + Vec3::new(0.0, 0.0, 0.0),
        [0.0, 0.0],
        f_id,
    ));

    chunk_mesh_data.indices.push(v4_id);
    chunk_mesh_data.indices.push(v2_id);
    chunk_mesh_data.indices.push(v3_id);
    chunk_mesh_data.indices.push(v4_id);
    chunk_mesh_data.indices.push(v1_id);
    chunk_mesh_data.indices.push(v2_id);
}

pub fn add_front_face(chunk_mesh_data: &mut ChunkMeshData, pos: U8Vec3, chunk_pos: IVec3) {
    let wp = world_pos(chunk_pos, pos);
    let lp = pos.as_vec3();

    let v1_id = chunk_mesh_data.vertices.len() as u32;
    let v2_id = v1_id + 1;
    let v3_id = v1_id + 2;
    let v4_id = v1_id + 3;

    let f_id = Face::Front.shader_id();

    chunk_mesh_data.vertices.push(Vertex::new(
        wp + Vec3::new(0.0, 0.0, 1.0),
        lp + Vec3::new(0.0, 0.0, 1.0),
        [0.0, 1.0],
        f_id,
    ));
    chunk_mesh_data.vertices.push(Vertex::new(
        wp + Vec3::new(1.0, 0.0, 1.0),
        lp + Vec3::new(1.0, 0.0, 1.0),
        [1.0, 1.0],
        f_id,
    ));
    chunk_mesh_data.vertices.push(Vertex::new(
        wp + Vec3::new(1.0, 1.0, 1.0),
        lp + Vec3::new(1.0, 1.0, 1.0),
        [1.0, 0.0],
        f_id,
    ));
    chunk_mesh_data.vertices.push(Vertex::new(
        wp + Vec3::new(0.0, 1.0, 1.0),
        lp + Vec3::new(0.0, 1.0, 1.0),
        [0.0, 0.0],
        f_id,
    ));

    chunk_mesh_data.indices.push(v4_id);
    chunk_mesh_data.indices.push(v3_id);
    chunk_mesh_data.indices.push(v2_id);
    chunk_mesh_data.indices.push(v4_id);
    chunk_mesh_data.indices.push(v2_id);
    chunk_mesh_data.indices.push(v1_id);
}

pub fn add_back_face(chunk_mesh_data: &mut ChunkMeshData, pos: U8Vec3, chunk_pos: IVec3) {
    let wp = world_pos(chunk_pos, pos);
    let lp = pos.as_vec3();

    let v1_id = chunk_mesh_data.vertices.len() as u32;
    let v2_id = v1_id + 1;
    let v3_id = v1_id + 2;
    let v4_id = v1_id + 3;

    let f_id = Face::Back.shader_id();

    chunk_mesh_data
        .vertices
        .push(Vertex::new(wp, lp, [0.0, 1.0], f_id));
    chunk_mesh_data.vertices.push(Vertex::new(
        wp + Vec3::new(1.0, 0.0, 0.0),
        lp + Vec3::new(1.0, 0.0, 0.0),
        [1.0, 1.0],
        f_id,
    ));
    chunk_mesh_data.vertices.push(Vertex::new(
        wp + Vec3::new(1.0, 1.0, 0.0),
        lp + Vec3::new(1.0, 1.0, 0.0),
        [1.0, 0.0],
        f_id,
    ));
    chunk_mesh_data.vertices.push(Vertex::new(
        wp + Vec3::new(0.0, 1.0, 0.0),
        lp + Vec3::new(0.0, 1.0, 0.0),
        [0.0, 0.0],
        f_id,
    ));

    chunk_mesh_data.indices.push(v4_id);
    chunk_mesh_data.indices.push(v2_id);
    chunk_mesh_data.indices.push(v3_id);
    chunk_mesh_data.indices.push(v4_id);
    chunk_mesh_data.indices.push(v1_id);
    chunk_mesh_data.indices.push(v2_id);
}

pub fn add_left_face(chunk_mesh_data: &mut ChunkMeshData, pos: U8Vec3, chunk_pos: IVec3) {
    let wp = world_pos(chunk_pos, pos);
    let lp = pos.as_vec3();

    let v1_id = chunk_mesh_data.vertices.len() as u32;
    let v2_id = v1_id + 1;
    let v3_id = v1_id + 2;
    let v4_id = v1_id + 3;

    let f_id = Face::Left.shader_id();

    chunk_mesh_data
        .vertices
        .push(Vertex::new(wp, lp, [0.0, 1.0], f_id));
    chunk_mesh_data.vertices.push(Vertex::new(
        wp + Vec3::new(0.0, 0.0, 1.0),
        lp + Vec3::new(0.0, 0.0, 1.0),
        [1.0, 1.0],
        f_id,
    ));
    chunk_mesh_data.vertices.push(Vertex::new(
        wp + Vec3::new(0.0, 1.0, 1.0),
        lp + Vec3::new(0.0, 1.0, 1.0),
        [1.0, 0.0],
        f_id,
    ));
    chunk_mesh_data.vertices.push(Vertex::new(
        wp + Vec3::new(0.0, 1.0, 0.0),
        lp + Vec3::new(0.0, 1.0, 0.0),
        [0.0, 0.0],
        f_id,
    ));

    chunk_mesh_data.indices.push(v4_id);
    chunk_mesh_data.indices.push(v3_id);
    chunk_mesh_data.indices.push(v2_id);
    chunk_mesh_data.indices.push(v4_id);
    chunk_mesh_data.indices.push(v2_id);
    chunk_mesh_data.indices.push(v1_id);
}

pub fn add_right_face(chunk_mesh_data: &mut ChunkMeshData, pos: U8Vec3, chunk_pos: IVec3) {
    let wp = world_pos(chunk_pos, pos);
    let lp = pos.as_vec3();

    let v1_id = chunk_mesh_data.vertices.len() as u32;
    let v2_id = v1_id + 1;
    let v3_id = v1_id + 2;
    let v4_id = v1_id + 3;

    let f_id = Face::Right.shader_id();

    chunk_mesh_data.vertices.push(Vertex::new(
        wp + Vec3::new(1.0, 0.0, 0.0),
        lp + Vec3::new(1.0, 0.0, 0.0),
        [0.0, 1.0],
        f_id,
    ));
    chunk_mesh_data.vertices.push(Vertex::new(
        wp + Vec3::new(1.0, 0.0, 1.0),
        lp + Vec3::new(1.0, 0.0, 1.0),
        [1.0, 1.0],
        f_id,
    ));
    chunk_mesh_data.vertices.push(Vertex::new(
        wp + Vec3::new(1.0, 1.0, 1.0),
        lp + Vec3::new(1.0, 1.0, 1.0),
        [1.0, 0.0],
        f_id,
    ));
    chunk_mesh_data.vertices.push(Vertex::new(
        wp + Vec3::new(1.0, 1.0, 0.0),
        lp + Vec3::new(1.0, 1.0, 0.0),
        [0.0, 0.0],
        f_id,
    ));

    chunk_mesh_data.indices.push(v4_id);
    chunk_mesh_data.indices.push(v2_id);
    chunk_mesh_data.indices.push(v3_id);
    chunk_mesh_data.indices.push(v4_id);
    chunk_mesh_data.indices.push(v1_id);
    chunk_mesh_data.indices.push(v2_id);
}
