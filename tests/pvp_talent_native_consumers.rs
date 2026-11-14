#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;
use wow_ui_sim::c_api::c_tooltip_info_pvp_talent::{PvpTalentSlotInput, PvpTalentTooltipRecord};
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
fn required_pvp_talent_arguments_report_verified_usage_strings() {
    let env = WowLuaEnv::new().expect("create PvP talent API environment");

    let results: (bool, bool, bool, bool, bool) = env
        .eval(
            r#"
            local function hasUsageError(call, usage)
                local ok, message = pcall(call)
                return not ok and string.find(message, usage, 1, true) ~= nil
            end
            return hasUsageError(function() C_SpecializationInfo.GetPvpTalentUnlockLevel() end,
                    "Usage: local requiredLevel = C_SpecializationInfo.GetPvpTalentUnlockLevel(talentID)"),
                hasUsageError(function() C_SpecializationInfo.GetPvpTalentSlotUnlockLevel() end,
                    "Usage: local requiredLevel = C_SpecializationInfo.GetPvpTalentSlotUnlockLevel(talentIndex)"),
                hasUsageError(function() C_SpecializationInfo.IsPvpTalentLocked() end,
                    "Usage: local locked = C_SpecializationInfo.IsPvpTalentLocked(talentID)"),
                hasUsageError(function() C_SpecializationInfo.SetPvpTalentLocked() end,
                    "Usage: C_SpecializationInfo.SetPvpTalentLocked(talentID, locked)"),
                hasUsageError(function() C_SpecializationInfo.SetPvpTalentLocked(123, nil) end,
                    "Usage: C_SpecializationInfo.SetPvpTalentLocked(talentID, locked)")
            "#,
        )
        .expect("query required PvP talent argument diagnostics");

    assert_eq!(results, (true, true, true, true, true));
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
            "local value = C_SpecializationInfo.GetPvpTalentUnlockLevel(777.9); return select('#', C_SpecializationInfo.GetPvpTalentUnlockLevel(777.9)), type(value)",
        )
        .expect("query unlock level for a catalog row without a player");

    assert_eq!(result, (1, "number".to_string()));
}

#[test]
fn player_spells_talent_sort_uses_catalog_unlock_levels() {
    with_blizzard_addon_closure(
        &["Blizzard_GameTooltip", "Blizzard_PlayerSpells"],
        &[],
        |env, loaded| {
            assert!(loaded.iter().any(|name| name == "Blizzard_PlayerSpells"));
            {
                let mut sim = env.state().borrow_mut();
                sim.pvp_talent_tooltip.slots.insert(
                    1,
                    PvpTalentSlotInput {
                        enabled: true,
                        level: 60,
                        available_talent_ids: vec![900, 901],
                    },
                );
                let mut later_unlock = PvpTalentTooltipRecord::default();
                later_unlock.level_required = 60;
                sim.pvp_talent_tooltip.catalog.insert(900, later_unlock);
                let mut earlier_unlock = PvpTalentTooltipRecord::default();
                earlier_unlock.level_required = 50;
                sim.pvp_talent_tooltip.catalog.insert(901, earlier_unlock);
            }

            let result: (i32, i32) = env
                .eval(
                    r#"
                    C_SpecializationInfo.GetPvpTalentInfo = function()
                        return { unlocked = false }
                    end
                    local slotInfo = C_SpecializationInfo.GetPvpTalentSlotInfo(1)
                    assert(slotInfo and #slotInfo.availableTalentIDs == 2,
                        "host-supplied slot data should reach Player Spells")
                    assert(C_SpecializationInfo.GetPvpTalentUnlockLevel(900) == 60)
                    assert(C_SpecializationInfo.GetPvpTalentUnlockLevel(901) == 50)
                    local inserted = {}
                    CreateDataProvider = function()
                        return {
                            Insert = function(_, talent)
                                table.insert(inserted, talent)
                            end,
                        }
                    end
                    local list = {
                        slotIndex = 1,
                        ScrollBox = { SetDataProvider = function() end },
                    }
                    local ok, err = pcall(function()
                        PvPTalentListMixin:Update(list)
                    end)
                    local first = inserted[1] and inserted[1].talentID or -1
                    local second = inserted[2] and inserted[2].talentID or -1
                    return ok, tostring(err), #inserted, first, second
                    "#,
                )
                .expect("run the selected Player Spells talent-list consumer");

            assert_eq!(result, (true, "nil".to_string(), 2, 901, 900));
        },
    );
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
                    C_SpecializationInfo.SetPvpTalentLocked(tostring(secondTalentID + 0.9), 0)
                    local bothLocked = C_SpecializationInfo.IsPvpTalentLocked(firstTalentID + 0.9)
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
