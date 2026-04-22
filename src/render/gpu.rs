use anyhow::bail;
use std::sync::Arc;
use wgpu::{Buffer, BufferUsages, Instance, InstanceDescriptor, SurfaceTexture, util::DeviceExt};
use winit::window::Window;

use crate::render::vertex::Vertex;

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
                required_features: wgpu::Features::POLYGON_MODE_LINE,
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
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

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
    pub fn create_chunk_buffers(&self, verts: Vec<Vertex>, indices: Vec<u32>) -> (Buffer, Buffer) {
        let vert_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("between chunk vertex buffer"),
                contents: bytemuck::cast_slice(&verts),
                usage: BufferUsages::VERTEX,
            });

        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("between chunk index buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: BufferUsages::INDEX,
            });

        (vert_buffer, index_buffer)
    }

    pub fn get_surface_texture(&mut self) -> anyhow::Result<Option<SurfaceTexture>> {
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => Ok(Some(surface_texture)),
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => {
                self.surface.configure(&self.device, &self.config);
                Ok(Some(surface_texture))
            }
            // skip these frames
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Validation
            | wgpu::CurrentSurfaceTexture::Occluded => Ok(None),
            wgpu::CurrentSurfaceTexture::Outdated => {
                // reconfigure outdated configuration then skip frame
                self.surface.configure(&self.device, &self.config);
                Ok(None)
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                // device and resources lost, just bail
                bail!("lost device");
            }
        }
    }
}
