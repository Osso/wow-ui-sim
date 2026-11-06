//! Runtime addon dependency screen filtering.

use std::fs;

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn glue_only_required_dependency_is_not_loaded_in_game() {
    let root = tempfile::tempdir().unwrap();
    let game_addon = root.path().join("GameRuntimeProbe");
    fs::create_dir(&game_addon).unwrap();
    fs::write(
        game_addon.join("GameRuntimeProbe.toc"),
        "## Title: GameRuntimeProbe\n## Dep: GlueRuntimeProbe\nGameRuntimeProbe.lua\n",
    )
    .unwrap();
    fs::write(
        game_addon.join("GameRuntimeProbe.lua"),
        "GameRuntimeProbeRan = true",
    )
    .unwrap();
    let glue_addon = root.path().join("GlueRuntimeProbe");
    fs::create_dir(&glue_addon).unwrap();
    fs::write(
        glue_addon.join("GlueRuntimeProbe.toc"),
        "## Title: GlueRuntimeProbe\n## AllowLoad: Glue\nGlueRuntimeProbe.lua\n",
    )
    .unwrap();
    fs::write(
        glue_addon.join("GlueRuntimeProbe.lua"),
        "GlueRuntimeProbeRan = true",
    )
    .unwrap();

    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().addon_base_paths = vec![root.path().to_path_buf()];
    let (loaded, reason): (bool, Option<String>) = env
        .eval("return C_AddOns.LoadAddOn('GameRuntimeProbe')")
        .unwrap();
    assert!(loaded, "{reason:?}");
    let (game_ran, glue_ran, glue_loaded): (bool, bool, bool) = env
        .eval(
            "return GameRuntimeProbeRan == true, GlueRuntimeProbeRan == true, \
             C_AddOns.IsAddOnLoaded('GlueRuntimeProbe')",
        )
        .unwrap();
    assert!(game_ran, "the game addon's own files should run");
    assert!(!glue_ran, "a Glue-only dependency must not run in Game");
    assert!(
        !glue_loaded,
        "a Glue-only dependency must not be marked loaded"
    );
}
