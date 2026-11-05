//! Load smoke for `Blizzard_ActionBar`.
//!
//! TOC reference (active profile's `Blizzard_ActionBar` TOC):
//!
//! ```text
//! ## Title: Blizzard_ActionBar
//! ## Author: Blizzard Entertainment
//! ## DefaultState: enabled
//! ## Dependencies: Blizzard_StoreUI, Blizzard_QuickKeybind, Blizzard_EditMode,
//!                  Blizzard_UIPanels_Game, Blizzard_TextStatusBar, Blizzard_Flyout,
//!                  Blizzard_Colors, Blizzard_HelpPlate, Blizzard_MicroMenu, Blizzard_PingUI,
//!                  Blizzard_GameMenuEsc
//! ## AllowLoad: Game
//! ```
//!
//! Why this lane uses the `with_blizzard_addon_startup_shape` harness rather
//! than the smoke-shape counterpart used by AchievementUI: action bars
//! register many events at OnLoad (`PLAYER_ENTERING_WORLD`,
//! `UPDATE_BINDINGS`, `ACTIONBAR_SLOT_CHANGED`, etc., across
//! `ActionBarMixin:OnLoad` at `Shared/ActionBar.lua`,
//! `ActionBarActionButtonMixin:OnLoad` at `Shared/ActionButton.lua:442`,
//! `MainActionBarMixin:OnLoad` at `Shared/MainActionBar.lua:3`) and rely on
//! `PLAYER_ENTERING_WORLD` to populate their first visual state — startup
//! settling is required before any Lua-error pinning is meaningful. The
//! startup-shape harness invokes `settle_headless_startup` after the closure
//! load, which fires the headless startup-event sequence and lets the OnLoad
//! handlers run to completion.
//!
//! The harness may satisfy a dependency through its panel baseline or through
//! the closure walk. Either route must leave every declared dependency runtime-
//! loaded after startup; the dependency test below pins that observable state.
//!
//! Assertion pinned: loading the startup-shape closure rooted at
//! `Blizzard_ActionBar` completes cleanly with zero lane-specific Lua errors
//! recorded. The lane spans 22 Lua files plus 14 XML siblings; any
//! template-resolution failure (XML inheritance from `ActionButtonTemplate`
//! / `MainActionBarTemplate` / `MultiActionBarTemplate`), nil-call (e.g.
//! `Mixin(...)`) at file scope, or missing global from a panel-baseline
//! gap would surface in `state.lua_errors` and fall through the
//! lane-specific filter below.

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use crate::common::panel_fixtures::blizzard_ui_dir;
use wow_ui_sim::loader::find_toc_file;
use wow_ui_sim::toc::TocFile;

const ROOT: &str = "Blizzard_ActionBar";
const CURRENT_DECLARED_DEPS: &[&str] = match wow_ui_sim::client_profile::ACTIVE {
    wow_ui_sim::client_profile::ClientProfile::Retail
    | wow_ui_sim::client_profile::ClientProfile::Ptr
    | wow_ui_sim::client_profile::ClientProfile::WowForever => &[
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
    ],
    wow_ui_sim::client_profile::ClientProfile::Mists
    | wow_ui_sim::client_profile::ClientProfile::Era
    | wow_ui_sim::client_profile::ClientProfile::Anniversary => &[
        "Blizzard_QuickKeybind",
        "Blizzard_UIParent",
        "Blizzard_EditMode",
        "Blizzard_FrameXML",
        "Blizzard_TextStatusBar",
        "Blizzard_Flyout",
        "Blizzard_Minimap",
        "Blizzard_MicroMenu",
    ],
    // Wrath uses native_dependencies before reading this modern addon snapshot.
    wow_ui_sim::client_profile::ClientProfile::Wrath => &[],
};
const LANE_FILE_SCOPE_MIXINS: &[&str] = &[
    "ActionBarMixin",
    "EditModeActionBarMixin",
    "MainActionBarMixin",
    "ActionBarActionButtonMixin",
    "BaseActionButtonMixin",
    "ActionBarButtonMixin",
    "SmallActionButtonMixin",
];
const LANE_FILE_SCOPE_TABLES: &[&str] = &["ActionButtonUtil", "AssistedCombatManager"];

#[cfg(feature = "optional-tests")]
#[test]
fn action_bar_load_emits_no_lane_specific_lua_errors() {
    wrath_action_case!(callbacks::load_errors);
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, loaded| {
        assert!(
            loaded.iter().any(|name| name == ROOT),
            "Startup-shape harness MUST end up loading `{ROOT}` itself when it is the closure \
             root. The TOC carries `## AllowLoad: Game`, which the closure walker accepts via \
             `allows_screen(Game)`. A regression that filtered the root by AllowLoad would land \
             here. Loaded set: {loaded:?}"
        );

        let lane_lua_errors: Vec<String> = env
            .state()
            .borrow()
            .lua_errors
            .iter()
            .filter(|message| {
                message.contains("ActionBar")
                    || message.contains("ActionButton")
                    || message.contains("MainActionBar")
                    || message.contains("StanceBar")
                    || message.contains("ExtraActionBar")
                    || message.contains("PossessActionBar")
                    || message.contains("PetActionBar")
                    || message.contains("VehicleLeaveButton")
                    || message.contains("StatusTrackingBar")
                    || message.contains("StatusTrackingManager")
                    || message.contains("ExpBar")
                    || message.contains("ReputationBar")
                    || message.contains("AzeriteBar")
                    || message.contains("ArtifactBar")
                    || message.contains("HonorBar")
                    || message.contains("HouseFavorBar")
                    || message.contains("AssistedCombatManager")
                    || message.contains("SpellFlyout")
            })
            .cloned()
            .collect();

        assert!(
            lane_lua_errors.is_empty(),
            "Blizzard_ActionBar emitted lane-specific Lua errors during the startup-shape closure \
             load. The lane spans 22 Lua files (`ActionButtonUtil`, `ActionButtonSpellAlerts`, \
             `AssistedCombatManager`, `StatusTrackingBar`, `ExpBar`, `ReputationBar`, \
             `AzeriteBar`, `ArtifactBar`, `HonorBar`, `HouseFavorBar`, `ActionButton`, \
             `ActionBar`, `MultiActionBars`, `MainActionBar`, `VehicleLeaveButton`, \
             `StatusTrackingManager`, `StanceBar`, `ExtraActionBar`, `PossessActionBar`, \
             `PetActionBar`, `SpellFlyout`, `Localization`) plus 14 XML siblings; any \
             template-resolution failure, nil-call at file scope, or missing global from a \
             panel-baseline gap would surface here. The filter substring-matches the file/global \
             names of every Lua chunk in the lane. Got:\n  {}",
            lane_lua_errors.join("\n  ")
        );
    });
}

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn action_bar_load_executes_file_scope_mixin_declarations() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        for mixin_name in LANE_FILE_SCOPE_MIXINS {
            let is_table = env
                .eval::<bool>(&format!(r#"return type({mixin_name}) == "table""#))
                .expect("file-scope mixin type probe must run cleanly");

            assert!(
                is_table,
                "Mixin `{mixin_name}` MUST be a table after the startup-shape harness loads \
                 `Blizzard_ActionBar`. Each entry in `LANE_FILE_SCOPE_MIXINS` is declared via \
                 `Mixin = {{}}` at file scope across the lane's Lua files \
                 (ActionBar.lua:1/254, MainActionBar.lua:3, ActionButton.lua:442/1500/1603/1625). \
                 If the closure walker silently skipped this addon's load — e.g. because a panel \
                 baseline pre-load shadowed the dep without the closure walker noticing — the \
                 file chunks would never run, leaving these globals as nil. A nil reading here \
                 means the load step regressed: the addon was discovered but its file chunks \
                 didn't execute. Got `type({mixin_name}) == \"table\"` returned false."
            );
        }

        for table_name in LANE_FILE_SCOPE_TABLES {
            let is_table = env
                .eval::<bool>(&format!(r#"return type({table_name}) == "table""#))
                .expect("file-scope table type probe must run cleanly");

            assert!(
                is_table,
                "Table `{table_name}` MUST be a table after the startup-shape harness loads \
                 `Blizzard_ActionBar`. `ActionButtonUtil` is declared at \
                 ActionButtonUtil.lua:10 and `AssistedCombatManager` at \
                 AssistedCombatManager.lua:3 — both as `Name = {{}}` at file scope. A nil \
                 reading here means the file chunk failed before reaching the declaration. Got \
                 `type({table_name}) == \"table\"` returned false."
            );
        }
    });
}
}

/// Pin the TOC-declared dependency contract via TWO independent grounds:
///
/// 1. **Parser-grounded.** Parse the active profile's `Blizzard_ActionBar` TOC via
///    `TocFile::from_file` (the same parser the closure walker uses at
///    `src/loader/mod.rs:454` via `toc.dependencies().chain(toc.optional_deps())`)
///    and assert the parser-extracted dep set matches the active profile's list
///    BIT-FOR-BIT. A regression where Blizzard adds, removes, or reorders a
///    `## Dependencies:` entry would trip this before the runtime-load
///    round-trip matters.
/// 2. **Runtime-loaded round-trip.** After the startup-shape harness runs,
///    assert `C_AddOns.IsAddOnLoaded(dep) == true` for every parser-extracted
///    dep. The closure-walked `loaded` set does NOT contain these deps —
///    they're pre-loaded by the panel-addons baseline at
///    `tests/common/panel_fixtures.rs:104-115` BEFORE the closure walker
///    runs, and the walker skips already-loaded entries when adding to the
///    new closure's `loaded` list. So `IsAddOnLoaded` (which reads the
///    addon registry's loaded-flag, set by `mark_addon_loaded` at
///    `src/c_api/c_addons.rs:661`) is the correct ground for the
///    "dep is satisfied at runtime" contract.
///
/// This test reads runtime-loaded rather than closure-walked state: the
/// closure-walked list contains only addons loaded by this walk, while the
/// registry flag represents what Lua can use after startup.
#[cfg(feature = "optional-tests")]
#[test]
fn action_bar_every_declared_toc_dependency_is_loaded_after_harness_runs() {
    wrath_action_case!(native_dependencies);

    let toc_path = find_toc_file(&blizzard_ui_dir().join(ROOT))
        .expect("Blizzard_ActionBar TOC for active profile");
    let toc = TocFile::from_file(&toc_path).unwrap_or_else(|e| {
        panic!(
            "TOC at `{}` MUST parse cleanly. The closure walker reads this same file via \
             `TocFile::from_file` (src/c_api/c_addons.rs:639) when servicing LoadAddOn — a \
             parser failure here would prove the simulator's runtime LoadAddOn dispatch \
             cannot resolve this addon either. Got: {e}",
            toc_path.display()
        )
    });
    let mut declared_deps: Vec<String> = toc.dependencies();
    declared_deps.extend(toc.optional_deps());

    let current_deps: Vec<String> = CURRENT_DECLARED_DEPS
        .iter()
        .map(|dependency| dependency.to_string())
        .collect();
    assert_eq!(
        declared_deps, current_deps,
        "Parser-extracted dep list from `{ROOT}` MUST match the active profile's TOC \
         BIT-FOR-BIT, including dependency order and any optional dependencies."
    );

    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        for dep in &declared_deps {
            let is_loaded = env
                .eval::<bool>(&format!(
                    r#"return C_AddOns.IsAddOnLoaded("{dep}") == true"#
                ))
                .expect("C_AddOns.IsAddOnLoaded probe must run cleanly");

            assert!(
                is_loaded,
                "Declared TOC dependency `{dep}` (parsed from `{ROOT}` via \
                 `TocFile::from_file`) MUST be runtime-loaded after the startup-shape harness \
                 finishes — `C_AddOns.IsAddOnLoaded(\"{dep}\")` returned false. Each declared \
                 dep is pre-loaded by the panel-addons baseline (cross-references in this \
                 file's module doc) BEFORE the closure walker runs; the walker then sees the \
                 dep as already satisfied and skips re-loading. A false reading here means \
                 either (a) the panel baseline regressed (entry missing or load failure \
                 silently swallowed at `panel_fixtures.rs:111-113`), OR (b) `mark_addon_loaded` \
                 (src/c_api/c_addons.rs:661) didn't run for this dep, OR (c) the TOC's \
                 declared name doesn't match the addon folder name discovered by \
                 `discover_blizzard_addons_for_screen`. Downstream addons inheriting templates \
                 from `{dep}` would fail to resolve in production."
            );
        }
    });
}
