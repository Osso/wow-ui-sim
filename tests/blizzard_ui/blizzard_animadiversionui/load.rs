//! Load smoke for `Blizzard_AnimaDiversionUI`.
//!
//! TOC reference (`Interface/BlizzardUI/Blizzard_AnimaDiversionUI/
//! Blizzard_AnimaDiversionUI.toc`):
//!
//! ```text
//! ## Title: Blizzard Anima Diversion UI
//! ## LoadOnDemand: 1
//! ```

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use crate::common::panel_fixtures::{blizzard_ui_dir, recorded_lua_errors};
use std::path::PathBuf;
use wow_ui_sim::loader::BlizzardAddonOverride;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::toc::TocFile;

const ROOT: &str = "Blizzard_AnimaDiversionUI";
const ROOT_TOC_FILE: &str = "Blizzard_AnimaDiversionUI.toc";
const IMPLICIT_DEPS: &[&str] = &["Blizzard_MapCanvas", "Blizzard_SharedMapDataProviders"];
const CLOSURE_OVERRIDES: &[BlizzardAddonOverride<'_>] = &[BlizzardAddonOverride {
    addon: ROOT,
    extra_roots: IMPLICIT_DEPS,
}];
const MIXIN_TABLES: &[&str] = &[
    "AnimaDiversionFrameMixin",
    "AnimaDiversionCurrencyFrameMixin",
    "ReinforceProgressFrameMixin",
    "ReinforceInfoFrameMixin",
    "AnimaNodeReinforceButtonMixin",
    "AnimaDiversionDataProviderMixin",
    "AnimaDiversionModelScenePinMixin",
    "AnimaDiversionPinMixin",
    "AnimaDiversionConnectionMixin",
    "AnimaDiversion_WorldQuestDataProviderMixin",
    "AnimaDiversion_WorldQuestPinMixin",
    "AnimaDiversionUtil",
];

#[cfg(feature = "optional-tests")]
#[test]
fn anima_diversion_ui_loads_with_dependency_closure_and_no_lua_errors() {
    assert_toc_declares_no_required_dependencies();

    with_blizzard_addon_startup_shape(&[ROOT], CLOSURE_OVERRIDES, |env, loaded| {
        assert_loaded(loaded);
        assert_implicit_dependencies_loaded(loaded);
        assert_no_lua_errors(env);
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn anima_diversion_ui_publishes_expected_mixin_tables() {
    with_blizzard_addon_startup_shape(&[ROOT], CLOSURE_OVERRIDES, |env, loaded| {
        assert_loaded(loaded);
        assert_implicit_dependencies_loaded(loaded);

        let missing_or_wrong_type = probe_missing_mixin_tables(env);
        assert!(
            missing_or_wrong_type.is_empty(),
            "`{ROOT}` must publish all expected mixins/util tables. Missing or wrong type: \
             {missing_or_wrong_type:?}"
        );
    });
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
        .expect("AnimaDiversionUI mixin table probe must run cleanly");

    missing
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn assert_toc_declares_no_required_dependencies() {
    let toc = load_root_toc();
    assert!(
        toc.dependencies().is_empty(),
        "`{ROOT}` currently declares no required dependencies; this test documents the implicit \
         MapCanvas dependencies separately via closure overrides"
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

fn assert_loaded(loaded: &[String]) {
    assert!(
        loaded.iter().any(|name| name == ROOT),
        "`{ROOT}` must be present in its dependency closure. Loaded set: {loaded:?}"
    );
}

fn assert_implicit_dependencies_loaded(loaded: &[String]) {
    for dependency in IMPLICIT_DEPS {
        assert!(
            loaded.iter().any(|name| name == dependency),
            "`{ROOT}` uses MapCanvas and shared map data-provider mixins at load time, so the harness \
             closure must include implicit dependency `{dependency}`. Loaded set: {loaded:?}"
        );
    }
}

fn assert_no_lua_errors(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let errors = recorded_lua_errors(env);
    assert!(
        errors.is_empty(),
        "`{ROOT}` dependency-closure load must emit zero recorded Lua errors after the \
         startup-shape harness ticks startup events. Got:\n  {}",
        errors.join("\n  ")
    );
}
