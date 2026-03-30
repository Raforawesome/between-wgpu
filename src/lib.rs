pub mod gpu;

use std::sync::Arc;
use gpu::Gpu;
use winit::{event_loop::ActiveEventLoop, keyboard::KeyCode, window::Window};

pub struct RenderState {
    gpu: Gpu,
    is_surface_configured: bool,
    render_pipeline: wgpu::RenderPipeline,
}

impl RenderState {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let mut gpu = Gpu::new(window).await?;

        // set up the render pipeline
        let shader = gpu.device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let pipeline_layout = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("between render pipeline layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        let render_pipeline = gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("between render pipeline"),
            layout: Some(&pipeline_layout),
            // set up vertex shader entry point
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
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
                compilation_options: wgpu::PipelineCompilationOptions::default(),
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

        Ok(Self {
            gpu,
            is_surface_configured: false,
            render_pipeline,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.gpu.resize(width, height);
        self.is_surface_configured = true;
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, key: KeyCode, is_pressed: bool) {
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
            self.gpu.device
                .create_command_encoder(&wgpu::wgt::CommandEncoderDescriptor {
                    label: Some("Between Render Encoder"),
                });

        // begin render pass
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.2,
                        g: 0.2,
                        b: 0.4,
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
        render_pass.draw(0..3, 0..1); // draw something with 3 vertices and 1 instance
        drop(render_pass); // drop render pass to release borrow

        let queue_jobs = std::iter::once(encoder.finish());
        self.gpu.queue.submit(queue_jobs);
        output.present();

        Ok(())
    }

    pub fn update(&mut self) {}
}
