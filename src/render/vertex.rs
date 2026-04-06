use std::mem;

use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3};
use wgpu::{VertexBufferLayout, VertexFormat, VertexStepMode};

#[derive(Pod, Zeroable, Debug, Copy, Clone)]
#[repr(C)]
/// This Vertex struct represents the unit of data given to each instance of
/// a vertex shader.
pub struct Vertex {
    pub position: Vec3,
    pub tex_coords: Vec2,
}

impl Vertex {
    pub fn new(position: impl Into<Vec3>, tex_coords: impl Into<Vec2>) -> Self {
        let position = position.into();
        let tex_coords = tex_coords.into();

        Self {
            position,
            tex_coords,
        }
    }

    /// Uses [std::mem::transmute] to form a [Vertex] struct with arrays of floats.
    /// This allows the method to be `const`. This method is not marked as unsafe since
    /// the use of transmute is safe, as [Vec3] and [Vec2] are marked with `#[repr(C)]`.
    pub const fn from_arrays(position: [f32; 3], tex_coords: [f32; 2]) -> Self {
        Self {
            position: unsafe { mem::transmute::<[f32; 3], Vec3>(position) },
            tex_coords: unsafe { mem::transmute::<[f32; 2], Vec2>(tex_coords) },
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
                // attribute descriptor for tex_coords (field 1)
                wgpu::VertexAttribute {
                    offset: mem::size_of::<Vec3>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
            ],
        }
    }
}
