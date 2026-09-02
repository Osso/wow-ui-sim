use crate::common;

use std::path::PathBuf;

use common::blizzard_addon_harness::{
    with_blizzard_addon_closure, with_blizzard_addon_glue_smoke_shape,
    with_blizzard_addon_smoke_shape,
};
use common::panel_fixtures::{blizzard_ui_dir, clear_recorded_lua_errors, recorded_lua_errors};
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

const ROOT: &str = "Blizzard_AutoComplete";
const ROOT_TOC_FILE: &str = "Blizzard_AutoComplete.toc";
const FONTS_SHARED: &str = "Blizzard_Fonts_Shared";

#[test]
fn blizzard_auto_complete_loads_without_ingestion_errors() {
    common::with_perf_lock(|| {
        common::with_timeout(240, || {
            with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
                clear_recorded_lua_errors(env);

                let (loaded, reason): (bool, Option<String>) = env
                    .eval(r#"return C_AddOns.LoadAddOn("Blizzard_AutoComplete")"#)
                    .expect("C_AddOns.LoadAddOn should return");
                assert!(loaded, "`{ROOT}` should load: {reason:?}");

                let errors = recorded_lua_errors(env);
                assert!(
                    errors.is_empty(),
                    "`{ROOT}` emitted Lua errors during load:\n{}",
                    errors.join("\n")
                );
            });
        });
    });
}

#[test]
fn blizzard_auto_complete_allowload_both_loads_in_game_and_glue_scopes() {
    assert_toc_allows_game_and_glue_scopes();

    common::with_perf_lock(|| {
        common::with_timeout(240, || {
            with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, loaded| {
                assert_loaded_in_scope(env, loaded, false, "game");
            });

            with_blizzard_addon_glue_smoke_shape(&[ROOT], &[], |env, loaded| {
                assert_loaded_in_scope(env, loaded, true, "glue");
            });
        });
    });
}

#[test]
fn blizzard_auto_complete_fonts_shared_loads_before_autocomplete() {
    assert_toc_declares_fonts_shared_dep();

    common::with_perf_lock(|| {
        common::with_timeout(240, || {
            with_blizzard_addon_closure(&[ROOT], &[], |env, loaded| {
                assert_loaded_before(loaded, FONTS_SHARED, ROOT);

                let (fonts_shared_loaded, autocomplete_loaded): (bool, bool) = env
                    .eval(
                        r#"
                        return C_AddOns.IsAddOnLoaded("Blizzard_Fonts_Shared"),
                            C_AddOns.IsAddOnLoaded("Blizzard_AutoComplete")
                        "#,
                    )
                    .expect("game closure loaded-state probe should return");
                assert!(
                    fonts_shared_loaded,
                    "`{FONTS_SHARED}` must be loaded before `{ROOT}` evaluates"
                );
                assert!(
                    autocomplete_loaded,
                    "`{ROOT}` must be loaded by the game closure"
                );
            });
        });
    });
}

fn assert_toc_declares_fonts_shared_dep() {
    let toc = load_root_toc();
    assert_eq!(
        toc.dependencies(),
        [FONTS_SHARED],
        "`{ROOT}` must keep its sole current `## Dep: {FONTS_SHARED}` declaration"
    );
}

fn assert_toc_allows_game_and_glue_scopes() {
    let toc = load_root_toc();
    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`{ROOT}` has `## AllowLoad: Both`, so game-scope discovery must include it"
    );
    assert!(
        toc.allows_screen(ScreenKind::CharacterSelect),
        "`{ROOT}` has `## AllowLoad: Both`, so glue-scope discovery must include it"
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

fn assert_loaded_in_scope(
    env: &wow_ui_sim::lua_api::WowLuaEnv,
    loaded: &[String],
    expected_in_glue: bool,
    scope: &str,
) {
    assert!(
        loaded.iter().any(|name| name == ROOT),
        "`{ROOT}` must load in the {scope} closure. Loaded set: {loaded:?}"
    );

    let is_loaded: bool = env
        .eval(r#"return C_AddOns.IsAddOnLoaded("Blizzard_AutoComplete")"#)
        .unwrap_or_else(|err| panic!("C_AddOns.IsAddOnLoaded must run in {scope}: {err}"));
    assert!(
        is_loaded,
        "`{ROOT}` must be reported loaded in the {scope} scope"
    );

    let in_glue: bool = env
        .eval("return InGlue()")
        .unwrap_or_else(|err| panic!("InGlue probe must run in {scope}: {err}"));
    assert_eq!(
        in_glue, expected_in_glue,
        "test harness must execute the expected {scope} branch"
    );

    let errors = recorded_lua_errors(env);
    assert!(
        errors.is_empty(),
        "`{ROOT}` emitted Lua errors in the {scope} scope:\n{}",
        errors.join("\n")
    );
}

fn assert_loaded_before(loaded: &[String], before: &str, after: &str) {
    let before_index = loaded
        .iter()
        .position(|name| name == before)
        .unwrap_or_else(|| panic!("`{before}` must appear in the loaded closure: {loaded:?}"));
    let after_index = loaded
        .iter()
        .position(|name| name == after)
        .unwrap_or_else(|| panic!("`{after}` must appear in the loaded closure: {loaded:?}"));
    assert!(
        before_index < after_index,
        "`{before}` must load before `{after}`. Loaded closure: {loaded:?}"
    );
}
