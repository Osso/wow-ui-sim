//! Integration tests for the original `GetColorForCurrencyReward`
//! (`Blizzard_FrameXMLUtil/CurrencyContainer.lua`). d509ed6aec removed the
//! simulator fallback; only the Retail, PTR and Forever caches ship the
//! original function.
#![cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_FrameXMLUtil"], &[]);
    env
}

#[test]
fn returns_white_when_no_default_color_passed() {
    let env = env();
    let (r, g, b): (f64, f64, f64) = env
        .eval("return GetColorForCurrencyReward(2245, 35):GetRGB()")
        .unwrap();
    assert_eq!((r, g, b), (1.0, 1.0, 1.0));
}

#[test]
fn returns_default_color_when_one_is_provided() {
    let env = env();
    let (r, g, b): (f64, f64, f64) = env
        .eval(
            r#"
            local override = CreateColor(0.25, 0.5, 0.75, 1)
            return GetColorForCurrencyReward(2245, 1, override):GetRGB()
            "#,
        )
        .unwrap();
    assert_eq!((r, g, b), (0.25, 0.5, 0.75));
}

#[test]
fn return_is_callable_with_get_rgba() {
    let env = env();
    let (r, g, b, a): (f64, f64, f64, f64) = env
        .eval("return GetColorForCurrencyReward(2245, 0):GetRGBA()")
        .unwrap();
    assert_eq!((r, g, b, a), (1.0, 1.0, 1.0, 1.0));
}

#[test]
fn returned_value_is_a_table() {
    let env = env();
    let returned_type: String = env
        .eval("return type(GetColorForCurrencyReward(2245, 0))")
        .unwrap();
    assert_eq!(returned_type, "table");
}

#[test]
fn handles_zero_quantity_without_error() {
    let env = env();
    let r: f64 = env
        .eval("local c = GetColorForCurrencyReward(2245, 0); return ({c:GetRGB()})[1]")
        .unwrap();
    assert_eq!(r, 1.0);
}

#[test]
fn default_color_is_returned_unchanged_for_red() {
    let env = env();
    let (r, g, b): (f64, f64, f64) = env
        .eval(
            r#"
            local red = CreateColor(1, 0, 0, 1)
            return GetColorForCurrencyReward(2245, 9999, red):GetRGB()
            "#,
        )
        .unwrap();
    assert_eq!((r, g, b), (1.0, 0.0, 0.0));
}
