//! GPU texture atlas for efficient texture management.
//!
//! Uses a texture array approach where each texture gets its own layer.
//! This avoids complex packing algorithms while still allowing batched rendering.

use std::collections::HashMap;

/// Maximum number of textures in the atlas.
/// wgpu guarantees at least 256 texture array layers.
const MAX_TEXTURES: u32 = 256;

/// Default texture size (textures are resized to fit).
const DEFAULT_TEXTURE_SIZE: u32 = 512;

/// GPU texture atlas managing multiple textures as array layers.
pub struct GpuTextureAtlas {
    /// The texture array on GPU.
    texture: wgpu::Texture,
    /// Texture view for binding (stored for bind_group recreation).
    _view: wgpu::TextureView,
    /// Sampler for texture lookups (stored for bind_group recreation).
    _sampler: wgpu::Sampler,
    /// Bind group for shader access.
    bind_group: wgpu::BindGroup,
    /// Bind group layout (needed for pipeline creation).
    bind_group_layout: wgpu::BindGroupLayout,
    /// Map from texture path to (layer_index, uv_rect).
    /// UV rect stores (u, v, width, height) as normalized coordinates.
    texture_map: HashMap<String, TextureEntry>,
    /// Next available layer index.
    next_layer: u32,
    /// Texture array size.
    texture_size: u32,
}

/// Entry for a texture in the atlas.
#[derive(Debug, Clone, Copy)]
pub struct TextureEntry {
    /// Layer index in the texture array.
    pub layer: i32,
    /// Original texture dimensions (for UV calculation).
    pub original_width: u32,
    pub original_height: u32,
    /// UV rectangle within the layer (for sub-region textures).
    /// For full textures, this is (0, 0, 1, 1).
    pub uv_x: f32,
    pub uv_y: f32,
    pub uv_width: f32,
    pub uv_height: f32,
}

impl TextureEntry {
    /// Get the texture index for the shader.
    pub fn tex_index(&self) -> i32 {
        self.layer
    }

    /// Get UV rectangle for the shader.
    pub fn uv_rect(&self) -> iced::Rectangle {
        iced::Rectangle::new(
            iced::Point::new(self.uv_x, self.uv_y),
            iced::Size::new(self.uv_width, self.uv_height),
        )
    }
}

impl GpuTextureAtlas {
    /// Create a new GPU texture atlas.
    pub fn new(device: &wgpu::Device) -> Self {
        let texture_size = DEFAULT_TEXTURE_SIZE;

        // Create texture array
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("WoW UI Texture Atlas"),
            size: wgpu::Extent3d {
                width: texture_size,
                height: texture_size,
                depth_or_array_layers: MAX_TEXTURES,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,  // Linear, not sRGB (avoid double gamma)
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("WoW UI Texture Atlas View"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("WoW UI Texture Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("WoW UI Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("WoW UI Texture Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            texture,
            _view: view,
            _sampler: sampler,
            bind_group,
            bind_group_layout,
            texture_map: HashMap::new(),
            next_layer: 0,
            texture_size,
        }
    }

    /// Get the bind group layout for pipeline creation.
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    /// Get the bind group for rendering.
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    /// Check if a texture is already in the atlas.
    pub fn get(&self, path: &str) -> Option<&TextureEntry> {
        self.texture_map.get(path)
    }

    /// Get texture index by path, returning -1 if not found.
    pub fn get_index(&self, path: &str) -> i32 {
        self.texture_map.get(path).map(|e| e.layer).unwrap_or(-1)
    }

    /// Upload a texture to the atlas, returning its entry.
    ///
    /// If the texture already exists, returns the existing entry.
    /// If the atlas is full, returns None.
    pub fn upload(
        &mut self,
        queue: &wgpu::Queue,
        path: &str,
        width: u32,
        height: u32,
        rgba_data: &[u8],
    ) -> Option<TextureEntry> {
        // Check if already uploaded
        if let Some(entry) = self.texture_map.get(path) {
            return Some(*entry);
        }

        // Check if atlas is full
        if self.next_layer >= MAX_TEXTURES {
            tracing::warn!("Texture atlas full, cannot upload: {}", path);
            return None;
        }

        let layer = self.next_layer;
        self.next_layer += 1;

        // Resize texture data to fit atlas layer
        let resized = self.resize_texture(width, height, rgba_data);

        // Upload to GPU
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: layer,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &resized,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(self.texture_size * 4),
                rows_per_image: Some(self.texture_size),
            },
            wgpu::Extent3d {
                width: self.texture_size,
                height: self.texture_size,
                depth_or_array_layers: 1,
            },
        );

        // Calculate UV coordinates for the texture within the layer
        // If texture fits within atlas, use its proportional size
        // If texture was scaled down, it fills the entire layer (0-1)
        let (uv_width, uv_height) = if width <= self.texture_size && height <= self.texture_size {
            // Texture fits, use proportional UVs
            (width as f32 / self.texture_size as f32, height as f32 / self.texture_size as f32)
        } else {
            // Texture was scaled to fit, fills entire layer
            (1.0, 1.0)
        };

        let entry = TextureEntry {
            layer: layer as i32,
            original_width: width,
            original_height: height,
            uv_x: 0.0,
            uv_y: 0.0,
            uv_width,
            uv_height,
        };

        self.texture_map.insert(path.to_string(), entry);
        Some(entry)
    }

    /// Resize texture data to fit the atlas layer size.
    fn resize_texture(&self, width: u32, height: u32, rgba_data: &[u8]) -> Vec<u8> {
        let target_size = self.texture_size;

        // If texture fits exactly, just pad with zeros
        if width <= target_size && height <= target_size {
            let mut padded = vec![0u8; (target_size * target_size * 4) as usize];

            // Copy row by row
            for y in 0..height {
                let src_offset = (y * width * 4) as usize;
                let dst_offset = (y * target_size * 4) as usize;
                let row_bytes = (width * 4) as usize;

                if src_offset + row_bytes <= rgba_data.len() {
                    padded[dst_offset..dst_offset + row_bytes]
                        .copy_from_slice(&rgba_data[src_offset..src_offset + row_bytes]);
                }
            }

            return padded;
        }

        // Texture is larger than atlas layer - need to scale down
        // Use simple bilinear-ish sampling
        let mut scaled = vec![0u8; (target_size * target_size * 4) as usize];

        let x_ratio = width as f32 / target_size as f32;
        let y_ratio = height as f32 / target_size as f32;

        for dst_y in 0..target_size {
            for dst_x in 0..target_size {
                let src_x = ((dst_x as f32 * x_ratio) as u32).min(width - 1);
                let src_y = ((dst_y as f32 * y_ratio) as u32).min(height - 1);

                let src_offset = ((src_y * width + src_x) * 4) as usize;
                let dst_offset = ((dst_y * target_size + dst_x) * 4) as usize;

                if src_offset + 4 <= rgba_data.len() {
                    scaled[dst_offset..dst_offset + 4]
                        .copy_from_slice(&rgba_data[src_offset..src_offset + 4]);
                }
            }
        }

        scaled
    }

    /// Clear the atlas (for reload).
    pub fn clear(&mut self) {
        self.texture_map.clear();
        self.next_layer = 0;
    }

    /// Number of textures in the atlas.
    pub fn len(&self) -> usize {
        self.texture_map.len()
    }

    /// Check if atlas is empty.
    pub fn is_empty(&self) -> bool {
        self.texture_map.is_empty()
    }
}

impl std::fmt::Debug for GpuTextureAtlas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GpuTextureAtlas")
            .field("texture_count", &self.texture_map.len())
            .field("next_layer", &self.next_layer)
            .field("texture_size", &self.texture_size)
            .finish()
    }
}
