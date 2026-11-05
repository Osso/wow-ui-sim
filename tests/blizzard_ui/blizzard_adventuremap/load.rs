//! Load smoke for `Blizzard_AdventureMap`.
//!
//! TOC reference (`Interface/BlizzardUI/Blizzard_AdventureMap/
//! Blizzard_AdventureMap.toc`):
//!
//! ```text
//! ## Title: Blizzard Adventure Map UI
//! ## RequiredDep: Blizzard_GarrisonTemplates, Blizzard_MapCanvas, Blizzard_SharedMapDataProviders
//! ## LoadOnDemand: 1
//! ```

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use crate::common::panel_fixtures::{blizzard_ui_dir, recorded_lua_errors};
use std::path::PathBuf;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::toc::TocFile;

const ROOT: &str = "Blizzard_AdventureMap";
const ROOT_TOC_FILE: &str = "Blizzard_AdventureMap.toc";
const REQUIRED_DEPS: &[&str] = &[
    "Blizzard_GarrisonTemplates",
    "Blizzard_MapCanvas",
    "Blizzard_SharedMapDataProviders",
];
const MIXIN_TABLES: &[&str] = &[
    "AdventureMapMixin",
    "AdventureMapInsetMixin",
    "AdventureMapQuestChoiceDialogMixin",
    "AdventureMap_QuestChoiceDataProviderMixin",
    "AdventureMap_QuestOfferDataProviderMixin",
    "AdventureMap_ZoneSummaryProviderMixin",
    "AdventureMap_QuestChoicePinMixin",
    "AdventureMap_QuestOfferPinMixin",
    "AdventureMap_FogPinMixin",
    "AdventureMap_ZoneSummaryPinMixin",
    "AdventureMap_ZoneSummaryInsetPinMixin",
    "AdventureMapQuestRewardMixin",
];

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn adventure_map_loads_with_dependency_closure_and_no_lua_errors() {
    assert_toc_declares_required_dependencies();

    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, loaded| {
        assert_loaded(loaded, ROOT);
        for dependency in REQUIRED_DEPS {
            assert_loaded(loaded, dependency);
        }

        let errors = recorded_lua_errors(env);
        assert!(
            errors.is_empty(),
            "`{ROOT}` dependency-closure load must emit zero recorded Lua errors after the \
             startup-shape harness ticks startup events. Got:\n  {}",
            errors.join("\n  ")
        );
    });
}
}

// Wrath has no AdventureMap vendor widget/handler input; mixed and modeled probes remain enabled.
post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn adventure_map_publishes_expected_mixin_tables() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, loaded| {
        assert_loaded(loaded, ROOT);

        let missing_or_wrong_type = probe_missing_mixin_tables(env);
        assert!(
            missing_or_wrong_type.is_empty(),
            "`{ROOT}` must publish all expected mixins as tables. Missing or wrong type: \
             {missing_or_wrong_type:?}"
        );
    });
}
}

fn probe_missing_mixin_tables(env: &WowLuaEnv) -> Vec<String> {
    let mixin_list = MIXIN_TABLES
        .iter()
        .map(|name| format!("{name:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    let probe = format!(
        r#"
        local missing = {{}}
        for _, name in ipairs({{{mixin_list}}}) do
            if type(_G[name]) ~= "table" then
                table.insert(missing, name .. ":" .. type(_G[name]))
            end
        end
        return table.concat(missing, "\n")
        "#
    );
    let missing: String = env
        .eval(&probe)
        .expect("AdventureMap mixin table probe must run cleanly");

    missing
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn assert_toc_declares_required_dependencies() {
    let toc = load_root_toc();
    let dependencies = toc.dependencies();

    assert_eq!(
        dependencies, REQUIRED_DEPS,
        "`{ROOT}` TOC dependency contract changed. Expected required deps: {REQUIRED_DEPS:?}; \
         parsed deps: {dependencies:?}"
    );
}

fn load_root_toc() -> TocFile {
    let toc_path = root_toc_path();
    TocFile::from_file(&toc_path).unwrap_or_else(|err| {
        panic!(
            "TOC at `{}` MUST parse cleanly before the load contract can be checked: {err}",
            toc_path.display()
        )
    })
}

fn root_toc_path() -> PathBuf {
    blizzard_ui_dir().join(ROOT).join(ROOT_TOC_FILE)
}

fn assert_loaded(loaded: &[String], addon: &str) {
    assert!(
        loaded.iter().any(|name| name == addon),
        "`{addon}` must be present in the `{ROOT}` dependency closure. Loaded set: {loaded:?}"
    );
}
