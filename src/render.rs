pub mod camera;
pub mod gpu;
pub mod texture;
pub mod vertex;

use gpu::Gpu;
use texture::Texture;
use vertex::Vertex;

use wgpu::{BufferUsages, PipelineCompilationOptions, util::DeviceExt as _};
use winit::{event_loop::ActiveEventLoop, keyboard::KeyCode, window::Window};

use std::sync::Arc;

use crate::render::camera::Camera;

const VERTICES: [Vertex; 4] = [
    Vertex::from_arrays([-0.5, 0.5, 0.0], [0.0, 0.0]), // Top Left
    Vertex::from_arrays([-0.5, -0.5, 0.0], [0.0, 1.0]), // Bottom Left
    Vertex::from_arrays([0.5, -0.5, 0.0], [1.0, 1.0]), // Bottom Right
    Vertex::from_arrays([0.5, 0.5, 0.0], [1.0, 0.0]),  // Top Right
];

const INDICES: [u16; 6] = [
    0, 1, 2, // Triangle 1
    0, 2, 3, // Triangle 2
];

/// Handles setting up a renderer with [wgpu]. Contains an abstraction for the GPU device.
pub struct RenderState {
    is_surface_configured: bool,
    /// Abstraction for the physical GPU device which handles interfacing and surfaces.
    gpu: Gpu,
    /// The render pipeline contains the buffers, shaders, and post-processing stages.
    render_pipeline: wgpu::RenderPipeline,
    /// Stores a list of vertices to pass to the GPU.
    vertex_buffer: wgpu::Buffer,
    /// Stores a list of vertex indices to draw. Each pair of 3 represents one triangle.
    index_buffer: wgpu::Buffer,
    /// Bind group for the diffuse texture array and sampler.
    diffuse_bind_group: wgpu::BindGroup,
    /// Camera for the scene.
    camera: Camera,
    /// Buffer for the camera matrix.
    cam_buffer: wgpu::Buffer,
    /// Bind group for the camera matrix.
    cam_bind_group: wgpu::BindGroup,
}

impl RenderState {
    /// Creates the render state using the gpu device.
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let gpu = Gpu::new(window).await?;

        // set up texture array & texture bind group
        let texture_array = Texture::create_texture_array(&gpu);
        let tex_group_layout = gpu.device.create_bind_group_layout(&Texture::layout_desc());
        let diffuse_bind_group = gpu.create_texture_bind_group(&texture_array, &tex_group_layout);

        // set up camera & camera bind group (creates camera buffer)
        let camera = Camera::new(
            (0., 1., 2.),
            (0., 0., 0.),
            gpu.config.width as f32 / gpu.config.height as f32,
        );
        let cam_buffer = gpu.create_camera_buffer(&camera);
        let cam_group_layout = gpu.device.create_bind_group_layout(&Camera::layout_desc());
        let cam_bind_group = gpu.create_camera_bind_group(&cam_buffer, &cam_group_layout);

        // set up the render pipeline
        let shader = gpu
            .device
            .create_shader_module(wgpu::include_wgsl!("shaders/shader.wgsl"));
        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("between render pipeline layout"),
                bind_group_layouts: &[Some(&tex_group_layout), Some(&cam_group_layout)],
                immediate_size: 0,
            });

        let render_pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("between render pipeline"),
                layout: Some(&pipeline_layout),
                // set up vertex shader entry point
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Vertex::buffer_layout()],
                    compilation_options: PipelineCompilationOptions::default(),
                },
                // set up fragment shader entry point
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: gpu.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: PipelineCompilationOptions::default(),
                }),
                // define how vertex data is interpreted
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                // z-buffer
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                cache: None,          // not useful for this project
                multiview_mask: None, // not useful for this project
            });

        let vertex_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("between vertex buffer"),
                contents: bytemuck::cast_slice(&VERTICES),
                usage: BufferUsages::VERTEX,
            });

        let index_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("between index buffer"),
                contents: bytemuck::cast_slice(&INDICES),
                usage: BufferUsages::INDEX,
            });

        Ok(Self {
            is_surface_configured: false,
            camera,
            gpu,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            diffuse_bind_group,
            cam_buffer,
            cam_bind_group,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.gpu.resize(width, height);
        self.is_surface_configured = true;
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, key: KeyCode, is_pressed: bool) {
        #[allow(clippy::single_match)]
        match (key, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.gpu.window.request_redraw();
        if !self.is_surface_configured {
            return Ok(());
        }

        let output = match self.gpu.get_surface_texture()? {
            Some(output) => output,
            None => return Ok(()),
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            self.gpu
                .device
                .create_command_encoder(&wgpu::wgt::CommandEncoderDescriptor {
                    label: Some("Between Render Encoder"),
                });

        // begin render pass. scoped to automatically drop it and release the borrow on encoder
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.cam_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            let idx_count = INDICES.len() as u32;
            render_pass.draw_indexed(0..idx_count, 0, 0..1);
        }

        self.gpu.queue.submit([encoder.finish()]);
        output.present();

        Ok(())
    }

    pub fn update(&mut self) {}
}
