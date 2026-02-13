//! Atlas lookup with fallback resolution for WoW-style atlas names.
//!
//! Wraps the auto-generated atlas data and adds resolution logic for
//! size-suffixed entries (e.g. "coin-copper" → "coin-copper-20x20").

pub use crate::atlas_data::{AtlasInfo, AtlasLookup, ATLAS_DB};
pub use crate::atlas_elements::get_atlas_name_by_element_id;

/// A single piece of a nine-slice atlas kit.
#[derive(Debug, Clone)]
pub struct NineSlicePiece {
    /// Texture file path (WoW-style).
    pub file: &'static str,
    /// UV coordinates (left, right, top, bottom).
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
    /// Piece dimensions in pixels.
    pub width: u32,
    pub height: u32,
}

/// Nine-slice atlas kit: 4 corners + 4 tiling edges + optional center.
#[derive(Debug, Clone)]
pub struct NineSliceAtlasInfo {
    pub corner_tl: NineSlicePiece,
    pub corner_tr: NineSlicePiece,
    pub corner_bl: NineSlicePiece,
    pub corner_br: NineSlicePiece,
    pub edge_top: NineSlicePiece,
    pub edge_bottom: NineSlicePiece,
    pub edge_left: NineSlicePiece,
    pub edge_right: NineSlicePiece,
    pub center: Option<NineSlicePiece>,
}

/// Check if an atlas name is a nine-slice kit prefix and return all pieces.
///
/// Detection: if `{lowercase(name)}-nineslice-cornertopleft` exists in ATLAS_DB,
/// this is a nine-slice kit. Returns `None` if any required piece is missing.
pub fn get_nine_slice_atlas_info(name: &str) -> Option<NineSliceAtlasInfo> {
    let kit = name.to_lowercase();
    let probe = format!("{kit}-nineslice-cornertopleft");
    ATLAS_DB.get(&probe as &str)?;

    let piece = |key: &str| -> Option<NineSlicePiece> {
        ATLAS_DB.get(key).map(|info| NineSlicePiece {
            file: info.file,
            left: info.left_tex_coord,
            right: info.right_tex_coord,
            top: info.top_tex_coord,
            bottom: info.bottom_tex_coord,
            width: info.width,
            height: info.height,
        })
    };

    Some(NineSliceAtlasInfo {
        corner_tl: piece(&format!("{kit}-nineslice-cornertopleft"))?,
        corner_tr: piece(&format!("{kit}-nineslice-cornertopright"))?,
        corner_bl: piece(&format!("{kit}-nineslice-cornerbottomleft"))?,
        corner_br: piece(&format!("{kit}-nineslice-cornerbottomright"))?,
        edge_top: piece(&format!("_{kit}-nineslice-edgetop"))?,
        edge_bottom: piece(&format!("_{kit}-nineslice-edgebottom"))?,
        edge_left: piece(&format!("!{kit}-nineslice-edgeleft"))?,
        edge_right: piece(&format!("!{kit}-nineslice-edgeright"))?,
        center: piece(&format!("{kit}-nineslice-center")),
    })
}

/// Common square sizes used in WoW's size-suffixed atlas entries.
const SIZE_SUFFIXES: &[u32] = &[16, 20, 32, 48, 64];

/// Get atlas info by name (case-insensitive).
///
/// Resolution order:
/// 1. Exact match, then `-2x` / strip `-2x` (from generated lookup)
/// 2. With `-NxN` size suffix (e.g. `coin-copper` → `coin-copper-20x20`)
/// 3. With `_1x` / `_2x` underscore suffix (e.g. `Unit_Evoker_EbonMight_EndCap` → `_2x`)
pub fn get_atlas_info(name: &str) -> Option<AtlasLookup> {
    if let Some(lookup) = crate::atlas_data::get_atlas_info(name) {
        return Some(lookup);
    }

    let lower = name.to_lowercase();

    // Try with -NxN size suffixes
    for &size in SIZE_SUFFIXES {
        let suffixed = format!("{lower}-{size}x{size}");
        if let Some(info) = ATLAS_DB.get(&suffixed as &str) {
            return Some(AtlasLookup { info, is_2x_fallback: false });
        }
    }

    // Try with _2x then _1x underscore suffixes
    let with_2x = format!("{lower}_2x");
    if let Some(info) = ATLAS_DB.get(&with_2x as &str) {
        return Some(AtlasLookup { info, is_2x_fallback: true });
    }
    let with_1x = format!("{lower}_1x");
    if let Some(info) = ATLAS_DB.get(&with_1x as &str) {
        return Some(AtlasLookup { info, is_2x_fallback: false });
    }

    // Blizzard typo corrections (divider→devider in atlas DB)
    try_spelling_corrections(&lower)
}

/// Atlas DB has some Blizzard typos. Try known corrections.
fn try_spelling_corrections(lower: &str) -> Option<AtlasLookup> {
    let corrected = lower.replace("divider", "devider");
    if corrected != *lower {
        return crate::atlas_data::get_atlas_info(&corrected);
    }
    None
}
