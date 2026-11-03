//! Integration tests for the action-button input globals registered in
//! `src/lua_api/globals/combat_verbs.rs`.
//!
//! Verifies that `ActionButtonDown`/`Up`, `MultiActionButtonDown`/`Up`,
//! `ExtraActionButtonKey`, and `TryUseActionButton` correctly mirror the
//! WoW keybind dispatch path: read `button.action`, look up the slot in
//! `state.action_bars`, and route through the cast pipeline so a key
//! press yields an active `UnitCastingInfo` entry.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

fn install_button(env: &WowLuaEnv, name: &str, slot: u32) {
    env.exec(&format!(
        r#"
        local f = CreateFrame("Button", "{name}", nil, "SecureActionButtonTemplate")
        f.action = {slot}
        f:SetButtonState("NORMAL")
        "#
    ))
    .expect("create button");
}

#[cfg(not(feature = "client-wrath"))]
#[test]
fn try_use_action_button_fires_cast_when_checking_from_down() {
    let env = env();
    install_button(&env, "ActionButton1", 1);
    let cast_name: String = env
        .eval(
            r#"
            A_Admin.SetActionSlot(1, 12345)
            TryUseActionButton(_G.ActionButton1, true)
            return (UnitCastingInfo("player"))
            "#,
        )
        .unwrap();
    assert_eq!(cast_name, "Spell 12345");
}

#[test]
fn try_use_action_button_skips_cast_when_not_from_down() {
    let env = env();
    install_button(&env, "ActionButton1", 1);
    let cast_name: Option<String> = env
        .eval(
            r#"
            A_Admin.SetActionSlot(1, 12345)
            TryUseActionButton(_G.ActionButton1, false)
            return (UnitCastingInfo("player"))
            "#,
        )
        .unwrap();
    assert!(cast_name.is_none(), "release should not start a cast");
}

#[cfg(not(feature = "client-wrath"))]
#[test]
fn action_button_down_pushes_state_and_fires_cast() {
    let env = env();
    install_button(&env, "ActionButton1", 1);
    let (state_after, cast_name): (String, String) = env
        .eval(
            r#"
            A_Admin.SetActionSlot(1, 4242)
            ActionButtonDown(1)
            return _G.ActionButton1:GetButtonState(), (UnitCastingInfo("player"))
            "#,
        )
        .unwrap();
    assert_eq!(state_after, "PUSHED");
    assert_eq!(cast_name, "Spell 4242");
}

#[test]
fn action_button_up_releases_pushed_state() {
    let env = env();
    install_button(&env, "ActionButton1", 1);
    let state_after: String = env
        .eval(
            r#"
            A_Admin.SetActionSlot(1, 4242)
            ActionButtonDown(1)
            ActionButtonUp(1)
            return _G.ActionButton1:GetButtonState()
            "#,
        )
        .unwrap();
    assert_eq!(state_after, "NORMAL");
}

#[test]
fn action_button_down_no_op_when_slot_empty() {
    let env = env();
    install_button(&env, "ActionButton1", 1);
    let cast_name: Option<String> = env
        .eval(
            r#"
            A_Admin.ClearActionSlot(1)
            ActionButtonDown(1)
            return (UnitCastingInfo("player"))
            "#,
        )
        .unwrap();
    assert!(
        cast_name.is_none(),
        "down on empty slot must not start a cast"
    );
}

#[test]
fn action_button_up_only_acts_when_currently_pushed() {
    let env = env();
    install_button(&env, "ActionButton1", 1);
    let state_after: String = env
        .eval(
            r#"
            A_Admin.SetActionSlot(1, 4242)
            ActionButtonUp(1)
            return _G.ActionButton1:GetButtonState()
            "#,
        )
        .unwrap();
    assert_eq!(state_after, "NORMAL", "up on NORMAL stays NORMAL");
}

#[cfg(not(feature = "client-wrath"))]
#[test]
fn multi_action_button_down_dispatches_via_action_buttons_table() {
    let env = env();
    let cast_name: String = env
        .eval(
            r#"
            local bar = CreateFrame("Frame", "MultiBarBottomLeft")
            local btn = CreateFrame("Button", "MultiBarBottomLeftButton1", bar, "SecureActionButtonTemplate")
            btn.action = 13
            btn:SetButtonState("NORMAL")
            bar.actionButtons = { [1] = btn }
            A_Admin.SetActionSlot(13, 7777)
            MultiActionButtonDown("MultiBarBottomLeft", 1)
            return (UnitCastingInfo("player"))
            "#,
        )
        .unwrap();
    assert_eq!(cast_name, "Spell 7777");
}

#[test]
fn multi_action_button_up_releases_pushed_state() {
    let env = env();
    let state_after: String = env
        .eval(
            r#"
            local bar = CreateFrame("Frame", "MultiBarBottomLeft")
            local btn = CreateFrame("Button", "MultiBarBottomLeftButton1", bar, "SecureActionButtonTemplate")
            btn.action = 13
            btn:SetButtonState("NORMAL")
            bar.actionButtons = { [1] = btn }
            A_Admin.SetActionSlot(13, 7777)
            MultiActionButtonDown("MultiBarBottomLeft", 1)
            MultiActionButtonUp("MultiBarBottomLeft", 1)
            return btn:GetButtonState()
            "#,
        )
        .unwrap();
    assert_eq!(state_after, "NORMAL");
}

#[cfg(not(feature = "client-wrath"))]
#[test]
fn extra_action_button_key_down_fires_cast() {
    let env = env();
    install_button(&env, "ExtraActionButton1", 169);
    let cast_name: String = env
        .eval(
            r#"
            A_Admin.SetActionSlot(169, 9999)
            ExtraActionButtonKey(1, true)
            return (UnitCastingInfo("player"))
            "#,
        )
        .unwrap();
    assert_eq!(cast_name, "Spell 9999");
}

#[test]
fn extra_action_button_key_up_releases_pushed_state() {
    let env = env();
    install_button(&env, "ExtraActionButton1", 169);
    let state_after: String = env
        .eval(
            r#"
            A_Admin.SetActionSlot(169, 9999)
            ExtraActionButtonKey(1, true)
            ExtraActionButtonKey(1, false)
            return _G.ExtraActionButton1:GetButtonState()
            "#,
        )
        .unwrap();
    assert_eq!(state_after, "NORMAL");
}

#[test]
fn action_button_down_with_unknown_button_is_silent_noop() {
    let env = env();
    let cast_name: Option<String> = env
        .eval(
            r#"
            ActionButtonDown(99)
            return (UnitCastingInfo("player"))
            "#,
        )
        .unwrap();
    assert!(cast_name.is_none());
}
