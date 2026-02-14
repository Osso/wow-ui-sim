//! GPU pipeline for WoW UI rendering.

use super::atlas::GpuTextureAtlas;
use super::quad::{QuadBatch, QuadVertex};
use iced::widget::shader;
use iced::Rectangle;
use std::mem;
use wgpu::util::DeviceExt;

/// Uniform buffer data for the shader.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    /// Projection matrix (orthographic, screen coords to clip space).
    projection: [[f32; 4]; 4],
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
        Self { projection }
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
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("WoW UI Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("quad.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("WoW UI Pipeline Layout"),
            bind_group_layouts: &[uniform_bind_group_layout, texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("WoW UI Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[QuadVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // No culling for 2D UI
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        })
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
            queue.write_buffer(
                &buf.vertex_buffer,
                0,
                bytemuck::cast_slice(&quads.vertices),
            );
        }
        if !quads.indices.is_empty() {
            queue.write_buffer(&buf.index_buffer, 0, bytemuck::cast_slice(&quads.indices));
        }
        buf.index_count = quads.indices.len();
    }

    /// Clear the index count for a strata slot (keeps buffer allocated).
    pub fn clear_strata(&mut self, slot: usize) {
        self.strata_buffers[slot].index_count = 0;
    }

    /// Render all strata + overlay using per-strata GPU buffers.
    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("WoW UI Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

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

        // Draw each strata + overlay in order.
        for buf in &self.strata_buffers {
            if buf.index_count > 0 {
                render_pass.set_vertex_buffer(0, buf.vertex_buffer.slice(..));
                render_pass.set_index_buffer(buf.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..buf.index_count as u32, 0, 0..1);
            }
        }
    }

    /// Render with a clear operation (for standalone/headless rendering).
    pub fn render_clear(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
        clear_color: [f32; 4],
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("WoW UI Render Pass (Clear)"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: clear_color[0] as f64,
                        g: clear_color[1] as f64,
                        b: clear_color[2] as f64,
                        a: clear_color[3] as f64,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

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
}

impl shader::Pipeline for WowUiPipeline {
    fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self {
        let texture_atlas = GpuTextureAtlas::new(device);

        let (uniform_buffer, uniform_bind_group_layout, uniform_bind_group) =
            create_uniform_resources(device);

        let pipeline = Self::create_pipeline(
            device,
            format,
            &uniform_bind_group_layout,
            texture_atlas.bind_group_layout(),
        );

        let strata_buffers: Vec<StrataGpuBuffer> =
            (0..BUFFER_SLOTS).map(|i| create_strata_buffer(device, i)).collect();

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
            visibility: wgpu::ShaderStages::VERTEX,
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
