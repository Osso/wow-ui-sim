#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn assert_bonus(env: &WowLuaEnv, battleground: bool, brawl: bool) {
    env.exec(&format!(
        "assert(select('#', C_PvP.IsBattlegroundEnlistmentBonusActive()) == 2); \
         local a, b = C_PvP.IsBattlegroundEnlistmentBonusActive(); \
         assert(a == {battleground} and b == {brawl})"
    ))
    .unwrap();
}

#[test]
fn battleground_enlistment_reads_gates_and_selected_world_states() {
    let env = WowLuaEnv::new().unwrap();
    assert_bonus(&env, false, false);
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        assert!(sim.special_event_brawl.is_none());
        let input = &mut sim.battleground_enlistment;
        input.player_level = Some(89);
        input.prerequisites_present = true;
        input.flags_allow = true;
        input.configuration_key = Some(1);
        input.cache_enabled = true;
        input
            .cached_values
            .extend([(10542, 3), (13646, -2), (10537, 0), (13647, 7)]);
    }
    assert_bonus(&env, false, false);
    env.state()
        .borrow_mut()
        .battleground_enlistment
        .player_level = Some(90);
    assert_bonus(&env, true, true);
    env.state()
        .borrow_mut()
        .battleground_enlistment
        .configuration_key = Some(0);
    assert_bonus(&env, false, true);
    env.state()
        .borrow_mut()
        .battleground_enlistment
        .cache_enabled = false;
    assert_bonus(&env, false, false);
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        let input = &mut sim.battleground_enlistment;
        input.cache_enabled = true;
        input.prerequisites_present = false;
    }
    assert_bonus(&env, false, false);
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        let input = &mut sim.battleground_enlistment;
        input.prerequisites_present = true;
        input.flags_allow = false;
    }
    assert_bonus(&env, false, false);
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        let input = &mut sim.battleground_enlistment;
        input.flags_allow = true;
        input.player_level = None;
    }
    assert_bonus(&env, false, false);
}
