#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn load_full_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    wow_ui_sim::xml::register_intrinsic_templates();

    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc_path) in &addons {
        wow_ui_sim::loader::load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

prefork_full_ui_case! {
fn blizzard_battlefield_map_loads_without_errors(env: &WowLuaEnv) {

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    let (loaded, reason): (bool, Option<String>) = env
        .eval("return C_AddOns.LoadAddOn('Blizzard_BattlefieldMap')")
        .expect("LoadAddOn('Blizzard_BattlefieldMap') should return");
    assert!(
        loaded,
        "Blizzard_BattlefieldMap should load successfully: reason={reason:?}"
    );

    let load_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    assert!(
        load_errors.is_empty(),
        "Blizzard_BattlefieldMap emitted Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );

    let frame_present: bool = env
        .eval("return BattlefieldMapFrame ~= nil and BattlefieldMapTab ~= nil")
        .expect("frame globals query should succeed");
    assert!(
        frame_present,
        "BattlefieldMapFrame and BattlefieldMapTab should be created by XML"
    );

    let mixin_loaded: bool = env
        .eval(
            "return type(BattlefieldMapMixin) == 'table' \
             and type(BattlefieldMapMixin.OnLoad) == 'function' \
             and type(BattlefieldMapTabMixin) == 'table'",
        )
        .expect("mixin query should succeed");
    assert!(
        mixin_loaded,
        "BattlefieldMapMixin/BattlefieldMapTabMixin should be defined"
    );
}
}

prefork_full_ui_case! {
fn battlefield_map_show_runs_without_errors(env: &WowLuaEnv) {

    let (loaded, _): (bool, Option<String>) = env
        .eval("return C_AddOns.LoadAddOn('Blizzard_BattlefieldMap')")
        .expect("LoadAddOn return");
    assert!(loaded);

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    env.exec("BattlefieldMapFrame:Show(); BattlefieldMapFrame:Hide()")
        .expect("Show/Hide should run");

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    assert!(
        errors.is_empty(),
        "BattlefieldMapFrame Show/Hide emitted Lua errors:\n  {}",
        errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn battlefield_map_options_persist_after_addon_loaded_event(env: &WowLuaEnv) {

    let (loaded, _): (bool, Option<String>) = env
        .eval("return C_AddOns.LoadAddOn('Blizzard_BattlefieldMap')")
        .expect("LoadAddOn return");
    assert!(loaded);

    let options_present: bool = env
        .eval(
            "return type(BattlefieldMapOptions) == 'table' \
             and BattlefieldMapOptions.locked == true \
             and BattlefieldMapOptions.showPlayers == true \
             and BattlefieldMapOptions.opacity == 0.7",
        )
        .expect("options query should succeed");
    assert!(
        options_present,
        "ADDON_LOADED handler should seed BattlefieldMapOptions defaults"
    );
}
}
