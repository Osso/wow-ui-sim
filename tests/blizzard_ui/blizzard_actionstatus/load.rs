//! Load smoke for `Blizzard_ActionStatus`.
//!
//! TOC reference (`Interface/BlizzardUI/Blizzard_ActionStatus/
//! Blizzard_ActionStatus.toc`):
//!
//! ```text
//! ## Title: Blizzard_ActionStatus
//! ## Dep: Blizzard_FrameXML [AllowLoad game]
//! ## Dep: Blizzard_GlueXML [AllowLoad glue]
//! ## AllowLoad: Both
//! ```

use crate::common::blizzard_addon_harness::{
    with_blizzard_addon_glue_smoke_shape, with_blizzard_addon_smoke_shape,
};
use crate::common::panel_fixtures::{blizzard_ui_dir, recorded_lua_errors};
use std::path::PathBuf;
use wow_ui_sim::loader::find_toc_file;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

const ROOT: &str = "Blizzard_ActionStatus";
const ROOT_TOC_FILE: &str = "Blizzard_ActionStatus.toc";
const REQUIRED_DEPS: [&str; 2] = ["Blizzard_FrameXML", "Blizzard_GlueXML"];
const RAW_GAME_DEP_LINE: &str = "## Dep: Blizzard_FrameXML [AllowLoad game]";
const RAW_GLUE_DEP_LINE: &str = "## Dep: Blizzard_GlueXML [AllowLoad glue]";
const EXPECTED_BODY: [&str; 2] = ["Mainline/ActionStatus.lua", "Mainline/ActionStatus.xml"];

#[test]
fn action_status_loads_with_current_required_deps_and_no_lua_errors() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, loaded| {
        assert_loaded(loaded, "game");

        assert_toc_declares_current_required_deps();

        assert_no_lua_errors(env, "game");
    });
}

#[test]
fn action_status_allowload_both_loads_in_game_and_glue_screens() {
    assert_toc_allows_game_and_glue_screens();

    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, loaded| {
        assert_loaded(loaded, "game");
        assert_in_glue(env, false, "game");
        assert_no_lua_errors(env, "game");
    });

    with_blizzard_addon_glue_smoke_shape(&[ROOT], &[], |env, loaded| {
        assert_loaded(loaded, "glue");
        assert_in_glue(env, true, "glue");
        assert_no_lua_errors(env, "glue");
    });
}

fn assert_toc_declares_current_required_deps() {
    let toc = load_root_toc();

    assert_eq!(
        toc.dependencies(),
        REQUIRED_DEPS,
        "`{ROOT}` must declare the current game/glue scoped dependencies"
    );
    assert_eq!(
        toc.metadata.get("AllowLoad"),
        Some(&"Both".to_string()),
        "`{ROOT}` must allow both game and glue screens"
    );
    assert_eq!(
        toc.metadata.get("AllowLoadGameType"),
        Some(&"mainline".to_string()),
        "`{ROOT}` must be limited to the current mainline game type"
    );
    let body: Vec<String> = toc
        .files
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        body,
        EXPECTED_BODY,
        "`{ROOT}` must retain its current Mainline Lua/XML body order"
    );

    let toc_path = root_toc_path();
    let raw_toc = std::fs::read_to_string(&toc_path).unwrap_or_else(|err| {
        panic!(
            "TOC at `{}` MUST be readable for raw dependency verification: {err}",
            toc_path.display()
        )
    });

    for line in [RAW_GAME_DEP_LINE, RAW_GLUE_DEP_LINE] {
        assert!(
            raw_toc.contains(line),
            "`{ROOT}` must keep the scoped dependency line `{line}`. Raw TOC:\n{raw_toc}"
        );
    }
}

fn assert_toc_allows_game_and_glue_screens() {
    let toc = load_root_toc();
    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`{ROOT}` has `## AllowLoad: Both`, so the game-screen closure must discover it"
    );
    assert!(
        toc.allows_screen(ScreenKind::CharacterSelect),
        "`{ROOT}` has `## AllowLoad: Both`, so the glue-screen closure must discover it"
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
    let addon_dir = blizzard_ui_dir().join(ROOT);
    let toc_path = find_toc_file(&addon_dir).unwrap_or_else(|| {
        panic!(
            "`{ROOT}` must have a compatible TOC under {}",
            addon_dir.display()
        )
    });
    assert_eq!(
        toc_path.file_name().and_then(|name| name.to_str()),
        Some(ROOT_TOC_FILE),
        "loader discovery must select the current retail ActionStatus TOC"
    );
    toc_path
}

fn assert_loaded(loaded: &[String], screen: &str) {
    assert!(
        loaded.iter().any(|name| name == ROOT),
        "`{ROOT}` must load in the {screen} closure. Loaded set: {loaded:?}"
    );
}

fn assert_in_glue(env: &wow_ui_sim::lua_api::WowLuaEnv, expected: bool, screen: &str) {
    let in_glue: bool = env
        .eval("return InGlue()")
        .unwrap_or_else(|err| panic!("InGlue probe must run cleanly in {screen} screen: {err}"));
    assert_eq!(
        in_glue, expected,
        "test harness must execute the {screen} branch with the expected InGlue() value"
    );
}

fn assert_no_lua_errors(env: &wow_ui_sim::lua_api::WowLuaEnv, screen: &str) {
    let errors = recorded_lua_errors(env);
    assert!(
        errors.is_empty(),
        "`{ROOT}` must emit zero recorded Lua errors in the {screen} closure. Got:\n  {}",
        errors.join("\n  ")
    );
}
