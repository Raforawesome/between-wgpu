use glam::{Mat4, Vec3};
use wgpu::{BufferUsages, util::DeviceExt as _};
use winit::keyboard::KeyCode;

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
        self.up.cross(self.forward()).normalize()
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
        // We use an infinite reversed-Z projection for optimal depth precision.
        let proj =
            Mat4::perspective_infinite_reverse_lh(self.fovy.to_radians(), self.aspect, self.znear);

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

pub struct CameraController {
    speed: f32,
    sensitivity_x: f32,
    sensitivity_y: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
}

impl Default for CameraController {
    fn default() -> Self {
        Self::new(0.2, (0.15, 0.1))
    }
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: (f32, f32)) -> Self {
        let (sensitivity_x, sensitivity_y) = sensitivity;

        Self {
            speed,
            sensitivity_x,
            sensitivity_y,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64, camera: &mut Camera) {
        camera.yaw += mouse_dx as f32 * self.sensitivity_x;
        camera.pitch -= mouse_dy as f32 * self.sensitivity_y;

        camera.pitch = camera.pitch.clamp(-89.0, 89.0);
    }

    pub fn process_events(&mut self, key: KeyCode, is_pressed: bool) -> bool {
        match key {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_forward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
                true
            }
            KeyCode::Space => {
                self.is_up_pressed = is_pressed;
                true
            }
            KeyCode::ShiftLeft => {
                self.is_down_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        let forward = camera.forward();
        let right = camera.right();

        if self.is_forward_pressed {
            camera.position += forward * self.speed;
        }
        if self.is_backward_pressed {
            camera.position -= forward * self.speed;
        }
        if self.is_right_pressed {
            camera.position += right * self.speed;
        }
        if self.is_left_pressed {
            camera.position -= right * self.speed;
        }
        if self.is_up_pressed {
            camera.position += camera.up * self.speed;
        }
        if self.is_down_pressed {
            camera.position -= camera.up * self.speed;
        }
    }
}
