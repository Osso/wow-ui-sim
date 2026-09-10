//! Real producer callbacks; no synthetic spellcast event injection.
use super::*;
use crate::lua_api::WowLuaEnv;

fn listen(env: &WowLuaEnv) {
    env.exec(
        r#"
        events = {}
        local f = CreateFrame('Frame')
        for _, name in ipairs({'UNIT_SPELLCAST_START', 'UNIT_SPELLCAST_STOP',
                               'UNIT_SPELLCAST_SUCCEEDED'}) do f:RegisterEvent(name) end
        f:SetScript('OnEvent', function(_, event, ...)
            local row = {event=event, n=select('#', ...), ...}
            row.queryID = select(10, UnitCastingInfo('player'))
            row.activeSpec = GetSpecialization()
            table.insert(events, row)
        end)
    "#,
    )
    .unwrap();
}

fn assert_start(env: &WowLuaEnv) -> (u32, u32) {
    let (id, spell) = {
        let state = env.state().borrow();
        let cast = state.casting.as_ref().unwrap();
        (cast.cast_id, cast.spell_id)
    };
    env.exec(&format!(
        r#"
        local e = events[#events]
        assert(e.event == 'UNIT_SPELLCAST_START')
        assert(e.n == 4, 'START payload count: ' .. e.n)
        assert(e[1] == 'player' and type(e[2]) == 'string' and #e[2] > 0)
        assert(e[3] == {spell} and e[4] == {id})
        assert(e.queryID == {id})
    "#
    ))
    .unwrap();
    (id, spell)
}

fn complete(env: &WowLuaEnv, expected: (u32, u32)) {
    assert!(extract_completed_cast(env.state()).is_none());
    env.state().borrow_mut().casting.as_mut().unwrap().end_time = 0.0;
    let completed = extract_completed_cast(env.state()).unwrap();
    assert_eq!(completed, expected);
    fire_cast_complete_events(env, completed.0, completed.1);
    env.exec(
        r#"
        local n = #events
        local start, stop, success = events[n-2], events[n-1], events[n]
        assert(stop.event == 'UNIT_SPELLCAST_STOP')
        assert(success.event == 'UNIT_SPELLCAST_SUCCEEDED')
        for _, e in ipairs({stop, success}) do
            assert(e.n == 4, e.event .. ' payload count: ' .. e.n)
            for i = 1, 4 do assert(e[i] == start[i], e.event .. ' slot ' .. i) end
            assert(e.queryID == nil)
        end
    "#,
    )
    .unwrap();
    assert!(extract_completed_cast(env.state()).is_none());
}

#[test]
fn spellcast_payload_action_and_timed_completion_share_identity() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("TargetUnit('party1')").unwrap();
    listen(&env);
    env.exec("UseAction(1)").unwrap();
    let first = assert_start(&env);
    complete(&env, first);
    env.exec("firstGUID = events[1][2]").unwrap();
    env.state().borrow_mut().gcd = None;
    env.exec("UseAction(1)").unwrap();
    let second = assert_start(&env);
    assert_ne!(first.0, second.0);
    env.exec("assert(events[4][2] ~= firstGUID)").unwrap();
    complete(&env, second);
}

#[test]
fn spellcast_payload_crafting_uses_allocated_identity() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        A_Admin.SetSelectedProfession(164)
        A_Admin.LearnRecipe(100001)
        A_Admin.SeedReagentsForRecipe(100001, 1)
    "#,
    )
    .unwrap();
    listen(&env);
    env.exec("assert(C_TradeSkillUI.CraftRecipe(100001, 1))")
        .unwrap();
    let identity = assert_start(&env);
    assert_eq!(identity.1, 100001);
    complete(&env, identity);
}

#[test]
fn spellcast_payload_spec_change_preserves_deferred_state() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().player.active_spec_index = 1;
    listen(&env);
    env.exec("assert(C_SpecializationInfo.SetSpecialization(2))")
        .unwrap();
    let identity = assert_start(&env);
    complete(&env, identity);
    env.exec("for _, e in ipairs(events) do assert(e.activeSpec == 1) end")
        .unwrap();
    assert_eq!(env.state().borrow().player.pending_spec_change, Some(2));
    apply_spec_change(env.state(), &env);
    assert_eq!(env.state().borrow().player.active_spec_index, 2);
    env.exec("assert(C_SpecializationInfo.SetSpecialization(2)); assert(#events == 3)")
        .unwrap();
}
