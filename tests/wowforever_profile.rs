//! Profile isolation and the TOC rules used by the Forever source tree.

use std::path::{Path, PathBuf};
use wow_ui_sim::toc::TocFile;

#[test]
#[cfg(any(feature = "profile-retail", feature = "client-ptr"))]
fn wowforever_profile_exclusion_annotation_filters_files_and_dependencies() {
    let toc = TocFile::parse(
        Path::new("/addons/Example"),
        "## Dep: Common\n## Dep: OtherGame [ExcludeLoadGameType mainline]\nCommon.lua\nOther.lua [ExcludeLoadGameType mainline]\n",
    );

    assert_eq!(toc.dependencies(), vec!["Common"]);
    assert_eq!(toc.files, vec![PathBuf::from("Common.lua")]);
}

#[test]
#[cfg(feature = "client-wowforever")]
fn wowforever_profile_identity_and_manifest_are_distinct() {
    use wow_ui_sim::client_profile::{ACTIVE, ACTIVE_INTERFACE_VERSION, ClientProfile};

    assert_eq!(ACTIVE, ClientProfile::WowForever);
    assert_eq!(ACTIVE.subdir(), "WowForever");
    assert_eq!(ACTIVE.cache_subdir(), "wowforever");
    assert_eq!(ACTIVE.interface_version(), 16001);
    assert_eq!(ACTIVE_INTERFACE_VERSION, 16001);
    assert!(!cfg!(feature = "retail-12-0-0"));
    assert!(!cfg!(feature = "client-anniversary"));
    let entries: Vec<_> = wow_ui_sim::blizzard_ui_sync::manifest_entries().collect();
    assert_eq!(entries.len(), 4398);
    assert!(entries.contains(&"Blizzard_FrameXML/Camelot/StackSplitFrame.xml"));
    assert!(entries.contains(&"Blizzard_FrameXMLBase/Blizzard_FrameXMLBase.toc"));
    assert!(
        wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
            .unwrap()
            .ends_with("blizzard-ui/wowforever/AddOns")
    );
}

#[test]
#[cfg(feature = "client-wowforever")]
fn wowforever_profile_reports_build_identity_and_finite_event_validation() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    let identity: (String, String, i32) = env
        .eval(
            "local version, build, _, interface = GetBuildInfo(); return version, build, interface",
        )
        .unwrap();
    assert_eq!(identity, ("1.60.1".to_owned(), "69913".to_owned(), 16001));
    assert!(wow_ui_sim::event::is_registerable_event("PLAYER_LOGIN"));
    for event in [
        "CONFIRM_BATTLE_NET_FRIEND_INVITE_SHOW",
        "DIEL_CYCLE_CHANGED",
        "DISCORD_LINK_UPDATE",
        "EXTERNAL_EVENT_LAUNCH_URL_FAILED",
        "GROUP_BUFF_VISUAL_ALERTS_CHANGED",
        "GUILD_RANKS_UPDATE_ACTIVE_PLAYER",
        "INPUT_DEVICE_INTERFACE_TRANSITION",
        "LFG_LIST_REVEALED_CENSORED_ACTIVE_ENTRY",
        "SOCIAL_UI_SYSTEM_STATUS_UPDATED",
        "UNIT_HAPPINESS",
        "UNIT_PET_TRAINING_POINTS",
        "UNIT_PING_PIN_ADDED",
        "UNIT_PING_PIN_REMOVED",
        "DISCORD_STATUS_UPDATE",
    ] {
        assert!(wow_ui_sim::event::is_registerable_event(event), "{event}");
    }
    assert!(!wow_ui_sim::event::is_registerable_event(
        "WOWFOREVER_INVENTED_EVENT"
    ));
    assert!(!wow_ui_sim::event::is_registerable_event(""));
    let retail_accessor: String = env
        .eval("return type(GetSecurePendingButtonCallback)")
        .unwrap();
    assert_eq!(retail_accessor, "nil");
}

#[test]
#[cfg(feature = "client-wowforever")]
fn wowforever_profile_loads_camelot_mainline_toc_entries() {
    // Forever 1.60.1.69913 Blizzard_FrameXML and FrameXMLBase TOC excerpts.
    let toc = TocFile::parse(
        Path::new("/addons/Blizzard_FrameXML"),
        "## Dep: Blizzard_UnitPopup [AllowLoadGameType classic]\n\
         ## Dep: Blizzard_UIParentPanelManager [AllowLoadGameType mainline]\n\
         [Family]\\StackSplitFrame.lua\n\
         [Family]\\StackSplitFrame.xml [ExcludeLoadGameType camelot]\n\
         [Game]\\StackSplitFrame.xml\t[AllowLoadGameType camelot]\n\
         Vanilla\\Constants.lua [AllowLoadGameType vanilla]\n",
    );
    assert_eq!(toc.dependencies(), vec!["Blizzard_UIParentPanelManager"]);
    assert_eq!(
        toc.files,
        vec![
            PathBuf::from("Mainline/StackSplitFrame.lua"),
            PathBuf::from("Camelot/StackSplitFrame.xml"),
        ]
    );
}

#[test]
#[cfg(feature = "client-wowforever")]
fn wowforever_profile_applies_header_filters_without_vanilla_alias() {
    for tag in ["camelot", "mainline"] {
        let toc = TocFile::parse(
            Path::new("/addons/Test"),
            &format!("## AllowLoadGameType: {tag}\n"),
        );
        assert!(!toc.is_game_type_restricted(), "{tag}");
    }
    for header in [
        "## AllowLoadGameType: vanilla",
        "## AllowLoadGameType: classic",
        "## AllowLoadGameType: standard",
        "## ExcludeLoadGameType: camelot",
        "## ExcludeLoadGameType: mainline",
    ] {
        assert!(
            TocFile::parse(Path::new("/addons/Test"), header).is_game_type_restricted(),
            "{header}"
        );
    }
    assert!(!TocFile::parse(Path::new("/addons/Test"), "## OnlyBetaAndPTR: 1").is_ptr_only());
}

#[test]
#[cfg(feature = "client-wowforever")]
fn wowforever_profile_selects_generic_toc_not_other_client_flavors() {
    let root = tempfile::tempdir().unwrap();
    let addon = root.path().join("Example");
    std::fs::create_dir(&addon).unwrap();
    for suffix in [
        "_Vanilla",
        "_Mainline",
        "_Classic",
        "_Mists",
        "_Wrath",
        "_TBC",
        "_Cata",
        "",
    ] {
        std::fs::write(addon.join(format!("Example{suffix}.toc")), "Core.lua\n").unwrap();
    }
    assert_eq!(
        wow_ui_sim::loader::find_toc_file(&addon),
        Some(addon.join("Example.toc"))
    );
    std::fs::remove_file(addon.join("Example.toc")).unwrap();
    assert_eq!(wow_ui_sim::loader::find_toc_file(&addon), None);
}

#[test]
#[cfg(feature = "client-wowforever")]
fn wowforever_profile_loads_pinned_world_map_filename() {
    let root = tempfile::tempdir().unwrap();
    let addon = root.path().join("Blizzard_WorldMap");
    std::fs::create_dir(&addon).unwrap();
    let toc_path = addon.join("Blizzard_WorldMap_Mainline.toc");
    std::fs::write(
        &toc_path,
        "[Game]/Blizzard_WorldMapConstants.lua [AllowLoadGameType camelot]\n",
    )
    .unwrap();
    assert_eq!(
        wow_ui_sim::loader::find_toc_file(&addon),
        Some(toc_path.clone())
    );
    let toc = TocFile::from_file(&toc_path).unwrap();
    assert_eq!(
        toc.files,
        vec![PathBuf::from("Camelot/Blizzard_WorldMapConstants.lua")]
    );
}
