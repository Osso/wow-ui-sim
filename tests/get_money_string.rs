//! Integration tests for the original `GetMoneyString`
//! (`Blizzard_SharedXML/FormattingUtil.lua:72`). d509ed6aec removed the
//! simulator's duplicate plain-text fallback; the original formats each
//! denomination with its coin texture. The Wrath cache has no
//! Blizzard_SharedXML.
#![cfg(not(feature = "client-wrath"))]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::lua_api::WowLuaEnv;

const GOLD: &str = r"|TInterface\MoneyFrame\UI-GoldIcon:0:0:2:0|t";
const SILVER: &str = r"|TInterface\MoneyFrame\UI-SilverIcon:0:0:2:0|t";
const COPPER: &str = r"|TInterface\MoneyFrame\UI-CopperIcon:0:0:2:0|t";

fn env() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_SharedXML"], &[]);
    env
}

fn money_string(call: &str) -> String {
    env().eval(&format!("return {call}")).unwrap()
}

#[test]
fn formats_full_gold_silver_copper() {
    let expected = format!("123{GOLD} 45{SILVER} 67{COPPER}");
    assert_eq!(money_string("GetMoneyString(1234567)"), expected);
}

#[test]
fn pure_copper_returns_copper_only() {
    assert_eq!(money_string("GetMoneyString(42)"), format!("42{COPPER}"));
}

#[test]
fn pure_silver_returns_silver_only() {
    assert_eq!(money_string("GetMoneyString(500)"), format!("5{SILVER}"));
}

#[test]
fn pure_gold_returns_gold_only() {
    assert_eq!(money_string("GetMoneyString(70000)"), format!("7{GOLD}"));
}

#[test]
fn zero_money_returns_zero_copper() {
    assert_eq!(money_string("GetMoneyString(0)"), format!("0{COPPER}"));
}

#[test]
fn elides_zero_silver_segment() {
    let expected = format!("1{GOLD} 5{COPPER}");
    assert_eq!(money_string("GetMoneyString(10005)"), expected);
}

#[test]
fn elides_zero_copper_segment() {
    let expected = format!("1{GOLD} 5{SILVER}");
    assert_eq!(money_string("GetMoneyString(10500)"), expected);
}

#[test]
fn separate_thousands_inserts_commas_in_gold() {
    let expected = format!("12,345{GOLD} 67{SILVER} 89{COPPER}");
    assert_eq!(money_string("GetMoneyString(123456789, true)"), expected);
}

#[test]
fn separate_thousands_omitted_for_small_gold() {
    let expected = format!("123{GOLD} 45{SILVER} 67{COPPER}");
    assert_eq!(money_string("GetMoneyString(1234567, true)"), expected);
}
