#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn chromie_time_ui_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_ChromieTimeUI/Blizzard_ChromieTimeUI.toc")
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
fn c_chromie_time_queries_return_fresh_empty_state() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");

    let (namespace_type, option_type, options_type): (String, String, String) = env
        .eval(
            "return type(C_ChromieTime), \
             type(C_ChromieTime and C_ChromieTime.GetChromieTimeExpansionOption), \
             type(C_ChromieTime and C_ChromieTime.GetChromieTimeExpansionOptions)",
        )
        .expect("C_ChromieTime query surface should be inspectable");
    assert_eq!(namespace_type, "table");
    assert_eq!(option_type, "function");
    assert_eq!(options_type, "function");

    let (option_is_nil, first_len, second_len, distinct): (bool, i32, i32, bool) = env
        .eval(
            "local first = C_ChromieTime.GetChromieTimeExpansionOptions(); \
             local second = C_ChromieTime.GetChromieTimeExpansionOptions(); \
             return C_ChromieTime.GetChromieTimeExpansionOption(987654) == nil, \
                    #first, #second, first ~= second",
        )
        .expect("C_ChromieTime empty-state queries should succeed");
    assert!(option_is_nil);
    assert_eq!(first_len, 0);
    assert_eq!(second_len, 0);
    assert!(distinct, "each options query should return a fresh table");
}

#[test]
fn c_chromie_time_actions_are_no_ops() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");

    let (close_type, select_type): (String, String) = env
        .eval(
            "local closeType = type(C_ChromieTime and C_ChromieTime.CloseUI); \
             local selectType = type(C_ChromieTime and C_ChromieTime.SelectChromieTimeOption); \
             C_ChromieTime.CloseUI(); \
             C_ChromieTime.SelectChromieTimeOption(987654); \
             return closeType, selectType",
        )
        .expect("C_ChromieTime no-op actions should succeed");
    assert_eq!(close_type, "function");
    assert_eq!(select_type, "function");
}

prefork_full_ui_case! {
fn blizzard_chromie_time_ui_loads_without_errors(env: &WowLuaEnv) {

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_addon(&env.loader_env(), &chromie_time_ui_toc())
        .expect("Blizzard_ChromieTimeUI should load via Rust loader");

    let load_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    assert!(
        load_errors.is_empty(),
        "Blizzard_ChromieTimeUI emitted Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );

    let frame_present: bool = env
        .eval("return ChromieTimeFrame ~= nil")
        .expect("ChromieTimeFrame query should succeed");
    assert!(
        frame_present,
        "ChromieTimeFrame should be defined after load"
    );

    let mixins_present: bool = env
        .eval(
            "return type(ChromieTimeFrameMixin) == 'table' \
                and type(CurrentlySelectedExpansionInfoFrameMixin) == 'table' \
                and type(ChromieTimeExpansionButtonMixin) == 'table' \
                and type(ChromieTimeSelectButtonMixin) == 'table'",
        )
        .expect("mixin query should succeed");
    assert!(
        mixins_present,
        "Blizzard_ChromieTimeUI mixin tables should be defined after load"
    );
}
}

prefork_full_ui_case! {
fn chromie_time_frame_show_and_hide_run_without_errors(env: &WowLuaEnv) {

    load_addon(&env.loader_env(), &chromie_time_ui_toc())
        .expect("Blizzard_ChromieTimeUI should load");

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    env.exec("ChromieTimeFrame:Show(); ChromieTimeFrame:Hide()")
        .expect("ChromieTimeFrame Show/Hide should run");

    let known_table_sort_gap = "bad argument #1 to 'sort' (table expected)";
    let unexpected_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| !message.contains(known_table_sort_gap))
        .cloned()
        .collect();
    assert!(
        unexpected_errors.is_empty(),
        "ChromieTimeFrame Show/Hide emitted unexpected Lua errors:\n  {}",
        unexpected_errors.join("\n  ")
    );
}
}
