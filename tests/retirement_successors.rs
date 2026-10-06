//! Native successor contracts for the retired 11.x surface.
#![cfg(feature = "retail-12-0-0")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn spell_book_name_returns_name_and_sub_name() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local name, subName = C_SpellBook.GetSpellBookItemName(1, Enum.SpellBookSpellBank.Player)
        assert(select('#', C_SpellBook.GetSpellBookItemName(1, 0)) == 2)
        assert(name == 'Auto Attack' and subName == '')
        assert(C_SpellBook.GetSpellBookItemName(9999, 0) == nil)
    "#).unwrap();
}

#[test]
fn spell_texture_returns_file_ids_not_paths() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local info = C_Spell.GetSpellInfo(19750)
        local icon, original, conditional = C_Spell.GetSpellTexture(19750)
        assert(icon == info.iconID and original == info.iconID and conditional == nil)
        assert(C_Spell.GetSpellTexture('Flash of Light') == icon)
        assert(select('#', C_Spell.GetSpellTexture(999999999)) == 0)
    "#).unwrap();
}

#[test]
fn merchant_item_info_reads_ordered_inventory_and_missing_slots() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().merchant_items = vec![6948];
    env.exec(r#"
        local info = C_MerchantFrame.GetItemInfo(1)
        assert(type(info) == 'table' and info.name == 'Hearthstone')
        assert(info.texture == C_Item.GetItemIconByID(6948))
        assert(type(info.price) == 'number' and info.stackCount == 1)
        assert(info.numAvailable == -1 and info.isPurchasable and info.isUsable)
        assert(info.hasExtendedCost == false and info.isQuestStartItem == false)
        assert(info.currencyID == nil and info.spellID == nil)
        assert(C_MerchantFrame.GetItemInfo(0) == nil)
        assert(C_MerchantFrame.GetItemInfo(2) == nil)
    "#).unwrap();
    env.state().borrow_mut().merchant_items.clear();
    env.exec("assert(C_MerchantFrame.GetItemInfo(1) == nil)").unwrap();
}

#[test]
fn challenge_completion_has_documented_empty_record() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(select('#', C_ChallengeMode.GetChallengeCompletionInfo()) == 1)
        local info = C_ChallengeMode.GetChallengeCompletionInfo()
        assert(info.mapChallengeModeID == 0 and info.level == 0 and info.time == 0)
        assert(info.onTime == false and info.keystoneUpgradeLevels == 0)
        assert(info.practiceRun == false and info.isMapRecord == false and info.isAffixRecord == false)
        assert(info.isEligibleForScore == false)
        assert(info.oldOverallDungeonScore == nil and info.newOverallDungeonScore == nil)
        assert(type(info.members) == 'table' and #info.members == 0)
    "#).unwrap();
}

#[test]
fn challenge_completion_reads_host_snapshot_and_returns_detached_records() {
    use wow_ui_sim::c_api::c_challenge_mode::{ChallengeCompletionInfo, ChallengeCompletionMember};
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().challenge_completion = ChallengeCompletionInfo {
        map_challenge_mode_id: 503,
        level: 12,
        time: 1234567.0,
        on_time: true,
        keystone_upgrade_levels: 2,
        practice_run: true,
        old_overall_dungeon_score: Some(1800.0),
        new_overall_dungeon_score: Some(1825.5),
        is_map_record: true,
        is_affix_record: true,
        is_eligible_for_score: true,
        members: vec![ChallengeCompletionMember {
            member_guid: "Player-1-42".into(), name: "RetirementPaladin".into(),
        }],
    };
    env.exec(r#"
        local info = C_ChallengeMode.GetChallengeCompletionInfo()
        assert(info.mapChallengeModeID == 503 and info.level == 12 and info.time == 1234567)
        assert(info.onTime and info.keystoneUpgradeLevels == 2 and info.practiceRun)
        assert(info.oldOverallDungeonScore == 1800 and info.newOverallDungeonScore == 1825.5)
        assert(info.isMapRecord and info.isAffixRecord and info.isEligibleForScore)
        assert(#info.members == 1 and info.members[1].memberGUID == 'Player-1-42')
        assert(info.members[1].name == 'RetirementPaladin')
        info.members[1].name = 'changed locally'
        assert(C_ChallengeMode.GetChallengeCompletionInfo().members[1].name == 'RetirementPaladin')
    "#).unwrap();
    env.state().borrow_mut().challenge_completion = Default::default();
    env.exec("local info=C_ChallengeMode.GetChallengeCompletionInfo(); assert(info.level == 0 and #info.members == 0)").unwrap();
}

#[test]
fn log_message_records_message_and_returns_no_values() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(select('#', C_Log.LogMessage('retirement log proof')) == 0)").unwrap();
    assert_eq!(env.state().borrow().console_output.last().map(String::as_str), Some("retirement log proof"));
    env.exec("C_Log.LogMessage('second message')").unwrap();
    assert_eq!(env.state().borrow().console_output.last().map(String::as_str), Some("second message"));
}

#[test]
fn spell_overlay_reads_active_proc_membership() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(select('#', C_SpellActivationOverlay.IsSpellOverlayed(19750)) == 1)
        assert(C_SpellActivationOverlay.IsSpellOverlayed(19750) == false)
        assert(C_SpellActivationOverlay.IsSpellOverlayed(999999999) == false)
    "#).unwrap();
    env.state().borrow_mut().spell_activation_overlays.insert(19750);
    env.exec("assert(C_SpellActivationOverlay.IsSpellOverlayed(19750) == true); assert(C_SpellActivationOverlay.IsSpellOverlayed(642) == false)").unwrap();
    env.state().borrow_mut().spell_activation_overlays.remove(&19750);
    env.exec("assert(C_SpellActivationOverlay.IsSpellOverlayed(19750) == false)").unwrap();
}
