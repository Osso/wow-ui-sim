use image_blp::parser::{LoadError, load_blp, load_blp_from_buf};
use image_blp::types::{
    BlpContent, BlpContentTag, BlpFlags, BlpHeader, BlpImage, BlpVersion, Compression,
    MipmapLocator,
    direct::dxtn::{BlpDxtn, DxtnFormat, DxtnImage},
};
use std::path::Path;

const MAX_MIPS: usize = 16;
const MIP_OFFSETS: usize = 20;
const MIP_SIZES: usize = MIP_OFFSETS + MAX_MIPS * 4;
const DXT_DATA_START: usize = MIP_SIZES + MAX_MIPS * 4 + 1024; // includes the color map

pub(super) fn load(path: &Path) -> Result<BlpImage, LoadError> {
    let bytes = std::fs::read(path).map_err(|err| LoadError::FileSystem(path.to_owned(), err))?;
    if bytes.starts_with(b"BLP0") {
        // BLP0 can use mipmaps in separate files next to the texture.
        return load_blp(path);
    }
    match parse_partial_dxt(&bytes)? {
        Some(image) => Ok(image),
        None => load_blp_from_buf(&bytes),
    }
}

fn parse_partial_dxt(bytes: &[u8]) -> Result<Option<BlpImage>, LoadError> {
    if bytes.len() < DXT_DATA_START
        || !bytes.starts_with(b"BLP2")
        || read_word(bytes, 4) != 1 // direct content
        || bytes[8] != 2 // DXT compression
        || bytes[11] == 0
    {
        return Ok(None);
    }
    let format = match bytes[10] {
        0 => DxtnFormat::Dxt1,
        1 => DxtnFormat::Dxt3,
        7 => DxtnFormat::Dxt5,
        _ => return Ok(None),
    };
    let width = read_word(bytes, 12);
    let height = read_word(bytes, 16);
    if width == 0 || height == 0 {
        return Err(LoadError::Parsing("Invalid BLP2 DXT dimensions".into()));
    }

    let sizes: [u32; MAX_MIPS] =
        std::array::from_fn(|level| read_word(bytes, MIP_SIZES + 4 * level));
    let full_mip_count = (width.max(height).ilog2() as usize + 1).min(MAX_MIPS);
    let Some(mip_count) = sizes[..full_mip_count].iter().position(|&size| size == 0) else {
        return Ok(None);
    };
    // image-blp assumes a full DXT mip chain. A trailing zero size ends the chain;
    // some writers give these unused entries an offset at end of file.
    if mip_count == 0 || sizes[mip_count..].iter().any(|&size| size != 0) {
        return Err(LoadError::Parsing("Invalid BLP2 DXT mip chain".into()));
    }

    let offsets = std::array::from_fn(|level| read_word(bytes, MIP_OFFSETS + 4 * level));
    let header = BlpHeader {
        version: BlpVersion::Blp2,
        content: BlpContentTag::Direct,
        flags: BlpFlags::Blp2 {
            compression: Compression::Dxtc,
            alpha_bits: bytes[9],
            alpha_type: bytes[10],
            has_mipmaps: bytes[11],
        },
        width,
        height,
        mipmap_locator: MipmapLocator::Internal { offsets, sizes },
    };
    let mut images = Vec::with_capacity(mip_count);
    for level in 0..mip_count {
        let (width, height) = header.mipmap_size(level);
        let blocks = u64::from(width.div_ceil(4)) * u64::from(height.div_ceil(4));
        let offset = offsets[level] as usize;
        let size = sizes[level] as usize;
        let invalid_mip = || LoadError::Parsing(format!("Invalid BLP2 DXT data for mip {level}"));
        let expected_size = blocks
            .checked_mul(format.block_size() as u64)
            .filter(|&expected| expected <= u64::from(sizes[level]))
            .ok_or_else(invalid_mip)? as usize;
        let end = offset.checked_add(size).ok_or_else(invalid_mip)?;
        if offset < DXT_DATA_START || end > bytes.len() {
            return Err(invalid_mip());
        }
        images.push(DxtnImage {
            content: bytes[offset..offset + expected_size].to_vec(),
        });
    }
    let dxt = BlpDxtn {
        format,
        cmap: Vec::new(), // The DXT converter uses block data directly.
        images,
    };
    let content = match format {
        DxtnFormat::Dxt1 => BlpContent::Dxt1(dxt),
        DxtnFormat::Dxt3 => BlpContent::Dxt3(dxt),
        DxtnFormat::Dxt5 => BlpContent::Dxt5(dxt),
    };
    Ok(Some(BlpImage { header, content }))
}

fn read_word(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_dxt5_mips_preserve_base_texture() {
        // A 6×6 base image needs four blocks. The 3×3 mip needs one block.
        // The 1×1 mip and every unused entry have a zero size and EOF offset.
        let mut bytes = vec![0; DXT_DATA_START + 80];
        bytes[..12].copy_from_slice(b"BLP2\x01\0\0\0\x02\x08\x07\x01");
        bytes[12..20].copy_from_slice(&[6, 0, 0, 0, 6, 0, 0, 0]);
        let mut offsets = [bytes.len() as u32; MAX_MIPS];
        offsets[0] = DXT_DATA_START as u32;
        offsets[1] = (DXT_DATA_START + 64) as u32;
        for (level, offset) in offsets.into_iter().enumerate() {
            let slot = MIP_OFFSETS + 4 * level;
            bytes[slot..slot + 4].copy_from_slice(&offset.to_le_bytes());
        }
        bytes[MIP_SIZES] = 64;
        bytes[MIP_SIZES + 4] = 16;

        let image = parse_partial_dxt(&bytes).unwrap().unwrap();
        assert_eq!(image.image_count(), 2);
        let rgba = image_blp::convert::blp_to_image(&image, 0)
            .unwrap()
            .to_rgba8();
        assert_eq!(rgba.dimensions(), (6, 6));
        assert_eq!(rgba.as_raw(), &[0; 6 * 6 * 4]);
        let bc = crate::bc_texture_result(6, 6, image.content).unwrap();
        assert_eq!(bc.format, crate::BcTextureFormat::Bc3);
        assert_eq!(bc.bc_data.as_ref(), &[0; 64]);

        bytes.pop();
        assert!(parse_partial_dxt(&bytes).is_err());
    }
}
