//! Bounded current backing behavior, not full 2013 API parity.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn assert_raid_difficulty_backing(env: &WowLuaEnv) {
    for difficulty in [14, 16] {
        env.state().borrow_mut().world.instance_difficulty = difficulty;
        let observed: i32 = env.eval("return GetRaidDifficultyID()").unwrap();
        assert_eq!(observed, difficulty);
    }
}

#[test]
fn patch_5_2_0_raid_difficulty_backing_integration() {
    let env = WowLuaEnv::new().unwrap();
    assert_raid_difficulty_backing(&env);
}

prefork_full_ui_case! {
fn patch_5_2_0_raid_difficulty_backing_prefork(env: &WowLuaEnv) {
    assert_raid_difficulty_backing(env);
}
}
