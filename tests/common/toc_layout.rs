//! Active-profile TOC file layout.
//!
//! The pinned Retail cache ships flavor-suffixed TOC variants
//! (`<AddOn>_Mainline.toc`, `<AddOn>_Classic.toc`, ...). The pinned PTR and
//! Forever caches ship one `<AddOn>.toc` per addon that selects per-flavor
//! files through `[Family]` / `[Game]` placeholders and per-line
//! `[AllowLoadGameType ...]` annotations. The classic caches (Mists, Era,
//! Anniversary, Wrath) ship the same flavor-suffixed shape as Retail, but with
//! `_Mists`, `_Vanilla`, `_Wrath` and shared `_Classic` companions instead of
//! `_Mainline`.

use std::path::{Path, PathBuf};

use wow_ui_sim::client_profile::{ClientProfile, ACTIVE};

/// True when the pinned active-profile cache stores one unified TOC per addon
/// instead of flavor-suffixed variants.
pub fn unified_toc_tree() -> bool {
    matches!(ACTIVE, ClientProfile::Ptr | ClientProfile::WowForever)
}

/// True for the Forever profile, whose pinned 1.60.1 TOCs differ in content
/// from the PTR 12.1.7 ones even though both use the unified layout.
pub fn forever() -> bool {
    ACTIVE == ClientProfile::WowForever
}

/// True for the profiles whose pinned cache stores the 218-addon classic set
/// (`data/blizzard-ui-files/{mists,era,anniversary}.txt`), plus Wrath's
/// 23-addon set. Those caches ship flavor-suffixed TOCs such as
/// `<AddOn>_Mists.toc`, `<AddOn>_Vanilla.toc`, `<AddOn>_Wrath.toc` and
/// `<AddOn>_Classic.toc` instead of the Retail `_Mainline` / bare pair.
pub fn classic_cache() -> bool {
    matches!(
        ACTIVE,
        ClientProfile::Mists
            | ClientProfile::Era
            | ClientProfile::Anniversary
            | ClientProfile::Wrath
    )
}

/// True for Mists, whose classic cache resolves `_Mists` before the shared
/// `_Classic` companion.
pub fn mists() -> bool {
    ACTIVE == ClientProfile::Mists
}

/// True for Wrath, whose 23-addon cache resolves `_Wrath` first.
pub fn wrath() -> bool {
    ACTIVE == ClientProfile::Wrath
}

/// Path of `addon` in the active profile's pinned Blizzard UI cache.
pub fn pinned_addon_dir(addon: &str) -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
        .join(addon)
}

/// True when the active profile's game-type allow-list admits `token`.
///
/// Mirrors `active_game_types` in `src/toc/mod.rs`. `toc_parsing.rs`
/// cross-checks this mirror against `is_game_type_restricted()` so drift shows
/// up as a test failure instead of a silently wrong expectation.
pub fn admits_game_type(token: &str) -> bool {
    match ACTIVE {
        ClientProfile::Retail | ClientProfile::Ptr => {
            matches!(token, "mainline" | "standard")
        }
        ClientProfile::Wrath => matches!(token, "wrath" | "wrath_classic" | "classic"),
        ClientProfile::Mists => matches!(token, "mists" | "mists_classic" | "classic"),
        ClientProfile::Era => matches!(token, "vanilla" | "classic_era" | "classic"),
        ClientProfile::Anniversary => {
            matches!(token, "vanilla" | "classic_anniversary" | "classic")
        }
        ClientProfile::WowForever => matches!(token, "camelot" | "mainline"),
    }
}

/// The TOC file name that `find_toc_file` must resolve for a mainline-capable
/// addon under the active profile.
///
/// PTR probes `_Mainline` first and Forever `_Camelot`. The classic caches
/// probe their own flavor first, then the bare name, then the shared
/// `_Classic` companion; the presence check picks the variant the pinned cache
/// actually ships (for example `Blizzard_QueueStatusFrame_Classic.toc` on
/// every classic profile, and `Blizzard_WorldMap_Mainline.toc` on PTR).
pub fn expected_toc_name(addon: &str) -> String {
    let preferences: &[&str] = match ACTIVE {
        ClientProfile::WowForever => &["_Camelot", "", "_Mainline"],
        ClientProfile::Mists => &["_Mists", "", "_Classic"],
        ClientProfile::Era | ClientProfile::Anniversary => &["_Vanilla", "", "_Classic"],
        ClientProfile::Wrath => &["_Wrath", "", "_Classic"],
        _ => &["_Mainline", "", "_Standard"],
    };
    let dir = pinned_addon_dir(addon);
    for suffix in preferences {
        let candidate = format!("{addon}{suffix}.toc");
        if dir.join(&candidate).is_file() {
            return candidate;
        }
    }
    format!("{addon}_Mainline.toc")
}

/// The TOC path that `find_toc_file` must resolve for a mainline-capable addon
/// under the active profile.
pub fn expected_toc_path(addon_dir: &Path, addon: &str) -> PathBuf {
    addon_dir.join(expected_toc_name(addon))
}
