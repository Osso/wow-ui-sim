//! Atlas lookup with fallback resolution for WoW-style atlas names.
//!
//! Wraps the auto-generated atlas data and adds resolution logic for
//! size-suffixed entries (e.g. "coin-copper" → "coin-copper-20x20").

pub use crate::atlas_data::{AtlasInfo, AtlasLookup, ATLAS_DB};

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
