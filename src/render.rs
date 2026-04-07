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

use crate::{
    render::camera::{Camera, CameraController},
    world::chunk_manager::ChunkManager,
};

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
    /// Number of indices to draw
    index_count: u32,
    /// Bind group for the diffuse texture array and sampler.
    diffuse_bind_group: wgpu::BindGroup,
    /// Camera for the scene.
    camera: Camera,
    /// Buffer for the camera matrix.
    cam_buffer: wgpu::Buffer,
    /// Bind group for the camera matrix.
    cam_bind_group: wgpu::BindGroup,
    /// Controller for moving the camera.
    camera_controller: CameraController,
    /// Stores a texture to sample a pixel's 3D depth.
    depth_texture: Texture,
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
            (0., 90., -2.),
            (0., 0., 0.),
            gpu.config.width as f32 / gpu.config.height as f32,
        );
        let cam_buffer = gpu.create_camera_buffer(&camera);
        let cam_group_layout = gpu.device.create_bind_group_layout(&Camera::layout_desc());
        let cam_bind_group = gpu.create_camera_bind_group(&cam_buffer, &cam_group_layout);

        let camera_controller = CameraController::default();

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

        let depth_texture = Texture::create_depth_texture(&gpu);
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
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: Texture::DEPTH_FORMAT,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                cache: None,          // not useful for this project
                multiview_mask: None, // not useful for this project
            });

        let chunk_man = ChunkManager::default();
        let mut verts: Vec<Vertex> = vec![];
        let mut indices: Vec<u32> = vec![];
        for x in -16..=16 {
            for z in -16..=16 {
                for y in 4..=5 {
                    let chunk = chunk_man.get_or_generate_chunk((x, y, z).into());
                    let (lv, li) = chunk_man.mesh_chunk(&chunk);

                    let base = verts.len() as u32;
                    verts.extend_from_slice(&lv);
                    indices.extend(li.iter().map(|i| base + *i));
                }
            }
        }
        // let chunk = chunk_man.get_or_generate_chunk((1, 5, -1).into());
        // let (verts, indices) = chunk_man.mesh_chunk(&chunk);

        let vertex_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("between vertex buffer"),
                contents: bytemuck::cast_slice(&verts),
                usage: BufferUsages::VERTEX,
            });

        let index_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("between index buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: BufferUsages::INDEX,
            });

        Ok(Self {
            is_surface_configured: false,
            camera,
            gpu,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            diffuse_bind_group,
            cam_buffer,
            cam_bind_group,
            camera_controller,
            depth_texture,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.gpu.resize(width, height);
        self.depth_texture = Texture::create_depth_texture(&self.gpu);
        self.camera.resize_aspect(&self.gpu.config);
        self.is_surface_configured = true;
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, key: KeyCode, is_pressed: bool) {
        if self.camera_controller.process_events(key, is_pressed) {
            return;
        }

        #[allow(clippy::single_match)]
        match (key, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }

    pub fn handle_mouse_motion(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.camera_controller
            .process_mouse(mouse_dx, mouse_dy, &mut self.camera);
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.cam_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            render_pass.draw_indexed(0..self.index_count, 0, 0..1);
        }

        self.gpu.queue.submit([encoder.finish()]);
        output.present();

        Ok(())
    }

    pub fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.gpu.queue.write_buffer(
            &self.cam_buffer,
            0,
            bytemuck::bytes_of(&self.camera.build_view_projection_matrix()),
        );
    }
}
