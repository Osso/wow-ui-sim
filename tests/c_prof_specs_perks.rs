#![cfg(feature = "profession-spec-paths")]

use wow_ui_sim::c_api::profession_spec_inputs::ProfessionPerkRankEntry;
use wow_ui_sim::lua_api::WowLuaEnv;

const PERK_ARGUMENT_ERRORS_LUA: &str = r#"
    local usage = {
        'Usage: local perkInfos = C_ProfSpecs.GetPerksForPath(pathID)',
        'Usage: local state = C_ProfSpecs.GetStateForPerk(perkID, configID)',
        'Usage: local unlockRank = C_ProfSpecs.GetUnlockRankForPerk(perkID)',
        'Usage: local entryID = C_ProfSpecs.GetEntryIDForPerk(perkID)',
    }
    local calls = {
        function() C_ProfSpecs.GetPerksForPath('bad') end,
        function() C_ProfSpecs.GetStateForPerk(2001) end,
        function() C_ProfSpecs.GetUnlockRankForPerk(math.huge) end,
        function() C_ProfSpecs.GetEntryIDForPerk({}) end,
    }
    for index, call in ipairs(calls) do
        local ok, err = pcall(call)
        if ok or not string.find(err, usage[index], 1, true) then return false end
    end
    local expected = 'outside of expected range -2147483648 to 2147483647 - ' .. usage[1]
    local ok, err = pcall(C_ProfSpecs.GetPerksForPath, 2147483648)
    return not ok and string.find(err, expected, 1, true) ~= nil
"#;

fn perk_rank(learned: i32, granted: i32, max_ranks: i32) -> ProfessionPerkRankEntry {
    ProfessionPerkRankEntry {
        learned,
        granted,
        max_ranks,
    }
}

fn env_with_perk_rows() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create profession perk environment");
    let mut sim = env.state().borrow_mut();
    let inputs = &mut sim.profession_spec_inputs;
    inputs.trait_config_ids.insert(700);
    inputs.path_perk_group_ids.extend([(1001, 10), (2001, 20)]);
    inputs
        .group_node_ids
        .extend([(10, vec![2001, 2002, 2003]), (20, vec![3001])]);
    inputs
        .perk_flags_by_node
        .extend([(2001, 2), (2002, 1), (3001, 4)]);
    inputs.perk_unlock_ranks.insert(2001, 3);
    inputs.perk_entry_ids.insert(2001, 8201);
    inputs.perk_rank_entries_by_config.extend([
        ((2001, 700), vec![perk_rank(1, 0, 3)]),
        ((2002, 700), vec![perk_rank(1, 2, 2)]),
        ((2003, 700), vec![perk_rank(0, 0, 1)]),
    ]);
    inputs.stored_perk_ranks_by_config.insert((2001, 700), 1);
    drop(sim);
    env
}

#[test]
fn get_perks_preserves_group_order_and_uses_major_bonus_flag() {
    let env = env_with_perk_rows();
    assert!(env.eval::<bool>(r#"
        local count, perks = select('#', C_ProfSpecs.GetPerksForPath(1001)), C_ProfSpecs.GetPerksForPath(1001)
        if count ~= 1 or type(perks) ~= 'table' or #perks ~= 2 then return false end
        if perks[1].perkID ~= 2001 or perks[1].isMajorPerk ~= true then return false end
        if perks[2].perkID ~= 2002 or perks[2].isMajorPerk ~= false then return false end
        local other = C_ProfSpecs.GetPerksForPath(2001)
        local missing = C_ProfSpecs.GetPerksForPath(9999)
        return #other == 1 and other[1].perkID == 3001 and other[1].isMajorPerk == false
            and type(missing) == 'table' and next(missing) == nil
    "#).expect("perk list includes only backed nodes in group order"));
}

#[test]
fn perk_queries_return_rank_entry_and_state_values() {
    let env = env_with_perk_rows();
    assert!(
        env.eval::<bool>(
            r#"
        local state = C_ProfSpecs.GetStateForPerk
        local states = Enum.ProfessionsSpecPerkState
        local n, pending = select('#', state(2001, 700)), state(2001, 700)
        return states.Unearned == 0 and states.Pending == 1 and states.Earned == 2
            and n == 1 and pending == states.Pending
            and state(2002, 700) == states.Earned
            and state(2003, 700) == states.Unearned
            and state(9999, 700) == states.Unearned
            and state(2001, 701) == states.Unearned
            and C_ProfSpecs.GetUnlockRankForPerk(2001) == 3
            and C_ProfSpecs.GetUnlockRankForPerk(9999) == nil
            and C_ProfSpecs.GetEntryIDForPerk(2001) == 8201
            and C_ProfSpecs.GetEntryIDForPerk(9999) == 0
    "#
        )
        .expect("perk rank, entry, and state results follow supplied rows")
    );
}

#[test]
fn perk_queries_use_verified_usage_and_range_errors() {
    let env = env_with_perk_rows();
    assert!(
        env.eval::<bool>(PERK_ARGUMENT_ERRORS_LUA)
            .expect("perk Usage and range errors")
    );
}
