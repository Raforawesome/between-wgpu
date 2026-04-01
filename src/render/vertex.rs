use std::mem;

use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use wgpu::{VertexBufferLayout, VertexFormat, VertexStepMode};

#[derive(Pod, Zeroable, Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    pub position: Vec3,
    pub colour: Vec3,
}

impl Vertex {
    pub fn new(position: impl Into<Vec3>, colour: impl Into<Vec3>) -> Self {
        let position = position.into();
        let colour = colour.into();

        Self { position, colour }
    }
}

impl Vertex {
    pub fn desc() -> VertexBufferLayout<'static> {
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
                // attribute descriptor for colour (field 1)
                wgpu::VertexAttribute {
                    offset: mem::size_of::<Vec3>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }
}
