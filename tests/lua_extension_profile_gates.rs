//! API-level coverage for the Lua extension globals that Retail 12.1.5 and
//! Forever 70707 both declare with the same signatures:
//! LuaStringExtensionsDocumentation.lua:11-79, LuaTableExtensionsDocumentation.lua:79/:163,
//! LocalizationDocumentation.lua:42 and SystemTimeDocumentation.lua:39.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common;
use wow_ui_sim::lua_api::WowLuaEnv;

const STRING_EXTENSIONS: &str = r#"
    assert(string.startswith("Hearthstone", "Hearth") == true)
    assert(string.startswith("Hearthstone", "stone") == false)
    assert(string.endswith("Hearthstone", "stone") == true)
    assert(string.endswith("Hearthstone", "Hearth") == false)
    assert(string.contains("Hearthstone", "rths") == true)
    assert(string.contains("Hearthstone", "xyz") == false)
    assert(string.ltrim("  a b \t") == "a b \t")
    assert(string.rtrim("  a b \t") == "  a b")
    assert(string.ltrim("xxaxx", "x") == "axx")
    assert(string.rtrim("xxaxx", "x") == "xxa")
"#;

const TABLE_FREEZE: &str = r#"
    local frozen, open = { 1 }, { 2 }
    assert(table.isfrozen(frozen) == false)
    assert(rawequal(table.freeze(frozen), frozen))
    assert(table.isfrozen(frozen) == true)
    assert(table.isfrozen(open) == false)
    assert(not pcall(table.insert, frozen, 3))
    table.insert(open, 3)
    assert(#open == 2)
"#;

const LARGE_NUMBERS: &str = r#"
    assert(BreakUpLargeNumbers(12345) == "12,345", BreakUpLargeNumbers(12345))
    assert(BreakUpLargeNumbers(9876543) == "9,876,543", BreakUpLargeNumbers(9876543))
    assert(BreakUpLargeNumbers(999) == "999")
"#;

const SESSION_TIME: &str = r#"
    A_Admin.SetManualClock(100)
    assert(GetSessionTime() == 100, GetSessionTime())
    A_Admin.AdvanceTime(25)
    assert(GetSessionTime() == 125, GetSessionTime())
"#;

#[cfg(feature = "client-retail")]
#[test]
fn bnet_time_alert_text_tracks_the_session_clock() {
    common::with_timeout(120, || {
        let env = WowLuaEnv::new().expect("Lua environment");
        common::load_game_startup_inventory(&env);
        env.exec("A_Admin.SetManualClock(0)")
            .expect("set the session clock");
        env.sync_addon_names_to_lua();
        env.apply_post_load_workarounds();
        wow_ui_sim::startup::fire_startup_events_for_screen(
            &env,
            wow_ui_sim::screen::ScreenKind::Game,
        );
        env.exec(
            r#"
            A_Admin.FireEvent("SESSION_TIME_ALERT", 100000000)
            A_Admin.AdvanceTime(0)
            local first = TimeAlertFrame.Text:GetText()
            A_Admin.AdvanceTime(3600)
            local second = TimeAlertFrame.Text:GetText()
            assert(TimeAlertFrame:IsShown())
            assert(first ~= second, first .. " / " .. second)
            "#,
        )
        .expect("BNet time alert reflects the advanced session time");
    });
}

#[test]
fn lua_extensions_number_grouping_and_session_time_follow_their_inputs() {
    let env = WowLuaEnv::new().expect("Lua environment");
    for (label, chunk) in [
        ("string extensions", STRING_EXTENSIONS),
        ("table freeze", TABLE_FREEZE),
        ("large numbers", LARGE_NUMBERS),
        ("session time", SESSION_TIME),
    ] {
        env.exec(chunk).unwrap_or_else(|error| panic!("{label}: {error}"));
    }
}
