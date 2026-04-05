use glam::{Mat4, Vec3};
use wgpu::{BufferUsages, util::DeviceExt as _};

use crate::render::gpu::Gpu;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub up: Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
}

impl Camera {
    pub fn forward(&self) -> Vec3 {
        // the sine of the pitch will oscillate between 1 at 90 deg (upwards) and 0 at
        // 0 degrees (forwards), making it perfect for the y coordinate of the forward vector.
        //
        // The x and z coordinates are determined on the 2d circle of their combined rotation plane
        // by cos(a) and sin(a), where a (angle) is the yaw in this case.
        // x will be sin(yaw) as the x coordinate of a circle is equal to sin(a).
        // z will be cos(yaw) for a similar reason, as +z points forwards in our LH system.
        //
        // we will need to multiply x and z by cos(pitch), which will make them 0 when sin(pitch)
        // is 1, and scale them accordingly otherwise. this gives us a unit vector.
        let (pitch_sin, pitch_cos) = self.pitch.to_radians().sin_cos();
        let (yaw_sin, yaw_cos) = self.yaw.to_radians().sin_cos();

        (yaw_sin * pitch_cos, pitch_sin, yaw_cos * pitch_cos).into()
    }

    pub fn right(&self) -> Vec3 {
        todo!()
    }

    pub fn new(pos: impl Into<Vec3>, target: impl Into<Vec3>, aspect: f32) -> Self {
        let position = pos.into();
        let target = target.into();
        let up = Vec3::Y;

        let dir = (target - position).normalize();
        let pitch = dir.y.asin().to_degrees();
        let yaw = dir.x.atan2(dir.z).to_degrees();

        Self {
            // position the camera 1 unit up and 2 units back
            position,
            yaw,
            pitch,
            // which direction is up
            up,
            // set aspect ratio to surface config
            aspect,
            // fov along y-axis
            fovy: 45.0,
            // near plane of frustum
            znear: 0.1,
        }
    }

    pub fn build_view_projection_matrix(&self) -> Mat4 {
        // multiply forward vector by some constant to avoid precision-related camera snapping
        let target = self.position + self.forward() * 5.0;

        // The view matrix moves the world to be at the position and rotation of the camera.
        // It's essentially an inverse of whatever the transform matrix of the camera would be.
        let view = Mat4::look_at_lh(self.position, target, self.up);

        // The projection matrix warps the scene to give the effect of depth. Without this,
        // objects up close would be the same size as objects far away.
        let proj = Mat4::perspective_rh(self.fovy.to_radians(), self.aspect, self.znear, self.zfar);

        // we return proj * view because the model -> view -> projection transformations apply
        // right to left. we dont need the model transformation since our vertices are already
        // in 3D space, so we combine the other two to obtain our final transformation matrix.
        proj * view
    }

    pub fn resize_aspect(&mut self, config: &wgpu::SurfaceConfiguration) {
        self.aspect = config.width as f32 / config.height as f32;
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
