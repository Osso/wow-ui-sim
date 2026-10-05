//! Retail 12.0.7 publication-sweep gap closures backed by simulator state.
#![cfg(feature = "retail-12-0-7")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn quest_short_expiration_warning_reads_host_flags() {
    let env = WowLuaEnv::new().unwrap();
    env.state()
        .borrow_mut()
        .quest_short_expiration_warnings
        .extend([76586, u32::MAX]);
    env.exec(
        r#"
        local query = C_QuestInfoSystem.GetQuestHasShortExpirationWarning
        assert(select('#', query(76586)) == 1)
        assert(query(76586) == true and query(4294967295) == true)
        assert(query(76587) == false and query(0) == false)
        assert(query(nil) == false and query() == false, 'INFERRED nil quest has no warning')
        for _, bad in ipairs({-1, 1.5, 4294967296, 0/0, '76586', true, {}}) do
            local ok, err = pcall(query, bad)
            assert(not ok and tostring(err):find('GetQuestHasShortExpirationWarning'), tostring(err))
        end
        "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .quest_short_expiration_warnings
        .remove(&76586);
    let live: bool = env
        .eval("return C_QuestInfoSystem.GetQuestHasShortExpirationWarning(76586)")
        .unwrap();
    assert!(!live, "removing the host flag is visible immediately");
}

#[test]
fn maw_power_rarity_info_requires_both_rarity_and_atlas() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        let powers = &mut state.maw_powers;
        powers.rarity_ids.extend([(101, 3), (202, 1)]);
        powers.border_atlases.extend([
            (101, "jailerstower-animapowerlist-powerborder-purple".to_string()),
            (303, "atlas-without-rarity".to_string()),
        ]);
    }
    env.exec(
        r#"
        local rarityID, atlas = C_Spell.GetMawPowerRarityInfoBySpellID(101)
        assert(select('#', C_Spell.GetMawPowerRarityInfoBySpellID(101)) == 2)
        assert(rarityID == 3 and atlas == 'jailerstower-animapowerlist-powerborder-purple')
        assert(select('#', C_Spell.GetMawPowerRarityInfoBySpellID(202)) == 0, 'rarity without atlas')
        assert(select('#', C_Spell.GetMawPowerRarityInfoBySpellID(303)) == 0, 'atlas without rarity')
        assert(select('#', C_Spell.GetMawPowerRarityInfoBySpellID(404)) == 0, 'unknown spell')
        assert(not pcall(C_Spell.GetMawPowerRarityInfoBySpellID, -1))
        assert(not pcall(C_Spell.GetMawPowerRarityInfoBySpellID))
        "#,
    )
    .unwrap();
}

#[test]
fn base_difficulty_is_identity_until_difficulty_variants_are_modeled() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(rawget(_G, 'GetBaseDifficultyID')) == 'function')
        for _, id in ipairs({1, 14, 15, 16, 23, 233}) do
            assert(GetBaseDifficultyID(id) == id)
        end
        local ok, err = pcall(GetBaseDifficultyID, '16')
        assert(not ok and tostring(err):find('difficultyID'), tostring(err))
        "#,
    )
    .unwrap();
}

#[test]
fn patch_12_0_7_cvars_publish_page_defaults() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local expected = {
            Aftermath = '1', AftermathCallstacks = '0', enableMemoryTrap = '1',
            unlockedExpansionLandingPages = '0',
        }
        for name, default in pairs(expected) do
            assert(C_CVar.GetCVarDefault(name) == default, name)
            assert(C_CVar.GetCVar(name) == default, name)
        end
        "#,
    )
    .unwrap();
}

#[test]
fn simulated_mouse_input_queues_only_for_secure_callers() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        SimulateMouseClick('LeftButton')
        SimulateMouseWheel(-1)
        local function addon() SimulateMouseDown('RightButton') end
        debug.setobjecttaint(addon, 'SimulateMouseAddon')
        addon()
        "#,
    )
    .unwrap();
    let queued = env.state().borrow().simulated_mouse_inputs.len();
    assert_eq!(queued, 3, "secure click (down+up) and wheel queue; addon call refused");
}

#[test]
fn club_battle_tag_friend_request_records_guild_members_once() {
    let env = WowLuaEnv::new().unwrap();
    let member_two: String = env
        .eval("return C_Club.GetMemberInfo('guild-0', C_Club.GetClubMembers('guild-0')[2]).name")
        .unwrap();
    env.exec(
        r#"
        local ids = C_Club.GetClubMembers('guild-0')
        assert(select('#', C_Club.SendBattleTagFriendRequest('guild-0', ids[2])) == 0)
        C_Club.SendBattleTagFriendRequest('guild-0', ids[2])
        C_Club.SendBattleTagFriendRequest('guild-0', ids[1])
        C_Club.SendBattleTagFriendRequest('guild-0', 'missing-member')
        C_Club.SendBattleTagFriendRequest('other-club', ids[2])
        "#,
    )
    .unwrap();
    let state = env.state().borrow();
    assert_eq!(state.club_battle_tag_friend_requests, vec![member_two]);
    assert!(state.title_friend_requests.is_empty(), "BattleTag and title requests are distinct");
}
