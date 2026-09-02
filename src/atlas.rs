//! Atlas lookup with fallback resolution for WoW-style atlas names.
//!
//! Wraps the auto-generated atlas data and adds resolution logic for
//! size-suffixed entries (e.g. "coin-copper" → "coin-copper-20x20").

pub use crate::atlas_data::{
    ATLAS_DB, AtlasInfo, AtlasLookup, AtlasSliceInfo, AtlasSliceMode, get_atlas_slice_info,
};
pub use crate::atlas_elements::get_atlas_name_by_element_id;

use std::sync::atomic::{AtomicBool, Ordering};

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
    ensure_nine_slice_kit_exists(&kit)?;

    Some(NineSliceAtlasInfo {
        corner_tl: nine_slice_piece(&format!("{kit}-nineslice-cornertopleft"))?,
        corner_tr: nine_slice_piece(&format!("{kit}-nineslice-cornertopright"))?,
        corner_bl: nine_slice_piece(&format!("{kit}-nineslice-cornerbottomleft"))?,
        corner_br: nine_slice_piece(&format!("{kit}-nineslice-cornerbottomright"))?,
        edge_top: nine_slice_piece(&format!("_{kit}-nineslice-edgetop"))?,
        edge_bottom: nine_slice_piece(&format!("_{kit}-nineslice-edgebottom"))?,
        edge_left: nine_slice_piece(&format!("!{kit}-nineslice-edgeleft"))?,
        edge_right: nine_slice_piece(&format!("!{kit}-nineslice-edgeright"))?,
        center: nine_slice_piece(&format!("{kit}-nineslice-center")),
    })
}

fn ensure_nine_slice_kit_exists(kit: &str) -> Option<()> {
    let probe = format!("{kit}-nineslice-cornertopleft");
    nine_slice_piece(&probe).map(|_| ())
}

fn nine_slice_piece(key: &str) -> Option<NineSlicePiece> {
    let (lookup, from_2x) = resolve_nine_slice_lookup(key)?;
    let (width, height) = logical_nine_slice_piece_size(lookup, from_2x);

    Some(NineSlicePiece {
        file: lookup.file,
        left: lookup.left_tex_coord,
        right: lookup.right_tex_coord,
        top: lookup.top_tex_coord,
        bottom: lookup.bottom_tex_coord,
        width,
        height,
    })
}

fn resolve_nine_slice_lookup(key: &str) -> Option<(&'static AtlasInfo, bool)> {
    for candidate in nine_slice_key_candidates(key) {
        let base_lookup = ATLAS_DB.get(candidate.as_str());
        if let Some(lookup) = paired_2x_variant(candidate.as_str()).or(base_lookup) {
            return Some((lookup, base_lookup.is_none()));
        }
    }
    None
}

fn nine_slice_key_candidates(key: &str) -> [String; 2] {
    [key.to_string(), key.replacen("-nineslice", "", 1)]
}

fn logical_nine_slice_piece_size(lookup: &AtlasInfo, from_2x: bool) -> (u32, u32) {
    if from_2x && !lookup.size_is_override {
        (
            (lookup.width as f32 / 2.0).round() as u32,
            (lookup.height as f32 / 2.0).round() as u32,
        )
    } else {
        (lookup.width, lookup.height)
    }
}

/// Common square sizes used in WoW's size-suffixed atlas entries.
const SIZE_SUFFIXES: &[u32] = &[16, 20, 32, 48, 64];
const RENDER_PREFERRED_2X_ATLASES: &[&str] = &["questlog-icon-ticksquare"];

fn exact_atlas_info(name: &str) -> Option<AtlasLookup> {
    crate::atlas_data::get_atlas_info(name)
}

fn paired_2x_variant(lower: &str) -> Option<&'static AtlasInfo> {
    if lower.ends_with("_1x")
        || lower.ends_with("-1x")
        || lower.ends_with("_2x")
        || lower.ends_with("-2x")
    {
        return None;
    }

    for sep in ["_", "-"] {
        let with_2x = format!("{lower}{sep}2x");
        if let Some(info) = ATLAS_DB.get(&with_2x as &str) {
            return Some(info);
        }
    }

    None
}

fn render_preferred_2x_variant(lower: &str, prefer_hires: bool) -> Option<&'static AtlasInfo> {
    if !prefer_hires && !RENDER_PREFERRED_2X_ATLASES.contains(&lower) {
        return None;
    }
    paired_2x_variant(lower)
}

/// Whether render lookups source texels from the paired `-2x` atlas entry
/// whenever one exists. The client draws its 2x art once a UI unit spans more
/// than one pixel (a 1440p client at uiScale 0.9 is 1.6875 px per unit); at
/// 1 px per unit the 1x art is the authored one.
static PREFER_HIRES_ATLASES: AtomicBool = AtomicBool::new(false);

pub fn set_prefer_hires_atlases(prefer: bool) {
    PREFER_HIRES_ATLASES.store(prefer, Ordering::Relaxed);
}

pub fn prefer_hires_atlases() -> bool {
    PREFER_HIRES_ATLASES.load(Ordering::Relaxed)
}

/// Get atlas info by name (case-insensitive).
///
/// Resolution order:
/// 1. Exact match
/// 2. With `-NxN` size suffix (e.g. `coin-copper` → `coin-copper-20x20`)
/// 3. With `_2x` / `-2x` / `_1x` / `-1x` suffixes (e.g. `bags-item-slot64` → `-2x`)
pub fn get_atlas_info(name: &str) -> Option<AtlasLookup> {
    let lower = name.to_lowercase();

    if let Some(lookup) = exact_atlas_info(name) {
        return Some(lookup);
    }

    // Try with -NxN size suffixes
    for &size in SIZE_SUFFIXES {
        let suffixed = format!("{lower}-{size}x{size}");
        if let Some(info) = ATLAS_DB.get(&suffixed as &str) {
            return Some(AtlasLookup {
                info,
                is_2x_fallback: false,
                logical_size: None,
            });
        }
    }

    // Try with _2x/_1x underscore and -2x/-1x hyphen suffixes
    for sep in ["_", "-"] {
        let with_2x = format!("{lower}{sep}2x");
        if let Some(info) = ATLAS_DB.get(&with_2x as &str) {
            return Some(AtlasLookup {
                info,
                is_2x_fallback: true,
                logical_size: None,
            });
        }
        let with_1x = format!("{lower}{sep}1x");
        if let Some(info) = ATLAS_DB.get(&with_1x as &str) {
            return Some(AtlasLookup {
                info,
                is_2x_fallback: false,
                logical_size: None,
            });
        }
    }

    // Blizzard typo corrections (divider→devider in atlas DB)
    try_spelling_corrections(&lower)
}

/// Get atlas info for rendering, preferring a paired 2x entry when one exists.
///
/// This keeps logical atlas dimensions unchanged while sourcing texels from
/// the higher-resolution atlas file.
pub fn get_render_atlas_info(name: &str) -> Option<AtlasLookup> {
    get_render_atlas_info_with(name, prefer_hires_atlases())
}

/// `get_render_atlas_info` with the 2x preference passed explicitly; the
/// public entry point reads the process-wide setting.
pub fn get_render_atlas_info_with(name: &str, prefer_hires: bool) -> Option<AtlasLookup> {
    let lower = name.to_lowercase();

    if let Some(base) = exact_atlas_info(name) {
        if let Some(info) = render_preferred_2x_variant(&lower, prefer_hires) {
            return Some(AtlasLookup {
                info,
                is_2x_fallback: true,
                logical_size: Some((base.width(), base.height())),
            });
        }
        return Some(base);
    }

    get_atlas_info(name)
}

/// Atlas DB has some Blizzard typos. Try known corrections.
fn try_spelling_corrections(lower: &str) -> Option<AtlasLookup> {
    let corrected = lower.replace("divider", "devider");
    if corrected != *lower {
        return crate::atlas_data::get_atlas_info(&corrected);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{
        ATLAS_DB, get_atlas_info, get_render_atlas_info, logical_nine_slice_piece_size,
        nine_slice_piece,
    };

    #[test]
    fn nine_slice_uses_2x_fallback_with_logical_sizes() {
        let ns_info = super::get_nine_slice_atlas_info("ui-frame-metal")
            .expect("metal nineslice should exist from 2x atlas fallback");
        let corner = ATLAS_DB
            .get("ui-frame-metal-cornertopleft-2x")
            .expect("metal corner +2x entry should exist");
        let edge_top = ATLAS_DB
            .get("_ui-frame-metal-edgetop-2x")
            .expect("metal edge top +2x entry should exist");

        assert_eq!(
            ns_info.corner_tl.width,
            (corner.width as f32 / 2.0).round() as u32
        );
        assert_eq!(
            ns_info.edge_top.width,
            (edge_top.width as f32 / 2.0).round() as u32
        );
        assert_eq!(
            ns_info.edge_top.height,
            (edge_top.height as f32 / 2.0).round() as u32
        );
    }

    #[test]
    fn override_sized_2x_fallback_is_not_halved() {
        // uimicromenu2x has no 1x sibling; its members carry OverrideWidth /
        // OverrideHeight, so the DB width (32x41) is the logical size already
        // and the plate must fill the 32x40 micro button. bagslots2x stores
        // the pixel rect (96) and halves to the 48-unit bag slot.
        let plate = get_atlas_info("ui-hud-micromenu-buttonbg-up").expect("micro-menu plate");
        assert!(plate.is_2x_fallback);
        assert!(plate.info.size_is_override);
        assert_eq!((plate.width(), plate.height()), (32, 41));

        let bag = get_atlas_info("bag-main").expect("bag slot");
        assert!(bag.is_2x_fallback);
        assert!(!bag.info.size_is_override);
        assert_eq!((bag.width(), bag.height()), (48, 48));
    }

    #[test]
    fn nine_slice_piece_uses_2x_dimensions_when_base_piece_is_missing() {
        let corner = ATLAS_DB
            .get("ui-frame-metal-cornertopleft-2x")
            .expect("metal corner +2x entry should exist");

        let piece = nine_slice_piece("ui-frame-metal-nineslice-cornertopleft")
            .expect("metal corner should resolve through paired 2x fallback");

        assert_eq!(piece.file, corner.file);
        assert_eq!(piece.width, logical_nine_slice_piece_size(corner, true).0);
        assert_eq!(piece.height, logical_nine_slice_piece_size(corner, true).1);
    }

    #[test]
    fn exact_unsuffixed_atlas_beats_2x_fallback() {
        let lookup = get_atlas_info("glues-characterselect-card-singles")
            .expect("character select singles atlas should exist");
        assert!(!lookup.is_2x_fallback);
        assert_eq!(
            lookup.info.file,
            r"Interface\glues\characterselect\uicharacterselectglues"
        );
        assert_eq!(lookup.width(), 310);
        assert_eq!(lookup.height(), 89);
    }

    #[test]
    fn render_lookup_prefers_paired_2x_atlas_without_changing_logical_size() {
        let lookup = get_render_atlas_info("questlog-icon-ticksquare")
            .expect("quest log checkbox atlas should exist");
        assert!(lookup.is_2x_fallback);
        assert_eq!(lookup.info.file, r"Interface\questframe\questlogframe2x");
        assert_eq!(lookup.width(), 14);
        assert_eq!(lookup.height(), 14);
    }

    #[test]
    fn render_lookup_keeps_other_exact_atlases_on_their_base_texture() {
        let lookup =
            get_render_atlas_info("questlog-tab-side").expect("quest log tab atlas should exist");
        assert!(!lookup.is_2x_fallback);
        assert_eq!(lookup.info.file, r"Interface\questframe\questlogframe");
        assert_eq!(lookup.width(), 51);
        assert_eq!(lookup.height(), 60);
    }

    #[test]
    fn hires_preference_sources_texels_from_the_2x_atlas_with_the_1x_logical_size() {
        let hires = super::get_render_atlas_info_with("ui-hud-minimap-frame", true)
            .expect("minimap frame atlas should exist");
        assert_eq!(hires.info.file, r"Interface\hud\uiminimap2x");
        assert!(hires.is_2x_fallback);
        assert_eq!((hires.width(), hires.height()), (215, 226));
        assert_eq!((hires.info.width, hires.info.height), (438, 460));

        let lores = super::get_render_atlas_info_with("ui-hud-minimap-frame", false)
            .expect("minimap frame atlas should exist");
        assert_eq!(lores.info.file, r"Interface\hud\uiminimap");
        assert!(!lores.is_2x_fallback);
        assert_eq!((lores.width(), lores.height()), (215, 226));
    }

    #[test]
    fn hires_preference_leaves_explicit_2x_names_alone() {
        let explicit = super::get_render_atlas_info_with("ui-hud-minimap-frame-2x", true)
            .expect("explicit 2x entry should exist");
        assert_eq!(explicit.info.file, r"Interface\hud\uiminimap2x");
        assert!(!explicit.is_2x_fallback);
        assert_eq!(explicit.width(), 438);
    }
}
