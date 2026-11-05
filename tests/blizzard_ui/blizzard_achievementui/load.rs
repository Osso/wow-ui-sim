//! Load contracts for current retail `Blizzard_AchievementUI`.
//!
//! The active profile's `Blizzard_AchievementUI` TOC loads on demand in
//! the mainline game client and declares `Blizzard_FrameXMLUtil`.
//! Its Plunderstorm dependency is game-type-filtered. Its bootstrap precedes the Lua, XML,
//! and localization files.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use crate::common::panel_fixtures::blizzard_ui_dir;
use wow_ui_sim::loader::find_toc_file;
use wow_ui_sim::toc::TocFile;

const ROOT: &str = "Blizzard_AchievementUI";
const LANE_FILE_SCOPE_MIXINS: &[&str] = &[
    "AchievementCategoryTemplateMixin",
    "AchievementCategoryTemplateButtonMixin",
    "AchievementTemplateMixin",
    "AchivementButtonCheckMixin",
    "AchievementsObjectivesMixin",
    "AchievementStatTemplateMixin",
    "AchievementMetaCriteriaMixin",
    "AchievementComparisonTemplateMixin",
    "AchivementComparisonStatMixin",
    "AchievementFullSearchResultsButtonMixin",
];
const LANE_FILE_SCOPE_DISPATCH_TABLES: &[&str] = &[
    "ACHIEVEMENT_FUNCTIONS",
    "GUILD_ACHIEVEMENT_FUNCTIONS",
    "STAT_FUNCTIONS",
    "COMPARISON_ACHIEVEMENT_FUNCTIONS",
    "COMPARISON_STAT_FUNCTIONS",
    "AchievementFrameFilterStrings",
    "AchievementFrameFilters",
];
const LANE_FILE_SCOPE_PANEL_REGISTRATION_KEY: &str = "AchievementFrame";

#[cfg(feature = "optional-tests")]
#[test]
fn achievement_ui_load_emits_no_lane_specific_lua_errors() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        assert(AchievementFrame and AchievementFrameCategories)
        assert(AchievementFrameCategoriesContainer.buttons[1])
        assert(AchievementFrameAchievements.buttons[1])
        assert(AchievementFrameComparisonContainer.buttons[1])
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, loaded| {
        assert!(
            loaded.iter().any(|name| name == ROOT),
            "Smoke-shape harness MUST end up loading `{ROOT}` itself when it is the closure root \
             — even though the TOC carries `## LoadOnDemand: 1`, the closure walker chains the \
             LoD pool into the main pool when an LoD addon is requested as a root \
             (src/loader/mod.rs:410). A regression that routed LoD roots away from the closure \
             walker would land here. Loaded set: {loaded:?}"
        );

        let lane_lua_errors: Vec<String> = env
            .state()
            .borrow()
            .lua_errors
            .iter()
            .filter(|message| message.contains("Achievement") || message.contains("Achivement"))
            .cloned()
            .collect();

        assert!(
            lane_lua_errors.is_empty(),
            "Blizzard_AchievementUI emitted lane-specific Lua errors during the smoke-shape \
             closure load. The addon defines AchievementCategoryTemplateMixin / \
             AchievementCategoryTemplateButtonMixin / AchievementTemplateMixin / \
             AchivementButtonCheckMixin / AchievementsObjectivesMixin / \
             AchievementStatTemplateMixin / AchievementMetaCriteriaMixin / \
             AchievementComparisonTemplateMixin / AchivementComparisonStatMixin / \
             AchievementFullSearchResultsButtonMixin at file scope across one Lua file plus its \
             XML sibling — any nil-call, missing global, or template-resolution failure would \
             surface here. The filter matches any error message containing the substring \
             `Achievement` or `Achivement` (the source has typo'd mixin names \
             `AchivementButtonCheckMixin` / `AchivementComparisonStatMixin` at \
             Blizzard_AchievementUI.lua:1629,2908 — preserved verbatim because Blizzard's API \
             contract uses the misspelling). The disjunction covers both file paths \
             (`Interface/BlizzardUI/Blizzard_AchievementUI/...`) and global identifiers \
             (`Achievement*Mixin`, `Achivement*Mixin`). Got:\n  {}",
            lane_lua_errors.join("\n  ")
        );
    });
}

// Retail/PTR filter the inline Plunderstorm dependency; Forever keeps it optional.
const CURRENT_TOC_DEPENDENCIES: &[&str] = match wow_ui_sim::client_profile::ACTIVE {
    wow_ui_sim::client_profile::ClientProfile::Retail
    | wow_ui_sim::client_profile::ClientProfile::Ptr => &["Blizzard_FrameXMLUtil"],
    wow_ui_sim::client_profile::ClientProfile::WowForever => {
        &["Blizzard_FrameXMLUtil", "Blizzard_Plunderstorm"]
    }
    wow_ui_sim::client_profile::ClientProfile::Wrath
    | wow_ui_sim::client_profile::ClientProfile::Mists
    | wow_ui_sim::client_profile::ClientProfile::Era
    | wow_ui_sim::client_profile::ClientProfile::Anniversary => &[],
};
const PANEL_ADDON_DEPENDENCIES: &[&str] = &["Blizzard_FrameXMLUtil"];
const STANDARD_GAME_TYPE_FILTERED_DEPENDENCIES: &[&str] = &["Blizzard_Plunderstorm"];
const CLOSURE_LOADED_ADDONS: &[&str] = &[ROOT];

#[cfg(feature = "optional-tests")]
#[test]
fn achievement_ui_dependency_closure_includes_current_declared_dependencies() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        assert(type(AchievementFrame_OnLoad) == "function")
        assert(type(UIPanelWindows.AchievementFrame) == "table")
        assert(IsAddOnLoaded("Blizzard_AchievementUI"))
        assert(type(PanelTemplates_SetNumTabs) == "function")
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, loaded| {
        for required in PANEL_ADDON_DEPENDENCIES {
            let is_loaded: bool = env
                .eval(&format!(r#"return C_AddOns.IsAddOnLoaded("{required}")"#))
                .expect("panel baseline dependency load-state probe must run cleanly");

            assert!(
                is_loaded,
                "`{required}` is a declared AchievementUI dependency already loaded by \
                 PANEL_ADDONS. Validate its runtime loaded state through C_AddOns rather than \
                 requiring it in this harness's newly-loaded closure result."
            );
        }

        for required in CLOSURE_LOADED_ADDONS {
            assert!(
                loaded.iter().any(|entry| entry == required),
                "The AchievementUI closure must newly load its requested root `{required}`. \
                 `Blizzard_FrameXMLUtil` is already in PANEL_ADDONS, while optional \
                 `Blizzard_Plunderstorm` is excluded by its game-type restriction. Got: {loaded:?}"
            );
        }
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn achievement_ui_load_on_demand_root_executes_file_scope_code() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        assert(type(AchievementFrame_OnLoad) == "function")
        for _, name in ipairs({"ACHIEVEMENT_FUNCTIONS", "STAT_FUNCTIONS", "COMPARISON_ACHIEVEMENT_FUNCTIONS", "COMPARISON_STAT_FUNCTIONS", "AchievementFrameFilters"}) do
            assert(type(_G[name]) == "table", name)
        end
        assert(UIPanelWindows.AchievementFrame.area == "doublewide")
        assert(UIPanelWindows.AchievementFrame.width == 840)
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, loaded| {
        assert!(
            loaded.iter().any(|name| name == ROOT),
            "`{ROOT}` MUST appear in the closure-walked `loaded` set despite carrying \
             `## LoadOnDemand: 1`. The closure walker chains the LoD pool into the main pool \
             when an LoD addon is requested as a root (src/loader/mod.rs:410). A regression \
             that excluded LoD addons from being closure roots would prevent any of this lane's \
             file-scope code from executing — the global-existence assertions below would all \
             fail, but this top-level check pins the root cause. Loaded set: {loaded:?}"
        );

        for mixin_name in LANE_FILE_SCOPE_MIXINS {
            let is_table = env
                .eval::<bool>(&format!(r#"return type({mixin_name}) == "table""#))
                .expect("file-scope mixin type probe must run cleanly");

            assert!(
                is_table,
                "Mixin `{mixin_name}` MUST be a table after the smoke-shape harness loads \
                 `Blizzard_AchievementUI`. Each entry in `LANE_FILE_SCOPE_MIXINS` is declared \
                 via `Mixin = {{}}` at file scope across the lane's single Lua file \
                 (Blizzard_AchievementUI.lua:496,571,1039,1629,1674,2125,2580,2682,2908,3392). \
                 If the LoadOnDemand flag silently skipped the addon's load, the closure walker \
                 would still list it in `loaded` (since LoD routing happens upstream) but the \
                 file chunks would never run — leaving these globals as nil. A nil reading here \
                 means the LoadOnDemand handling in `load_addon` regressed: the addon was \
                 discovered but its file chunks didn't execute. Got \
                 `type({mixin_name}) == \"table\"` returned false."
            );
        }

        for table_name in LANE_FILE_SCOPE_DISPATCH_TABLES {
            let is_table = env
                .eval::<bool>(&format!(r#"return type({table_name}) == "table""#))
                .expect("file-scope dispatch table type probe must run cleanly");

            assert!(
                is_table,
                "Dispatch table `{table_name}` MUST be a table after the smoke-shape harness \
                 loads `Blizzard_AchievementUI`. These tables (declared at \
                 Blizzard_AchievementUI.lua:56,144,149,154,159,164,1846) carry the per-mode \
                 dispatch handlers used by the achievement / guild-achievement / stat / \
                 comparison panels. A nil reading here means the file chunk failed before \
                 reaching the dispatch-table block. Got `type({table_name}) == \"table\"` \
                 returned false."
            );
        }

        let panel_registration_present = env
            .eval::<bool>(&format!(
                r#"return type(UIPanelWindows) == "table"
                    and type(UIPanelWindows["{LANE_FILE_SCOPE_PANEL_REGISTRATION_KEY}"]) == "table""#
            ))
            .expect("UIPanelWindows registration probe must run cleanly");

        assert!(
            panel_registration_present,
            "After loading `Blizzard_AchievementUI`, \
             `UIPanelWindows[\"{LANE_FILE_SCOPE_PANEL_REGISTRATION_KEY}\"]` MUST be a table — \
             populated at Blizzard_AchievementUI.lua:2 via \
             `UIPanelWindows[\"AchievementFrame\"] = {{ area = \"doublewide\", pushable = 0, \
             xoffset = 80, whileDead = 1 }}`. This is the addon's registration with the panel \
             manager (driven by `Blizzard_UIParentPanelManager`, preloaded by the \
             panel-addons baseline). A nil reading would prove either (a) the file chunk \
             aborted at line 2 because `UIPanelWindows` was nil at that moment (panel-addons \
             baseline regressed or load order changed), OR (b) the assignment was moved \
             behind a deferred path (e.g. `OnLoad`) — both meaningful contract changes."
        );
    });
}

/// Verify the parser and closure walker preserve retail's direct TOC dependencies.

#[cfg(feature = "optional-tests")]
#[test]
fn achievement_ui_loaded_set_contains_every_declared_toc_dependency() {
    if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::Wrath {
        let addon = blizzard_ui_dir().join(ROOT);
        let path = wow_ui_sim::loader::find_toc_file(&addon).expect("native AchievementUI TOC");
        let toc = TocFile::from_file(&path).expect("parse native AchievementUI TOC");
        assert!(toc.dependencies().is_empty());
        assert!(toc.optional_deps().is_empty());
        assert!(toc.is_load_on_demand());
    }
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        assert(type(AchievementFrame_OnLoad) == "function")
        assert(type(UIPanelWindows.AchievementFrame) == "table")
        assert(IsAddOnLoaded("Blizzard_AchievementUI"))
        assert(type(PanelTemplates_SetNumTabs) == "function")
        "#,
    ) {
        return;
    }
    let toc_path = find_toc_file(&blizzard_ui_dir().join(ROOT))
        .expect("Blizzard_AchievementUI TOC for active profile");
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

    assert_eq!(
        declared_deps,
        CURRENT_TOC_DEPENDENCIES,
        "`{ROOT}` currently declares these direct dependencies. Update this source \
         contract with the TOC if retail changes it."
    );

    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, loaded| {
        for dep in &declared_deps {
            let is_loaded: bool = env
                .eval(&format!(r#"return C_AddOns.IsAddOnLoaded("{dep}")"#))
                .expect("declared dependency load-state probe must run cleanly");

            if PANEL_ADDON_DEPENDENCIES.contains(&dep.as_str()) {
                assert!(
                    is_loaded,
                    "Declared TOC dependency `{dep}` is already loaded by PANEL_ADDONS, so \
                     C_AddOns.IsAddOnLoaded must report it at runtime."
                );
            } else if STANDARD_GAME_TYPE_FILTERED_DEPENDENCIES.contains(&dep.as_str()) {
                assert!(
                    !is_loaded,
                    "Declared TOC dependency `{dep}` is restricted to the plunderstorm game \
                     type, so the standard retail panel fixture must not load it."
                );
            } else {
                panic!("unclassified declared AchievementUI dependency `{dep}`");
            }
        }

        assert!(
            loaded.iter().any(|name| name == ROOT),
            "The newly-loaded closure must include its requested root `{ROOT}`. Got: {loaded:?}"
        );
    });
}

/// Pin the LoD-trigger contract: `C_AddOns.LoadAddOn` from Lua resolves and
/// loads the addon when called against a fresh env that has NOT pre-loaded it.
///
/// Earlier `achievement_ui_load_on_demand_root_executes_file_scope_code` runs
/// the smoke-shape harness with `&[ROOT]` as roots, which loads the addon via
/// the closure walker — that test pins "LoD as a closure root works", not "LoD
/// as a Lua-driven runtime dispatch works". THIS test exercises the second
/// path: passing `&[]` to the harness loads only the panel-addons baseline
/// (`tests/common/panel_fixtures.rs:53-56`), leaving the AchievementUI TOC
/// discoverable via `addon_base_paths` but unloaded. The Lua call
/// `C_AddOns.LoadAddOn("Blizzard_AchievementUI")` then exercises
/// `c_addons_load_addon` (src/c_api/c_addons.rs:570), which finds the TOC via
/// `find_runtime_addon_toc`, parses it, and runs the file chunks.
#[cfg(feature = "optional-tests")]
#[test]
fn achievement_ui_load_on_demand_triggers_via_lua_load_addon_api() {
    if crate::common::wrath_achievement_fixtures::assert_lazy(
        r#"
        assert(not IsAddOnLoaded("Blizzard_AchievementUI"))
        assert(AchievementFrame == nil)
        local loaded, reason = LoadAddOn("Blizzard_AchievementUI")
        assert(loaded, tostring(reason))
        assert(IsAddOnLoaded("Blizzard_AchievementUI"))
        assert(type(AchievementFrame_OnLoad) == "function")
        assert(AchievementFrame and AchievementFrame.selectedTab == 1)
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[], &[], |env, loaded| {
        assert!(
            !loaded.iter().any(|name| name == ROOT),
            "Pre-condition violated: `{ROOT}` MUST NOT be in the closure-walked `loaded` set \
             when the harness is invoked with empty roots. The smoke-shape harness only loads \
             the closure of `roots` plus the panel-addons baseline — and the panel baseline does \
             not include `{ROOT}`. A non-empty reading here means a baseline pre-load regressed \
             into pulling AchievementUI, which would invalidate this test's LoD-trigger \
             assertion (the pre-loaded addon would short-circuit the LoadAddOn call). Loaded \
             set: {loaded:?}"
        );

        let pre_loaded = env
            .eval::<bool>(&format!(
                r#"return C_AddOns.IsAddOnLoaded("{ROOT}") == true"#
            ))
            .expect("IsAddOnLoaded probe must run cleanly");

        assert!(
            !pre_loaded,
            "Pre-condition violated: `C_AddOns.IsAddOnLoaded(\"{ROOT}\")` returned true BEFORE \
             the LoadAddOn dispatch ran. This means a panel-baseline workaround or runtime \
             preload (e.g. `apply_for_runtime_addon_preload` at src/c_api/c_addons.rs:584) \
             auto-loaded AchievementUI as a side effect — which would short-circuit the \
             LoadAddOn call below and invalidate this test's contract. If this assertion ever \
             trips, audit the panel baseline and any workaround paths for AchievementUI \
             auto-load."
        );

        let load_addon_returned_true = env
            .eval::<bool>(&format!(
                r#"local loaded, _reason = C_AddOns.LoadAddOn("{ROOT}")
                return loaded == true"#
            ))
            .expect("C_AddOns.LoadAddOn dispatch must run cleanly");

        assert!(
            load_addon_returned_true,
            "`C_AddOns.LoadAddOn(\"{ROOT}\")` MUST return `true` when invoked from Lua against \
             a discoverable LoD addon. The dispatch lives at \
             `c_addons_load_addon` (src/c_api/c_addons.rs:570): it locates the TOC via \
             `find_runtime_addon_toc`, parses it via `TocFile::from_file`, and walks deps + \
             foundations recursively before executing the file chunks. A `false` return means \
             one of: (a) `find_runtime_addon_toc` failed to locate the addon (addon_base_paths \
             regression), (b) the TOC parser tripped, (c) a dependency closure walked through \
             a disabled addon, OR (d) `load_addon_from_toc` itself errored. Tear-down line: \
             this is the FIRST gate keeping LoD-only addons usable from runtime Lua code."
        );

        let post_loaded = env
            .eval::<bool>(&format!(
                r#"return C_AddOns.IsAddOnLoaded("{ROOT}") == true"#
            ))
            .expect("post-load IsAddOnLoaded probe must run cleanly");

        assert!(
            post_loaded,
            "After `C_AddOns.LoadAddOn(\"{ROOT}\")` returned true, \
             `C_AddOns.IsAddOnLoaded(\"{ROOT}\")` MUST also return true — the LoadAddOn path \
             ends with `mark_addon_loaded` (src/c_api/c_addons.rs:661), which sets the addon's \
             `loaded` flag. A false reading here means the load completed but the loaded-state \
             bookkeeping wasn't updated — downstream `IsAddOnLoaded` callers would see the addon \
             as not-loaded and re-dispatch LoadAddOn, potentially infinitely."
        );

        let mixin_present = env
            .eval::<bool>(r#"return type(AchievementTemplateMixin) == "table""#)
            .expect("file-scope mixin probe after LoadAddOn must run cleanly");

        assert!(
            mixin_present,
            "After LoadAddOn returned true, the file-scope global \
             `AchievementTemplateMixin` (declared at Blizzard_AchievementUI.lua:1039) MUST be a \
             table. A nil reading here means the addon's loaded-state bookkeeping was updated \
             but its file chunks didn't actually execute — proving LoadAddOn took a fast path \
             past `load_addon_from_toc`'s file-load step. This is a stronger pin than \
             IsAddOnLoaded alone because it surfaces the case where the addon registry says \
             loaded but no Lua code actually ran."
        );
    });
}
