//! Return-tuple identity only; event payloads and native identifier types are not covered.
#![cfg(feature = "retail-12-1-0")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn start_cast(env: &WowLuaEnv) -> u32 {
    env.exec("A_Admin.SetCasting(19750, 'Flash of Light', 'cast-icon', 5)")
        .unwrap();
    env.state().borrow().casting.as_ref().unwrap().cast_id
}

fn assert_tuple(env: &WowLuaEnv, channel: bool, id: u32, stages: u32) {
    let (start, end) = {
        let state = env.state().borrow();
        let cast = if channel { &state.channeling } else { &state.casting };
        let cast = cast.as_ref().unwrap();
        (cast.start_time * 1000.0, cast.end_time * 1000.0)
    };
    let query = if channel { "UnitChannelInfo" } else { "UnitCastingInfo" };
    let tail = if channel {
        format!("false, 19750, {}, {stages}, {id}", stages > 0)
    } else {
        format!("'Cast-Sim-{id}', false, 19750, {id}, 0")
    };
    env.exec(&format!(
        r#"
        local function pack(...) return {{n=select('#', ...), ...}} end
        local actual = pack({query}('player'))
        local expected = {{'Flash of Light', 'Flash of Light', 'cast-icon',
                           {start:?}, {end:?}, false, {tail}}}
        assert(actual.n == 11, '{query} arity: ' .. actual.n)
        for index = 1, 11 do
            assert(actual[index] == expected[index], '{query} slot ' .. index)
        end
        "#
    ))
    .unwrap();
}

#[test]
fn cast_bar_id_cast_lifecycle_preserves_tuple_and_identity() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(UnitCastingInfo('player') == nil); assert(UnitChannelInfo('player') == nil)")
        .unwrap();
    let first = start_cast(&env);
    assert_tuple(&env, false, first, 0);
    env.state().borrow_mut().casting.as_mut().unwrap().end_time += 0.5;
    assert_tuple(&env, false, first, 0);
    env.exec("assert(UnitChannelInfo('player') == nil); A_Admin.StopCasting(); assert(UnitCastingInfo('player') == nil)")
        .unwrap();
    let second = start_cast(&env);
    assert_ne!(first, second);
    assert_tuple(&env, false, second, 0);
    env.exec("assert(UnitCastingInfo('target') == nil)").unwrap();
}

#[test]
fn cast_bar_id_channel_uses_allocated_identity_across_updates() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("A_Admin.StartChannel(19750, 'Flash of Light', 'cast-icon', 5)").unwrap();
    let channel_id = env.state().borrow().channeling.as_ref().unwrap().cast_id;
    assert_tuple(&env, true, channel_id, 0);
    env.exec("assert(UnitCastingInfo('player') == nil); assert(A_Admin.UpdateChannel(5.75))").unwrap();
    assert_tuple(&env, true, channel_id, 0);
    env.exec("A_Admin.StartEmpower(19750, 'Flash of Light', 'cast-icon', {1,1,1,1}, 0)").unwrap();
    let empower_id = env.state().borrow().channeling.as_ref().unwrap().cast_id;
    assert_ne!(empower_id, channel_id);
    assert_tuple(&env, true, empower_id, 4);
    env.exec("assert(A_Admin.UpdateEmpower({1,1,1,2}, 0.5))").unwrap();
    assert_tuple(&env, true, empower_id, 4);
    let casting_id = start_cast(&env);
    assert_ne!(casting_id, empower_id);
    assert_tuple(&env, false, casting_id, 0);
    env.exec("assert(UnitChannelInfo('player') == nil); assert(UnitChannelInfo('target') == nil)").unwrap();
    env.exec("A_Admin.StartChannel(19750, 'Flash of Light', 'cast-icon', 5)").unwrap();
    let next_channel = env.state().borrow().channeling.as_ref().unwrap().cast_id;
    assert_ne!(next_channel, channel_id);
    assert_ne!(next_channel, casting_id);
    assert_tuple(&env, true, next_channel, 0);
    env.exec("assert(UnitCastingInfo('player') == nil)").unwrap();
}
