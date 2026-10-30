//! Tests for PLAN row 643: party members have distinct buff/debuff distributions.
//!
//! - party1 (Thrynn):   buff only
//! - party2 (Kazzara):  debuff only
//! - party3 (Sylvanas): buff + debuff
//! - party4 (Jaina):    neither

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.state().borrow_mut().party_group_active = true;
    env
}

#[test]
fn test_party1_has_buff_no_debuff() {
    let env = env();
    let (has_buff, has_debuff): (bool, bool) = env
        .eval(
            r#"
            local buff = C_UnitAuras.GetAuraDataByIndex("party1", 1, "HELPFUL")
            local debuff = C_UnitAuras.GetAuraDataByIndex("party1", 1, "HARMFUL")
            return buff ~= nil, debuff ~= nil
        "#,
        )
        .unwrap();
    assert!(has_buff, "party1 should have a buff");
    assert!(!has_debuff, "party1 should not have a debuff");
}

#[test]
fn test_party2_has_debuff_no_buff() {
    let env = env();
    let (has_buff, has_debuff): (bool, bool) = env
        .eval(
            r#"
            local buff = C_UnitAuras.GetAuraDataByIndex("party2", 1, "HELPFUL")
            local debuff = C_UnitAuras.GetAuraDataByIndex("party2", 1, "HARMFUL")
            return buff ~= nil, debuff ~= nil
        "#,
        )
        .unwrap();
    assert!(!has_buff, "party2 should not have a buff");
    assert!(has_debuff, "party2 should have a debuff");
}

#[test]
fn test_party3_has_both_buff_and_debuff() {
    let env = env();
    let (has_buff, has_debuff): (bool, bool) = env
        .eval(
            r#"
            local buff = C_UnitAuras.GetAuraDataByIndex("party3", 1, "HELPFUL")
            local debuff = C_UnitAuras.GetAuraDataByIndex("party3", 1, "HARMFUL")
            return buff ~= nil, debuff ~= nil
        "#,
        )
        .unwrap();
    assert!(has_buff, "party3 should have a buff");
    assert!(has_debuff, "party3 should have a debuff");
}

#[test]
fn test_party4_has_neither_buff_nor_debuff() {
    let env = env();
    let (has_buff, has_debuff): (bool, bool) = env
        .eval(
            r#"
            local buff = C_UnitAuras.GetAuraDataByIndex("party4", 1, "HELPFUL")
            local debuff = C_UnitAuras.GetAuraDataByIndex("party4", 1, "HARMFUL")
            return buff ~= nil, debuff ~= nil
        "#,
        )
        .unwrap();
    assert!(!has_buff, "party4 should not have a buff");
    assert!(!has_debuff, "party4 should not have a debuff");
}
