//! WoW client profile selection — retail, PTR, wrath, mists, era, anniversary, Forever.
//!
//! Exactly one client profile marker must be enabled. The active profile
//! determines which profile-scoped Blizzard UI cache the addon loader reads;
//! cumulative retail epoch features select the exposed API version separately.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientProfile {
    Retail,
    Ptr,
    Wrath,
    Mists,
    Era,
    Anniversary,
    WowForever,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetailApiEpoch {
    Retail12_0_0,
    Retail12_0_5,
    Retail12_0_7,
    Retail12_1_0,
    Retail12_1_5,
}

impl RetailApiEpoch {
    pub const fn interface_version(self) -> u32 {
        match self {
            RetailApiEpoch::Retail12_0_0 => 120000,
            RetailApiEpoch::Retail12_0_5 => 120005,
            RetailApiEpoch::Retail12_0_7 => 120007,
            RetailApiEpoch::Retail12_1_0 => 120100,
            RetailApiEpoch::Retail12_1_5 => 120105,
        }
    }
}

#[cfg(feature = "retail-12-1-5")]
pub const ACTIVE_RETAIL_API_EPOCH: RetailApiEpoch = RetailApiEpoch::Retail12_1_5;

#[cfg(all(not(feature = "retail-12-1-5"), feature = "retail-12-1-0"))]
pub const ACTIVE_RETAIL_API_EPOCH: RetailApiEpoch = RetailApiEpoch::Retail12_1_0;

#[cfg(all(
    not(feature = "retail-12-1-5"),
    not(feature = "retail-12-1-0"),
    feature = "retail-12-0-7"
))]
pub const ACTIVE_RETAIL_API_EPOCH: RetailApiEpoch = RetailApiEpoch::Retail12_0_7;

#[cfg(all(
    not(feature = "retail-12-1-5"),
    not(feature = "retail-12-1-0"),
    not(feature = "retail-12-0-7"),
    feature = "retail-12-0-5"
))]
pub const ACTIVE_RETAIL_API_EPOCH: RetailApiEpoch = RetailApiEpoch::Retail12_0_5;

#[cfg(all(
    not(feature = "retail-12-1-5"),
    not(feature = "retail-12-1-0"),
    not(feature = "retail-12-0-7"),
    not(feature = "retail-12-0-5"),
    feature = "retail-12-0-0"
))]
pub const ACTIVE_RETAIL_API_EPOCH: RetailApiEpoch = RetailApiEpoch::Retail12_0_0;

#[cfg(not(feature = "retail-12-0-0"))]
pub const ACTIVE_RETAIL_API_EPOCH: RetailApiEpoch = RetailApiEpoch::Retail12_0_7;

pub const RETAIL_API_INTERFACE_VERSION: u32 = ACTIVE_RETAIL_API_EPOCH.interface_version();

#[cfg(any(feature = "profile-retail", feature = "client-ptr"))]
pub const ACTIVE_INTERFACE_VERSION: u32 = RETAIL_API_INTERFACE_VERSION;

#[cfg(feature = "client-wrath")]
pub const ACTIVE_INTERFACE_VERSION: u32 = 38001;

#[cfg(feature = "client-mists")]
pub const ACTIVE_INTERFACE_VERSION: u32 = 50504;

#[cfg(any(feature = "client-era", feature = "client-anniversary"))]
pub const ACTIVE_INTERFACE_VERSION: u32 = 11507;

// Forever 1.60.1's published TOC version, distinct from Era/Anniversary.
#[cfg(feature = "client-wowforever")]
pub const ACTIVE_INTERFACE_VERSION: u32 = 16001;

impl ClientProfile {
    pub fn subdir(self) -> &'static str {
        match self {
            ClientProfile::Retail => "Retail",
            ClientProfile::Ptr => "Ptr",
            ClientProfile::Wrath => "Wrath",
            ClientProfile::Mists => "Mists",
            ClientProfile::Era => "Era",
            ClientProfile::Anniversary => "Anniversary",
            ClientProfile::WowForever => "WowForever",
        }
    }

    pub fn cache_subdir(self) -> &'static str {
        match self {
            ClientProfile::Retail => "retail",
            ClientProfile::Ptr => "ptr",
            ClientProfile::Wrath => "wrath",
            ClientProfile::Mists => "mists",
            ClientProfile::Era => "era",
            ClientProfile::Anniversary => "anniversary",
            ClientProfile::WowForever => "wowforever",
        }
    }

    pub const fn interface_version(self) -> u32 {
        match self {
            ClientProfile::Retail | ClientProfile::Ptr => RETAIL_API_INTERFACE_VERSION,
            ClientProfile::Wrath => 38001,
            ClientProfile::Mists => 50504,
            ClientProfile::Era | ClientProfile::Anniversary => 11507,
            ClientProfile::WowForever => 16001,
        }
    }
}

#[cfg(all(
    not(feature = "client-wrath"),
    not(feature = "client-mists"),
    not(feature = "client-era"),
    not(feature = "client-anniversary"),
    not(feature = "client-ptr"),
    feature = "profile-retail",
    not(feature = "client-wowforever"),
))]
pub const ACTIVE: ClientProfile = ClientProfile::Retail;

#[cfg(all(
    not(feature = "profile-retail"),
    not(feature = "client-wrath"),
    not(feature = "client-mists"),
    not(feature = "client-era"),
    not(feature = "client-anniversary"),
    feature = "client-ptr",
    not(feature = "client-wowforever"),
))]
pub const ACTIVE: ClientProfile = ClientProfile::Ptr;

#[cfg(all(
    not(feature = "profile-retail"),
    not(feature = "client-ptr"),
    not(feature = "client-mists"),
    not(feature = "client-era"),
    not(feature = "client-anniversary"),
    feature = "client-wrath",
    not(feature = "client-wowforever"),
))]
pub const ACTIVE: ClientProfile = ClientProfile::Wrath;

#[cfg(all(
    not(feature = "profile-retail"),
    not(feature = "client-ptr"),
    not(feature = "client-wrath"),
    not(feature = "client-era"),
    not(feature = "client-anniversary"),
    feature = "client-mists",
    not(feature = "client-wowforever"),
))]
pub const ACTIVE: ClientProfile = ClientProfile::Mists;

#[cfg(all(
    not(feature = "profile-retail"),
    not(feature = "client-ptr"),
    not(feature = "client-wrath"),
    not(feature = "client-mists"),
    not(feature = "client-anniversary"),
    feature = "client-era",
    not(feature = "client-wowforever"),
))]
pub const ACTIVE: ClientProfile = ClientProfile::Era;

#[cfg(all(
    not(feature = "profile-retail"),
    not(feature = "client-ptr"),
    not(feature = "client-wrath"),
    not(feature = "client-mists"),
    not(feature = "client-era"),
    feature = "client-anniversary",
    not(feature = "client-wowforever"),
))]
pub const ACTIVE: ClientProfile = ClientProfile::Anniversary;

#[cfg(all(
    not(feature = "profile-retail"),
    not(feature = "client-ptr"),
    not(feature = "client-wrath"),
    not(feature = "client-mists"),
    not(feature = "client-era"),
    not(feature = "client-anniversary"),
    feature = "client-wowforever",
))]
pub const ACTIVE: ClientProfile = ClientProfile::WowForever;

#[cfg(any(
    all(feature = "profile-retail", feature = "client-wrath"),
    all(feature = "profile-retail", feature = "client-mists"),
    all(feature = "profile-retail", feature = "client-era"),
    all(feature = "profile-retail", feature = "client-anniversary"),
    all(feature = "profile-retail", feature = "client-ptr"),
    all(feature = "client-ptr", feature = "client-wrath"),
    all(feature = "client-ptr", feature = "client-mists"),
    all(feature = "client-ptr", feature = "client-era"),
    all(feature = "client-ptr", feature = "client-anniversary"),
    all(feature = "client-wrath", feature = "client-mists"),
    all(feature = "client-wrath", feature = "client-era"),
    all(feature = "client-wrath", feature = "client-anniversary"),
    all(feature = "client-mists", feature = "client-era"),
    all(feature = "client-mists", feature = "client-anniversary"),
    all(feature = "client-era", feature = "client-anniversary"),
    all(
        feature = "client-wowforever",
        any(
            feature = "profile-retail",
            feature = "client-ptr",
            feature = "client-wrath",
            feature = "client-mists",
            feature = "client-era",
            feature = "client-anniversary"
        )
    ),
))]
compile_error!(
    "Exactly one profile marker must be enabled: profile-retail (normally via client-retail), client-ptr, client-wrath, client-mists, client-era, client-anniversary, or client-wowforever"
);

#[cfg(not(any(
    feature = "profile-retail",
    feature = "client-ptr",
    feature = "client-wrath",
    feature = "client-mists",
    feature = "client-era",
    feature = "client-anniversary",
    feature = "client-wowforever",
)))]
compile_error!(
    "Exactly one profile marker must be enabled: profile-retail (normally via client-retail), client-ptr, client-wrath, client-mists, client-era, client-anniversary, or client-wowforever"
);

#[cfg(all(feature = "client-ptr", not(feature = "retail-12-1-5")))]
compile_error!("client-ptr must enable the retail-12-1-5 API epoch");

#[cfg(all(
    any(
        feature = "retail-12-0-0",
        feature = "retail-12-0-5",
        feature = "retail-12-0-7",
        feature = "retail-12-1-0",
        feature = "retail-12-1-5"
    ),
    not(any(feature = "profile-retail", feature = "client-ptr"))
))]
compile_error!("retail API epoch features require profile-retail or client-ptr");

/// Path to the AddOns directory for the active profile.
///
/// Prefer a completed cache-managed Blizzard UI source tree for every client
/// profile, otherwise return the profile-scoped default cache path so startup
/// can sync it from CASC.
pub fn blizzard_ui_addons_dir() -> PathBuf {
    blizzard_ui_addons_dir_under_with_cache(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")),
        crate::blizzard_ui_sync::cached_blizzard_ui_addons_path(),
    )
}

/// Absolute path to the AddOns directory for the active profile, anchored at `root`.
///
/// Tests typically pass `Path::new(env!("CARGO_MANIFEST_DIR"))` so the path resolves
/// regardless of the test's current working directory.
pub fn blizzard_ui_addons_dir_under(root: &Path) -> PathBuf {
    blizzard_ui_addons_dir_under_with_cache(
        root,
        crate::blizzard_ui_sync::cached_blizzard_ui_addons_path(),
    )
}

fn blizzard_ui_addons_dir_under_with_cache(root: &Path, cache_path: Option<PathBuf>) -> PathBuf {
    if let Some(cache_path) = cache_path {
        return cache_path;
    }

    crate::blizzard_ui_sync::default_cache_addons_path().unwrap_or_else(|_| {
        root.join(".cache/wow-ui-sim/blizzard-ui")
            .join(ACTIVE.cache_subdir())
            .join("AddOns")
    })
}

/// Path to the FrameXML.toc under the active profile, if it exists on disk.
///
/// Wrath ships its UI as a flat `Interface/FrameXML/` tree alongside `Interface/AddOns/`;
/// retail and mists collapsed FrameXML into `Blizzard_*` addons and have no top-level
/// FrameXML directory. Callers use this to load a synthetic "FrameXML" addon before
/// the regular Blizzard_* discovery pass.
pub fn blizzard_ui_framexml_toc() -> Option<PathBuf> {
    let addons_dir = blizzard_ui_addons_dir();
    [
        addons_dir.join("FrameXML").join("FrameXML.toc"),
        addons_dir
            .parent()
            .unwrap_or(&addons_dir)
            .join("FrameXML")
            .join("FrameXML.toc"),
    ]
    .into_iter()
    .find(|toc| toc.exists())
}

#[cfg(test)]
#[cfg_attr(not(feature = "client-mists"), allow(unused_imports))]
mod tests {
    use super::*;

    #[test]
    fn retail_api_epoch_maps_to_interface_versions() {
        let expected = [
            (RetailApiEpoch::Retail12_0_0, 120000),
            (RetailApiEpoch::Retail12_0_5, 120005),
            (RetailApiEpoch::Retail12_0_7, 120007),
            (RetailApiEpoch::Retail12_1_0, 120100),
            (RetailApiEpoch::Retail12_1_5, 120105),
        ];

        for (epoch, interface_version) in expected {
            assert_eq!(epoch.interface_version(), interface_version);
        }
    }

    #[test]
    #[cfg(all(
        feature = "profile-retail",
        feature = "retail-12-0-0",
        not(feature = "retail-12-0-5"),
        not(feature = "retail-12-0-7"),
        not(feature = "retail-12-1-0"),
    ))]
    fn historical_retail_profile_uses_12_0_0_epoch() {
        assert_eq!(ACTIVE, ClientProfile::Retail);
        assert_eq!(ACTIVE.cache_subdir(), "retail");
        assert_eq!(ACTIVE.interface_version(), 120000);
    }

    #[test]
    #[cfg(feature = "client-retail")]
    fn retail_client_points_at_current_retail_api_epoch() {
        assert_eq!(ACTIVE, ClientProfile::Retail);
        assert_eq!(ACTIVE_INTERFACE_VERSION, 120100);
        assert!(cfg!(feature = "retail-12-1-0"));
    }

    #[test]
    #[cfg(all(
        feature = "profile-retail",
        feature = "retail-12-1-0",
        not(feature = "retail-12-1-5")
    ))]
    fn retail_client_can_point_at_patch_12_1_0_api_epoch() {
        assert_eq!(ACTIVE, ClientProfile::Retail);
        assert_eq!(ACTIVE_INTERFACE_VERSION, 120100);
        assert_eq!(RETAIL_API_INTERFACE_VERSION, 120100);
    }

    #[test]
    #[cfg(feature = "client-ptr")]
    fn ptr_client_points_at_patch_12_1_5_api_epoch() {
        assert!(cfg!(feature = "retail-12-0-7"));
        assert!(cfg!(feature = "retail-12-1-0"));
        assert!(cfg!(feature = "retail-12-1-5"));
        assert_eq!(ACTIVE_INTERFACE_VERSION, 120105);
    }

    #[test]
    #[cfg(feature = "client-mists")]
    fn mists_prefers_completed_cache_over_default_cache_path() {
        let root = tempfile::tempdir().expect("tempdir");
        let cache_path = root.path().join("cache/blizzard-ui/mists/AddOns");
        let resolved =
            blizzard_ui_addons_dir_under_with_cache(root.path(), Some(cache_path.clone()));

        assert_eq!(resolved, cache_path);
    }

    #[test]
    fn missing_cache_resolves_to_profile_scoped_cache_path() {
        let root = tempfile::tempdir().expect("tempdir");
        let resolved = blizzard_ui_addons_dir_under_with_cache(root.path(), None);

        assert!(
            resolved.ends_with(Path::new(ACTIVE.cache_subdir()).join("AddOns")),
            "Blizzard UI fallback path should be profile-scoped cache AddOns root, got {}",
            resolved.display()
        );
    }

    #[test]
    #[cfg(feature = "client-retail")]
    fn retail_interface_matches_current_live_build() {
        assert_eq!(ClientProfile::Retail.interface_version(), 120100);
    }

    #[test]
    #[cfg(feature = "client-ptr")]
    fn ptr_uses_12_1_interface_and_cache_scope() {
        assert_eq!(ACTIVE, ClientProfile::Ptr);
        assert_eq!(ACTIVE.interface_version(), 120105);
        assert_eq!(ACTIVE.cache_subdir(), "ptr");
    }
}
