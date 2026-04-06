#![allow(clippy::too_many_arguments)]
use crate::constants::{ATLAS_SIZE, TILE_STEP};
use crate::meshing::ChunkMeshData;
use glam::U8Vec3;

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
pub fn add_top_face(chunk_mesh_data: &mut ChunkMeshData, pos: U8Vec3, texture_index: usize) {
    let ChunkMeshData(vertices, normals, uvs, indices) = chunk_mesh_data;
    let (x, y, z) = (pos.x as f32, pos.y as f32, pos.z as f32);

    // vert_count = vertex 1
    // vert_count + 1 = vertex 2
    // ...
    let v1_id = vertices.len() as u32;
    let v2_id = v1_id + 1;
    let v3_id = v1_id + 2;
    let v4_id = v1_id + 3;

    // push 4 vertices to form the below triangles
    vertices.push([x, y + 1.0, z + 1.0]);
    vertices.push([x + 1.0, y + 1.0, z + 1.0]);
    vertices.push([x + 1.0, y + 1.0, z]);
    vertices.push([x, y + 1.0, z]);

    // form two triangles:
    // - 4, 3, 2
    // - 4, 2, 1
    indices.push(v4_id);
    indices.push(v3_id);
    indices.push(v2_id);
    indices.push(v4_id);
    indices.push(v2_id);
    indices.push(v1_id);

    // one normal per vertex
    for _ in 0..4 {
        normals.push(Face::Top.to_norm());
    }

    // handle UVs for textures
    let u_min = (texture_index % ATLAS_SIZE) as f32 * TILE_STEP; // steps to skip right
    let v_min = (texture_index / ATLAS_SIZE) as f32 * TILE_STEP; // steps to skip down
    let u_max = u_min + TILE_STEP;
    let v_max = v_min + TILE_STEP;

    uvs.push([u_min, v_max]);
    uvs.push([u_max, v_max]);
    uvs.push([u_max, v_min]);
    uvs.push([u_min, v_min]);
}

pub fn add_bottom_face(chunk_mesh_data: &mut ChunkMeshData, pos: U8Vec3, texture_index: usize) {
    let ChunkMeshData(vertices, normals, uvs, indices) = chunk_mesh_data;
    let (x, y, z) = (pos.x as f32, pos.y as f32, pos.z as f32);

    // vert_count = vertex 1
    // vert_count + 1 = vertex 2
    // ...
    let v1_id = vertices.len() as u32;
    let v2_id = v1_id + 1;
    let v3_id = v1_id + 2;
    let v4_id = v1_id + 3;

    // push 4 vertices to form the below triangles
    vertices.push([x, y, z + 1.0]);
    vertices.push([x + 1.0, y, z + 1.0]);
    vertices.push([x + 1.0, y, z]);
    vertices.push([x, y, z]);

    // form two triangles:
    // - 4, 2, 3
    // - 4, 1, 2
    indices.push(v4_id);
    indices.push(v2_id);
    indices.push(v3_id);
    indices.push(v4_id);
    indices.push(v1_id);
    indices.push(v2_id);

    // one normal per vertex
    for _ in 0..4 {
        normals.push(Face::Bottom.to_norm());
    }

    // handle UVs for textures
    let u_min = (texture_index % ATLAS_SIZE) as f32 * TILE_STEP; // steps to skip right
    let v_min = (texture_index / ATLAS_SIZE) as f32 * TILE_STEP; // steps to skip down
    let u_max = u_min + TILE_STEP;
    let v_max = v_min + TILE_STEP;

    uvs.push([u_min, v_max]);
    uvs.push([u_max, v_max]);
    uvs.push([u_max, v_min]);
    uvs.push([u_min, v_min]);
}

pub fn add_front_face(chunk_mesh_data: &mut ChunkMeshData, pos: U8Vec3, texture_index: usize) {
    let ChunkMeshData(vertices, normals, uvs, indices) = chunk_mesh_data;
    let (x, y, z) = (pos.x as f32, pos.y as f32, pos.z as f32);

    // vert_count = vertex 1
    // vert_count + 1 = vertex 2
    // ...
    let v1_id = vertices.len() as u32;
    let v2_id = v1_id + 1;
    let v3_id = v1_id + 2;
    let v4_id = v1_id + 3;

    // push 4 vertices to form the below triangles
    vertices.push([x, y, z + 1.0]);
    vertices.push([x + 1.0, y, z + 1.0]);
    vertices.push([x + 1.0, y + 1.0, z + 1.0]);
    vertices.push([x, y + 1.0, z + 1.0]);

    // form two triangles:
    // - 4, 3, 2
    // - 4, 2, 1
    indices.push(v4_id);
    indices.push(v3_id);
    indices.push(v2_id);
    indices.push(v4_id);
    indices.push(v2_id);
    indices.push(v1_id);

    // one normal per vertex
    for _ in 0..4 {
        normals.push(Face::Front.to_norm());
    }

    // handle UVs for textures
    let u_min = (texture_index % ATLAS_SIZE) as f32 * TILE_STEP; // steps to skip right
    let v_min = (texture_index / ATLAS_SIZE) as f32 * TILE_STEP; // steps to skip down
    let u_max = u_min + TILE_STEP;
    let v_max = v_min + TILE_STEP;

    uvs.push([u_min, v_max]);
    uvs.push([u_max, v_max]);
    uvs.push([u_max, v_min]);
    uvs.push([u_min, v_min]);
}

pub fn add_back_face(chunk_mesh_data: &mut ChunkMeshData, pos: U8Vec3, texture_index: usize) {
    let ChunkMeshData(vertices, normals, uvs, indices) = chunk_mesh_data;
    let (x, y, z) = (pos.x as f32, pos.y as f32, pos.z as f32);

    // vert_count = vertex 1
    // vert_count + 1 = vertex 2
    // ...
    let v1_id = vertices.len() as u32;
    let v2_id = v1_id + 1;
    let v3_id = v1_id + 2;
    let v4_id = v1_id + 3;

    // push 4 vertices to form the below triangles
    vertices.push([x, y, z]);
    vertices.push([x + 1.0, y, z]);
    vertices.push([x + 1.0, y + 1.0, z]);
    vertices.push([x, y + 1.0, z]);

    // form two triangles:
    // - 4, 2, 3
    // - 4, 1, 2
    indices.push(v4_id);
    indices.push(v2_id);
    indices.push(v3_id);
    indices.push(v4_id);
    indices.push(v1_id);
    indices.push(v2_id);

    // one normal per vertex
    for _ in 0..4 {
        normals.push(Face::Back.to_norm());
    }

    // handle UVs for textures
    let u_min = (texture_index % ATLAS_SIZE) as f32 * TILE_STEP; // steps to skip right
    let v_min = (texture_index / ATLAS_SIZE) as f32 * TILE_STEP; // steps to skip down
    let u_max = u_min + TILE_STEP;
    let v_max = v_min + TILE_STEP;

    uvs.push([u_min, v_max]);
    uvs.push([u_max, v_max]);
    uvs.push([u_max, v_min]);
    uvs.push([u_min, v_min]);
}

pub fn add_left_face(chunk_mesh_data: &mut ChunkMeshData, pos: U8Vec3, texture_index: usize) {
    let ChunkMeshData(vertices, normals, uvs, indices) = chunk_mesh_data;
    let (x, y, z) = (pos.x as f32, pos.y as f32, pos.z as f32);

    // vert_count = vertex 1
    // vert_count + 1 = vertex 2
    // ...
    let v1_id = vertices.len() as u32;
    let v2_id = v1_id + 1;
    let v3_id = v1_id + 2;
    let v4_id = v1_id + 3;

    // push 4 vertices to form the below triangles
    vertices.push([x, y, z]);
    vertices.push([x, y, z + 1.0]);
    vertices.push([x, y + 1.0, z + 1.0]);
    vertices.push([x, y + 1.0, z]);

    // form two triangles:
    // - 4, 3, 2
    // - 4, 2, 1
    indices.push(v4_id);
    indices.push(v3_id);
    indices.push(v2_id);
    indices.push(v4_id);
    indices.push(v2_id);
    indices.push(v1_id);

    // one normal per vertex
    for _ in 0..4 {
        normals.push(Face::Left.to_norm());
    }

    // handle UVs for textures
    let u_min = (texture_index % ATLAS_SIZE) as f32 * TILE_STEP; // steps to skip right
    let v_min = (texture_index / ATLAS_SIZE) as f32 * TILE_STEP; // steps to skip down
    let u_max = u_min + TILE_STEP;
    let v_max = v_min + TILE_STEP;

    uvs.push([u_min, v_max]);
    uvs.push([u_max, v_max]);
    uvs.push([u_max, v_min]);
    uvs.push([u_min, v_min]);
}

pub fn add_right_face(chunk_mesh_data: &mut ChunkMeshData, pos: U8Vec3, texture_index: usize) {
    let ChunkMeshData(vertices, normals, uvs, indices) = chunk_mesh_data;
    let (x, y, z) = (pos.x as f32, pos.y as f32, pos.z as f32);

    // vert_count = vertex 1
    // vert_count + 1 = vertex 2
    // ...
    let v1_id = vertices.len() as u32;
    let v2_id = v1_id + 1;
    let v3_id = v1_id + 2;
    let v4_id = v1_id + 3;

    // push 4 vertices to form the below triangles
    vertices.push([x + 1.0, y, z]);
    vertices.push([x + 1.0, y, z + 1.0]);
    vertices.push([x + 1.0, y + 1.0, z + 1.0]);
    vertices.push([x + 1.0, y + 1.0, z]);

    // form two triangles:
    // - 4, 2, 3
    // - 4, 1, 2
    indices.push(v4_id);
    indices.push(v2_id);
    indices.push(v3_id);
    indices.push(v4_id);
    indices.push(v1_id);
    indices.push(v2_id);

    // one normal per vertex
    for _ in 0..4 {
        normals.push(Face::Right.to_norm());
    }

    // handle UVs for textures
    let u_min = (texture_index % ATLAS_SIZE) as f32 * TILE_STEP; // steps to skip right
    let v_min = (texture_index / ATLAS_SIZE) as f32 * TILE_STEP; // steps to skip down
    let u_max = u_min + TILE_STEP;
    let v_max = v_min + TILE_STEP;

    uvs.push([u_min, v_max]);
    uvs.push([u_max, v_max]);
    uvs.push([u_max, v_min]);
    uvs.push([u_min, v_min]);
}
