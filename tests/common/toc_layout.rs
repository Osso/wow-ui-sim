//! Active-profile TOC file layout.
//!
//! The pinned Retail cache ships flavor-suffixed TOC variants
//! (`<AddOn>_Mainline.toc`, `<AddOn>_Classic.toc`, ...). The pinned PTR and
//! Forever caches ship one `<AddOn>.toc` per addon that selects per-flavor
//! files through `[Family]` / `[Game]` placeholders and per-line
//! `[AllowLoadGameType ...]` annotations.

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
pub fn expected_toc_name(addon: &str) -> String {
    if unified_toc_tree() {
        format!("{addon}.toc")
    } else {
        format!("{addon}_Mainline.toc")
    }
}

/// The TOC path that `find_toc_file` must resolve for a mainline-capable addon
/// under the active profile.
pub fn expected_toc_path(addon_dir: &Path, addon: &str) -> PathBuf {
    addon_dir.join(expected_toc_name(addon))
}
