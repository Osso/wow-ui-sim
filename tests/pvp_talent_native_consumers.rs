#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn pvp_slot_unlock_level_is_nil_without_player_source() {
    let env = WowLuaEnv::new().expect("create PvP talent API environment");
    env.state().borrow_mut().player.present = false;

    let result: Option<i32> = env
        .eval("return C_SpecializationInfo.GetPvpTalentSlotUnlockLevel(1)")
        .expect("query PvP talent slot unlock level");

    assert_eq!(result, None);
}

#[test]
fn inspect_selected_pvp_talent_is_nil_without_matching_cache() {
    let env = WowLuaEnv::new().expect("create PvP talent API environment");

    let result: Option<i32> = env
        .eval("return C_SpecializationInfo.GetInspectSelectedPvpTalent(\"target\", 0)")
        .expect("query inspected PvP talent selection");

    assert_eq!(result, None);
}

#[test]
fn set_pvp_talent_locked_returns_no_values() {
    let env = WowLuaEnv::new().expect("create PvP talent API environment");

    let result: i32 = env
        .eval("return select('#', C_SpecializationInfo.SetPvpTalentLocked(123, true))")
        .expect("call PvP talent lock setter");

    assert_eq!(result, 0);
}

#[test]
fn player_spells_slot_tooltip_preserves_enabled_slot_consumer() {
    with_blizzard_addon_closure(
        &["Blizzard_GameTooltip", "Blizzard_PlayerSpells"],
        &[],
        |env, loaded| {
            assert!(loaded.iter().any(|name| name == "Blizzard_PlayerSpells"));

            let shown: bool = env
                .eval(
                    r#"
                    local parent = CreateFrame("Frame")
                    function parent:AcknowledgeNewNotification() end

                    local slot = CreateFrame("Button", nil, parent)
                    slot.slotIndex = 1
                    function slot:GetSelectedTalent() return nil end
                    function slot:IsInspecting() return false end

                    PvPTalentSlotButtonMixin.OnEnter(slot)
                    return GameTooltip:IsShown()
                    "#,
                )
                .expect("run the selected Player Spells slot-tooltip consumer");

            assert!(shown, "the enabled-slot tooltip consumer should complete");
        },
    );
}
