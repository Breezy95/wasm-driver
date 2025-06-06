use std::num::{NonZero, NonZeroU32};

use image::{GenericImageView, ImageReader};
use wgpu::{
    Device, Origin3d, SamplerDescriptor, core::resource::CreateTextureError, hal::TextureBinding,
};

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn create_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img_file: &str,
        u_mode: wgpu::AddressMode,
        v_mode: wgpu::AddressMode,
    ) -> Result<Self, CreateTextureError> {
        let img = ImageReader::open(img_file).expect("fauiled to find and open image file").decode().unwrap();
        //println!("image width is {:?}",img.asr);
        let rgba = img.to_rgba8();
        let rgba_bytes: &[u8] = rgba.as_raw();
        
        let dim = img.dimensions();

        let size = wgpu::Extent3d {
            width: dim.0,        
            height: dim.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(img_file),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba_bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(dim.0 * 4).unwrap().get().into(),
                rows_per_image: NonZeroU32::new(dim.1).unwrap().get().into(),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: u_mode,
            address_mode_v: v_mode,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }
}
