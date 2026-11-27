//! AddOnPerformance initialization behavior for `Blizzard_AddOnPerformance`.

use crate::common::blizzard_addon_harness::{
    load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use crate::common::panel_fixtures::blizzard_ui_dir;
use wow_ui_sim::loader::load_addon;

const ROOT: &str = "Blizzard_AddOnPerformance";
const ROOT_TOC_FILE: &str = "Blizzard_AddOnPerformance.toc";

#[test]
fn init_creates_message_tables_and_arms_ten_second_ticker() {
    let env = build_env_with_ticker_spy();
    load_root_addon(&env);

    let probe: InitTickerProbe = env
        .eval(
            r#"
            return type(AddOnPerformance.shownPerformanceMessages),
                   next(AddOnPerformance.shownPerformanceMessages) == nil,
                   type(AddOnPerformance.addOnHasPerformanceWarning),
                   next(AddOnPerformance.addOnHasPerformanceWarning) == nil,
                   __addonPerformanceTickerSeconds,
                   type(__addonPerformanceTickerCallback),
                   __addonPerformanceTickerHandle ~= nil
            "#,
        )
        .expect("AddOnPerformance init ticker probe must run cleanly");

    assert_init_ticker_probe(probe);
}

type InitTickerProbe = (String, bool, String, bool, i64, String, bool);

fn build_env_with_ticker_spy() -> wow_ui_sim::lua_api::WowLuaEnv {
    let ui_dir = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui_dir);
    // AddOnPerformance.lua registers StaticPopupDialogs at load; the client
    // loads Blizzard_StaticPopup_Game first, and Retail/Forever no longer seed a
    // placeholder table (c1b07efb79).
    load_blizzard_addon_closure_into_env(
        &env,
        &ui_dir,
        &["Blizzard_SharedXML", "Blizzard_StaticPopup_Game"],
        &[],
    );
    install_ticker_spy(&env);
    env
}

fn install_ticker_spy(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.exec(
        r#"
        __addonPerformanceTickerHandle = { Cancel = function() end }
        __addonPerformanceTickerSeconds = nil
        __addonPerformanceTickerCallback = nil
        C_Timer.NewTicker = function(seconds, callback)
            __addonPerformanceTickerSeconds = seconds
            __addonPerformanceTickerCallback = callback
            return __addonPerformanceTickerHandle
        end
        "#,
    )
    .expect("ticker spy must install before Blizzard_AddOnPerformance loads");
}

fn load_root_addon(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let toc_path = blizzard_ui_dir().join(ROOT).join(ROOT_TOC_FILE);
    load_addon(&env.loader_env(), &toc_path)
        .unwrap_or_else(|err| panic!("`{ROOT}` must load cleanly under ticker spy: {err}"));
}

fn assert_init_ticker_probe(probe: InitTickerProbe) {
    let (
        shown_messages_type,
        shown_messages_empty,
        warnings_type,
        warnings_empty,
        ticker_seconds,
        ticker_callback_type,
        ticker_handle_exists,
    ) = probe;

    assert_eq!(shown_messages_type, "table");
    assert!(
        shown_messages_empty,
        "`AddOnPerformanceMixin:Init` must start with no shown performance messages"
    );
    assert_eq!(warnings_type, "table");
    assert!(
        warnings_empty,
        "`AddOnPerformanceMixin:Init` must start with no addon warning flags"
    );
    assert_eq!(ticker_seconds, 10, "`Init` must arm a 10 second ticker");
    assert_eq!(
        ticker_callback_type, "function",
        "`Init` must pass a callback to `C_Timer.NewTicker`"
    );
    assert!(
        ticker_handle_exists,
        "`C_Timer.NewTicker` must return a non-nil ticker handle"
    );
}
