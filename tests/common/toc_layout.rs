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
