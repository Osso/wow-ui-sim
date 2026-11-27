//! Tests for A_Admin action bar API.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ============================================================================
// SetActionSlot / HasAction
// ============================================================================

#[test]
fn test_set_action_slot_has_action_returns_true() {
    let env = env();
    let got: bool = env
        .eval(
            r#"
            A_Admin.SetActionSlot(1, 12345)
            return HasAction(1)
            "#,
        )
        .unwrap();
    assert!(got);
}

#[test]
fn test_set_action_slot_get_action_info_returns_spell_type_and_id() {
    let env = env();
    let (action_type, id): (String, i64) = env
        .eval(
            r#"
            A_Admin.SetActionSlot(1, 12345)
            local actionType, id, subType = GetActionInfo(1)
            return actionType, id
            "#,
        )
        .unwrap();
    assert_eq!(action_type, "spell");
    assert_eq!(id, 12345);
}

// ============================================================================
// ClearActionSlot
// ============================================================================

#[test]
fn test_clear_action_slot_has_action_returns_false() {
    let env = env();
    let got: bool = env
        .eval(
            r#"
            A_Admin.SetActionSlot(1, 12345)
            A_Admin.ClearActionSlot(1)
            return HasAction(1)
            "#,
        )
        .unwrap();
    assert!(!got);
}

#[test]
fn test_clear_action_slot_only_clears_targeted_slot() {
    let env = env();
    let (slot1, slot2): (bool, bool) = env
        .eval(
            r#"
            A_Admin.SetActionSlot(1, 111)
            A_Admin.SetActionSlot(2, 222)
            A_Admin.ClearActionSlot(1)
            return HasAction(1), HasAction(2)
            "#,
        )
        .unwrap();
    assert!(!slot1);
    assert!(slot2);
}

// ============================================================================
// ClearActionBars
// ============================================================================

#[test]
fn test_clear_action_bars_empties_all_slots() {
    let env = env();
    let (s1, s2, s3): (bool, bool, bool) = env
        .eval(
            r#"
            A_Admin.SetActionSlot(1, 111)
            A_Admin.SetActionSlot(2, 222)
            A_Admin.SetActionSlot(3, 333)
            A_Admin.ClearActionBars()
            return HasAction(1), HasAction(2), HasAction(3)
            "#,
        )
        .unwrap();
    assert!(!s1);
    assert!(!s2);
    assert!(!s3);
}

// ============================================================================
// Multiple slots
// ============================================================================

#[test]
fn test_multiple_action_slots_independent() {
    let env = env();
    // Use high slot numbers (>12) to avoid default Paladin action bar pre-population
    let (s101, s120, s102): (bool, bool, bool) = env
        .eval(
            r#"
            A_Admin.SetActionSlot(101, 111)
            A_Admin.SetActionSlot(120, 222)
            return HasAction(101), HasAction(120), HasAction(102)
            "#,
        )
        .unwrap();
    assert!(s101);
    assert!(s120);
    assert!(!s102);
}

// ============================================================================
// HasAction on unset slot
// ============================================================================

#[test]
fn test_has_action_returns_false_for_empty_slot() {
    let env = env();
    let got: bool = env.eval("return HasAction(99)").unwrap();
    assert!(!got);
}

// ============================================================================
// GetActionInfo on unset slot
// ============================================================================

#[test]
fn test_get_action_info_returns_nil_for_empty_slot() {
    let env = env();
    let is_nil: bool = env.eval("return GetActionInfo(99) == nil").unwrap();
    assert!(is_nil);
}

// Retail and Forever publish the legacy global from the original
// Blizzard_DeprecatedActionBar addon (86dc74b163 retired the forwarding
// fallback there).
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn bonus_bar_env() -> WowLuaEnv {
    use crate::common::blizzard_addon_harness::{
        blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
    };
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_DeprecatedActionBar"], &[]);
    env
}

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
fn bonus_bar_env() -> WowLuaEnv {
    env()
}

#[test]
fn test_bonus_bar_offset_tracks_bonus_bar_index_above_main_pages() {
    let env = bonus_bar_env();
    let (namespace_offset, global_offset): (i32, i32) = env
        .eval(
            r#"
            local original = C_ActionBar.GetBonusBarIndex
            C_ActionBar.GetBonusBarIndex = function() return 11 end
            local namespaceOffset = C_ActionBar.GetBonusBarOffset()
            local globalOffset = GetBonusBarOffset()
            C_ActionBar.GetBonusBarIndex = original
            return namespaceOffset, globalOffset
            "#,
        )
        .unwrap();

    assert_eq!(namespace_offset, 5);
    assert_eq!(global_offset, 5);
}
