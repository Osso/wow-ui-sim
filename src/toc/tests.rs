use super::*;

#[test]
fn test_parse_simple_toc() {
    let contents = r#"
## Title: MyAddon
## Interface: 110000
## Dependencies: Ace3, LibStub

Core.lua
UI/Main.lua
UI/Options.xml
"#;
    let toc = TocFile::parse(Path::new("/addons/MyAddon"), contents);

    assert_eq!(toc.name, "MyAddon");
    assert_eq!(toc.interface_versions(), vec![110000]);
    assert_eq!(toc.dependencies(), vec!["Ace3", "LibStub"]);
    assert_eq!(toc.files.len(), 3);
    assert_eq!(toc.files[0], PathBuf::from("Core.lua"));
    assert_eq!(toc.files[1], PathBuf::from("UI/Main.lua"));
    assert_eq!(toc.files[2], PathBuf::from("UI/Options.xml"));
}

#[test]
fn test_parse_space_separated_dependencies() {
    let contents = r#"
## Title: Blizzard_BattlefieldMap
## Dependencies: Blizzard_MapCanvas Blizzard_SharedMapDataProviders Blizzard_ObjectiveTracker
BattlefieldMap.lua
"#;
    let toc = TocFile::parse(Path::new("/addons/Blizzard_BattlefieldMap"), contents);

    assert_eq!(
        toc.dependencies(),
        vec![
            "Blizzard_MapCanvas",
            "Blizzard_SharedMapDataProviders",
            "Blizzard_ObjectiveTracker",
        ]
    );
}

#[test]
#[cfg(feature = "client-ptr")]
fn ptr_profile_allows_beta_and_ptr_tocs() {
    let toc = TocFile::parse(
        Path::new("/addons/Blizzard_PTRFeedback"),
        "## Title: Blizzard_PTRFeedback\n## OnlyBetaAndPTR: 1\nPTRFeedback.lua\n",
    );

    assert!(!toc.is_ptr_only());
}

#[test]
#[cfg(not(any(feature = "client-ptr", feature = "client-wowforever")))]
fn non_ptr_profiles_restrict_beta_and_ptr_tocs() {
    let toc = TocFile::parse(
        Path::new("/addons/Blizzard_PTRFeedback"),
        "## Title: Blizzard_PTRFeedback\n## OnlyBetaAndPTR: 1\nPTRFeedback.lua\n",
    );

    assert!(toc.is_ptr_only());
}

#[test]
fn test_parse_repeated_dep_metadata() {
    let contents = r#"
## Title: Blizzard_CatalogShopRefundFlow
## Dep: Blizzard_SharedXML
## Dep: Blizzard_CatalogShopSharedUtil
## Dep: Blizzard_AsyncRequest
Core.lua
"#;
    let toc = TocFile::parse(
        Path::new("/addons/Blizzard_CatalogShopRefundFlow"),
        contents,
    );

    assert_eq!(
        toc.dependencies(),
        vec![
            "Blizzard_SharedXML",
            "Blizzard_CatalogShopSharedUtil",
            "Blizzard_AsyncRequest",
        ]
    );
}

#[test]
fn test_parse_repeated_optional_dep_metadata() {
    let contents = r#"
## Title: Blizzard_OptionalChain
## OptionalDep: Blizzard_A
## OptionalDep: Blizzard_B
Core.lua
"#;
    let toc = TocFile::parse(Path::new("/addons/Blizzard_OptionalChain"), contents);

    assert_eq!(toc.optional_deps(), vec!["Blizzard_A", "Blizzard_B"]);
}

#[test]
fn test_parse_blizzard_toc() {
    let contents = r#"
## Title: Blizzard_SharedXMLBase
## AllowLoad: Both
Compat.lua
Mixin.lua
TableUtil.lua
"#;
    let toc = TocFile::parse(
        Path::new("/Interface/AddOns/Blizzard_SharedXMLBase"),
        contents,
    );

    assert_eq!(toc.name, "Blizzard_SharedXMLBase");
    assert!(toc.is_blizzard_addon());
    assert_eq!(toc.files.len(), 3);
}

#[test]
fn test_parse_with_comments() {
    let contents = r#"
## Title: TestAddon
# This is a comment
#@no-lib-strip@
Libs/LibStub.lua
#@end-no-lib-strip@
Core.lua
"#;
    let toc = TocFile::parse(Path::new("/addons/TestAddon"), contents);

    // Comments and directives should be skipped
    assert_eq!(toc.files.len(), 2);
    assert_eq!(toc.files[0], PathBuf::from("Libs/LibStub.lua"));
    assert_eq!(toc.files[1], PathBuf::from("Core.lua"));
}

#[test]
fn test_parse_backslash_paths() {
    let contents = r#"
## Title: TestAddon
Libs\LibStub\LibStub.lua
Core\Init.lua
"#;
    let toc = TocFile::parse(Path::new("/addons/TestAddon"), contents);

    // Backslashes should be normalized to forward slashes
    assert_eq!(toc.files[0], PathBuf::from("Libs/LibStub/LibStub.lua"));
    assert_eq!(toc.files[1], PathBuf::from("Core/Init.lua"));
}

#[test]
fn test_optional_deps() {
    let contents = r#"
## Title: TestAddon
## OptionalDeps: Ace3, LibDBIcon-1.0, LibSharedMedia-3.0
Core.lua
"#;
    let toc = TocFile::parse(Path::new("/addons/TestAddon"), contents);

    assert_eq!(
        toc.optional_deps(),
        vec!["Ace3", "LibDBIcon-1.0", "LibSharedMedia-3.0"]
    );
}

#[test]
fn test_load_first_metadata() {
    let contents = r#"
## Title: TestAddon
## LoadFirst: 1
Core.lua
"#;
    let toc = TocFile::parse(Path::new("/addons/TestAddon"), contents);

    assert!(toc.is_load_first());
}

#[test]
fn test_saved_variables() {
    let contents = r#"
## Title: TestAddon
## SavedVariables: TestAddonDB, TestAddonPerCharDB
Core.lua
"#;
    let toc = TocFile::parse(Path::new("/addons/TestAddon"), contents);

    assert_eq!(
        toc.saved_variables(),
        vec!["TestAddonDB", "TestAddonPerCharDB"]
    );
}

#[test]
fn test_saved_variables_split_whitespace_names() {
    let contents = r#"
## Title: TestAddon
## SavedVariables: TestAddonDB TestAddonMinimapDB
## SavedVariablesPerCharacter: TestAddonCharDB TestAddonCharMinimapDB
Core.lua
"#;
    let toc = TocFile::parse(Path::new("/addons/TestAddon"), contents);

    assert_eq!(
        toc.saved_variables(),
        vec!["TestAddonDB", "TestAddonMinimapDB"]
    );
    assert_eq!(
        toc.saved_variables_per_character(),
        vec!["TestAddonCharDB", "TestAddonCharMinimapDB"]
    );
}

#[test]
fn test_multiple_interface_versions() {
    let contents = r#"
## Title: TestAddon
## Interface: 110107, 50500, 11507
Core.lua
"#;
    let toc = TocFile::parse(Path::new("/addons/TestAddon"), contents);

    assert_eq!(toc.interface_versions(), vec![110107, 50500, 11507]);
}

/// Wrath 3.3.5 vendors write `## Interface: 30300`. Parser must accept the
/// single legacy value.
#[test]
fn test_wrath_interface_version() {
    let contents = r#"
## Title: WrathAddon
## Interface: 30300
WrathCore.lua
"#;
    let toc = TocFile::parse(Path::new("/addons/WrathAddon"), contents);

    assert_eq!(toc.interface_versions(), vec![30300]);
    assert!(!toc.is_game_type_restricted());
}

/// Mists profile vendors write `## Interface: 50500` for MoP-Classic
/// addons. Parser must accept the single value (the existing
/// `test_multiple_interface_versions` covers 50500 only as a list element).
#[test]
fn test_mists_interface_version() {
    let contents = r#"
## Title: MistsAddon
## Interface: 50500
MistsCore.lua
"#;
    let toc = TocFile::parse(Path::new("/addons/MistsAddon"), contents);

    assert_eq!(toc.interface_versions(), vec![50500]);
    assert!(!toc.is_game_type_restricted());
}

#[test]
fn test_supports_interface_version() {
    let contents = format!(
        "## Interface: {}, 120001\nCore.lua\n",
        RETAIL_INTERFACE_VERSION
    );
    let current = TocFile::parse(Path::new("/addons/TestAddon"), &contents);
    let old = TocFile::parse(
        Path::new("/addons/TestAddon"),
        "## Interface: 120001\nCore.lua\n",
    );
    let missing = TocFile::parse(Path::new("/addons/TestAddon"), "Core.lua\n");

    assert!(current.supports_interface_version(RETAIL_INTERFACE_VERSION));
    assert!(!old.supports_interface_version(RETAIL_INTERFACE_VERSION));
    assert!(missing.supports_interface_version(RETAIL_INTERFACE_VERSION));
}

#[test]
fn test_parse_inline_annotations() {
    let contents = r#"
## Title: TestAddon
Core.lua
Dump.lua [AllowLoadEnvironment Global]
Debug.lua [AllowLoadEnvironment Global, SomeFlag]
"#;
    let toc = TocFile::parse(Path::new("/addons/TestAddon"), contents);

    // Annotations should be stripped, only filenames kept
    assert_eq!(toc.files.len(), 3);
    assert_eq!(toc.files[0], PathBuf::from("Core.lua"));
    assert_eq!(toc.files[1], PathBuf::from("Dump.lua"));
    assert_eq!(toc.files[2], PathBuf::from("Debug.lua"));
    assert_eq!(toc.file_use_secure_env(0), None);
    assert_eq!(toc.file_use_secure_env(1), None);
    assert_eq!(toc.file_use_secure_env(2), None);
    assert_eq!(toc.file_allow_load_environment(0), None);
    assert_eq!(toc.file_allow_load_environment(1), Some(false));
    assert_eq!(toc.file_allow_load_environment(2), None);
}

#[test]
fn test_parse_native_target_aura_multiannotations() {
    let toc = TocFile::parse(
        Path::new("/addons/Blizzard_UnitFrame"),
        "Before.lua\n\
         Shared\\TargetFrameAuraContainer.lua\t\t[AllowLoadGameType mainline] [LoadIntoEnvironment secure]\n\
         Shared\\TargetFrameAuraButton.lua\t\t[AllowLoadGameType mainline] [LoadIntoEnvironment secure]\n\
         After.lua\n",
    );
    let mut expected = vec![PathBuf::from("Before.lua")];
    if matches_active_game_type("mainline") {
        expected.extend([
            PathBuf::from("Shared/TargetFrameAuraContainer.lua"),
            PathBuf::from("Shared/TargetFrameAuraButton.lua"),
        ]);
        assert_eq!(toc.file_use_secure_env(1), Some(true));
        assert_eq!(toc.file_use_secure_env(2), Some(true));
    }
    expected.push(PathBuf::from("After.lua"));
    assert_eq!(toc.files, expected);
    assert_eq!(toc.file_use_secure_env(0), None);
    assert_eq!(toc.file_use_secure_env(toc.files.len() - 1), None);
}

#[test]
fn test_parse_multiannotation_permutations_preserve_flags_and_order() {
    let active = active_game_types()[0];
    let annotations = [
        format!("[AllowLoadGameType {active}]"),
        "[LoadIntoEnvironment secure]".to_string(),
        "[Bootstrap]".to_string(),
        "[ExcludeLoadGameType plunderstorm]".to_string(),
        "[AllowLoadEnvironment secure]".to_string(),
    ];
    for reverse in [false, true] {
        for rotation in 0..annotations.len() {
            let mut ordered = annotations.clone();
            ordered.rotate_left(rotation);
            if reverse {
                ordered.reverse();
            }
            let suffix = ordered.join(" \t");
            let toc = TocFile::parse(
                Path::new("/addons/AnnotationProbe"),
                &format!("Before.lua\n[Game]\\Bootstrap.lua\t\t{suffix}\nAfter.lua\n"),
            );
            assert_eq!(
                toc.files,
                vec![
                    PathBuf::from("Before.lua"),
                    PathBuf::from(format!("{}/Bootstrap.lua", game_subdir())),
                    PathBuf::from("After.lua"),
                ],
                "{suffix}",
            );
            assert_eq!(toc.file_is_bootstrap, vec![false, true, false]);
            assert_eq!(toc.file_env_overrides, vec![None, Some(true), None]);
            assert_eq!(toc.file_env_allows, vec![None, Some(true), None]);
        }
    }
}

#[test]
fn test_parse_multiannotations_preserve_game_type_exclusions() {
    let active = active_game_types()[0];
    let toc = TocFile::parse(
        Path::new("/addons/AnnotationProbe"),
        &format!(
            "Before.lua\n\
             Excluded.lua\t[LoadIntoEnvironment secure] [AllowLoadGameType {active}] [ExcludeLoadGameType {active}] [Bootstrap]\n\
             OtherGame.lua\t[Bootstrap] [AllowLoadGameType plunderstorm] [LoadIntoEnvironment secure]\n\
             Public.lua\t[LoadIntoEnvironment global] [AllowLoadGameType {active}]\n\
             After.lua\n"
        ),
    );
    assert_eq!(
        toc.files,
        vec![
            PathBuf::from("Before.lua"),
            PathBuf::from("Public.lua"),
            PathBuf::from("After.lua")
        ],
    );
    assert_eq!(toc.file_env_overrides, vec![None, Some(false), None]);
    assert_eq!(toc.file_is_bootstrap, vec![false, false, false]);
}

#[test]
fn test_parse_bootstrap_annotation_keeps_regular_file_order() {
    let contents = r#"
## Title: TestAddon
Core.lua
Bootstrap.lua [Bootstrap]
After.lua
"#;
    let toc = TocFile::parse(Path::new("/addons/TestAddon"), contents);

    assert_eq!(
        toc.files,
        vec![
            PathBuf::from("Core.lua"),
            PathBuf::from("Bootstrap.lua"),
            PathBuf::from("After.lua"),
        ]
    );
    assert!(!toc.file_is_bootstrap(0));
    assert!(toc.file_is_bootstrap(1));
    assert!(!toc.file_is_bootstrap(2));
    assert_eq!(
        toc.bootstrap_files()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>(),
        vec![PathBuf::from("Bootstrap.lua")]
    );
}

#[test]
fn test_parse_bootstrap_annotation_keeps_path_substitutions_and_filters() {
    let contents = r#"
## Title: TestAddon
[Game]\Bootstrap.lua [Bootstrap]
Ignored.lua [Bootstrap] [AllowLoadGameType plunderstorm]
"#;
    let toc = TocFile::parse(Path::new("/addons/TestAddon"), contents);

    assert_eq!(
        toc.files,
        vec![PathBuf::from(format!("{}/Bootstrap.lua", game_subdir()))]
    );
    assert!(toc.file_is_bootstrap(0));
    assert_eq!(
        toc.bootstrap_files()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>(),
        vec![PathBuf::from(format!("{}/Bootstrap.lua", game_subdir()))]
    );
}

#[test]
fn test_parse_load_into_environment_annotations() {
    let contents = r#"
## Title: TestAddon
Core.lua
Restricted.lua [LoadIntoEnvironment secure]
Public.lua [LoadIntoEnvironment global]
"#;
    let toc = TocFile::parse(Path::new("/addons/TestAddon"), contents);

    assert_eq!(toc.files.len(), 3);
    assert_eq!(toc.files[0], PathBuf::from("Core.lua"));
    assert_eq!(toc.files[1], PathBuf::from("Restricted.lua"));
    assert_eq!(toc.files[2], PathBuf::from("Public.lua"));
    assert_eq!(toc.file_use_secure_env(0), None);
    assert_eq!(toc.file_use_secure_env(1), Some(true));
    assert_eq!(toc.file_use_secure_env(2), Some(false));
}

#[test]
fn test_parse_allow_load_environment_annotations_as_pass_filters() {
    let contents = r#"
## Title: TestAddon
Core.lua
Restricted.lua [AllowLoadEnvironment secure]
Public.lua [AllowLoadEnvironment global]
Debug.lua [AllowLoadEnvironment Global, SomeFlag]
"#;
    let toc = TocFile::parse(Path::new("/addons/TestAddon"), contents);

    assert_eq!(toc.files.len(), 4);
    assert_eq!(toc.files[0], PathBuf::from("Core.lua"));
    assert_eq!(toc.files[1], PathBuf::from("Restricted.lua"));
    assert_eq!(toc.files[2], PathBuf::from("Public.lua"));
    assert_eq!(toc.files[3], PathBuf::from("Debug.lua"));
    assert_eq!(toc.file_use_secure_env(0), None);
    assert_eq!(toc.file_use_secure_env(1), None);
    assert_eq!(toc.file_use_secure_env(2), None);
    assert_eq!(toc.file_use_secure_env(3), None);
    assert_eq!(toc.file_allow_load_environment(0), None);
    assert_eq!(toc.file_allow_load_environment(1), Some(true));
    assert_eq!(toc.file_allow_load_environment(2), Some(false));
    assert_eq!(toc.file_allow_load_environment(3), None);
    assert!(toc.file_allows_environment(1, true));
    assert!(!toc.file_allows_environment(1, false));
    assert!(toc.file_allows_environment(2, false));
    assert!(!toc.file_allows_environment(2, true));
}

#[test]
fn test_family_placeholder_resolves_to_mainline() {
    let contents = r#"
## Title: Blizzard_Colors
Shared\ColorOverrides.lua
[Family]\ColorConstants.lua
[Family]\ColorManager.lua
"#;
    let toc = TocFile::parse(Path::new("/addons/Blizzard_Colors"), contents);

    assert_eq!(toc.files.len(), 3);
    assert_eq!(toc.files[0], PathBuf::from("Shared/ColorOverrides.lua"));
    assert_eq!(toc.files[1], PathBuf::from("Mainline/ColorConstants.lua"));
    assert_eq!(toc.files[2], PathBuf::from("Mainline/ColorManager.lua"));
}

#[test]
fn test_game_type_filter_skips_plunderstorm() {
    let contents = r#"
## Title: Blizzard_FrameXMLBase
Constants.lua
[Game]\GameModeConstants.lua [AllowLoadGameType plunderstorm]
"#;
    let toc = TocFile::parse(Path::new("/addons/Blizzard_FrameXMLBase"), contents);

    assert_eq!(toc.files.len(), 1);
    assert_eq!(toc.files[0], PathBuf::from("Constants.lua"));
}

#[test]
fn test_game_type_filter_allows_mainline_and_standard() {
    let contents = r#"
## Title: TestAddon
Core.lua
Mainline\Override.lua [AllowLoadGameType mainline]
Standard\Mode.lua [AllowLoadGameType standard]
Standard\Multi.lua [AllowLoadGameType standard, wowhack, plunderstorm]
WoWLabs\Mode.lua [AllowLoadGameType plunderstorm]
Classic\Mode.lua [AllowLoadGameType classic]
Cata\Mode.lua [AllowLoadGameType wrath, cata, mists]
"#;
    let toc = TocFile::parse(Path::new("/addons/TestAddon"), contents);

    assert_eq!(toc.files.len(), 4);
    assert_eq!(toc.files[0], PathBuf::from("Core.lua"));
    assert_eq!(toc.files[1], PathBuf::from("Mainline/Override.lua"));
    assert_eq!(toc.files[2], PathBuf::from("Standard/Mode.lua"));
    assert_eq!(toc.files[3], PathBuf::from("Standard/Multi.lua"));
}

#[test]
fn test_is_allowed_game_type() {
    assert!(is_allowed_game_type("Core.lua"));
    assert!(is_allowed_game_type(
        "File.lua [AllowLoadGameType mainline]"
    ));
    assert!(is_allowed_game_type(
        "File.lua [AllowLoadGameType standard]"
    ));
    assert!(is_allowed_game_type(
        "File.lua [AllowLoadGameType standard, wowhack]"
    ));
    assert!(is_allowed_game_type(
        "File.lua [AllowLoadGameType vanilla tbc mainline]"
    ));
    assert!(!is_allowed_game_type(
        "File.lua [AllowLoadGameType plunderstorm]"
    ));
    assert!(!is_allowed_game_type(
        "File.lua [AllowLoadGameType classic]"
    ));
    assert!(!is_allowed_game_type(
        "File.lua [AllowLoadGameType wrath, cata, mists]"
    ));
}

#[test]
#[cfg(feature = "client-mists")]
fn mists_allows_mainline_game_menu_shared_files() {
    let contents = r#"
## Title: Blizzard_GameMenu
Shared\GameMenuFrame.lua [AllowLoadGameType standard, wowhack]
Shared\GameMenuFrame.xml [AllowLoadGameType standard, wowhack]
WoWLabs\GameMenuFrame.lua [AllowLoadGameType plunderstorm]
Shared\Localization.lua
"#;
    let toc = TocFile::parse(Path::new("/addons/Blizzard_GameMenu"), contents);

    assert_eq!(toc.files.len(), 3);
    assert_eq!(toc.files[0], PathBuf::from("Shared/GameMenuFrame.lua"));
    assert_eq!(toc.files[1], PathBuf::from("Shared/GameMenuFrame.xml"));
    assert_eq!(toc.files[2], PathBuf::from("Shared/Localization.lua"));
}

#[test]
fn test_is_game_type_restricted() {
    let plunderstorm = TocFile::parse(
        Path::new("/addons/Test"),
        "## AllowLoadGameType: plunderstorm\nCore.lua",
    );
    assert!(plunderstorm.is_game_type_restricted());

    let mainline = TocFile::parse(
        Path::new("/addons/Test"),
        "## AllowLoadGameType: mainline\nCore.lua",
    );
    assert!(!mainline.is_game_type_restricted());

    let standard = TocFile::parse(
        Path::new("/addons/Test"),
        "## AllowLoadGameType: standard\nCore.lua",
    );
    assert!(!standard.is_game_type_restricted());

    let mixed = TocFile::parse(
        Path::new("/addons/Test"),
        "## AllowLoadGameType: plunderstorm, wowhack\nCore.lua",
    );
    assert!(mixed.is_game_type_restricted());

    let no_restriction = TocFile::parse(Path::new("/addons/Test"), "## Title: TestAddon\nCore.lua");
    assert!(!no_restriction.is_game_type_restricted());
}

#[test]
fn test_packager_debug_block_interface_version() {
    // BlizzMove-style TOC: template Interface line skipped, debug block wins
    let contents = r#"
## Interface: @toc-version-midnight@, @toc-version-retail@, @toc-version-classic@
#@debug@
## Interface: 120000
#@end-debug@
## Title: BlizzMove
## Version: @project-version@
Core.lua
"#;
    let toc = TocFile::parse(Path::new("/addons/BlizzMove"), contents);
    // Template-only Interface line is skipped; debug block provides version
    assert_eq!(toc.interface_versions(), vec![120000]);
    // @project-version@ replaced with "dev"
    assert_eq!(toc.metadata.get("Version").map(|s| s.as_str()), Some("dev"));
    assert_eq!(toc.files.len(), 1);
}

#[test]
fn test_packager_mixed_interface_version_kept() {
    // If Interface has at least one plain number alongside templates, keep it
    let contents = r#"
## Interface: @toc-version-retail@, 110000
Core.lua
"#;
    let toc = TocFile::parse(Path::new("/addons/TestAddon"), contents);
    // Mixed value retained; non-numeric tokens dropped by interface_versions()
    assert_eq!(toc.interface_versions(), vec![110000]);
}

#[test]
fn test_is_all_template_versions() {
    assert!(is_all_template_versions(
        "@toc-version-retail@, @toc-version-cata@"
    ));
    assert!(is_all_template_versions("@toc-version-retail@"));
    assert!(!is_all_template_versions("110000"));
    assert!(!is_all_template_versions("@toc-version-retail@, 110000"));
    assert!(!is_all_template_versions(""));
    assert!(!is_all_template_versions("@project-version@"));
}

#[test]
fn test_allows_screen_modes() {
    use crate::screen::ScreenKind;

    let both = TocFile::parse(Path::new("/addons/Both"), "## AllowLoad: Both\nCore.lua");
    assert!(both.allows_screen(ScreenKind::Game));
    assert!(both.allows_screen(ScreenKind::Login));
    assert!(both.allows_screen(ScreenKind::CharacterSelect));

    let game = TocFile::parse(Path::new("/addons/Game"), "## AllowLoad: Game\nCore.lua");
    assert!(game.allows_screen(ScreenKind::Game));
    assert!(!game.allows_screen(ScreenKind::Login));
    assert!(!game.allows_screen(ScreenKind::CharacterSelect));

    let glue = TocFile::parse(Path::new("/addons/Glue"), "## AllowLoad: Glue\nCore.lua");
    assert!(!glue.allows_screen(ScreenKind::Game));
    assert!(glue.allows_screen(ScreenKind::Login));
    assert!(glue.allows_screen(ScreenKind::CharacterSelect));

    let unrestricted = TocFile::parse(
        Path::new("/addons/Unrestricted"),
        "## Title: TestAddon\nCore.lua",
    );
    assert!(unrestricted.allows_screen(ScreenKind::Game));
    assert!(!unrestricted.allows_screen(ScreenKind::Login));
    assert!(!unrestricted.allows_screen(ScreenKind::CharacterSelect));
}
