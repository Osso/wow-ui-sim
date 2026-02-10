//! Headless GPU rendering for producing screenshots.
//!
//! Uses the same wgpu shader pipeline as the iced GUI but drives it
//! without a window. Produces pixel-identical output to the live renderer.

use iced::widget::shader::Primitive;
use image::RgbaImage;

use super::shader::{GpuTextureData, QuadBatch, WowUiPrimitive};
use crate::texture::TextureManager;

/// Load unique textures for all batch texture requests.
fn load_batch_textures(
    batch: &QuadBatch,
    tex_mgr: &mut TextureManager,
) -> Vec<GpuTextureData> {
    let mut textures = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for request in batch.texture_requests.iter().chain(&batch.mask_texture_requests) {
        if seen.contains(&request.path) {
            continue;
        }
        if let Some(tex_data) = tex_mgr.load(&request.path) {
            textures.push(GpuTextureData {
                path: request.path.clone(),
                width: tex_data.width,
                height: tex_data.height,
                rgba: tex_data.pixels.clone(),
            });
            seen.insert(request.path.clone());
        }
    }
    textures
}

/// Create a headless wgpu device and queue.
fn create_headless_device() -> (wgpu::Device, wgpu::Queue) {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    pollster::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find GPU adapter");

        adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("Failed to create GPU device")
    })
}

/// Create a render target texture and its view.
fn create_render_target(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Screenshot Render Target"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

/// Copy render target to a readable buffer and read back pixels into an image.
fn read_back_pixels(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: wgpu::CommandEncoder,
    render_texture: &wgpu::Texture,
    width: u32,
    height: u32,
) -> RgbaImage {
    let bytes_per_row = (width * 4 + 255) & !255; // Align to 256
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Screenshot Output Buffer"),
        size: (bytes_per_row * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = encoder;
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: render_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &output_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
    );

    queue.submit(std::iter::once(encoder.finish()));

    let buffer_slice = output_buffer.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).unwrap();
    });
    let _ = device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: Some(std::time::Duration::from_secs(10)),
    });
    receiver.recv().unwrap().expect("Failed to map buffer");

    let data = buffer_slice.get_mapped_range();
    let mut img = RgbaImage::new(width, height);
    for y in 0..height {
        let src_offset = (y * bytes_per_row) as usize;
        let row = &data[src_offset..src_offset + (width * 4) as usize];
        for x in 0..width {
            let i = (x * 4) as usize;
            img.put_pixel(x, y, image::Rgba([row[i], row[i + 1], row[i + 2], row[i + 3]]));
        }
    }

    img
}

/// Render a QuadBatch to an RGBA image using headless wgpu.
///
/// Creates a headless GPU device, sets up the same WowUiPipeline used by
/// the iced GUI, and renders to an offscreen texture. The result is read
/// back to CPU memory as an RgbaImage.
///
/// When `glyph_atlas_data` is provided, text glyphs are rendered using the
/// glyph atlas texture.
pub fn render_to_image(
    batch: &QuadBatch,
    tex_mgr: &mut TextureManager,
    width: u32,
    height: u32,
    glyph_atlas_data: Option<(&[u8], u32)>,
) -> RgbaImage {
    let textures = load_batch_textures(batch, tex_mgr);
    let mut primitive = WowUiPrimitive::with_textures(std::sync::Arc::new(batch.clone()), textures);

    if let Some((data, size)) = glyph_atlas_data {
        primitive.glyph_atlas_data = Some(data.to_vec());
        primitive.glyph_atlas_size = size;
    }

    let (device, queue) = create_headless_device();
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;

    use iced::widget::shader::Pipeline;
    let mut pipeline = super::shader::WowUiPipeline::new(&device, &queue, format);
    let (render_texture, render_view) = create_render_target(&device, width, height, format);

    // Prepare (uploads textures, resolves tex_index, uploads buffers)
    let bounds = iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(width as f32, height as f32));
    let viewport = iced::widget::shader::Viewport::with_physical_size(iced::Size::new(width, height), 1.0);
    primitive.prepare(&mut pipeline, &device, &queue, &bounds, &viewport);

    // Render
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenshot Encoder"),
    });
    let clip_bounds_u32 = iced::Rectangle { x: 0u32, y: 0u32, width, height };
    pipeline.render_clear(&mut encoder, &render_view, &clip_bounds_u32, [0.05, 0.05, 0.08, 1.0]);

    read_back_pixels(&device, &queue, encoder, &render_texture, width, height)
}
