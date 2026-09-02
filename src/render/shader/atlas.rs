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

use super::atlas_bc::{BcAtlasTier, build_bc_entry, init_bc_atlases, write_bc_slot};
use super::atlas_bind_groups::{
    create_atlas_bind_groups, create_glyph_atlas, create_texture_sampler,
};

pub use super::atlas_bc::{BC_CELL_SIZE, BcFormat, BcTextureEntry, is_bc_supported};

#[cfg(test)]
pub(crate) use super::atlas_bc::set_bc_supported_for_tests;

/// Cell sizes for each tier.
pub const TIER_SIZES: [u32; 5] = [64, 128, 256, 512, 2048];

/// Number of tiers.
pub const NUM_TIERS: usize = 5;

/// Size of each tier's atlas texture.
const ATLAS_SIZE: u32 = 4096;
const GLYPH_ATLAS_SIZE: u32 = 2048;

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
    /// True when every pixel's alpha is 255.
    ///
    /// WoW ships mask textures in two encodings and the file name does not say
    /// which: some carry coverage in RGB with a uniformly opaque alpha, others
    /// carry it in alpha with black RGB. Picking the wrong rule multiplies the
    /// masked texture to zero everywhere. This flag is the discriminator, and
    /// it is a property of the pixels rather than of the path.
    pub alpha_uniformly_opaque: bool,
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

/// Whether every pixel of an RGBA buffer is fully opaque.
///
/// This is the discriminator between WoW's two mask encodings, which the file
/// name does not reveal. A mask whose alpha is uniformly 255 carries its
/// coverage in RGB (white shows, black hides); anything else carries coverage
/// in alpha, and its RGB is frequently solid black. Applying the RGB rule to an
/// alpha-coverage mask multiplies the masked texture by zero everywhere, which
/// is why unit-frame portraits and status-bar fills rendered as empty holes.
pub fn alpha_is_uniformly_opaque(rgba_data: &[u8]) -> bool {
    rgba_data.chunks_exact(4).all(|px| px[3] == 255)
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

/// Texture index used for BC1 (DXT1) compressed textures.
pub const BC1_ATLAS_TEX_INDEX: i32 = 6;

/// Texture index used for BC3 (DXT3/DXT5) compressed textures.
pub const BC3_ATLAS_TEX_INDEX: i32 = 7;

/// GPU texture atlas with multiple size tiers.
pub struct GpuTextureAtlas {
    /// 2D texture atlases for each tier.
    tiers: [TierAtlas; NUM_TIERS],
    /// Glyph atlas texture for text rendering.
    glyph_texture: wgpu::Texture,
    glyph_atlas_size: u32,
    /// BC1 (DXT1) compressed texture atlas (or placeholder if no BC support).
    bc1_atlas: BcAtlasTier,
    /// BC3 (DXT3/DXT5) compressed texture atlas (or placeholder if no BC support).
    bc3_atlas: BcAtlasTier,
    /// Whether the GPU supports BC texture compression.
    has_bc_support: bool,
    /// Bind group for shader access.
    bind_group: wgpu::BindGroup,
    /// Bind group layout.
    bind_group_layout: wgpu::BindGroupLayout,
    /// Map from texture path to RGBA entry.
    texture_map: HashMap<String, TextureEntry>,
    /// Map from texture path to BC-compressed entry.
    bc_texture_map: HashMap<String, BcTextureEntry>,
}

struct AtlasBacking {
    glyph_texture: wgpu::Texture,
    bc1_atlas: BcAtlasTier,
    bc3_atlas: BcAtlasTier,
    has_bc_support: bool,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl GpuTextureAtlas {
    /// Create a new tiered GPU texture atlas.
    pub fn new(device: &wgpu::Device) -> Self {
        let tiers = create_tier_atlases(device);
        let backing = create_atlas_backing(device, &tiers);

        Self {
            tiers,
            glyph_texture: backing.glyph_texture,
            glyph_atlas_size: GLYPH_ATLAS_SIZE,
            bc1_atlas: backing.bc1_atlas,
            bc3_atlas: backing.bc3_atlas,
            has_bc_support: backing.has_bc_support,
            bind_group: backing.bind_group,
            bind_group_layout: backing.bind_group_layout,
            texture_map: HashMap::new(),
            bc_texture_map: HashMap::new(),
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
            AtlasCell {
                tier: &self.tiers[tier_idx],
                grid_x,
                grid_y,
                cell_size,
            },
            TextureUploadSource {
                width,
                height,
                rgba_data,
            },
        );

        let alpha_uniformly_opaque = alpha_is_uniformly_opaque(rgba_data);

        let entry = compute_texture_entry(
            &self.tiers[tier_idx],
            tier_idx,
            grid_x,
            grid_y,
            width,
            height,
            cell_size,
            alpha_uniformly_opaque,
        );

        self.texture_map.insert(path.to_string(), entry);
        Some(entry)
    }

    /// Check if a BC-compressed texture is already in the atlas.
    pub fn get_bc(&self, path: &str) -> Option<&BcTextureEntry> {
        self.bc_texture_map.get(path)
    }

    /// Upload a BC-compressed texture directly to the BC atlas.
    ///
    /// The `bc_data` must be raw DXT block data for mip level 0.
    /// Returns `None` if BC compression is unsupported, atlas is full, or dimensions are invalid.
    pub fn upload_bc(
        &mut self,
        queue: &wgpu::Queue,
        path: &str,
        width: u32,
        height: u32,
        bc_data: &[u8],
        format: BcFormat,
    ) -> Option<BcTextureEntry> {
        if !self.has_bc_support {
            return None;
        }
        if let Some(entry) = self.cached_bc_entry(path) {
            return Some(*entry);
        }
        let atlas = self.bc_atlas_mut(format);
        let entry = upload_new_bc_texture(queue, atlas, width, height, bc_data, format)?;
        self.bc_texture_map.insert(path.to_string(), entry);
        Some(entry)
    }

    /// Prepare texture data to fit the cell size.
    fn prepare_texture_data_static(
        width: u32,
        height: u32,
        rgba_data: &[u8],
        cell_size: u32,
    ) -> Vec<u8> {
        // If texture fits, pad with replicated edge pixels so bilinear
        // filtering at the texture's right / bottom edges blends against the
        // same colour instead of zero-bleeding. This matters for narrow
        // atlas slots (e.g. 1×N tiling strips) where bilinear sampling can
        // otherwise mix the real pixel with transparent-black padding and
        // produce a visibly darker interior.
        if width <= cell_size && height <= cell_size {
            return pad_texture_replicate(width, height, rgba_data, cell_size);
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

    fn cached_bc_entry(&self, path: &str) -> Option<&BcTextureEntry> {
        self.bc_texture_map.get(path)
    }

    fn bc_atlas_mut(&mut self, format: BcFormat) -> &mut BcAtlasTier {
        match format {
            BcFormat::Bc1 => &mut self.bc1_atlas,
            BcFormat::Bc3 => &mut self.bc3_atlas,
        }
    }

    /// Clear the atlas (for reload).
    pub fn clear(&mut self) {
        self.texture_map.clear();
        self.bc_texture_map.clear();
        for tier in &mut self.tiers {
            tier.next_slot = 0;
        }
        self.bc1_atlas.reset();
        self.bc3_atlas.reset();
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

fn create_tier_atlases(device: &wgpu::Device) -> [TierAtlas; NUM_TIERS] {
    std::array::from_fn(|i| TierAtlas::new(device, TIER_SIZES[i], i))
}

fn upload_new_bc_texture(
    queue: &wgpu::Queue,
    atlas: &mut BcAtlasTier,
    width: u32,
    height: u32,
    bc_data: &[u8],
    format: BcFormat,
) -> Option<BcTextureEntry> {
    let (grid_x, grid_y) = atlas.allocate_slot()?;
    write_bc_slot(queue, atlas, grid_x, grid_y, width, height, bc_data, format);
    Some(build_bc_entry(atlas, format, grid_x, grid_y, width, height))
}

fn create_atlas_backing(device: &wgpu::Device, tiers: &[TierAtlas; NUM_TIERS]) -> AtlasBacking {
    let (glyph_texture, glyph_view) = create_glyph_atlas(device, GLYPH_ATLAS_SIZE);
    let (bc1_atlas, bc3_atlas, has_bc_support) = init_bc_atlases(device);
    let sampler = create_texture_sampler(device);
    let tier_views = [
        &tiers[0].view,
        &tiers[1].view,
        &tiers[2].view,
        &tiers[3].view,
        &tiers[4].view,
    ];
    let (bind_group_layout, bind_group) = create_atlas_bind_groups(
        device,
        tier_views,
        &glyph_view,
        &bc1_atlas.view,
        &bc3_atlas.view,
        &sampler,
    );
    AtlasBacking {
        glyph_texture,
        bc1_atlas,
        bc3_atlas,
        has_bc_support,
        bind_group,
        bind_group_layout,
    }
}

/// Memory usage statistics for the atlas.
#[derive(Debug, Default)]
pub struct TierStats {
    pub allocated_bytes: usize,
    pub used_bytes: usize,
    pub used_slots: [usize; NUM_TIERS],
}

/// Copy a `width×height` RGBA texture into a `cell_size×cell_size` slot,
/// replicating the right and bottom edge pixels into the remaining padding.
///
/// Zero-padding narrow textures (e.g. a `1×42` tiling strip placed in a `64×64`
/// slot) causes bilinear sampling to blend the real pixels with transparent
/// black at the slot edges, which shows up as a visibly darker interior on
/// three-slice tabs. Replicating the edge pixels keeps bilinear taps on the
/// same colour they already sample from inside the texture.
fn pad_texture_replicate(width: u32, height: u32, rgba_data: &[u8], cell_size: u32) -> Vec<u8> {
    let mut padded = vec![0u8; (cell_size * cell_size * 4) as usize];
    if width == 0 || height == 0 {
        return padded;
    }
    let src_row_bytes = (width * 4) as usize;
    let dst_row_bytes = (cell_size * 4) as usize;

    for y in 0..height {
        let src_offset = (y * width * 4) as usize;
        let dst_offset = (y * cell_size * 4) as usize;
        if src_offset + src_row_bytes > rgba_data.len() {
            continue;
        }
        padded[dst_offset..dst_offset + src_row_bytes]
            .copy_from_slice(&rgba_data[src_offset..src_offset + src_row_bytes]);
        let last_pixel_offset = src_offset + src_row_bytes - 4;
        let last_pixel: [u8; 4] = rgba_data[last_pixel_offset..last_pixel_offset + 4]
            .try_into()
            .unwrap_or([0; 4]);
        for x in width..cell_size {
            let px_offset = dst_offset + (x * 4) as usize;
            padded[px_offset..px_offset + 4].copy_from_slice(&last_pixel);
        }
    }

    let last_row_offset = ((height - 1) * cell_size * 4) as usize;
    let (head, tail) = padded.split_at_mut(last_row_offset + dst_row_bytes);
    let last_row = &head[last_row_offset..last_row_offset + dst_row_bytes];
    for row_chunk in tail.chunks_exact_mut(dst_row_bytes) {
        row_chunk.copy_from_slice(last_row);
    }

    padded
}

/// Upload texture data to a specific cell in a tier atlas.
struct AtlasCell<'a> {
    tier: &'a TierAtlas,
    grid_x: u32,
    grid_y: u32,
    cell_size: u32,
}

struct TextureUploadSource<'a> {
    width: u32,
    height: u32,
    rgba_data: &'a [u8],
}

fn upload_cell_to_gpu(queue: &wgpu::Queue, cell: AtlasCell<'_>, source: TextureUploadSource<'_>) {
    let data = GpuTextureAtlas::prepare_texture_data_static(
        source.width,
        source.height,
        source.rgba_data,
        cell.cell_size,
    );
    let (pixel_x, pixel_y) = cell.tier.pixel_offset(cell.grid_x, cell.grid_y);

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &cell.tier.texture,
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
            bytes_per_row: Some(cell.cell_size * 4),
            rows_per_image: Some(cell.cell_size),
        },
        wgpu::Extent3d {
            width: cell.cell_size,
            height: cell.cell_size,
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
    alpha_uniformly_opaque: bool,
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
        alpha_uniformly_opaque,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_device() -> (wgpu::Device, wgpu::Queue) {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .expect("test adapter should exist");
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("test device should be created")
    }

    fn create_test_bc_atlas(device: &wgpu::Device) -> BcAtlasTier {
        let had_bc = set_bc_supported_for_tests(true);
        let (mut bc1_atlas, _bc3_atlas, _has_bc_support) = init_bc_atlases(device);
        set_bc_supported_for_tests(had_bc);
        bc1_atlas.reset();
        bc1_atlas
    }

    #[test]
    fn upload_new_bc_texture_returns_none_when_atlas_is_full() {
        let (device, queue) = create_test_device();
        let mut atlas = create_test_bc_atlas(&device);
        while atlas.allocate_slot().is_some() {}

        let entry = upload_new_bc_texture(&queue, &mut atlas, 256, 256, &[], BcFormat::Bc1);

        assert!(entry.is_none(), "full atlas should reject a new BC upload");
    }

    #[test]
    fn create_tier_atlases_uses_configured_sizes_and_empty_slots() {
        let (device, _queue) = create_test_device();
        let tiers = create_tier_atlases(&device);

        for (index, tier) in tiers.iter().enumerate() {
            assert_eq!(tier.cell_size, TIER_SIZES[index]);
            assert_eq!(tier.grid_size, ATLAS_SIZE / TIER_SIZES[index]);
            assert_eq!(tier.next_slot, 0);
        }
    }

    #[test]
    fn cached_bc_entry_returns_copied_entry() {
        let (device, _queue) = create_test_device();
        let mut atlas = GpuTextureAtlas::new(&device);
        let expected = BcTextureEntry {
            format: BcFormat::Bc1,
            grid_x: 1,
            grid_y: 2,
            original_width: 256,
            original_height: 256,
            uv_x: 0.0,
            uv_y: 0.0,
            uv_width: 0.25,
            uv_height: 0.25,
        };
        atlas.bc_texture_map.insert("foo".to_string(), expected);

        assert_eq!(
            atlas
                .cached_bc_entry("foo")
                .copied()
                .map(|entry| entry.grid_x),
            Some(1)
        );
    }

    #[test]
    fn pad_texture_replicate_fills_right_and_bottom_with_edge_pixels() {
        let src = [
            0xAA, 0xBB, 0xCC, 0xFF, // row 0
            0x10, 0x20, 0x30, 0xFF, // row 1
        ];
        let padded = pad_texture_replicate(1, 2, &src, 4);
        assert_eq!(padded.len(), 4 * 4 * 4);

        for x in 0..4 {
            let off = x * 4;
            assert_eq!(
                &padded[off..off + 4],
                &[0xAA, 0xBB, 0xCC, 0xFF],
                "row 0 x={x}"
            );
        }

        for x in 0..4 {
            let off = 4 * 4 + x * 4;
            assert_eq!(
                &padded[off..off + 4],
                &[0x10, 0x20, 0x30, 0xFF],
                "row 1 x={x}"
            );
        }

        for y in 2..4 {
            for x in 0..4 {
                let off = y * 4 * 4 + x * 4;
                assert_eq!(
                    &padded[off..off + 4],
                    &[0x10, 0x20, 0x30, 0xFF],
                    "row {y} x={x} should replicate last real row"
                );
            }
        }
    }

    #[test]
    fn pad_texture_replicate_zero_dimension_returns_zero_buffer() {
        let padded = pad_texture_replicate(0, 2, &[], 4);
        assert_eq!(padded.len(), 4 * 4 * 4);
        assert!(padded.iter().all(|&b| b == 0));
    }

    /// The two mask encodings WoW actually ships, taken from decoded BLPs.
    ///
    /// `uiunitframeplayerportraitmask` is black RGB with opaque alpha inside
    /// the circle and zero alpha outside; `uiminimapmask` is white-on-black RGB
    /// with alpha 255 everywhere. Reading the file name cannot tell them apart,
    /// and applying the RGB rule to the first one erases the portrait entirely.
    #[test]
    fn mask_coverage_encoding_is_decided_by_alpha_not_by_name() {
        // Alpha-coverage shape: visible region is black but fully opaque.
        let alpha_coverage: Vec<u8> = [
            [0u8, 0, 0, 255], // inside the circle
            [0, 0, 0, 255],
            [255, 255, 255, 0], // outside
            [0, 0, 0, 0],
        ]
        .concat();
        assert!(
            !alpha_is_uniformly_opaque(&alpha_coverage),
            "a mask with any transparent pixel carries coverage in alpha"
        );

        // RGB-intensity shape: coverage is the colour, alpha is uniformly 255.
        let rgb_coverage: Vec<u8> = [
            [255u8, 255, 255, 255], // shows
            [0, 0, 0, 255],         // hides
            [128, 128, 128, 255],   // partial
            [255, 255, 255, 255],
        ]
        .concat();
        assert!(
            alpha_is_uniformly_opaque(&rgb_coverage),
            "a mask whose alpha is uniformly 255 carries coverage in RGB"
        );

        // A single non-opaque pixel is enough to switch encodings.
        let mut nearly_opaque = rgb_coverage.clone();
        nearly_opaque[7] = 254;
        assert!(!alpha_is_uniformly_opaque(&nearly_opaque));

        assert!(
            alpha_is_uniformly_opaque(&[]),
            "an empty buffer is vacuously opaque"
        );
    }
}
