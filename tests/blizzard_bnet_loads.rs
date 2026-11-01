#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn bnet_toc() -> PathBuf {
    wow_ui_sim::loader::find_toc_file(&blizzard_ui_dir().join("Blizzard_BNet"))
        .expect("Blizzard_BNet TOC must resolve")
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
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn bnet_toast_global_strings_exist() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    let strings_present: bool = env
        .eval(
            "return type(BN_TOAST_NEW_INVITE) == 'string' \
             and type(BN_TOAST_PENDING_INVITES) == 'string' \
             and type(BN_TOAST_ONLINE) == 'string' \
             and type(BN_TOAST_OFFLINE) == 'string' \
             and type(BN_TOAST_NEW_CLUB_INVITATION) == 'string' \
             and type(TIME_PLAYED_ALERT) == 'string'",
        )
        .expect("global strings probe should succeed");
    assert!(
        strings_present,
        "BN_TOAST_* and TIME_PLAYED_ALERT global strings should be defined"
    );
}

prefork_full_ui_case! {
fn blizzard_bnet_loads_without_errors(env: &WowLuaEnv) {

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_addon(&env.loader_env(), &bnet_toc()).expect("Blizzard_BNet should load via Rust loader");

    let load_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    assert!(
        load_errors.is_empty(),
        "Blizzard_BNet emitted Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );

    let frames_present: bool = env
        .eval(
            "return BNToastFrame ~= nil \
             and TimeAlertFrame ~= nil \
             and type(BNToastMixin) == 'table' \
             and type(BNetTimeAlertMixin) == 'table' \
             and type(BNet_GetClientAtlas) == 'function' \
             and type(BNet_GetClientEmbeddedAtlas) == 'function' \
             and type(BNet_GetBattlenetClientAtlas) == 'function' \
             and type(BNet_GetClientEmbeddedTexture) == 'function'",
        )
        .expect("frame/mixin query should succeed");
    assert!(
        frames_present,
        "BNet frames, mixins, and helper globals should be defined after load"
    );
}
}

prefork_full_ui_case! {
fn bnet_toast_show_and_hide_run_without_errors(env: &WowLuaEnv) {

    load_addon(&env.loader_env(), &bnet_toc()).expect("Blizzard_BNet should load");

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    env.exec("BNToastFrame:Show(); BNToastFrame:Hide()")
        .expect("Show/Hide should run");

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    assert!(
        errors.is_empty(),
        "BNToastFrame Show/Hide emitted Lua errors:\n  {}",
        errors.join("\n  ")
    );
}
}
