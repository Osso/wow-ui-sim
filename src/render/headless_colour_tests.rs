//! Opt-in readback regression for WoW UI display-byte colours on the production sRGB target.
//!
//! Explicit opt-in invocation (requires the `gui` feature):
//! ```text
//! cargo test --lib render::headless::colour_tests::colour_readback_uses_display_byte_rgb_values -- --ignored --exact --nocapture
//! ```
//! Hardware and CPU/software adapters both exercise the production wgpu shader
//! path and are valid for functional readback. The adapter is logged; absence
//! fails the test instead of silently skipping it.

use super::{create_render_target, read_back_pixels};
use crate::render::shader::{BlendMode, GLYPH_ATLAS_TEX_INDEX, QuadBatch, WowUiPipeline};
use iced::widget::shader::Pipeline;
use iced::{Point, Rectangle, Size};

const WIDTH: u32 = 48;
const HEIGHT: u32 = 8;
const GLYPH_ATLAS_SIZE: u32 = 2048;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

fn request_colour_test_device() -> (wgpu::Device, wgpu::Queue) {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    pollster::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await
            .expect("color readback requires an available wgpu adapter (hardware or software)");
        let info = adapter.get_info();
        eprintln!("color readback adapter: {info:?}; target={FORMAT:?}");
        adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("color readback GPU device")
    })
}

fn swatch_bounds(index: u32) -> Rectangle {
    Rectangle::new(Point::new((index * 8) as f32, 0.0), Size::new(8.0, 8.0))
}

fn upload_colour_swatches(pipeline: &mut WowUiPipeline, queue: &wgpu::Queue) -> QuadBatch {
    let texels = [51, 102, 153, 255].repeat(4 * 4);
    let entry = pipeline
        .texture_atlas_mut()
        .upload(queue, "colour-regression-srgb", 4, 4, &texels)
        .expect("constant sRGB texture upload");
    upload_glyph_colour_pixels(pipeline, queue);
    let mut batch = QuadBatch::new();
    batch.push_solid(swatch_bounds(0), [0.2, 0.4, 0.6, 1.0]);
    batch.push_textured_uv(
        swatch_bounds(1),
        entry.uv_rect(),
        entry.tex_index(),
        [1.0; 4],
        BlendMode::Alpha,
    );
    batch.push_solid(swatch_bounds(2), [0.2, 0.4, 0.6, 0.5]);
    batch.push_textured_uv(
        swatch_bounds(3),
        Rectangle::new(Point::ORIGIN, Size::new(1.0, 1.0)),
        -1,
        [0.2, 0.4, 0.6, 1.0],
        BlendMode::Additive,
    );
    batch.push_textured_uv(
        swatch_bounds(4),
        Rectangle::new(Point::ORIGIN, Size::new(1.0, 0.4)),
        GLYPH_ATLAS_TEX_INDEX,
        [1.0; 4],
        BlendMode::Alpha,
    );
    batch.push_textured_uv(
        swatch_bounds(5),
        Rectangle::new(Point::new(0.0, 0.6), Size::new(1.0, 0.4)),
        GLYPH_ATLAS_TEX_INDEX,
        [0.5, 1.0, 1.0, 1.0],
        BlendMode::Alpha,
    );
    batch
}

fn upload_glyph_colour_pixels(pipeline: &mut WowUiPipeline, queue: &wgpu::Queue) {
    let mut glyph_pixels =
        [51, 102, 153, 255].repeat((GLYPH_ATLAS_SIZE * GLYPH_ATLAS_SIZE) as usize);
    let first_translucent_pixel = (GLYPH_ATLAS_SIZE * GLYPH_ATLAS_SIZE / 2) as usize;
    for pixel in glyph_pixels
        .chunks_exact_mut(4)
        .skip(first_translucent_pixel)
    {
        pixel[3] = 128;
    }
    pipeline
        .texture_atlas_mut()
        .upload_glyph_atlas(queue, &glyph_pixels, GLYPH_ATLAS_SIZE);
}

#[test]
fn colour_readback_uses_display_byte_rgb_values() {
    let _gpu = crate::render::headless::lock_gpu_device_lifetime();
    let (device, queue) = request_colour_test_device();
    let mut pipeline = WowUiPipeline::new(&device, &queue, FORMAT);
    let (target, view) = create_render_target(&device, WIDTH, HEIGHT, FORMAT);
    let batch = upload_colour_swatches(&mut pipeline, &queue);
    let viewport = Rectangle::new(Point::ORIGIN, Size::new(WIDTH as f32, HEIGHT as f32));
    pipeline.update_projection(&queue, &viewport);
    pipeline.upload_strata(&device, &queue, 0, &batch);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    let clip = Rectangle {
        x: 0,
        y: 0,
        width: WIDTH,
        height: HEIGHT,
    };
    pipeline.render_clear(&mut encoder, &view, &clip, [0.0, 0.0, 0.0, 1.0]);
    let image = read_back_pixels(&device, &queue, encoder, &target, WIDTH, HEIGHT);

    // UI and image RGB values are display bytes. Half-alpha blending still occurs
    // in linear space before the sRGB attachment encodes the result.
    let cases = [
        ("solid-display-bytes", 4, [51, 102, 153, 255]),
        ("texture-srgb-roundtrip", 12, [51, 102, 153, 255]),
        ("solid-half-alpha-over-black", 20, [35, 73, 111, 255]),
        ("additive-display-bytes", 28, [51, 102, 153, 255]),
        ("glyph-unorm-display-bytes", 36, [51, 102, 153, 255]),
        ("glyph-unorm-tint-and-alpha", 44, [16, 73, 111, 255]),
    ];
    for (name, x, expected) in &cases {
        eprintln!(
            "{name}: mapped={:?}, expected={expected:?}",
            image.get_pixel(*x, 4).0
        );
    }
    for (name, x, expected) in cases {
        let actual = image.get_pixel(x, 4).0;
        for channel in 0..4 {
            assert!(
                actual[channel].abs_diff(expected[channel]) <= 1,
                "{name}: channel {channel}, mapped={actual:?}, expected={expected:?} +/- 1"
            );
        }
    }
}
