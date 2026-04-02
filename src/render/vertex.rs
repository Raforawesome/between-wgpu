use std::mem;

use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use wgpu::{VertexBufferLayout, VertexFormat, VertexStepMode};

#[derive(Pod, Zeroable, Copy, Clone)]
#[repr(C)]
/// This Vertex struct represents the unit of data given to each instance of
/// a vertex shader.
pub struct Vertex {
    pub position: Vec3,
    pub color: Vec3,
}

impl Vertex {
    pub fn new(position: impl Into<Vec3>, color: impl Into<Vec3>) -> Self {
        let position = position.into();
        let color = color.into();

        Self { position, color }
    }

    /// Uses [std::mem::transmute] to form a [Vertex] struct with arrays of floats.
    /// This allows the method to be `const`. This method is not marked as unsafe since
    /// the use of transmute is safe, as [Vec3] is marked with `#[repr(C)]`.
    pub const fn from_arrays(position: [f32; 3], color: [f32; 3]) -> Self {
        Self {
            position: unsafe { mem::transmute::<[f32; 3], Vec3>(position) },
            color: unsafe { mem::transmute::<[f32; 3], Vec3>(color) },
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
                // attribute descriptor for color (field 1)
                wgpu::VertexAttribute {
                    offset: mem::size_of::<Vec3>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }
}
