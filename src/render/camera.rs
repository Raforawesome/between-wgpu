use glam::{Mat4, Vec3};
use wgpu::{BufferUsages, util::DeviceExt as _};

use crate::render::gpu::Gpu;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn new(pos: impl Into<Vec3>, target: impl Into<Vec3>, aspect: f32) -> Self {
        Self {
            // position the camera 1 unit up and 2 units back
            eye: pos.into(),
            // be looking at the origin
            target: target.into(),
            // which direction is up
            up: Vec3::Y,
            // set aspect ratio to surface config
            aspect,
            // fov along y-axis
            fovy: 45.0,
            // near plane of frustum
            znear: 0.1,
            // far plane of frustum
            zfar: 100.0,
        }
    }

    pub fn build_view_projection_matrix(&self) -> Mat4 {
        // The view matrix moves the world to be at the position and rotation of the camera.
        // It's essentially an inverse of whatever the transform matrix of the camera would be.
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);

        // The proj matrix warps the scene to give the effect of depth. Without this, objects
        // up close would be the same size as objects far away.
        let proj = Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);

        // we return proj * view because the model -> view -> projection transformations apply
        // right to left. we dont need the model transformation since our vertices are already
        // in 3D space, so we combine the other two to obtain our final transformation matrix.
        proj * view
    }

    /*
     * These methods help reduce the wgpu boilerplate around setting up the bind group
     * for the game camera.
     */
    pub const fn layout_desc() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        }
    }
}

impl Gpu {
    pub fn create_camera_buffer(&self, camera: &Camera) -> wgpu::Buffer {
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("between camera buffer"),
                contents: bytemuck::bytes_of(&camera.build_view_projection_matrix()),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
    }

    pub fn create_camera_bind_group(
        &self,
        camera_buffer: &wgpu::Buffer,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        })
    }
}
