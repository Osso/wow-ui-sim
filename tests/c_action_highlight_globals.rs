//! Integration tests for the action-highlight globals registered in
//! `src/lua_api/globals/real/action_highlights.rs`.
//!
//! Verifies that `MarkNewActionHighlight`/`ClearNewActionHighlight`/
//! `GetNewActionHighlightMark`, `ClearOnBarHighlightMarks`/
//! `GetOnBarHighlightMark`, the three `UpdateOnBarHighlightMarksBy*` verbs,
//! and `GetActionButtonForID` mirror the Blizzard `ActionButton.lua` shape
//! while reading and writing `state.action_highlights` (and the existing
//! `state.action_bars`).

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

#[test]
fn clearing_an_unbound_slot_preserves_other_new_action_marks() {
    let env = env();
    let (cleared, retained): (Option<bool>, Option<bool>) = env
        .eval(
            r#"
            MarkNewActionHighlight(4)
            MarkNewActionHighlight(9)
            ClearNewActionHighlight(4, false)
            return GetNewActionHighlightMark(4), GetNewActionHighlightMark(9)
            "#,
        )
        .unwrap();

    assert_eq!(cleared, None);
    assert_eq!(retained, Some(true));
}

#[test]
fn mark_and_get_new_action_highlight_round_trip() {
    let env = env();
    let (marked, missing): (Option<bool>, Option<bool>) = env
        .eval(
            r#"
            MarkNewActionHighlight(7)
            return GetNewActionHighlightMark(7), GetNewActionHighlightMark(8)
            "#,
        )
        .unwrap();
    assert_eq!(marked, Some(true));
    assert_eq!(missing, None);
}

#[test]
fn clear_new_action_highlight_with_prevent_keeps_other_slots() {
    let env = env();
    let (slot1, slot2): (Option<bool>, Option<bool>) = env
        .eval(
            r#"
            A_Admin.SetActionSlot(1, 555)
            A_Admin.SetActionSlot(2, 555)
            MarkNewActionHighlight(1)
            MarkNewActionHighlight(2)
            ClearNewActionHighlight(1, true)
            return GetNewActionHighlightMark(1), GetNewActionHighlightMark(2)
            "#,
        )
        .unwrap();
    assert_eq!(slot1, None, "explicit clear drops slot 1");
    assert_eq!(slot2, Some(true), "preventIdentical=true keeps slot 2");
}

#[test]
fn clear_new_action_highlight_cascades_through_identical_spell_slots() {
    let env = env();
    let (slot1, slot2, slot3): (Option<bool>, Option<bool>, Option<bool>) = env
        .eval(
            r#"
            A_Admin.SetActionSlot(1, 555)
            A_Admin.SetActionSlot(2, 555)
            A_Admin.SetActionSlot(3, 999)
            MarkNewActionHighlight(1)
            MarkNewActionHighlight(2)
            MarkNewActionHighlight(3)
            ClearNewActionHighlight(1, false)
            return GetNewActionHighlightMark(1), GetNewActionHighlightMark(2), GetNewActionHighlightMark(3)
            "#,
        )
        .unwrap();
    assert_eq!(slot1, None);
    assert_eq!(slot2, None, "duplicate spell slot is also cleared");
    assert_eq!(slot3, Some(true), "different spell stays marked");
}

#[test]
fn update_on_bar_highlight_marks_by_spell_tags_matching_slots() {
    let env = env();
    let (mark1, type1, mark2, type2, missing): (
        Option<bool>,
        Option<String>,
        Option<bool>,
        Option<String>,
        Option<bool>,
    ) = env
        .eval(
            r#"
            A_Admin.SetActionSlot(1, 4242)
            A_Admin.SetActionSlot(2, 4242)
            A_Admin.SetActionSlot(3, 9999)
            UpdateOnBarHighlightMarksBySpell(4242)
            local m1, t1 = GetOnBarHighlightMark(1)
            local m2, t2 = GetOnBarHighlightMark(2)
            local m3 = GetOnBarHighlightMark(3)
            return m1, t1, m2, t2, m3
            "#,
        )
        .unwrap();
    assert_eq!(mark1, Some(true));
    assert_eq!(type1.as_deref(), Some("spell"));
    assert_eq!(mark2, Some(true));
    assert_eq!(type2.as_deref(), Some("spell"));
    assert_eq!(missing, None, "non-matching slot has no on-bar mark");
}

#[test]
fn update_on_bar_highlight_marks_by_flyout_tags_with_flyout_kind() {
    let env = env();
    let (mark, kind): (Option<bool>, Option<String>) = env
        .eval(
            r#"
            A_Admin.SetActionSlot(1, 31415)
            UpdateOnBarHighlightMarksByFlyout(31415)
            return GetOnBarHighlightMark(1)
            "#,
        )
        .unwrap();
    assert_eq!(mark, Some(true));
    assert_eq!(kind.as_deref(), Some("flyout"));
}

#[test]
fn update_on_bar_highlight_marks_by_pet_action_tags_with_petaction_kind() {
    let env = env();
    let (mark, kind): (Option<bool>, Option<String>) = env
        .eval(
            r#"
            A_Admin.SetActionSlot(5, 271828)
            UpdateOnBarHighlightMarksByPetAction(271828)
            return GetOnBarHighlightMark(5)
            "#,
        )
        .unwrap();
    assert_eq!(mark, Some(true));
    assert_eq!(kind.as_deref(), Some("petaction"));
}

#[test]
fn update_on_bar_highlight_replaces_previous_marks() {
    let env = env();
    let (slot1, slot2): (Option<bool>, Option<bool>) = env
        .eval(
            r#"
            A_Admin.SetActionSlot(1, 1111)
            A_Admin.SetActionSlot(2, 2222)
            UpdateOnBarHighlightMarksBySpell(1111)
            UpdateOnBarHighlightMarksBySpell(2222)
            return GetOnBarHighlightMark(1), GetOnBarHighlightMark(2)
            "#,
        )
        .unwrap();
    assert_eq!(slot1, None, "first spell's marks were cleared");
    assert_eq!(slot2, Some(true));
}

#[test]
fn clear_on_bar_highlight_marks_wipes_all_kinds() {
    let env = env();
    let (slot1, slot2): (Option<bool>, Option<bool>) = env
        .eval(
            r#"
            A_Admin.SetActionSlot(1, 1111)
            A_Admin.SetActionSlot(2, 1111)
            UpdateOnBarHighlightMarksBySpell(1111)
            ClearOnBarHighlightMarks()
            return GetOnBarHighlightMark(1), GetOnBarHighlightMark(2)
            "#,
        )
        .unwrap();
    assert_eq!(slot1, None);
    assert_eq!(slot2, None);
}

#[test]
fn get_action_button_for_id_returns_named_global() {
    let env = env();
    let (named, missing): (Option<String>, bool) = env
        .eval(
            r#"
            CreateFrame("Button", "ActionButton7", nil, "SecureActionButtonTemplate")
            local btn = GetActionButtonForID(7)
            local missing_btn = GetActionButtonForID(99)
            return btn and btn:GetName(), missing_btn == nil
            "#,
        )
        .unwrap();
    assert_eq!(named.as_deref(), Some("ActionButton7"));
    assert!(missing, "unknown id resolves to nil");
}

#[test]
fn update_by_spell_with_no_matching_slots_clears_existing() {
    let env = env();
    let mark: Option<bool> = env
        .eval(
            r#"
            A_Admin.SetActionSlot(1, 4242)
            UpdateOnBarHighlightMarksBySpell(4242)
            UpdateOnBarHighlightMarksBySpell(0)
            return GetOnBarHighlightMark(1)
            "#,
        )
        .unwrap();
    assert_eq!(mark, None);
}
