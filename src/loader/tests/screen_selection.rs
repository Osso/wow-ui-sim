use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::loader::{
    BlizzardAddonOverride, discover_blizzard_addon_closure_for_screen,
    discover_blizzard_addon_closure_for_screen_with_overrides, discover_blizzard_addons_for_screen,
};
use crate::screen::ScreenKind;

// These discover actual named addons, not same-named native frames.
// Wrath manifest1-382/cache ships none of their required packaged addons.
post_wrath_ui_fixture! {
fn blizzard_ui_dir() -> PathBuf {
    crate::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn addon_names(screen: ScreenKind) -> Vec<String> {
    discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen)
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

#[test]
fn game_screen_discovers_game_addons_not_glue_base() {
    let addons = addon_names(ScreenKind::Game);
    assert!(addons.iter().any(|name| name == "Blizzard_UIParent"));
    assert!(!addons.iter().any(|name| name == "Blizzard_GlueParent"));
    assert!(!addons.iter().any(|name| name == "Blizzard_GlueXML"));
}

#[test]
fn login_screen_discovers_glue_addons_not_game_base() {
    let addons = addon_names(ScreenKind::Login);
    assert!(addons.iter().any(|name| name == "Blizzard_GlueParent"));
    assert!(addons.iter().any(|name| name == "Blizzard_GlueXML"));
    assert!(!addons.iter().any(|name| name == "Blizzard_UIParent"));
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::WowForever {
        assert!(!addons.iter().any(|name| name == "Blizzard_CharacterCreate"));
        assert!(
            !addons
                .iter()
                .any(|name| name == "Blizzard_CharacterCustomize")
        );
    }
}

#[test]
fn character_select_screen_uses_glue_addon_set() {
    let addons = addon_names(ScreenKind::CharacterSelect);
    assert!(addons.iter().any(|name| name == "Blizzard_GlueParent"));
    assert!(addons.iter().any(|name| name == "Blizzard_GlueXML"));
    assert!(!addons.iter().any(|name| name == "Blizzard_UIParent"));
}

character_create_ui_fixture! {
#[test]
fn character_select_screen_includes_character_create() {
    let addons = addon_names(ScreenKind::CharacterSelect);
    assert!(addons.iter().any(|name| name == "Blizzard_CharacterCreate"));
}
}

#[test]
fn character_create_screen_uses_glue_addon_set() {
    let addons = addon_names(ScreenKind::CharacterCreate);
    assert!(addons.iter().any(|name| name == "Blizzard_GlueParent"));
    assert!(addons.iter().any(|name| name == "Blizzard_GlueXML"));
    assert!(!addons.iter().any(|name| name == "Blizzard_UIParent"));
}

character_create_ui_fixture! {
#[test]
fn character_create_screen_includes_character_create() {
    let addons = addon_names(ScreenKind::CharacterCreate);
    assert!(addons.iter().any(|name| name == "Blizzard_CharacterCreate"));
}
}

}

struct TempBlizzardUiDir {
    path: PathBuf,
}

impl TempBlizzardUiDir {
    fn new(suffix: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("wow-sim-blizzard-ui-{suffix}-{unique}"));
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn add_addon(&self, name: &str, toc_contents: &str) {
        self.add_addon_toc(name, &format!("{name}.toc"), toc_contents);
    }

    fn add_addon_toc(&self, name: &str, toc_name: &str, toc_contents: &str) {
        let addon_dir = self.path.join(name);
        std::fs::create_dir_all(&addon_dir).unwrap();
        std::fs::write(addon_dir.join(toc_name), toc_contents).unwrap();
    }
}

impl Drop for TempBlizzardUiDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[test]
fn load_first_addons_sort_ahead_of_non_load_first_addons() {
    let ui = TempBlizzardUiDir::new("load-first-order");
    ui.add_addon(
        "Blizzard_Z_LoadFirst",
        r#"
## Title: Blizzard_Z_LoadFirst
## AllowLoad: Both
## LoadFirst: 1
Core.lua
"#,
    );
    ui.add_addon(
        "Blizzard_A_Normal",
        r#"
## Title: Blizzard_A_Normal
## AllowLoad: Both
Core.lua
"#,
    );

    let addons: Vec<String> = discover_blizzard_addons_for_screen(&ui.path, ScreenKind::Game)
        .into_iter()
        .map(|(name, _)| name)
        .collect();

    assert_eq!(
        addons,
        vec!["Blizzard_Z_LoadFirst", "Blizzard_A_Normal"],
        "LoadFirst addons should sort before normal addons",
    );
}

#[test]
fn packaged_startup_phases_put_secure_addons_before_packaged_defaults() {
    let ui = TempBlizzardUiDir::new("startup-phases");
    ui.add_addon(
        "Blizzard_Z_LoadFirst",
        "## AllowLoad: Both\n## LoadFirst: 1\nCore.lua\n",
    );
    ui.add_addon(
        "Blizzard_Y_Secure",
        "## AllowLoad: Both\n## UseSecureEnvironment: 1\nCore.lua\n",
    );
    ui.add_addon("Blizzard_A_Packaged", "## AllowLoad: Both\nCore.lua\n");

    let packaged: Vec<String> =
        crate::loader::discover_blizzard_startup_addons_for_screen(&ui.path, ScreenKind::Game)
            .into_iter()
            .map(|addon| addon.name)
            .collect();
    let load_first = packaged
        .iter()
        .position(|name| name == "Blizzard_Z_LoadFirst")
        .unwrap();
    let secure = packaged
        .iter()
        .position(|name| name == "Blizzard_Y_Secure")
        .unwrap();
    let ordinary = packaged
        .iter()
        .position(|name| name == "Blizzard_A_Packaged")
        .unwrap();
    assert!(load_first < ordinary);
    assert!(secure < ordinary);
}

#[test]
fn foundational_addon_names_do_not_create_load_first_ordering() {
    let ui = TempBlizzardUiDir::new("no-foundation-promotion");
    ui.add_addon(
        "Blizzard_Z_LoadFirst",
        "## AllowLoad: Both\n## LoadFirst: 1\nCore.lua\n",
    );
    ui.add_addon("Blizzard_SharedXMLBase", "## AllowLoad: Both\nCore.lua\n");

    let expected = vec!["Blizzard_Z_LoadFirst", "Blizzard_SharedXMLBase"];
    let for_screen: Vec<String> = discover_blizzard_addons_for_screen(&ui.path, ScreenKind::Game)
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    let startup: Vec<String> =
        crate::loader::discover_blizzard_startup_addons_for_screen(&ui.path, ScreenKind::Game)
            .into_iter()
            .map(|addon| addon.name)
            .collect();
    let all: Vec<String> = crate::loader::discover_all_blizzard_addons(&ui.path)
        .into_iter()
        .map(|(name, _)| name)
        .collect();

    assert_eq!(for_screen, expected);
    assert_eq!(startup, expected);
    assert_eq!(all, expected);
}

#[test]
#[cfg(feature = "client-mists")]
fn mists_profile_skips_retail_only_deprecated_arena_ui() {
    let ui = TempBlizzardUiDir::new("mists-retail-only-addon");
    ui.add_addon(
        "Blizzard_Deprecated_ArenaUI",
        r#"
## Title: Blizzard_Deprecated_ArenaUI
## AllowLoad: Game
Deprecated_ArenaUI.xml
"#,
    );
    ui.add_addon(
        "Blizzard_UnitFrame",
        r#"
## Title: Blizzard_UnitFrame
## AllowLoad: Game
UnitFrame.lua
"#,
    );

    let addons: Vec<String> = discover_blizzard_addons_for_screen(&ui.path, ScreenKind::Game)
        .into_iter()
        .map(|(name, _)| name)
        .collect();

    assert_eq!(
        addons,
        vec!["Blizzard_UnitFrame"],
        "Mists should not load Blizzard_Deprecated_ArenaUI from the retail cache because it is absent from the Classic PTR/Mists source tree"
    );
}

#[test]
#[cfg(feature = "client-mists")]
fn mists_profile_uses_mainline_game_menu_fallback() {
    let ui = TempBlizzardUiDir::new("mists-game-menu-mainline");
    ui.add_addon_toc(
        "Blizzard_GameMenu",
        "Blizzard_GameMenu_Mainline.toc",
        r#"
## Title: Blizzard_GameMenu
## AllowLoad: Game
Shared\GameMenuFrame.lua [AllowLoadGameType standard, wowhack]
"#,
    );

    let addons: Vec<(String, String)> =
        discover_blizzard_addons_for_screen(&ui.path, ScreenKind::Game)
            .into_iter()
            .map(|(name, toc_path)| {
                (
                    name,
                    toc_path.file_name().unwrap().to_string_lossy().into_owned(),
                )
            })
            .collect();

    assert_eq!(
        addons,
        vec![(
            "Blizzard_GameMenu".to_string(),
            "Blizzard_GameMenu_Mainline.toc".to_string()
        )],
        "Mists should load the only shipped GameMenu toc so the game menu buttons are real"
    );
}

#[test]
#[cfg(feature = "client-mists")]
fn mists_profile_uses_world_map_mists_toc() {
    let ui = TempBlizzardUiDir::new("mists-world-map-toc");
    ui.add_addon_toc(
        "Blizzard_WorldMap",
        "Blizzard_WorldMap_Mainline.toc",
        r#"
## Title: Blizzard_WorldMap
## AllowLoad: Game
Blizzard_WorldMap.xml
"#,
    );
    ui.add_addon_toc(
        "Blizzard_WorldMap",
        "Blizzard_WorldMap_Mists.toc",
        r#"
## Title: Blizzard_WorldMap
## AllowLoad: Game
## AllowLoadGameType: mists
Cata\Blizzard_WorldMap.xml
"#,
    );

    let addons: Vec<(String, String)> =
        discover_blizzard_addons_for_screen(&ui.path, ScreenKind::Game)
            .into_iter()
            .map(|(name, toc_path)| {
                (
                    name,
                    toc_path.file_name().unwrap().to_string_lossy().into_owned(),
                )
            })
            .collect();

    assert_eq!(
        addons,
        vec![(
            "Blizzard_WorldMap".to_string(),
            "Blizzard_WorldMap_Mists.toc".to_string()
        )],
        "Mists should prefer the shipped Mists WorldMap TOC over the retail Mainline TOC"
    );
}

#[test]
fn deprecated_chat_and_combat_compat_addons_load_after_base_addons() {
    let ui = TempBlizzardUiDir::new("deprecated-compat-order");
    for addon in [
        "Blizzard_DeprecatedChatInfo",
        "Blizzard_ChatFrameBase",
        "Blizzard_DeprecatedCombatLog",
        "Blizzard_CombatLogBase",
    ] {
        ui.add_addon(addon, "## AllowLoad: Game\nCore.lua\n");
    }

    let addons: Vec<String> = discover_blizzard_addons_for_screen(&ui.path, ScreenKind::Game)
        .into_iter()
        .map(|(name, _)| name)
        .collect();

    let index = |name: &str| addons.iter().position(|addon| addon == name).unwrap();
    assert!(index("Blizzard_ChatFrameBase") < index("Blizzard_DeprecatedChatInfo"));
    assert!(index("Blizzard_CombatLogBase") < index("Blizzard_DeprecatedCombatLog"));
}

#[test]
fn deprecated_combat_log_pulls_load_on_demand_combat_log_base() {
    let ui = TempBlizzardUiDir::new("deprecated-combat-log-lod");
    ui.add_addon(
        "Blizzard_DeprecatedCombatLog",
        "## AllowLoad: Game\nDeprecated.lua\n",
    );
    ui.add_addon(
        "Blizzard_CombatLogBase",
        "## AllowLoad: Game\n## LoadOnDemand: 1\nBase.lua\n",
    );

    let addons: Vec<String> = discover_blizzard_addons_for_screen(&ui.path, ScreenKind::Game)
        .into_iter()
        .map(|(name, _)| name)
        .collect();

    assert_eq!(
        addons,
        vec!["Blizzard_CombatLogBase", "Blizzard_DeprecatedCombatLog"],
        "implicit deprecated-combat-log dependency should pull the LOD base addon before loading deprecated globals"
    );
}

#[test]
fn objective_tracker_pulls_poi_button_owner_templates() {
    let ui = TempBlizzardUiDir::new("objective-tracker-poi-button");
    ui.add_addon(
        "Blizzard_ObjectiveTracker",
        "## AllowLoad: Game\nObjectiveTracker.lua\n",
    );
    ui.add_addon(
        "Blizzard_POIButton",
        "## AllowLoad: Game\n## LoadOnDemand: 1\nPOIButtonOwner.xml\n",
    );

    let addons: Vec<String> = discover_blizzard_addons_for_screen(&ui.path, ScreenKind::Game)
        .into_iter()
        .map(|(name, _)| name)
        .collect();

    assert_eq!(
        addons,
        vec!["Blizzard_POIButton", "Blizzard_ObjectiveTracker"],
        "ObjectiveTracker inherits POIButtonOwnerTemplate without declaring the addon"
    );
}

#[test]
fn only_forever_glue_feedback_pulls_customization_before_login_hooks() {
    let ui = TempBlizzardUiDir::new("glue-feedback-customization");
    ui.add_addon(
        "Blizzard_PTRFeedbackGlue",
        "## AllowLoad: Glue\nFeedback.lua\n",
    );
    ui.add_addon(
        "Blizzard_CharacterCustomize",
        "## AllowLoad: Both\n## LoadOnDemand: 1\n## Dependencies: Blizzard_CustomizationUI\nCustomize.xml\n",
    );
    ui.add_addon(
        "Blizzard_CharacterCreate",
        "## AllowLoad: Glue\n## Dependencies: Blizzard_CharacterCustomize\nCreate.xml\n",
    );
    ui.add_addon(
        "Blizzard_CustomizationUI",
        "## AllowLoad: Both\n## LoadOnDemand: 1\nCustomization.xml\n",
    );

    let expected =
        if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::WowForever {
            vec![
                "Blizzard_CustomizationUI",
                "Blizzard_CharacterCustomize",
                "Blizzard_CharacterCreate",
                "Blizzard_PTRFeedbackGlue",
            ]
        } else {
            vec!["Blizzard_PTRFeedbackGlue"]
        };
    let discovered: Vec<String> = discover_blizzard_addons_for_screen(&ui.path, ScreenKind::Login)
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    assert_eq!(discovered, expected);

    let startup =
        crate::loader::discover_blizzard_startup_addons_for_screen(&ui.path, ScreenKind::Login);
    assert!(
        startup
            .iter()
            .all(|addon| addon.kind == crate::loader::StartupAddonLoadKind::Full)
    );
    assert_eq!(
        startup
            .into_iter()
            .map(|addon| addon.name)
            .collect::<Vec<_>>(),
        expected,
    );
}

#[test]
fn cyclic_addons_still_emit_dependencies_before_load_first_addon() {
    let ui = TempBlizzardUiDir::new("load-first-cycle");
    ui.add_addon(
        "Blizzard_A_Normal",
        r#"
## Title: Blizzard_A_Normal
## AllowLoad: Both
## Dependencies: Blizzard_Z_LoadFirst
Core.lua
"#,
    );
    ui.add_addon(
        "Blizzard_Z_LoadFirst",
        r#"
## Title: Blizzard_Z_LoadFirst
## AllowLoad: Both
## LoadFirst: 1
## Dependencies: Blizzard_A_Normal
Core.lua
"#,
    );

    let addons: Vec<String> = discover_blizzard_addons_for_screen(&ui.path, ScreenKind::Game)
        .into_iter()
        .map(|(name, _)| name)
        .collect();

    assert_eq!(
        addons,
        vec!["Blizzard_A_Normal", "Blizzard_Z_LoadFirst"],
        "LoadFirst triggers first-pass loading, but declared dependencies still emit first",
    );
}

#[test]
fn addon_closure_resolves_transitive_optional_dependencies_for_roots() {
    let ui = TempBlizzardUiDir::new("closure");
    ui.add_addon(
        "Blizzard_B",
        r#"
## Title: Blizzard_B
## AllowLoad: Both
Core.lua
"#,
    );
    ui.add_addon(
        "Blizzard_C",
        r#"
## Title: Blizzard_C
## AllowLoad: Both
Core.lua
"#,
    );
    ui.add_addon(
        "Blizzard_D",
        r#"
## Title: Blizzard_D
## AllowLoad: Glue
Core.lua
"#,
    );
    ui.add_addon(
        "Blizzard_A",
        r#"
## Title: Blizzard_A
## AllowLoad: Both
## Dependencies: Blizzard_B
## OptionalDeps: Blizzard_C, Blizzard_D
Core.lua
"#,
    );

    let addons =
        discover_blizzard_addon_closure_for_screen(&ui.path, ScreenKind::Game, &["Blizzard_A"]);
    let names: Vec<String> = addons.into_iter().map(|(name, _)| name).collect();

    assert_eq!(
        names,
        vec!["Blizzard_B", "Blizzard_C", "Blizzard_A"],
        "dependency closure should include required and screen-allowed optional TOC deps only",
    );
    assert!(
        !names.iter().any(|name| name == "Blizzard_D"),
        "screen-filtered optional dependencies should not be included in the closure"
    );
}

#[test]
fn addon_closure_includes_load_on_demand_roots_and_their_dependencies() {
    let ui = TempBlizzardUiDir::new("lod-closure");
    ui.add_addon(
        "Blizzard_Dependency",
        r#"
## Title: Blizzard_Dependency
## AllowLoad: Both
Core.lua
"#,
    );
    ui.add_addon(
        "Blizzard_Optional",
        r#"
## Title: Blizzard_Optional
## AllowLoad: Both
Core.lua
"#,
    );
    ui.add_addon(
        "Blizzard_LoadOnDemandRoot",
        r#"
## Title: Blizzard_LoadOnDemandRoot
## AllowLoad: Both
## LoadOnDemand: 1
## Dependencies: Blizzard_Dependency
## OptionalDeps: Blizzard_Optional
Core.lua
"#,
    );

    let addons = discover_blizzard_addon_closure_for_screen(
        &ui.path,
        ScreenKind::Game,
        &["Blizzard_LoadOnDemandRoot"],
    );
    let names: Vec<String> = addons.into_iter().map(|(name, _)| name).collect();

    assert_eq!(
        names,
        vec![
            "Blizzard_Dependency",
            "Blizzard_Optional",
            "Blizzard_LoadOnDemandRoot",
        ],
        "load-on-demand roots should resolve against the full screen-allowed TOC set",
    );
}

#[test]
fn addon_closure_applies_override_manifest_extras_transitively() {
    let ui = TempBlizzardUiDir::new("closure-overrides");
    ui.add_addon(
        "Blizzard_B",
        r#"
## Title: Blizzard_B
## AllowLoad: Both
Core.lua
"#,
    );
    ui.add_addon(
        "Blizzard_C",
        r#"
## Title: Blizzard_C
## AllowLoad: Both
Core.lua
"#,
    );
    ui.add_addon(
        "Blizzard_A",
        r#"
## Title: Blizzard_A
## AllowLoad: Both
## Dependencies: Blizzard_B
Core.lua
"#,
    );

    let overrides = &[BlizzardAddonOverride {
        addon: "Blizzard_A",
        extra_roots: &["Blizzard_C"],
    }];

    let addons = discover_blizzard_addon_closure_for_screen_with_overrides(
        &ui.path,
        ScreenKind::Game,
        &["Blizzard_A"],
        overrides,
    );
    let names: Vec<String> = addons.into_iter().map(|(name, _)| name).collect();

    assert_eq!(
        names,
        vec!["Blizzard_B", "Blizzard_C", "Blizzard_A"],
        "override manifest extras should participate in the explicit closure",
    );
}

post_wrath_ui_fixture! {
#[test]
fn real_glue_load_first_addons_sort_before_non_load_first_addons() {
    let addons = addon_names(ScreenKind::Login);

    let glue_menu_frame = addons
        .iter()
        .position(|name| name == "Blizzard_GlueMenuFrame")
        .expect("Blizzard_GlueMenuFrame should be present");
    let login_warning_dialogs = addons
        .iter()
        .position(|name| name == "Blizzard_LoginWarningDialogs")
        .expect("Blizzard_LoginWarningDialogs should be present");

    // Pinned `Blizzard_GlueMenuFrame_Classic.toc:4` depends on
    // `Blizzard_LoginWarningDialogs`, so classic profiles must load the
    // dependency first regardless of LoadFirst. The Mainline variant depends on
    // GlueXMLBase/GlueParent only, so LoadFirst then sorts it ahead.
    let classic_dependency = matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Mists
            | crate::client_profile::ClientProfile::Era
            | crate::client_profile::ClientProfile::Anniversary
    );
    if classic_dependency {
        assert!(
            login_warning_dialogs < glue_menu_frame,
            "pinned GlueMenuFrame_Classic.toc:4 dependency must load before the dependent",
        );
    } else {
        assert!(
            glue_menu_frame < login_warning_dialogs,
            "Blizzard_GlueMenuFrame is tagged LoadFirst and should sort before comparable non-LoadFirst glue addons",
        );
    }
}
}
