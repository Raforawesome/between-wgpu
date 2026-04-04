use anyhow::*;
use image::GenericImageView;
use wgpu::{BindGroupEntry, BindGroupLayoutEntry, BindingResource, BindingType};

use crate::render::gpu::Gpu;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    /// Convenience method to create the default texture array for the game
    pub fn create_texture_array(gpu: &Gpu) -> Self {
        let bytes_grass_side = include_bytes!("../../assets/game/grass_side.png");
        let bytes_dirt_side = include_bytes!("../../assets/game/dirt.png");

        Self::from_bytes_array(
            &gpu.device,
            &gpu.queue,
            &[bytes_grass_side, bytes_dirt_side],
            Some("Block Texture Array"),
        )
        .expect("valid texture array")
    }

    /// Creates a 3D texture which consists of a series of 2D textures,
    /// sorted by the z coordinate.
    pub fn from_bytes_array(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes_array: &[&[u8]],
        label: Option<&str>,
    ) -> Result<Self> {
        if bytes_array.is_empty() {
            bail!("Texture array cannot be empty");
        }

        // load the first image to get dimensions
        let first_image = image::load_from_memory(bytes_array[0])?;
        let dimensions = first_image.dimensions();

        let diffuse_tex = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth_or_array_layers: bytes_array.len() as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        for (i, bytes) in bytes_array.iter().enumerate() {
            let img = image::load_from_memory(bytes)?;
            if img.dimensions() != dimensions {
                bail!("All images in a texture array must have the same dimensions");
            }
            let rgba = img.to_rgba8();

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &diffuse_tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: i as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * dimensions.0),
                    rows_per_image: Some(dimensions.1),
                },
                wgpu::Extent3d {
                    width: dimensions.0,
                    height: dimensions.1,
                    depth_or_array_layers: 1, // Copying one layer at a time
                },
            );
        }

        let view = diffuse_tex.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Texture Array View"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest, // Crisp pixel art
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture: diffuse_tex,
            view,
            sampler,
        })
    }

    /*
     * These methods help reduce the wgpu boilerplate around setting up the bind group
     * for game textures.
     */
    /// Returns the layout descriptor for the texture bind group.
    pub const fn layout_desc() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // this should match the filterable field of the corresponding Texture entry above.
                    ty: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        }
    }

    /// Creates bind group entries for a texture struct.
    pub const fn bind_entries<'a>(&'a self) -> [BindGroupEntry<'a>; 2] {
        [
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&self.view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&self.sampler),
            },
        ]
    }
}

impl Gpu {
    pub fn create_texture_bind_group(
        &self,
        texture: &Texture,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        let device = &self.device;

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &texture.bind_entries(),
            label: Some("diffuse_bind_group"),
        })
    }
}
