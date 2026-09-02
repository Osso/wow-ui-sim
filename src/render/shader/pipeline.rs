//! GPU pipeline for WoW UI rendering.

use super::atlas::GpuTextureAtlas;
use super::quad::{QuadBatch, QuadVertex};
use iced::Rectangle;
use iced::widget::shader;
use std::mem;
use wgpu::util::DeviceExt;

/// Uniform buffer data for the shader.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    /// Projection matrix (orthographic, screen coords to clip space).
    projection: [[f32; 4]; 4],
    /// x: gamma divisor for the brightness boost the fragment shader applies
    /// to non-additive pixels (`pow(rgb, 1/x)`). 1.5 is the historical
    /// on-screen aid that lifts dark UI against the black backdrop; 1.0 is the
    /// identity, which a capture meant to match the client wants. The other
    /// three lanes keep the struct 16-byte aligned for the uniform buffer.
    params: [f32; 4],
}

/// Gamma divisor the shader uses to lift dark pixels, from
/// `WOW_SIM_BRIGHTNESS_BOOST`: unset / "1" keeps the historical 1.5, "0" or
/// "false" disables the lift (divisor 1.0), any other number is used as the
/// divisor directly.
pub fn brightness_boost_divisor() -> f32 {
    brightness_boost_divisor_with(
        override_divisor(),
        std::env::var("WOW_SIM_BRIGHTNESS_BOOST").ok().as_deref(),
    )
}

/// Process-wide override of the divisor, set by a command that wants a
/// particular tone regardless of the environment (the screenshot command
/// captures texel-for-texel unless `WOW_SIM_BRIGHTNESS_BOOST` says
/// otherwise). Stored as the f32's bits; 0 means "no override".
static BRIGHTNESS_BOOST_OVERRIDE: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(0);

pub fn set_brightness_boost_divisor(divisor: Option<f32>) {
    let bits = divisor.filter(|d| *d > 0.0).map(f32::to_bits).unwrap_or(0);
    BRIGHTNESS_BOOST_OVERRIDE.store(bits, std::sync::atomic::Ordering::Relaxed);
}

fn override_divisor() -> Option<f32> {
    match BRIGHTNESS_BOOST_OVERRIDE.load(std::sync::atomic::Ordering::Relaxed) {
        0 => None,
        bits => Some(f32::from_bits(bits)),
    }
}

/// An explicit override wins; otherwise the environment decides.
fn brightness_boost_divisor_with(override_divisor: Option<f32>, env_value: Option<&str>) -> f32 {
    override_divisor.unwrap_or_else(|| brightness_boost_divisor_from(env_value))
}

fn brightness_boost_divisor_from(value: Option<&str>) -> f32 {
    const DEFAULT: f32 = 1.5;
    let Some(v) = value else {
        return DEFAULT;
    };
    let v = v.trim();
    if v == "0" || v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("off") {
        1.0
    } else if v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("on") {
        DEFAULT
    } else {
        v.parse::<f32>()
            .ok()
            .filter(|d| *d > 0.0)
            .unwrap_or(DEFAULT)
    }
}

impl Uniforms {
    fn new(width: f32, height: f32) -> Self {
        // Orthographic projection: (0,0) top-left, (width,height) bottom-right
        // Maps to clip space (-1,-1) to (1,1), with Y flipped
        let projection = [
            [2.0 / width, 0.0, 0.0, 0.0],
            [0.0, -2.0 / height, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ];
        Self {
            projection,
            params: [brightness_boost_divisor(), 0.0, 0.0, 0.0],
        }
    }
}

use crate::widget::FrameStrata;

/// Per-strata GPU vertex and index buffers.
struct StrataGpuBuffer {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    index_capacity: usize,
    index_count: usize,
    #[cfg(test)]
    last_uploaded_vertices: Vec<QuadVertex>,
}

/// Total number of GPU buffer slots: 9 strata + 1 overlay.
const BUFFER_SLOTS: usize = FrameStrata::COUNT + 1;

/// GPU pipeline holding persistent rendering resources.
pub struct WowUiPipeline {
    /// Render pipeline for quad drawing.
    pipeline: wgpu::RenderPipeline,
    /// Uniform buffer for projection matrix.
    uniform_buffer: wgpu::Buffer,
    /// Bind group for uniforms.
    uniform_bind_group: wgpu::BindGroup,
    /// Per-strata GPU buffers (9 strata + 1 overlay).
    strata_buffers: Vec<StrataGpuBuffer>,
    /// Texture format (stored for potential pipeline recreation).
    _format: wgpu::TextureFormat,
    /// Current viewport size.
    viewport_size: (u32, u32),
    /// GPU texture atlas for texture storage.
    texture_atlas: GpuTextureAtlas,
}

impl std::fmt::Debug for WowUiPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WowUiPipeline")
            .field("buffer_slots", &self.strata_buffers.len())
            .field("viewport_size", &self.viewport_size)
            .finish()
    }
}

/// Create a single strata GPU buffer pair (vertex + index) with initial capacity.
fn create_strata_buffer(device: &wgpu::Device, label_idx: usize) -> StrataGpuBuffer {
    let vb = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(&format!("WoW UI Vertex Buffer [strata {}]", label_idx)),
        size: 4096,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let ib = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(&format!("WoW UI Index Buffer [strata {}]", label_idx)),
        size: 4096,
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    StrataGpuBuffer {
        vertex_buffer: vb,
        index_buffer: ib,
        vertex_capacity: 4096,
        index_capacity: 4096,
        index_count: 0,
        #[cfg(test)]
        last_uploaded_vertices: Vec::new(),
    }
}

impl WowUiPipeline {
    /// Create the render pipeline.
    fn create_pipeline(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let shader = Self::create_shader_module(device);
        let pipeline_layout = Self::create_pipeline_layout(
            device,
            uniform_bind_group_layout,
            texture_bind_group_layout,
        );
        let vertex_buffers = [QuadVertex::desc()];
        let color_targets = [Some(Self::color_target_state(format))];

        let descriptor = Self::render_pipeline_descriptor(
            &pipeline_layout,
            &shader,
            &vertex_buffers,
            &color_targets,
        );
        device.create_render_pipeline(&descriptor)
    }

    fn create_shader_module(device: &wgpu::Device) -> wgpu::ShaderModule {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("WoW UI Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("quad.wgsl").into()),
        })
    }

    fn create_pipeline_layout(
        device: &wgpu::Device,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::PipelineLayout {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("WoW UI Pipeline Layout"),
            bind_group_layouts: &[uniform_bind_group_layout, texture_bind_group_layout],
            push_constant_ranges: &[],
        })
    }

    fn render_pipeline_descriptor<'a>(
        pipeline_layout: &'a wgpu::PipelineLayout,
        shader: &'a wgpu::ShaderModule,
        vertex_buffers: &'a [wgpu::VertexBufferLayout<'static>],
        color_targets: &'a [Option<wgpu::ColorTargetState>],
    ) -> wgpu::RenderPipelineDescriptor<'a> {
        wgpu::RenderPipelineDescriptor {
            label: Some("WoW UI Render Pipeline"),
            layout: Some(pipeline_layout),
            vertex: Self::vertex_state(shader, vertex_buffers),
            fragment: Some(Self::fragment_state(shader, color_targets)),
            primitive: Self::triangle_list_primitive_state(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        }
    }

    fn vertex_state<'a>(
        shader: &'a wgpu::ShaderModule,
        vertex_buffers: &'a [wgpu::VertexBufferLayout<'static>],
    ) -> wgpu::VertexState<'a> {
        wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: vertex_buffers,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }
    }

    fn fragment_state<'a>(
        shader: &'a wgpu::ShaderModule,
        targets: &'a [Option<wgpu::ColorTargetState>],
    ) -> wgpu::FragmentState<'a> {
        wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }
    }

    fn color_target_state(format: wgpu::TextureFormat) -> wgpu::ColorTargetState {
        wgpu::ColorTargetState {
            format,
            blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        }
    }

    fn triangle_list_primitive_state() -> wgpu::PrimitiveState {
        wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        }
    }

    /// Update the projection matrix if the viewport size changed.
    pub fn update_projection(&mut self, queue: &wgpu::Queue, bounds: &iced::Rectangle) {
        let width = bounds.width as u32;
        let height = bounds.height as u32;
        if self.viewport_size != (width, height) {
            self.viewport_size = (width, height);
            let uniforms = Uniforms::new(bounds.width, bounds.height);
            queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
        }
    }

    fn trace_strata_upload(slot: usize, quads: &QuadBatch) {
        let pending_tex_vertices = quads.vertices.iter().filter(|v| v.tex_index == -2).count();
        let pending_mask_vertices = quads
            .vertices
            .iter()
            .filter(|v| v.mask_tex_index == -2)
            .count();
        let sample = quads
            .vertices
            .iter()
            .find(|v| v.tex_index >= 0 || v.mask_tex_index >= 0)
            .or_else(|| quads.vertices.first());
        let sample_text = sample.map_or_else(
            || "sample=none".to_string(),
            |vertex| {
                format!(
                    "sample_tex_index={} sample_tex_coords=({:.3},{:.3}) sample_mask_tex_index={} sample_mask_tex_coords=({:.3},{:.3})",
                    vertex.tex_index,
                    vertex.tex_coords[0],
                    vertex.tex_coords[1],
                    vertex.mask_tex_index,
                    vertex.mask_tex_coords[0],
                    vertex.mask_tex_coords[1],
                )
            },
        );
        crate::logging::eprintln_gui_trace(&format!(
            "upload_strata slot={slot} vertices={} indices={} pending_tex_vertices={pending_tex_vertices} pending_mask_vertices={pending_mask_vertices} {sample_text}",
            quads.vertices.len(),
            quads.indices.len(),
        ));
    }

    /// Upload quad data for a single strata/overlay slot.
    ///
    /// Resizes the slot's vertex/index buffers if needed, then writes data.
    pub fn upload_strata(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        slot: usize,
        quads: &QuadBatch,
    ) {
        if crate::logging::gui_trace_enabled() {
            Self::trace_strata_upload(slot, quads);
        }

        let buf = &mut self.strata_buffers[slot];

        let vertex_size = quads.vertices.len() * mem::size_of::<QuadVertex>();
        if vertex_size > buf.vertex_capacity {
            buf.vertex_capacity = vertex_size.next_power_of_two().max(4096);
            buf.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("WoW UI Vertex Buffer [strata {}]", slot)),
                size: buf.vertex_capacity as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        let index_size = quads.indices.len() * mem::size_of::<u32>();
        if index_size > buf.index_capacity {
            buf.index_capacity = index_size.next_power_of_two().max(4096);
            buf.index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("WoW UI Index Buffer [strata {}]", slot)),
                size: buf.index_capacity as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        if !quads.vertices.is_empty() {
            queue.write_buffer(&buf.vertex_buffer, 0, bytemuck::cast_slice(&quads.vertices));
        }
        if !quads.indices.is_empty() {
            queue.write_buffer(&buf.index_buffer, 0, bytemuck::cast_slice(&quads.indices));
        }
        buf.index_count = quads.indices.len();
        #[cfg(test)]
        {
            buf.last_uploaded_vertices = quads.vertices.clone();
        }
    }

    /// Clear the index count for a strata slot (keeps buffer allocated).
    pub fn clear_strata(&mut self, slot: usize) {
        self.strata_buffers[slot].index_count = 0;
        #[cfg(test)]
        self.strata_buffers[slot].last_uploaded_vertices.clear();
    }

    /// Render all strata + overlay using per-strata GPU buffers.
    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        self.render_with_load_op(
            encoder,
            target,
            clip_bounds,
            "WoW UI Render Pass",
            wgpu::LoadOp::Load,
        );
    }

    /// Render with a clear operation (for standalone/headless rendering).
    pub fn render_clear(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
        clear_color: [f32; 4],
    ) {
        let clear_color = wgpu::Color {
            r: clear_color[0] as f64,
            g: clear_color[1] as f64,
            b: clear_color[2] as f64,
            a: clear_color[3] as f64,
        };
        self.render_with_load_op(
            encoder,
            target,
            clip_bounds,
            "WoW UI Render Pass (Clear)",
            wgpu::LoadOp::Clear(clear_color),
        );
    }

    fn render_with_load_op(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
        label: &'static str,
        load: wgpu::LoadOp<wgpu::Color>,
    ) {
        let color_attachment = Self::render_pass_color_attachment(target, load);
        let color_attachments = [Some(color_attachment)];
        let descriptor = Self::render_pass_descriptor(label, &color_attachments);
        let mut render_pass = encoder.begin_render_pass(&descriptor);

        self.configure_render_pass(&mut render_pass, clip_bounds);
        self.draw_strata_buffers(&mut render_pass);
    }

    fn render_pass_color_attachment<'a>(
        target: &'a wgpu::TextureView,
        load: wgpu::LoadOp<wgpu::Color>,
    ) -> wgpu::RenderPassColorAttachment<'a> {
        wgpu::RenderPassColorAttachment {
            view: target,
            resolve_target: None,
            ops: wgpu::Operations {
                load,
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        }
    }

    fn render_pass_descriptor<'a>(
        label: &'static str,
        color_attachments: &'a [Option<wgpu::RenderPassColorAttachment<'a>>],
    ) -> wgpu::RenderPassDescriptor<'a> {
        wgpu::RenderPassDescriptor {
            label: Some(label),
            color_attachments,
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        }
    }

    fn configure_render_pass(
        &self,
        render_pass: &mut wgpu::RenderPass<'_>,
        clip_bounds: &Rectangle<u32>,
    ) {
        render_pass.set_viewport(
            clip_bounds.x as f32,
            clip_bounds.y as f32,
            clip_bounds.width as f32,
            clip_bounds.height as f32,
            0.0,
            1.0,
        );
        render_pass.set_scissor_rect(
            clip_bounds.x,
            clip_bounds.y,
            clip_bounds.width,
            clip_bounds.height,
        );
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_bind_group(1, self.texture_atlas.bind_group(), &[]);
    }

    fn draw_strata_buffers(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        for buf in &self.strata_buffers {
            if buf.index_count > 0 {
                render_pass.set_vertex_buffer(0, buf.vertex_buffer.slice(..));
                render_pass.set_index_buffer(buf.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..buf.index_count as u32, 0, 0..1);
            }
        }
    }

    /// Get mutable access to the texture atlas.
    pub fn texture_atlas_mut(&mut self) -> &mut GpuTextureAtlas {
        &mut self.texture_atlas
    }

    #[cfg(test)]
    pub(crate) fn uploaded_vertices(&self, slot: usize) -> &[QuadVertex] {
        &self.strata_buffers[slot].last_uploaded_vertices
    }
}

impl shader::Pipeline for WowUiPipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let texture_atlas = GpuTextureAtlas::new(device);

        let (uniform_buffer, uniform_bind_group_layout, uniform_bind_group) =
            create_uniform_resources(device);

        let pipeline = Self::create_pipeline(
            device,
            format,
            &uniform_bind_group_layout,
            texture_atlas.bind_group_layout(),
        );

        let strata_buffers: Vec<StrataGpuBuffer> = (0..BUFFER_SLOTS)
            .map(|i| create_strata_buffer(device, i))
            .collect();

        Self {
            pipeline,
            uniform_buffer,
            uniform_bind_group,
            strata_buffers,
            _format: format,
            viewport_size: (0, 0),
            texture_atlas,
        }
    }
}

/// Create uniform buffer, bind group layout, and bind group for the projection matrix.
fn create_uniform_resources(
    device: &wgpu::Device,
) -> (wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
    let uniforms = Uniforms::new(1920.0, 1080.0);
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("WoW UI Uniform Buffer"),
        contents: bytemuck::cast_slice(&[uniforms]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("WoW UI Uniform Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("WoW UI Uniform Bind Group"),
        layout: &layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    (uniform_buffer, layout, bind_group)
}

#[cfg(test)]
mod brightness_boost_tests {
    use super::{brightness_boost_divisor_from, brightness_boost_divisor_with};

    #[test]
    fn explicit_divisor_override_wins_over_the_environment() {
        assert_eq!(brightness_boost_divisor_with(Some(1.0), None), 1.0);
        assert_eq!(brightness_boost_divisor_with(Some(1.0), Some("2.2")), 1.0);
        assert_eq!(brightness_boost_divisor_with(None, Some("2.2")), 2.2);
        assert_eq!(brightness_boost_divisor_with(None, None), 1.5);
    }

    #[test]
    fn unset_keeps_the_historical_lift() {
        assert_eq!(brightness_boost_divisor_from(None), 1.5);
        assert_eq!(brightness_boost_divisor_from(Some("1")), 1.5);
        assert_eq!(brightness_boost_divisor_from(Some("on")), 1.5);
    }

    #[test]
    fn zero_false_and_off_disable_the_lift() {
        assert_eq!(brightness_boost_divisor_from(Some("0")), 1.0);
        assert_eq!(brightness_boost_divisor_from(Some("false")), 1.0);
        assert_eq!(brightness_boost_divisor_from(Some(" OFF ")), 1.0);
    }

    #[test]
    fn a_positive_number_is_the_divisor_and_garbage_falls_back() {
        assert_eq!(brightness_boost_divisor_from(Some("2.2")), 2.2);
        assert_eq!(brightness_boost_divisor_from(Some("-1")), 1.5);
        assert_eq!(brightness_boost_divisor_from(Some("bright")), 1.5);
    }
}
