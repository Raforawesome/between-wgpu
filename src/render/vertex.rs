use std::mem;

use bytemuck::{Pod, Zeroable};
use glam::U8Vec3;
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

use crate::constants::CHUNK_SIZE;

const X_SHIFT: u32 = 0;
const Y_SHIFT: u32 = 5;
const Z_SHIFT: u32 = 10;
const UV_CORNER_SHIFT: u32 = 15;
const FACE_ID_SHIFT: u32 = 17;

const COORD_MASK: u32 = 0b11111;
const UV_CORNER_MASK: u32 = 0b11;
const FACE_ID_MASK: u32 = 0b111;

const _: () = assert!(CHUNK_SIZE <= COORD_MASK as usize);

#[derive(Pod, Zeroable, Debug, Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
/// Packed vertex data passed to the vertex shader.
///
/// Bit layout:
/// - bits 0..=4: chunk-local x vertex coordinate, range 0..=16
/// - bits 5..=9: chunk-local y vertex coordinate, range 0..=16
/// - bits 10..=14: chunk-local z vertex coordinate, range 0..=16
/// - bits 15..=16: UV corner id, range 0..=3
/// - bits 17..=19: face id, range 0..=5
/// - bits 20..=31: reserved for future use
pub struct Vertex {
    pub packed: u32,
}

impl Vertex {
    pub fn new(position: U8Vec3, uv_corner: u8, face_id: u32) -> Self {
        debug_assert!(position.x as usize <= CHUNK_SIZE);
        debug_assert!(position.y as usize <= CHUNK_SIZE);
        debug_assert!(position.z as usize <= CHUNK_SIZE);
        debug_assert!(u32::from(uv_corner) <= UV_CORNER_MASK);
        debug_assert!(face_id <= FACE_ID_MASK);

        Self {
            packed: (u32::from(position.x) << X_SHIFT)
                | (u32::from(position.y) << Y_SHIFT)
                | (u32::from(position.z) << Z_SHIFT)
                | (u32::from(uv_corner) << UV_CORNER_SHIFT)
                | (face_id << FACE_ID_SHIFT),
        }
    }
}

impl Vertex {
    /// Returns a `VertexBufferLayout` description for the packed vertex format.
    pub const fn buffer_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: VertexFormat::Uint32,
            }],
        }
    }
}
