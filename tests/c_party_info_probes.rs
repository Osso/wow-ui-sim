//! Tests for `C_PartyInfo` probes backed by
//! `SimState.party_members` / `party_group_active`.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[path = "common/selected_game_ui.rs"]
mod selected_game_ui;

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn original_leave_instance_button_observes_supplied_party_policy() {
    let env = selected_game_ui::load_selected_game_ui("", &["Blizzard_CompactRaidFrames"]);
    env.exec(r#"
        A_Admin.SetPartySize(2)
        CompactRaidFrameManager:Show()
        local button = CompactRaidFrameManagerLeaveInstanceGroupButton
        button:GetParent():Show()
        button:Show()
        assert(button:IsVisible())
        A_Admin.SetPartyPolicyFlags(true, false, true, true)
        assert(C_PartyInfo.AllowedToDoPartyConversion(true))
        assert(not C_PartyInfo.AllowedToDoPartyConversion(false))
        assert(C_PartyInfo.IsPartyInJailersTower())
    "#).unwrap();
    env.fire_on_update(0.016).unwrap();
    env.exec("assert(CompactRaidFrameManagerLeaveInstanceGroupButton:GetText() == INSTANCE_WALK_IN_LEAVE)").unwrap();
    env.exec(r#"
        A_Admin.SetPartyPolicyFlags(false, true, false, false)
        assert(not C_PartyInfo.AllowedToDoPartyConversion(true))
        assert(C_PartyInfo.AllowedToDoPartyConversion(false))
        assert(not C_PartyInfo.IsPartyInJailersTower())
    "#).unwrap();
    env.fire_on_update(0.016).unwrap();
    env.exec("assert(CompactRaidFrameManagerLeaveInstanceGroupButton:GetText() == INSTANCE_PARTY_LEAVE)").unwrap();
}

fn fired(env: &WowLuaEnv, name: &str) -> bool {
    env.state()
        .borrow()
        .events
        .pending()
        .iter()
        .any(|e| e.name == name)
}

// ── GetActiveGroupType ────────────────────────────────────────────────────────

#[test]
fn get_active_group_type_nil_when_solo() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_PartyInfo.GetActiveGroupType() == nil")
        .unwrap();
    assert!(is_nil, "solo player should return nil group type");
}

#[test]
fn get_active_group_type_zero_for_party() {
    let env = env();
    let group_type: i32 = env
        .eval(
            r#"
            A_Admin.SetPartySize(3)
            return C_PartyInfo.GetActiveGroupType()
            "#,
        )
        .unwrap();
    assert_eq!(group_type, 0, "party of 3 should return 0 (Party)");
}

#[test]
fn get_active_group_type_one_for_raid() {
    let env = env();
    let group_type: i32 = env
        .eval(
            r#"
            A_Admin.SetPartySize(10)
            return C_PartyInfo.GetActiveGroupType()
            "#,
        )
        .unwrap();
    assert_eq!(group_type, 1, "party of 10 (≥6) should return 1 (Raid)");
}

// ── IsPartyFull ───────────────────────────────────────────────────────────────

#[test]
fn is_party_full_false_when_solo() {
    let env = env();
    let full: bool = env.eval("return C_PartyInfo.IsPartyFull()").unwrap();
    assert!(!full, "solo player is not in a full party");
}

#[test]
fn is_party_full_false_for_small_party() {
    let env = env();
    let full: bool = env
        .eval(
            r#"
            A_Admin.SetPartySize(2)
            return C_PartyInfo.IsPartyFull()
            "#,
        )
        .unwrap();
    assert!(!full, "party of 2 others + player is not full");
}

#[test]
fn is_party_full_true_when_party_at_capacity() {
    let env = env();
    let full: bool = env
        .eval(
            r#"
            A_Admin.SetPartySize(4)
            return C_PartyInfo.IsPartyFull()
            "#,
        )
        .unwrap();
    assert!(full, "4 others + player = 5 total = full party");
}

// ── IsPartyInJailersTower ─────────────────────────────────────────────────────

#[test]
fn is_party_in_jailers_tower_always_false() {
    let env = env();
    let in_tower: bool = env
        .eval("return C_PartyInfo.IsPartyInJailersTower()")
        .unwrap();
    assert!(!in_tower, "Torghast is not modelled; always false");
}

// ── GetActiveCategories ───────────────────────────────────────────────────────

#[test]
fn get_active_categories_empty_when_solo() {
    let env = env();
    let count: i32 = env
        .eval("return #C_PartyInfo.GetActiveCategories()")
        .unwrap();
    assert_eq!(count, 0, "solo player has no active party categories");
}

#[test]
fn get_active_categories_home_when_grouped() {
    let env = env();
    let (count, first): (i32, i32) = env
        .eval(
            r#"
            A_Admin.SetPartySize(2)
            local cats = C_PartyInfo.GetActiveCategories()
            return #cats, cats[1]
            "#,
        )
        .unwrap();
    assert_eq!(count, 1, "one active category when grouped");
    assert_eq!(first, 1, "category id 1 = Home (Enum.PartyCategory.Home)");
}

// ── GetInviteConfirmationInfo ─────────────────────────────────────────────────

/// Retail/Forever publish the modeled global (fb72bce3a2; R88y `_G` row) and
/// retired the C_PartyInfo placeholder; other profiles keep the placeholder.
fn invite_confirmation_info_function() -> &'static str {
    if cfg!(any(feature = "client-retail", feature = "client-wowforever")) {
        "GetInviteConfirmationInfo"
    } else {
        "C_PartyInfo.GetInviteConfirmationInfo"
    }
}

#[test]
fn get_invite_confirmation_info_returns_nil() {
    let env = env();
    let count: i32 = env
        .eval(&format!(
            r#"return select('#', {}("Player-1234-ABCDEF"))"#,
            invite_confirmation_info_function()
        ))
        .unwrap();
    assert_eq!(count, 0, "no pending invite returns nothing");
}

// ── LeaveParty ────────────────────────────────────────────────────────────────

#[test]
fn leave_party_clears_roster_and_fires_event() {
    let env = env();
    let count: i32 = env
        .eval(
            r#"
            A_Admin.SetPartySize(3)
            C_PartyInfo.LeaveParty()
            return GetNumGroupMembers()
            "#,
        )
        .unwrap();
    assert_eq!(count, 0, "leaving party should clear group members");
    assert!(
        fired(&env, "GROUP_ROSTER_UPDATE"),
        "leaving party should fire GROUP_ROSTER_UPDATE"
    );
}
