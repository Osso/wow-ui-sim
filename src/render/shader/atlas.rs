//! GPU texture atlas with size tiers for efficient memory usage.
//!
//! Uses multiple texture arrays at different resolutions:
//! - Tier 0: 64x64 (icons, small UI elements)
//! - Tier 1: 128x128 (buttons, medium elements)
//! - Tier 2: 256x256 (panels, frames)
//! - Tier 3: 512x512 (large textures, backgrounds)
//!
//! Each texture is placed in the smallest tier that fits it.

use std::collections::HashMap;

/// Texture tiers with their sizes.
pub const TIER_SIZES: [u32; 4] = [64, 128, 256, 512];

/// Number of tiers.
pub const NUM_TIERS: usize = 4;

/// Maximum layers per tier.
const MAX_LAYERS_PER_TIER: u32 = 64;

/// Tier index constants for encoding in tex_index.
pub const TIER_64: u32 = 0;
pub const TIER_128: u32 = 1;
pub const TIER_256: u32 = 2;
pub const TIER_512: u32 = 3;

/// Encode tier and layer into a single tex_index.
/// Format: tier * 1000 + layer
pub fn encode_tex_index(tier: u32, layer: u32) -> i32 {
    (tier * 1000 + layer) as i32
}

/// Decode tex_index into (tier, layer).
pub fn decode_tex_index(tex_index: i32) -> (u32, u32) {
    let idx = tex_index as u32;
    (idx / 1000, idx % 1000)
}

/// Entry for a texture in the atlas.
#[derive(Debug, Clone, Copy)]
pub struct TextureEntry {
    /// Tier index (0-3).
    pub tier: u32,
    /// Layer index within the tier.
    pub layer: u32,
    /// Original texture dimensions.
    pub original_width: u32,
    pub original_height: u32,
    /// UV rectangle within the layer.
    pub uv_x: f32,
    pub uv_y: f32,
    pub uv_width: f32,
    pub uv_height: f32,
}

impl TextureEntry {
    /// Get the encoded texture index for the shader.
    pub fn tex_index(&self) -> i32 {
        encode_tex_index(self.tier, self.layer)
    }

    /// Get UV rectangle for the shader.
    pub fn uv_rect(&self) -> iced::Rectangle {
        iced::Rectangle::new(
            iced::Point::new(self.uv_x, self.uv_y),
            iced::Size::new(self.uv_width, self.uv_height),
        )
    }
}

/// A single tier's texture array.
struct TierArray {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    size: u32,
    next_layer: u32,
}

impl TierArray {
    fn new(device: &wgpu::Device, size: u32, tier_index: usize) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("WoW UI Texture Tier {} ({}x{})", tier_index, size, size)),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: MAX_LAYERS_PER_TIER,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("WoW UI Texture Tier {} View", tier_index)),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        Self {
            texture,
            view,
            size,
            next_layer: 0,
        }
    }

    fn is_full(&self) -> bool {
        self.next_layer >= MAX_LAYERS_PER_TIER
    }

    fn allocate_layer(&mut self) -> Option<u32> {
        if self.is_full() {
            return None;
        }
        let layer = self.next_layer;
        self.next_layer += 1;
        Some(layer)
    }
}

/// GPU texture atlas with multiple size tiers.
pub struct GpuTextureAtlas {
    /// Texture arrays for each tier.
    tiers: [TierArray; NUM_TIERS],
    /// Shared sampler.
    sampler: wgpu::Sampler,
    /// Bind group for shader access.
    bind_group: wgpu::BindGroup,
    /// Bind group layout.
    bind_group_layout: wgpu::BindGroupLayout,
    /// Map from texture path to entry.
    texture_map: HashMap<String, TextureEntry>,
}

impl GpuTextureAtlas {
    /// Create a new tiered GPU texture atlas.
    pub fn new(device: &wgpu::Device) -> Self {
        // Create tier arrays
        let tiers = std::array::from_fn(|i| TierArray::new(device, TIER_SIZES[i], i));

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
            label: Some("WoW UI Tiered Texture Bind Group Layout"),
            entries: &[
                // Tier 0 (64x64)
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
                // Tier 1 (128x128)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                // Tier 2 (256x256)
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                // Tier 3 (512x512)
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("WoW UI Tiered Texture Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&tiers[0].view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&tiers[1].view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&tiers[2].view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&tiers[3].view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            tiers,
            sampler,
            bind_group,
            bind_group_layout,
            texture_map: HashMap::new(),
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
        self.texture_map
            .get(path)
            .map(|e| e.tex_index())
            .unwrap_or(-1)
    }

    /// Select the best tier for a texture based on its dimensions.
    fn select_tier(&self, width: u32, height: u32) -> Option<usize> {
        let max_dim = width.max(height);
        for (i, &tier_size) in TIER_SIZES.iter().enumerate() {
            if max_dim <= tier_size && !self.tiers[i].is_full() {
                return Some(i);
            }
        }
        // If texture is larger than max tier or all appropriate tiers are full,
        // try to fit in the largest tier with scaling
        for i in (0..NUM_TIERS).rev() {
            if !self.tiers[i].is_full() {
                return Some(i);
            }
        }
        None
    }

    /// Upload a texture to the atlas, returning its entry.
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

        // Select tier
        let tier_idx = self.select_tier(width, height)?;
        let tier_size = self.tiers[tier_idx].size;

        // Allocate layer
        let layer = self.tiers[tier_idx].allocate_layer()?;

        // Prepare texture data (before borrowing tier for upload)
        let data = Self::prepare_texture_data_static(width, height, rgba_data, tier_size);

        // Upload to GPU
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.tiers[tier_idx].texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: layer,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(tier_size * 4),
                rows_per_image: Some(tier_size),
            },
            wgpu::Extent3d {
                width: tier_size,
                height: tier_size,
                depth_or_array_layers: 1,
            },
        );

        // Calculate UV coordinates
        let (uv_width, uv_height) = if width <= tier_size && height <= tier_size {
            (width as f32 / tier_size as f32, height as f32 / tier_size as f32)
        } else {
            (1.0, 1.0)
        };

        let entry = TextureEntry {
            tier: tier_idx as u32,
            layer,
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

    /// Prepare texture data to fit the tier size.
    fn prepare_texture_data_static(
        width: u32,
        height: u32,
        rgba_data: &[u8],
        tier_size: u32,
    ) -> Vec<u8> {
        // If texture fits, pad with zeros
        if width <= tier_size && height <= tier_size {
            let mut padded = vec![0u8; (tier_size * tier_size * 4) as usize];
            for y in 0..height {
                let src_offset = (y * width * 4) as usize;
                let dst_offset = (y * tier_size * 4) as usize;
                let row_bytes = (width * 4) as usize;
                if src_offset + row_bytes <= rgba_data.len() {
                    padded[dst_offset..dst_offset + row_bytes]
                        .copy_from_slice(&rgba_data[src_offset..src_offset + row_bytes]);
                }
            }
            return padded;
        }

        // Scale down to fit
        let mut scaled = vec![0u8; (tier_size * tier_size * 4) as usize];
        let x_ratio = width as f32 / tier_size as f32;
        let y_ratio = height as f32 / tier_size as f32;

        for dst_y in 0..tier_size {
            for dst_x in 0..tier_size {
                let src_x = ((dst_x as f32 * x_ratio) as u32).min(width - 1);
                let src_y = ((dst_y as f32 * y_ratio) as u32).min(height - 1);
                let src_offset = ((src_y * width + src_x) * 4) as usize;
                let dst_offset = ((dst_y * tier_size + dst_x) * 4) as usize;
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
        for tier in &mut self.tiers {
            tier.next_layer = 0;
        }
    }

    /// Number of textures in the atlas.
    pub fn len(&self) -> usize {
        self.texture_map.len()
    }

    /// Check if atlas is empty.
    pub fn is_empty(&self) -> bool {
        self.texture_map.is_empty()
    }

    /// Get memory usage statistics.
    pub fn memory_stats(&self) -> TierStats {
        let mut stats = TierStats::default();
        for (i, tier) in self.tiers.iter().enumerate() {
            let size = TIER_SIZES[i];
            let layer_bytes = (size * size * 4) as usize;
            let tier_bytes = layer_bytes * MAX_LAYERS_PER_TIER as usize;
            stats.allocated_bytes += tier_bytes;
            stats.used_layers[i] = tier.next_layer as usize;
            stats.used_bytes += layer_bytes * tier.next_layer as usize;
        }
        stats
    }
}

/// Memory usage statistics for the atlas.
#[derive(Debug, Default)]
pub struct TierStats {
    pub allocated_bytes: usize,
    pub used_bytes: usize,
    pub used_layers: [usize; NUM_TIERS],
}

impl std::fmt::Debug for GpuTextureAtlas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stats = self.memory_stats();
        f.debug_struct("GpuTextureAtlas")
            .field("texture_count", &self.texture_map.len())
            .field("tier_64_layers", &stats.used_layers[0])
            .field("tier_128_layers", &stats.used_layers[1])
            .field("tier_256_layers", &stats.used_layers[2])
            .field("tier_512_layers", &stats.used_layers[3])
            .field("used_mb", &(stats.used_bytes / 1024 / 1024))
            .field("allocated_mb", &(stats.allocated_bytes / 1024 / 1024))
            .finish()
    }
}
