use glam::IVec3;
use std::sync::Arc;
use wgpu::{
    BufferUsages, CurrentSurfaceTexture, Instance, InstanceDescriptor, SurfaceTexture,
    util::DeviceExt,
};
use winit::window::Window;

use crate::{
    constants::CHUNK_SIZEI,
    meshing::ChunkMeshData,
    render::{GpuChunk, texture::Texture},
    world::block::BlockType,
};

pub struct Gpu {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub window: Arc<Window>,
}

impl Gpu {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        // instance is mainly used for setup
        let instance = Instance::new(InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        // the surface is the area in the window we can draw directly to
        let surface = instance.create_surface(window.clone()).unwrap();

        // the adapter is a handle to the graphics card and contains information about it
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|_| anyhow::anyhow!("Failed to find adapter"))?;

        // here we can select a list of required features and change memory
        // allocation strategies.
        // TODO: look up raytracing features
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: adapter.features() & wgpu::Features::POLYGON_MODE_LINE,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        // pick srgb, or the first format available if not found.
        // if srgb is not found colours will probably look messed up
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        // configure the render surface
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);
        log::info!("GPU: {:?}", adapter.get_info());

        Ok(Self {
            surface,
            device,
            queue,
            config,
            window,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Uploads a chunk's vertex and index data to GPU buffers.
    pub fn create_chunk_buffers(
        &self,
        pos: IVec3,
        mesh_data: ChunkMeshData,
        blocks: &[BlockType],
        blocks_layout: &wgpu::BindGroupLayout,
    ) -> GpuChunk {
        let ChunkMeshData { vertices, indices } = &mesh_data;

        let vert_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("between chunk vertex buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: BufferUsages::VERTEX,
            });

        let idx_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("between chunk index buffer"),
                contents: bytemuck::cast_slice(indices),
                usage: BufferUsages::INDEX,
            });

        let block_texture = Texture::create_blocks_texture(self, blocks);

        // Create uniform buffer for world_pos
        let world_pos = (pos * CHUNK_SIZEI).as_vec3();
        let world_pos_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("chunk_world_pos_buffer"),
                // align data to 16 bytes because wgpu requires it
                contents: bytemuck::bytes_of(&glam::Vec4::new(
                    world_pos.x,
                    world_pos.y,
                    world_pos.z,
                    1.0,
                )),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let chunk_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: blocks_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: world_pos_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&block_texture.view),
                },
            ],
            label: Some("chunk_blocks_bind_group"),
        });

        GpuChunk {
            vert_buf,
            idx_buf,
            block_texture,
            chunk_bind_group,
        }
    }

    pub fn get_surface_texture(&mut self) -> Option<SurfaceTexture> {
        match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(surface_texture) => Some(surface_texture),
            CurrentSurfaceTexture::Suboptimal(surface_texture) => {
                drop(surface_texture);
                self.surface.configure(&self.device, &self.config);
                None
            }
            // skip these frames
            CurrentSurfaceTexture::Timeout
            | CurrentSurfaceTexture::Validation
            | CurrentSurfaceTexture::Occluded => None,
            CurrentSurfaceTexture::Outdated => {
                // reconfigure outdated configuration then skip frame
                self.surface.configure(&self.device, &self.config);
                None
            }
            CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                None
            }
        }
    }
}
