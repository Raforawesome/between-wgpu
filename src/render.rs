pub mod camera;
pub mod frustum;
pub mod gpu;
pub mod texture;
pub mod vertex;

use camera::Camera;
use glam::IVec3;
use gpu::Gpu;
use texture::Texture;
use vertex::Vertex;

use wgpu::{Buffer, PipelineCompilationOptions};
use winit::{event::ElementState, keyboard::KeyCode, window::Window};

use std::sync::Arc;

use crate::{HashMap, WorldState, util::LruCache, world::loader::ChunkLoader};

pub const GPU_CACHE_BYTES: u64 = 64 * 1024 * 1024;

#[allow(unused)] // uniform buffers are only accessed gpu side
pub struct GpuChunk {
    /// The GPU vertex buffer for the chunk.
    vert_buf: Buffer,
    /// The GPU index buffer for the chunk.
    idx_buf: Buffer,
    /// The GPU texture storing block ids for this chunk.
    block_texture: Texture,
    /// Bind group exposing the block-id volume to the shader.
    chunk_bind_group: wgpu::BindGroup,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct RenderStats {
    pub resident: usize,
    pub drawn: usize,
    pub culled: usize,
    pub triangles: usize,
}

impl GpuChunk {
    /// Payload estimate; driver allocation overhead is not included.
    fn bytes(&self) -> u64 {
        self.vert_buf.size() + self.idx_buf.size() + crate::constants::CHUNK_VOL as u64 + 16
    }
}

/// Handles setting up a renderer with [wgpu]. Contains an abstraction for the GPU device.
pub struct RenderState {
    /// Abstraction for the physical GPU device which handles interfacing and surfaces.
    pub gpu: Gpu,
    is_surface_configured: bool,
    /// The render pipeline contains the buffers, shaders, and post-processing stages.
    render_pipeline: wgpu::RenderPipeline,
    /// Render pipeline variant that draws triangle edges using polygon line mode.
    wireframe_pipeline: Option<wgpu::RenderPipeline>,
    culling_enabled: bool,
    pub stats: RenderStats,
    wireframe_enabled: bool,
    /// Stores a list of chunk vertex & index buffers to pass to the GPU.
    chunk_buffers: HashMap<IVec3, GpuChunk>,
    /// Contains a list of chunks that are outside view distance but being temporarily kept.
    chunk_lru_cache: LruCache<IVec3, GpuChunk, 2048>,
    cached_bytes: u64,
    /// Bind group for the diffuse texture array and sampler.
    diffuse_bind_group: wgpu::BindGroup,
    /// Layout for per-chunk block-id textures.
    blocks_group_layout: wgpu::BindGroupLayout,
    /// Camera for the scene.
    camera: Camera,
    /// Buffer for the camera matrix.
    cam_buffer: Buffer,
    /// Bind group for the camera matrix.
    cam_bind_group: wgpu::BindGroup,
    /// Stores a texture to sample a pixel's 3D depth.
    depth_texture: Texture,
}

impl RenderState {
    /// Creates the render state using the gpu device.
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let gpu = Gpu::new(window).await?;

        // set up texture array & texture bind group
        let texture_array = Texture::create_texture_array(&gpu);
        let tex_group_layout = gpu
            .device
            .create_bind_group_layout(&Texture::tex_layout_desc());
        let diffuse_bind_group = gpu.create_texture_bind_group(&texture_array, &tex_group_layout);
        let blocks_group_layout = gpu
            .device
            .create_bind_group_layout(&Texture::blocks_layout_desc());

        // set up camera & camera bind group (creates camera buffer)
        let camera = Camera::new(
            (0., 90., -2.),
            (0., 0., 0.),
            gpu.config.width as f32 / gpu.config.height as f32,
        );
        let cam_buffer = gpu.create_camera_buffer(&camera);
        let cam_group_layout = gpu.device.create_bind_group_layout(&Camera::layout_desc());
        let cam_bind_group = gpu.create_camera_bind_group(&cam_buffer, &cam_group_layout);

        // set up the render pipeline
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("between shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!(concat!(env!("OUT_DIR"), "/shader.wgsl")).into(),
                ),
            });
        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("between render pipeline layout"),
                bind_group_layouts: &[
                    Some(&tex_group_layout),
                    Some(&cam_group_layout),
                    Some(&blocks_group_layout),
                ],
                immediate_size: 0,
            });

        /*
         * set up render pipelines for normal view & wireframe view
         */
        let depth_texture = Texture::create_depth_texture(&gpu);

        // normal render pipeline (non-wireframe)
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

        // create render pipeline for wireframe view
        let wireframe_pipeline = gpu
            .device
            .features()
            .contains(wgpu::Features::POLYGON_MODE_LINE)
            .then(|| {
                gpu.device
                    .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("between wireframe pipeline"),
                        layout: Some(&pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &shader,
                            entry_point: Some("vs_main"),
                            buffers: &[Vertex::buffer_layout()],
                            compilation_options: PipelineCompilationOptions::default(),
                        },
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
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: Some(wgpu::Face::Back),
                            polygon_mode: wgpu::PolygonMode::Line,
                            unclipped_depth: false,
                            conservative: false,
                        },
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
                        cache: None,
                        multiview_mask: None,
                    })
            });

        Ok(Self {
            is_surface_configured: true,
            culling_enabled: true,
            stats: RenderStats::default(),
            camera,
            gpu,
            render_pipeline,
            wireframe_pipeline,
            wireframe_enabled: false,
            chunk_buffers: HashMap::default(),
            chunk_lru_cache: LruCache::default(),
            cached_bytes: 0,
            diffuse_bind_group,
            blocks_group_layout,
            cam_buffer,
            cam_bind_group,
            depth_texture,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            self.is_surface_configured = false;
            return;
        }
        self.gpu.resize(width, height);
        self.depth_texture = Texture::create_depth_texture(&self.gpu);
        self.camera.resize_aspect(&self.gpu.config);
        self.is_surface_configured = true;
    }

    pub fn render(&mut self, _game_state: &mut WorldState) {
        self.gpu.window.request_redraw();
        if !self.is_surface_configured {
            return;
        }

        let output = match self.gpu.get_surface_texture() {
            Some(output) => output,
            None => return,
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

        let frustum =
            frustum::Frustum::from_view_projection(self.camera.build_view_projection_matrix());
        self.stats = RenderStats {
            resident: self.chunk_buffers.len(),
            ..Default::default()
        };

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
                            r: 150. / 255.,
                            g: 150. / 255.,
                            b: 1.0,
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

            render_pass.set_pipeline(if self.wireframe_enabled {
                self.wireframe_pipeline
                    .as_ref()
                    .unwrap_or(&self.render_pipeline)
            } else {
                &self.render_pipeline
            });

            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.cam_bind_group, &[]);

            for chunk_pos in game_state.loader.loaded_chunks() {
                if let Some(gpu_chunk) = self.chunk_buffers.get(chunk_pos) {
                    let GpuChunk {
                        vert_buf,
                        idx_buf,
                        chunk_bind_group: block_bind_group,
                        .. // we dont need block_texture since only the gpu uses it
                    } = gpu_chunk;
                    let idx_count = idx_buf.size() as u32 / std::mem::size_of::<u32>() as u32;

                    render_pass.set_vertex_buffer(0, vert_buf.slice(..));
                    render_pass.set_index_buffer(idx_buf.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.set_bind_group(2, block_bind_group, &[]);
                    render_pass.draw_indexed(0..idx_count, 0, 0..1);
                    self.stats.drawn += 1;
                    self.stats.triangles += idx_count as usize / 3;
                };
            }
        }

        self.gpu.queue.submit([encoder.finish()]);
        output.present();
    }

    /// Handles key press inputs relating to render state, such as debug views.
    pub fn handle_input(&mut self, code: KeyCode, key_state: ElementState, repeat: bool) -> bool {
        match code {
            KeyCode::KeyP if key_state.is_pressed() && !repeat => {
                self.toggle_wireframe();
                true
            }
            _ => false,
        }
    }

    pub fn toggle_wireframe(&mut self) {
        self.wireframe_enabled = !self.wireframe_enabled;
    }

    pub fn update(&mut self, game_state: &mut WorldState) {
        // update the camera buffer
        let player = &game_state.player;
        self.camera.position = player.eye;
        self.camera.yaw = player.yaw;
        self.camera.pitch = player.pitch;

        self.gpu.queue.write_buffer(
            &self.cam_buffer,
            0,
            bytemuck::bytes_of(&self.camera.build_view_projection_matrix()),
        );
    }
}

// getters and setters
impl RenderState {
    pub fn has_chunk_buffer(&self, coords: &IVec3) -> bool {
        self.chunk_buffers.contains_key(coords)
    }

    pub fn insert_chunk_buffer(&mut self, coords: IVec3, chunk: GpuChunk) {
        self.take_cached_chunk(&coords);
        self.chunk_buffers.insert(coords, chunk);
    }

    pub fn cache_chunk_buffer(&mut self, coords: IVec3, chunk: GpuChunk) {
        self.chunk_buffers.remove(&coords);
        self.take_cached_chunk(&coords);
        self.cached_bytes += chunk.bytes();
        if let Some((_, evicted)) = self.chunk_lru_cache.put(coords, chunk) {
            self.cached_bytes -= evicted.bytes();
        }
        while self.cached_bytes > GPU_CACHE_BYTES {
            let (_, evicted) = self
                .chunk_lru_cache
                .pop_lru()
                .expect("nonempty cache exceeds budget");
            self.cached_bytes -= evicted.bytes();
        }
    }

    pub fn remove_chunk_buffer(&mut self, coords: &IVec3) {
        self.chunk_buffers.remove(coords);
        self.take_cached_chunk(coords);
    }

    pub fn move_chunk_buffer_to_cache(&mut self, coords: &IVec3) {
        if let Some(chunk) = self.chunk_buffers.remove(coords) {
            self.cache_chunk_buffer(*coords, chunk);
        }
    }

    fn take_cached_chunk(&mut self, coords: &IVec3) -> Option<GpuChunk> {
        let chunk = self.chunk_lru_cache.take(coords)?;
        self.cached_bytes -= chunk.bytes();
        Some(chunk)
    }

    pub fn restore_chunk_buffer_from_cache(&mut self, coords: &IVec3) -> bool {
        if let Some(chunk) = self.take_cached_chunk(coords) {
            self.chunk_buffers.insert(*coords, chunk);
            true
        } else {
            false
        }
    }

    pub fn cache_chunks_outside_view(&mut self, loader: &ChunkLoader) {
        let outside_view: Vec<_> = self
            .chunk_buffers
            .keys()
            .copied()
            .filter(|coords| !loader.contains(coords))
            .collect();

        for coords in outside_view {
            self.move_chunk_buffer_to_cache(&coords);
        }
    }

    pub fn blocks_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.blocks_group_layout
    }
}
