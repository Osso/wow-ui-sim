//! CPU-side linear-light premultiplied mipmap generation for GPU atlas cells.

use std::sync::OnceLock;

/// Number of mip levels inside an atlas cell of `cell_size × cell_size` texels.
///
/// Stops at 1×1 per cell so neighboring atlas slots never merge into the same texel.
pub(super) const fn tier_mip_level_count(cell_size: u32) -> u32 {
    cell_size.trailing_zeros() + 1
}

/// Total RGBA8 byte size of a square texture and its `mip_level_count` mip chain.
pub(super) fn mip_chain_bytes(base_size: u32, mip_level_count: u32) -> usize {
    (0..mip_level_count)
        .map(|level| {
            let dim = (base_size >> level).max(1) as usize;
            dim * dim * 4
        })
        .sum()
}

/// Upload a padded `cell_size × cell_size` cell and all its per-cell mip levels.
pub(super) fn upload_cell_mip_chain(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    origin: (u32, u32),
    cell_size: u32,
    mip_level_count: u32,
    base_rgba: &[u8],
) {
    write_cell_mip(queue, texture, 0, origin, cell_size, base_rgba);
    for (offset, (mip_dim, mip_rgba)) in generate_mip_levels(base_rgba, cell_size, mip_level_count)
        .into_iter()
        .enumerate()
    {
        let level = offset as u32 + 1;
        let mip_origin = (origin.0 >> level, origin.1 >> level);
        write_cell_mip(queue, texture, level, mip_origin, mip_dim, &mip_rgba);
    }
}

fn write_cell_mip(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    mip_level: u32,
    origin: (u32, u32),
    dim: u32,
    rgba: &[u8],
) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level,
            origin: wgpu::Origin3d {
                x: origin.0,
                y: origin.1,
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        },
        rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(dim * 4),
            rows_per_image: Some(dim),
        },
        wgpu::Extent3d {
            width: dim,
            height: dim,
            depth_or_array_layers: 1,
        },
    );
}

fn srgb_to_linear_lut() -> &'static [f32; 256] {
    static LUT: OnceLock<[f32; 256]> = OnceLock::new();
    LUT.get_or_init(|| {
        std::array::from_fn(|i| {
            let c = i as f32 / 255.0;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        })
    })
}

fn linear_to_srgb_u8(linear: f32) -> u8 {
    let c = linear.clamp(0.0, 1.0);
    let encoded = if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0).round().clamp(0.0, 255.0) as u8
}

fn to_premultiplied_linear(base_rgba: &[u8], cell_size: u32) -> Vec<[f32; 4]> {
    let lut = srgb_to_linear_lut();
    let texels = (cell_size * cell_size) as usize;
    let mut linear = Vec::with_capacity(texels);
    for px in base_rgba.chunks_exact(4).take(texels) {
        let a = f32::from(px[3]) / 255.0;
        linear.push([
            lut[usize::from(px[0])] * a,
            lut[usize::from(px[1])] * a,
            lut[usize::from(px[2])] * a,
            a,
        ]);
    }
    linear.resize(texels, [0.0; 4]);
    linear
}

fn downsample_premultiplied_2x2(prev: &[[f32; 4]], prev_dim: u32) -> Vec<[f32; 4]> {
    let next_dim = (prev_dim / 2).max(1) as usize;
    let stride = prev_dim as usize;
    let mut next = Vec::with_capacity(next_dim * next_dim);
    for y in 0..next_dim {
        let row0 = (y * 2) * stride;
        let row1 = row0 + stride;
        for x in 0..next_dim {
            let col = x * 2;
            let p00 = prev[row0 + col];
            let p10 = prev[row0 + col + 1];
            let p01 = prev[row1 + col];
            let p11 = prev[row1 + col + 1];
            next.push([
                0.25 * (p00[0] + p10[0] + p01[0] + p11[0]),
                0.25 * (p00[1] + p10[1] + p01[1] + p11[1]),
                0.25 * (p00[2] + p10[2] + p01[2] + p11[2]),
                0.25 * (p00[3] + p10[3] + p01[3] + p11[3]),
            ]);
        }
    }
    next
}

fn encode_srgb_premultiplied(linear: &[[f32; 4]]) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(linear.len() * 4);
    for &[ra, ga, ba, a] in linear {
        let alpha = (a * 255.0).round() as u8;
        if alpha != 0 {
            encoded.extend_from_slice(&[
                linear_to_srgb_u8(ra),
                linear_to_srgb_u8(ga),
                linear_to_srgb_u8(ba),
                alpha,
            ]);
        } else {
            encoded.extend_from_slice(&[0, 0, 0, 0]);
        }
    }
    encoded
}

/// Generate mip levels `1..mip_level_count` for a `cell_size × cell_size` sRGB RGBA buffer.
///
/// Keep RGB premultiplied in the generated levels so both box and GPU filtering
/// weight colors by coverage. Level 0 retains the original straight-alpha upload.
pub(super) fn generate_mip_levels(
    base_rgba: &[u8],
    cell_size: u32,
    mip_level_count: u32,
) -> Vec<(u32, Vec<u8>)> {
    if mip_level_count <= 1 || cell_size < 2 {
        return Vec::new();
    }
    let mut prev = to_premultiplied_linear(base_rgba, cell_size);
    let mut prev_dim = cell_size;
    let mut levels = Vec::with_capacity(mip_level_count as usize - 1);
    for _ in 1..mip_level_count {
        if prev_dim < 2 {
            break;
        }
        let next = downsample_premultiplied_2x2(&prev, prev_dim);
        prev_dim /= 2;
        levels.push((prev_dim, encode_srgb_premultiplied(&next)));
        prev = next;
    }
    levels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_mip_level_counts_stop_at_one_texel_per_cell() {
        let counts: Vec<u32> = [64, 128, 256, 512, 2048]
            .into_iter()
            .map(tier_mip_level_count)
            .collect();
        assert_eq!(counts, vec![7, 8, 9, 10, 12]);
    }

    #[test]
    fn mipmap_rgb_stays_premultiplied_for_gpu_filtering() {
        let mut rgba = vec![0u8; 4 * 4 * 4];
        rgba[0..4].copy_from_slice(&[255, 255, 255, 255]);

        let mips = generate_mip_levels(&rgba, 4, tier_mip_level_count(4));
        assert_eq!(mips.len(), 2);
        let (dim1, mip1) = &mips[0];
        assert_eq!(*dim1, 2);
        assert_eq!(&mip1[0..4], &[137, 137, 137, 64]);
        assert_eq!(&mip1[4..8], &[0, 0, 0, 0]);
    }

    #[test]
    fn constant_srgb_colour_roundtrips_across_all_mips() {
        let texel = [51u8, 102, 153, 255];
        let rgba = texel.repeat(8 * 8);

        let mips = generate_mip_levels(&rgba, 8, tier_mip_level_count(8));
        assert_eq!(mips.len(), 3);
        for (level_offset, (dim, data)) in mips.iter().enumerate() {
            assert_eq!(*dim, 4 >> level_offset);
            for chunk in data.chunks_exact(4) {
                for ch in 0..4 {
                    assert!(
                        chunk[ch].abs_diff(texel[ch]) <= 1,
                        "mip {}: channel {ch} drifted to {:?}",
                        level_offset + 1,
                        chunk
                    );
                }
            }
        }
    }
}
