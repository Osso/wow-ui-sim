#![cfg(any(feature = "client-retail", feature = "client-ptr"))]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::paths::default_blizzard_ui_addons_path;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;

fn blizzard_ui_dir() -> PathBuf {
    default_blizzard_ui_addons_path().expect("Blizzard UI cache should be available")
}

fn black_market_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_BlackMarketUI/Blizzard_BlackMarketUI.toc")
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
fn c_black_market_namespace_is_registered() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    let registered: bool = env
        .eval(
            "return type(C_BlackMarket) == 'table' \
             and type(C_BlackMarket.Close) == 'function' \
             and type(C_BlackMarket.RequestItems) == 'function' \
             and type(C_BlackMarket.ItemPlaceBid) == 'function' \
             and type(C_BlackMarket.IsViewOnly) == 'function' \
             and type(C_BlackMarket.GetNumItems) == 'function' \
             and type(C_BlackMarket.GetHotItem) == 'function' \
             and type(C_BlackMarket.GetItemInfoByID) == 'function' \
             and type(C_BlackMarket.GetItemInfoByIndex) == 'function'",
        )
        .expect("namespace probe should succeed");
    assert!(
        registered,
        "C_BlackMarket should expose all 8 functions used by the UI"
    );
}

#[test]
fn black_market_global_strings_exist() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    let strings_present: bool = env
        .eval(
            "return type(BLACK_MARKET_AUCTION_CONFIRMATION) == 'string' \
             and type(BLACK_MARKET_HOT_ITEM_TIME_LEFT) == 'string' \
             and type(BLACK_MARKET_YOUR_BID) == 'string' \
             and type(AUCTION_TIME_LEFT0) == 'string' \
             and type(AUCTION_TIME_LEFT4) == 'string' \
             and type(AUCTION_TIME_LEFT0_DETAIL) == 'string' \
             and type(AUCTION_TIME_LEFT4_DETAIL) == 'string'",
        )
        .expect("global strings probe should succeed");
    assert!(
        strings_present,
        "BLACK_MARKET_* and AUCTION_TIME_LEFT* global strings should be defined"
    );
}

#[test]
fn black_market_defaults_match_view_only_empty_state() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    let view_only: bool = env
        .eval("return C_BlackMarket.IsViewOnly()")
        .expect("IsViewOnly should return a value");
    assert!(!view_only, "IsViewOnly default should report false");

    let num_items: f64 = env
        .eval("return C_BlackMarket.GetNumItems()")
        .expect("GetNumItems should return a number");
    assert_eq!(num_items, 0.0, "GetNumItems default should report 0");
}

prefork_full_ui_case! {
fn blizzard_black_market_ui_loads_without_errors(env: &WowLuaEnv) {

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_addon(&env.loader_env(), &black_market_toc())
        .expect("Blizzard_BlackMarketUI should load via Rust loader");

    let load_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    assert!(
        load_errors.is_empty(),
        "Blizzard_BlackMarketUI emitted Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );

    let frames_present: bool = env
        .eval(
            "return BlackMarketFrame ~= nil \
             and BlackMarketBidPrice ~= nil \
             and BlackMarketMoneyFrame ~= nil \
             and type(BlackMarketItemMixin) == 'table' \
             and type(BlackMarketFrame_OnEvent) == 'function' \
             and type(BlackMarketFrame_OnShow) == 'function' \
             and type(BlackMarketFrame_OnHide) == 'function' \
             and type(BlackMarketScrollFrame_Update) == 'function' \
             and type(BlackMarketFrame_UpdateBidButton) == 'function'",
        )
        .expect("frame/mixin query should succeed");
    assert!(
        frames_present,
        "BlackMarket frames, mixin, and globals should be defined after load"
    );
}
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_black_market_does_not_publish_reversed_hide_wrapper() {
    let env = load_full_game_ui();
    load_addon(&env.loader_env(), &black_market_toc()).expect("BlackMarketUI should load");

    let result: (String, String, bool, bool) = env
        .eval(
            r#"
            BlackMarketFrame:Show()
            local playedSound
            local originalPlaySound = PlaySound
            PlaySound = function(soundKitID)
                playedSound = soundKitID
            end
            HideBlackMarketFrame()
            PlaySound = originalPlaySound
            return type(BlackMarketFrame_Hide), type(HideBlackMarketFrame),
                not BlackMarketFrame:IsShown(),
                playedSound == SOUNDKIT.AUCTION_WINDOW_CLOSE
            "#,
        )
        .expect("black market hide wrapper behavior should be queryable");
    assert_eq!(
        result,
        ("function".to_string(), "function".to_string(), true, true),
        "PTR BlackMarketUI bootstrap.lua:10-16 exposes a working interaction hide delegate"
    );
}

prefork_full_ui_case! {
fn black_market_show_runs_without_errors(env: &WowLuaEnv) {

    load_addon(&env.loader_env(), &black_market_toc()).expect("BlackMarketUI should load");

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    env.exec("BlackMarketFrame_Show(); BlackMarketFrame_Hide()")
        .expect("Show/Hide should run");

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    assert!(
        errors.is_empty(),
        "BlackMarketFrame Show/Hide emitted Lua errors:\n  {}",
        errors.join("\n  ")
    );
}
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn mainline_black_market_bootstrap_hide_wrapper_hides_the_open_frame() {
    let env = load_full_game_ui();
    load_addon(&env.loader_env(), &black_market_toc()).expect("BlackMarketUI should load");

    let result: (String, bool) = env
        .eval(
            r#"
            local frame = BlackMarketFrame
            local received
            local originalHide = HideUIPanel
            HideUIPanel = function(value) received = value end
            HideBlackMarketFrame()
            HideUIPanel = originalHide
            return type(HideBlackMarketFrame), received == frame
            "#,
        )
        .expect("selected BlackMarket bootstrap hide wrapper should run");
    assert_eq!(result, ("function".to_string(), true));
}
