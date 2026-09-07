use std::path::{Path, PathBuf};
use wow_ui_sim::toc::TocFile;

fn blizzard_shared_xml_base_toc() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
    .join("Blizzard_SharedXMLBase/Blizzard_SharedXMLBase.toc")
}

const ACE3_TOC: &str = "/home/osso/Projects/wow/reference-addons/Ace3/Ace3.toc";

fn is_mainline_profile() -> bool {
    matches!(
        wow_ui_sim::client_profile::ACTIVE,
        wow_ui_sim::client_profile::ClientProfile::Retail
            | wow_ui_sim::client_profile::ClientProfile::Ptr
    )
}

#[test]
fn metadata_game_type_filters_load_flags_and_interface_overrides() {
    let toc = TocFile::parse(
        Path::new("ConditionalMetadata"),
        "## LoadFirst: 1 [AllowLoadGameType mainline]\n\
         ## LoadOnDemand: 1 [AllowLoadGameType classic]\n\
         ## Interface: 120105 [AllowLoadGameType mainline]\n\
         ## Interface: 30403 [AllowLoadGameType classic]\n\
         ## Interface: 999999 [AllowLoadGameType plunderstorm]\n",
    );
    assert_eq!(toc.is_load_first(), is_mainline_profile());
    assert_eq!(toc.is_load_on_demand(), !is_mainline_profile());
    let expected_interface = if is_mainline_profile() { 120105 } else { 30403 };
    assert_eq!(toc.interface_versions(), vec![expected_interface]);
}

#[test]
fn metadata_game_type_filters_repeated_dependencies_before_merging() {
    let toc = TocFile::parse(
        Path::new("ConditionalDependencies"),
        "## Dep: Shared\n\
         ## Dep: MainlineOnly [AllowLoadGameType mainline]\n\
         ## Dep: ClassicOnly [AllowLoadGameType classic]\n\
         ## Dep: Both [AllowLoadGameType mainline, classic]\n\
         ## Dep: Unsupported [AllowLoadGameType plunderstorm]\n",
    );
    let family_dependency = if is_mainline_profile() {
        "MainlineOnly"
    } else {
        "ClassicOnly"
    };
    assert_eq!(
        toc.dependencies(),
        vec!["Shared", family_dependency, "Both"]
    );
}

#[test]
fn metadata_game_type_accepts_mixed_token_lists_after_other_annotations() {
    let toc = TocFile::parse(
        Path::new("MixedMetadata"),
        "## Dep: Commas [AllowLoadGameType plunderstorm, mainline, classic]\n\
         ## Dep: Spaces [AllowLoadGameType plunderstorm standard classic]\n\
         ## Dep: MultipleAnnotations [AllowLoad game] [AllowLoadGameType mainline, classic]\n\
         ## LoadFirst: 1 [AllowLoadGameType plunderstorm, mainline classic]\n",
    );
    assert!(toc.is_load_first());
    assert_eq!(
        toc.dependencies(),
        vec!["Commas", "Spaces", "MultipleAnnotations"]
    );
}

#[cfg(feature = "client-ptr")]
#[test]
fn metadata_game_type_excludes_classic_edges_in_pinned_ptr_addons() {
    let root = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )));
    let parse_addon = |name: &str| {
        TocFile::from_file(&root.join(name).join(format!("{name}.toc")))
            .unwrap_or_else(|error| panic!("parse pinned PTR {name}: {error}"))
    };
    let chat = parse_addon("Blizzard_ChatFrame");
    let chat_base = parse_addon("Blizzard_ChatFrameBase");
    let action_bar = parse_addon("Blizzard_ActionBar");
    assert!(!chat.is_load_first(), "ChatFrame LoadFirst is classic-only");
    assert!(
        !chat_base.is_load_first(),
        "ChatFrameBase LoadFirst is classic-only"
    );
    assert_eq!(
        chat.dependencies(),
        vec![
            "Blizzard_EditMode",
            "Blizzard_ChatFrameBase",
            "Blizzard_VoiceToggleButton",
        ]
    );
    assert_eq!(
        chat_base.dependencies(),
        vec![
            "Blizzard_AutoComplete",
            "Blizzard_Settings_Shared",
            "Blizzard_TimerunningUtil",
            "Blizzard_FrameXMLUtil",
            "Blizzard_GlueStubs",
            "Blizzard_EditMode",
            "Blizzard_TransmogShared",
            "Blizzard_GameMenuEsc",
        ]
    );
    assert_eq!(
        action_bar.dependencies(),
        vec![
            "Blizzard_StoreUI",
            "Blizzard_QuickKeybind",
            "Blizzard_EditMode",
            "Blizzard_UIPanels_Game",
            "Blizzard_TextStatusBar",
            "Blizzard_Flyout",
            "Blizzard_Colors",
            "Blizzard_HelpPlate",
            "Blizzard_MicroMenu",
            "Blizzard_PingUI",
            "Blizzard_GameMenuEsc",
        ]
    );
}

#[test]
fn test_parse_blizzard_shared_xml_base() {
    let toc_path = blizzard_shared_xml_base_toc();
    let toc = TocFile::from_file(&toc_path).expect("Failed to read TOC file");

    assert_eq!(toc.name, "Blizzard_SharedXMLBase");
    assert!(toc.is_blizzard_addon());

    // Should have many files
    assert!(
        toc.files.len() > 20,
        "Expected many files, got {}",
        toc.files.len()
    );

    // First file should be Compat.lua
    assert_eq!(toc.files[0].to_str().unwrap(), "Compat.lua");

    // Should contain Mixin.lua
    assert!(
        toc.files.iter().any(|f| f.to_str() == Some("Mixin.lua")),
        "Expected Mixin.lua in file list"
    );
}

#[test]
fn test_parse_ace3() {
    let path = Path::new(ACE3_TOC);
    if !path.exists() {
        eprintln!("Skipping test_parse_ace3: {} not found", ACE3_TOC);
        return;
    }

    let toc = TocFile::from_file(path).expect("Failed to read TOC file");

    assert_eq!(toc.name, "Lib: Ace3");
    assert!(!toc.is_blizzard_addon());

    let versions = toc.interface_versions();
    assert!(!versions.is_empty(), "Expected interface version");

    assert!(
        toc.files[0].to_str().unwrap().contains("LibStub"),
        "Expected LibStub as first file"
    );
}

#[test]
fn test_file_paths_absolute() {
    let toc_path = blizzard_shared_xml_base_toc();
    let toc = TocFile::from_file(&toc_path).expect("Failed to read TOC file");

    let paths = toc.file_paths();

    for path in &paths {
        assert!(path.is_absolute(), "Expected absolute path: {:?}", path);
        assert!(path.exists(), "File should exist: {:?}", path);
    }
}

#[test]
fn test_lua_and_xml_files() {
    let toc_path = blizzard_shared_xml_base_toc();
    let toc = TocFile::from_file(&toc_path).expect("Failed to read TOC file");

    let lua_count = toc
        .files
        .iter()
        .filter(|f| f.extension().map(|e| e == "lua").unwrap_or(false))
        .count();

    let xml_count = toc
        .files
        .iter()
        .filter(|f| f.extension().map(|e| e == "xml").unwrap_or(false))
        .count();

    assert!(lua_count > 0, "Expected Lua files");
    assert!(xml_count > 0, "Expected XML files");
}
