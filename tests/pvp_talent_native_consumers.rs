#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;
use wow_ui_sim::c_api::c_tooltip_info_pvp_talent::PvpTalentTooltipRecord;
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
fn pvp_talent_unlock_level_is_nil_when_catalog_row_is_missing() {
    let env = WowLuaEnv::new().expect("create PvP talent API environment");
    env.state().borrow_mut().pvp_talent_tooltip.catalog.clear();

    let result: Option<i32> = env
        .eval("return C_SpecializationInfo.GetPvpTalentUnlockLevel(123)")
        .expect("query PvP talent unlock level");

    assert_eq!(result, None);
}

#[test]
fn pvp_talent_unlock_level_publishes_one_integer_for_a_row_without_a_player() {
    let env = WowLuaEnv::new().expect("create PvP talent API environment");
    env.state().borrow_mut().player.present = false;
    env.state()
        .borrow_mut()
        .pvp_talent_tooltip
        .catalog
        .insert(777, PvpTalentTooltipRecord::default());

    let result: (i32, String) = env
        .eval(
            "local value = C_SpecializationInfo.GetPvpTalentUnlockLevel(777); return select('#', C_SpecializationInfo.GetPvpTalentUnlockLevel(777)), type(value)",
        )
        .expect("query unlock level for a catalog row without a player");

    assert_eq!(result, (1, "number".to_string()));
}

#[test]
fn player_spells_locked_talent_consumer_clears_only_the_selected_lock() {
    with_blizzard_addon_closure(
        &["Blizzard_GameTooltip", "Blizzard_PlayerSpells"],
        &[],
        |env, loaded| {
            assert!(loaded.iter().any(|name| name == "Blizzard_PlayerSpells"));

            let result: (bool, bool, bool, bool, bool, bool) = env
                .eval(
                    r#"
                    local firstTalentID, secondTalentID = 123, 456
                    local startsUnlocked = not C_SpecializationInfo.IsPvpTalentLocked(firstTalentID)
                        and not C_SpecializationInfo.IsPvpTalentLocked(secondTalentID)

                    C_SpecializationInfo.SetPvpTalentLocked(firstTalentID, true)
                    C_SpecializationInfo.SetPvpTalentLocked(secondTalentID, 1)
                    local bothLocked = C_SpecializationInfo.IsPvpTalentLocked(firstTalentID)
                        and C_SpecializationInfo.IsPvpTalentLocked(secondTalentID)

                    local button = {
                        talentID = firstTalentID,
                        talentInfo = { unlocked = true },
                        New = { Hide = function(self) self.hidden = true end },
                        NewGlow = { Hide = function(self) self.hidden = true end },
                    }
                    PvPTalentListButtonMixin.OnEnter(button)
                    local consumerClearedFirst = not C_SpecializationInfo.IsPvpTalentLocked(firstTalentID)
                    local secondRemainsLocked = C_SpecializationInfo.IsPvpTalentLocked(secondTalentID)
                    local indicatorHidden = button.New.hidden and button.NewGlow.hidden

                    C_SpecializationInfo.SetPvpTalentLocked(secondTalentID, false)
                    local falseRemovedSecond = not C_SpecializationInfo.IsPvpTalentLocked(secondTalentID)
                    return startsUnlocked, bothLocked, consumerClearedFirst,
                        secondRemainsLocked, indicatorHidden, falseRemovedSecond
                    "#,
                )
                .expect("run the selected Player Spells lock consumer");

            assert_eq!(result, (true, true, true, true, true, true));
        },
    );
}

#[test]
fn player_spells_slot_consumer_works_without_legacy_alias() {
    with_blizzard_addon_closure(
        &["Blizzard_GameTooltip", "Blizzard_PlayerSpells"],
        &[],
        |env, loaded| {
            assert!(loaded.iter().any(|name| name == "Blizzard_PlayerSpells"));

            let (has_legacy_alias, shown): (bool, bool) = env
                .eval(
                    r#"
                    local hasLegacyAlias = rawget(_G, "GetPvpTalentSlotInfo") ~= nil
                    local parent = CreateFrame("Frame")
                    function parent:AcknowledgeNewNotification() end

                    local slot = CreateFrame("Button", nil, parent)
                    slot.slotIndex = 1
                    function slot:GetSelectedTalent() return nil end
                    function slot:IsInspecting() return false end

                    PvPTalentSlotButtonMixin.OnEnter(slot)
                    return hasLegacyAlias, GameTooltip:IsShown()
                    "#,
                )
                .expect("run the selected Player Spells slot-tooltip consumer");

            assert!(
                !has_legacy_alias,
                "Retail has no legacy slot-info global alias"
            );
            assert!(shown, "the enabled-slot tooltip consumer should complete");
        },
    );
}
