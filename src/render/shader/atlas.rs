//! GPU texture atlas with size tiers and 2D grid packing.
//!
//! Uses multiple 2D textures at different cell sizes:
//! - Tier 0: 64x64 cells (icons, small UI elements)
//! - Tier 1: 128x128 cells (buttons, medium elements)
//! - Tier 2: 256x256 cells (panels, frames)
//! - Tier 3: 512x512 cells (large textures, backgrounds)
//! - Tier 4: 2048x2048 cells (full atlas textures like talents.blp)
//!
//! Each tier is a large 2D texture (ATLAS_SIZE x ATLAS_SIZE) with textures
//! packed in a grid. UV coordinates select the correct sub-region.
//! This avoids WGSL's "dynamically uniform" requirement for texture array indices.

use std::collections::HashMap;

/// Cell sizes for each tier.
pub const TIER_SIZES: [u32; 5] = [64, 128, 256, 512, 2048];

/// Number of tiers.
pub const NUM_TIERS: usize = 5;

/// Size of each tier's atlas texture.
const ATLAS_SIZE: u32 = 4096;

/// Entry for a texture in the atlas.
#[derive(Debug, Clone, Copy)]
pub struct TextureEntry {
    /// Tier index (0-4).
    pub tier: u32,
    /// Grid position X within the tier atlas.
    pub grid_x: u32,
    /// Grid position Y within the tier atlas.
    pub grid_y: u32,
    /// Original texture dimensions.
    pub original_width: u32,
    pub original_height: u32,
    /// UV rectangle within the atlas (pre-computed for the grid cell).
    pub uv_x: f32,
    pub uv_y: f32,
    pub uv_width: f32,
    pub uv_height: f32,
}

impl TextureEntry {
    /// Get the tier index for the shader.
    pub fn tex_index(&self) -> i32 {
        self.tier as i32
    }

    /// Get UV rectangle for the shader.
    pub fn uv_rect(&self) -> iced::Rectangle {
        iced::Rectangle::new(
            iced::Point::new(self.uv_x, self.uv_y),
            iced::Size::new(self.uv_width, self.uv_height),
        )
    }
}

/// A single tier's 2D texture atlas.
struct TierAtlas {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    cell_size: u32,
    /// Grid dimensions (how many cells fit in each direction).
    grid_size: u32,
    /// Next available grid position (linear index).
    next_slot: u32,
}

impl TierAtlas {
    fn new(device: &wgpu::Device, cell_size: u32, tier_index: usize) -> Self {
        let grid_size = ATLAS_SIZE / cell_size;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!(
                "WoW UI Tier {} Atlas ({}x{} cells)",
                tier_index, cell_size, cell_size
            )),
            size: wgpu::Extent3d {
                width: ATLAS_SIZE,
                height: ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("WoW UI Tier {} Atlas View", tier_index)),
            dimension: Some(wgpu::TextureViewDimension::D2),
            ..Default::default()
        });

        Self {
            texture,
            view,
            cell_size,
            grid_size,
            next_slot: 0,
        }
    }

    fn is_full(&self) -> bool {
        self.next_slot >= self.grid_size * self.grid_size
    }

    fn allocate_slot(&mut self) -> Option<(u32, u32)> {
        if self.is_full() {
            return None;
        }
        let slot = self.next_slot;
        self.next_slot += 1;
        let grid_x = slot % self.grid_size;
        let grid_y = slot / self.grid_size;
        Some((grid_x, grid_y))
    }

    /// Get pixel offset for a grid position.
    fn pixel_offset(&self, grid_x: u32, grid_y: u32) -> (u32, u32) {
        (grid_x * self.cell_size, grid_y * self.cell_size)
    }

    /// Get UV offset for a grid position.
    fn uv_offset(&self, grid_x: u32, grid_y: u32) -> (f32, f32) {
        let cell_uv = self.cell_size as f32 / ATLAS_SIZE as f32;
        (grid_x as f32 * cell_uv, grid_y as f32 * cell_uv)
    }
}

/// Texture index used for glyph atlas quads.
pub const GLYPH_ATLAS_TEX_INDEX: i32 = 5;

/// GPU texture atlas with multiple size tiers.
pub struct GpuTextureAtlas {
    /// 2D texture atlases for each tier.
    tiers: [TierAtlas; NUM_TIERS],
    /// Glyph atlas texture for text rendering.
    glyph_texture: wgpu::Texture,
    #[allow(dead_code)] // Kept alive for bind group reference
    glyph_view: wgpu::TextureView,
    glyph_atlas_size: u32,
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
        let tiers = std::array::from_fn(|i| TierAtlas::new(device, TIER_SIZES[i], i));

        let glyph_atlas_size = 2048;
        let (glyph_texture, glyph_view) = create_glyph_atlas(device, glyph_atlas_size);

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

        let (bind_group_layout, bind_group) =
            create_atlas_bind_groups(device, &tiers, &glyph_view, &sampler);

        Self {
            tiers,
            glyph_texture,
            glyph_view,
            glyph_atlas_size,
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
        (0..NUM_TIERS).rev().find(|&i| !self.tiers[i].is_full())
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
        if let Some(entry) = self.texture_map.get(path) {
            return Some(*entry);
        }

        let tier_idx = self.select_tier(width, height)?;
        let cell_size = self.tiers[tier_idx].cell_size;
        let (grid_x, grid_y) = self.tiers[tier_idx].allocate_slot()?;

        upload_cell_to_gpu(
            queue,
            &self.tiers[tier_idx],
            grid_x,
            grid_y,
            width,
            height,
            rgba_data,
            cell_size,
        );

        let entry = compute_texture_entry(
            &self.tiers[tier_idx],
            tier_idx,
            grid_x,
            grid_y,
            width,
            height,
            cell_size,
        );

        self.texture_map.insert(path.to_string(), entry);
        Some(entry)
    }

    /// Prepare texture data to fit the cell size.
    fn prepare_texture_data_static(
        width: u32,
        height: u32,
        rgba_data: &[u8],
        cell_size: u32,
    ) -> Vec<u8> {
        // If texture fits, pad with zeros
        if width <= cell_size && height <= cell_size {
            let mut padded = vec![0u8; (cell_size * cell_size * 4) as usize];
            for y in 0..height {
                let src_offset = (y * width * 4) as usize;
                let dst_offset = (y * cell_size * 4) as usize;
                let row_bytes = (width * 4) as usize;
                if src_offset + row_bytes <= rgba_data.len() {
                    padded[dst_offset..dst_offset + row_bytes]
                        .copy_from_slice(&rgba_data[src_offset..src_offset + row_bytes]);
                }
            }
            return padded;
        }

        // Scale down to fit
        let mut scaled = vec![0u8; (cell_size * cell_size * 4) as usize];
        let x_ratio = width as f32 / cell_size as f32;
        let y_ratio = height as f32 / cell_size as f32;

        for dst_y in 0..cell_size {
            for dst_x in 0..cell_size {
                let src_x = ((dst_x as f32 * x_ratio) as u32).min(width - 1);
                let src_y = ((dst_y as f32 * y_ratio) as u32).min(height - 1);
                let src_offset = ((src_y * width + src_x) * 4) as usize;
                let dst_offset = ((dst_y * cell_size + dst_x) * 4) as usize;
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
            tier.next_slot = 0;
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

    /// Upload glyph atlas RGBA data to the GPU.
    ///
    /// The data must be exactly `size * size * 4` bytes of RGBA.
    pub fn upload_glyph_atlas(&self, queue: &wgpu::Queue, rgba_data: &[u8], size: u32) {
        assert_eq!(size, self.glyph_atlas_size);
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.glyph_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(size * 4),
                rows_per_image: Some(size),
            },
            wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Get memory usage statistics (includes glyph atlas).
    pub fn memory_stats(&self) -> TierStats {
        let mut stats = TierStats::default();
        for (i, tier) in self.tiers.iter().enumerate() {
            let tier_bytes = (ATLAS_SIZE * ATLAS_SIZE * 4) as usize;
            stats.allocated_bytes += tier_bytes;
            stats.used_slots[i] = tier.next_slot as usize;
            let slot_bytes = (tier.cell_size * tier.cell_size * 4) as usize;
            stats.used_bytes += slot_bytes * tier.next_slot as usize;
        }
        // Glyph atlas
        let glyph_bytes = (self.glyph_atlas_size * self.glyph_atlas_size * 4) as usize;
        stats.allocated_bytes += glyph_bytes;
        stats
    }
}

/// Memory usage statistics for the atlas.
#[derive(Debug, Default)]
pub struct TierStats {
    pub allocated_bytes: usize,
    pub used_bytes: usize,
    pub used_slots: [usize; NUM_TIERS],
}

/// Create the glyph atlas texture and view.
fn create_glyph_atlas(
    device: &wgpu::Device,
    size: u32,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Glyph Atlas"),
        size: wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

/// Create bind group layout and bind group for tier textures, sampler, and glyph atlas.
fn create_atlas_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let texture_entry = |binding: u32| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("WoW UI Texture Bind Group Layout"),
        entries: &[
            texture_entry(0), // Tier 0 (64x64 cells)
            texture_entry(1), // Tier 1 (128x128 cells)
            texture_entry(2), // Tier 2 (256x256 cells)
            texture_entry(3), // Tier 3 (512x512 cells)
            texture_entry(4), // Tier 4 (2048x2048 cells)
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            texture_entry(6), // Glyph atlas
        ],
    })
}

fn create_atlas_bind_groups(
    device: &wgpu::Device,
    tiers: &[TierAtlas; NUM_TIERS],
    glyph_view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
    let layout = create_atlas_bind_group_layout(device);

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("WoW UI Texture Bind Group"),
        layout: &layout,
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
                resource: wgpu::BindingResource::TextureView(&tiers[4].view),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: wgpu::BindingResource::TextureView(glyph_view),
            },
        ],
    });

    (layout, bind_group)
}

/// Upload texture data to a specific cell in a tier atlas.
#[allow(clippy::too_many_arguments)]
fn upload_cell_to_gpu(
    queue: &wgpu::Queue,
    tier: &TierAtlas,
    grid_x: u32,
    grid_y: u32,
    width: u32,
    height: u32,
    rgba_data: &[u8],
    cell_size: u32,
) {
    let data = GpuTextureAtlas::prepare_texture_data_static(width, height, rgba_data, cell_size);
    let (pixel_x, pixel_y) = tier.pixel_offset(grid_x, grid_y);

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &tier.texture,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: pixel_x,
                y: pixel_y,
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        },
        &data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(cell_size * 4),
            rows_per_image: Some(cell_size),
        },
        wgpu::Extent3d {
            width: cell_size,
            height: cell_size,
            depth_or_array_layers: 1,
        },
    );
}

/// Compute the UV coordinates and TextureEntry for a newly uploaded texture.
fn compute_texture_entry(
    tier: &TierAtlas,
    tier_idx: usize,
    grid_x: u32,
    grid_y: u32,
    width: u32,
    height: u32,
    cell_size: u32,
) -> TextureEntry {
    let (uv_base_x, uv_base_y) = tier.uv_offset(grid_x, grid_y);
    let cell_uv_size = cell_size as f32 / ATLAS_SIZE as f32;

    let (uv_width, uv_height) = if width <= cell_size && height <= cell_size {
        (
            width as f32 / ATLAS_SIZE as f32,
            height as f32 / ATLAS_SIZE as f32,
        )
    } else {
        (cell_uv_size, cell_uv_size)
    };

    TextureEntry {
        tier: tier_idx as u32,
        grid_x,
        grid_y,
        original_width: width,
        original_height: height,
        uv_x: uv_base_x,
        uv_y: uv_base_y,
        uv_width,
        uv_height,
    }
}

impl std::fmt::Debug for GpuTextureAtlas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stats = self.memory_stats();
        f.debug_struct("GpuTextureAtlas")
            .field("texture_count", &self.texture_map.len())
            .field("tier_64_slots", &stats.used_slots[0])
            .field("tier_128_slots", &stats.used_slots[1])
            .field("tier_256_slots", &stats.used_slots[2])
            .field("tier_512_slots", &stats.used_slots[3])
            .field("tier_2048_slots", &stats.used_slots[4])
            .field("used_mb", &(stats.used_bytes / 1024 / 1024))
            .field("allocated_mb", &(stats.allocated_bytes / 1024 / 1024))
            .finish()
    }
}
