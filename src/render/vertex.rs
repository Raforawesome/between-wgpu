use std::mem;

use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

#[derive(Pod, Zeroable, Debug, Copy, Clone)]
#[repr(C)]
/// This Vertex struct represents the unit of data given to each instance of
/// a vertex shader.
pub struct Vertex {
    pub position: Vec3,
    pub face_uv: [f32; 2],
    pub face_id: u32,
}

impl Vertex {
    pub fn new(position: Vec3, face_uv: [f32; 2], face_id: u32) -> Self {
        Self {
            position,
            face_uv,
            face_id,
        }
    }
}

impl Vertex {
    /// Returns a `VertexBufferLayout` description for the vertex format.
    pub const fn buffer_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                // attribute descriptor for position (field 0)
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                // attribute descriptor for face_uv (field 1)
                VertexAttribute {
                    offset: mem::size_of::<Vec3>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
                // attribute descriptor for face_id (field 2)
                VertexAttribute {
                    offset: (mem::size_of::<Vec3>() + mem::size_of::<[f32; 2]>()) as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Uint32,
                },
            ],
        }
    }
}
