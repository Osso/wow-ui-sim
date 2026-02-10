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

/// GPU pipeline holding persistent rendering resources.
pub struct WowUiPipeline {
    /// Render pipeline for quad drawing.
    pipeline: wgpu::RenderPipeline,
    /// Uniform buffer for projection matrix.
    uniform_buffer: wgpu::Buffer,
    /// Bind group for uniforms.
    uniform_bind_group: wgpu::BindGroup,
    /// Vertex buffer (resized as needed).
    vertex_buffer: wgpu::Buffer,
    /// Index buffer (resized as needed).
    index_buffer: wgpu::Buffer,
    /// Current vertex buffer capacity.
    vertex_capacity: usize,
    /// Current index buffer capacity.
    index_capacity: usize,
    /// Texture format (stored for potential pipeline recreation).
    _format: wgpu::TextureFormat,
    /// Current viewport size.
    viewport_size: (u32, u32),
    /// GPU texture atlas for texture storage.
    texture_atlas: GpuTextureAtlas,
    /// Number of indices in the last uploaded batch (for render).
    last_index_count: usize,
}

impl std::fmt::Debug for WowUiPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WowUiPipeline")
            .field("vertex_capacity", &self.vertex_capacity)
            .field("index_capacity", &self.index_capacity)
            .field("viewport_size", &self.viewport_size)
            .finish()
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
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
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

    /// Prepare GPU buffers with quad data.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &iced::Rectangle,
        quads: &QuadBatch,
    ) {
        // Use widget bounds for projection (not full viewport)
        // This makes coordinates relative to the widget like canvas does
        let width = bounds.width as u32;
        let height = bounds.height as u32;

        // Update projection if size changed
        if self.viewport_size != (width, height) {
            self.viewport_size = (width, height);
            let uniforms = Uniforms::new(bounds.width, bounds.height);
            queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
        }

        // Resize vertex buffer if needed
        let vertex_size = quads.vertices.len() * mem::size_of::<QuadVertex>();
        if vertex_size > self.vertex_capacity {
            self.vertex_capacity = vertex_size.next_power_of_two().max(4096);
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("WoW UI Vertex Buffer"),
                size: self.vertex_capacity as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        // Resize index buffer if needed
        let index_size = quads.indices.len() * mem::size_of::<u32>();
        if index_size > self.index_capacity {
            self.index_capacity = index_size.next_power_of_two().max(4096);
            self.index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("WoW UI Index Buffer"),
                size: self.index_capacity as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        // Upload vertex and index data
        if !quads.vertices.is_empty() {
            queue.write_buffer(
                &self.vertex_buffer,
                0,
                bytemuck::cast_slice(&quads.vertices),
            );
        }
        if !quads.indices.is_empty() {
            queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&quads.indices));
        }

        // Store index count for render
        self.last_index_count = quads.indices.len();
    }

    /// Render the quads using data uploaded in prepare().
    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        // Use LoadOp::Load to preserve iced's UI elements (console border, etc.)
        // The scissor rect only affects draw calls, not clear operations.
        // We draw a background quad instead of using clear.
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

        // Set viewport to match clip bounds - this ensures coordinates map correctly
        render_pass.set_viewport(
            clip_bounds.x as f32,
            clip_bounds.y as f32,
            clip_bounds.width as f32,
            clip_bounds.height as f32,
            0.0,
            1.0,
        );

        // Set scissor rect to clip drawing to widget bounds
        render_pass.set_scissor_rect(
            clip_bounds.x,
            clip_bounds.y,
            clip_bounds.width,
            clip_bounds.height,
        );

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_bind_group(1, self.texture_atlas.bind_group(), &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        if self.last_index_count > 0 {
            render_pass.draw_indexed(0..self.last_index_count as u32, 0, 0..1);
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
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        if self.last_index_count > 0 {
            render_pass.draw_indexed(0..self.last_index_count as u32, 0, 0..1);
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

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("WoW UI Vertex Buffer"),
            size: 4096,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("WoW UI Index Buffer"),
            size: 4096,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let pipeline = Self::create_pipeline(
            device,
            format,
            &uniform_bind_group_layout,
            texture_atlas.bind_group_layout(),
        );

        Self {
            pipeline,
            uniform_buffer,
            uniform_bind_group,
            vertex_buffer,
            index_buffer,
            vertex_capacity: 4096,
            index_capacity: 4096,
            _format: format,
            viewport_size: (0, 0),
            texture_atlas,
            last_index_count: 0,
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
