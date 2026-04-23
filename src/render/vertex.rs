use std::mem;

use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use wgpu::{VertexBufferLayout, VertexFormat, VertexStepMode};

#[derive(Pod, Zeroable, Debug, Copy, Clone)]
#[repr(C)]
/// This Vertex struct represents the unit of data given to each instance of
/// a vertex shader.
pub struct Vertex {
    pub position: Vec3,
    pub local_pos: Vec3,
    pub face_uv: [f32; 2],
    pub face_id: u32,
}

impl Vertex {
    pub fn new(
        position: Vec3,
        local_pos: Vec3,
        face_uv: [f32; 2],
        face_id: u32,
    ) -> Self {
        Self {
            position,
            local_pos,
            face_uv,
            face_id,
        }
    }
}

impl Vertex {
    /// Returns a `VertexBufferLayout` description for the vertex format.
    pub const fn buffer_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                // attribute descriptor for position (field 0)
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                // attribute descriptor for local_pos (field 1)
                wgpu::VertexAttribute {
                    offset: mem::size_of::<Vec3>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x3,
                },
                // attribute descriptor for face_uv (field 2)
                wgpu::VertexAttribute {
                    offset: (mem::size_of::<Vec3>() * 2) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32x2,
                },
                // attribute descriptor for face_id (field 3)
                wgpu::VertexAttribute {
                    offset: (mem::size_of::<Vec3>() * 2 + mem::size_of::<[f32; 2]>())
                        as wgpu::BufferAddress,
                    shader_location: 3,
                    format: VertexFormat::Uint32,
                },
            ],
        }
    }
}
