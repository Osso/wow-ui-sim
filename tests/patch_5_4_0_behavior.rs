//! Bounded current-retail proof, not wholesale 2013 contract parity.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_5_4_0_standalone_instance_group_size() {
    let env = WowLuaEnv::new().expect("create standalone instance fixture");
    patch_5_4_0_instance_group_size_tracks_world_state::run(&env);
}

#[test]
fn patch_5_4_0_standalone_forbidden_frame_flag() {
    let env = WowLuaEnv::new().expect("create standalone forbidden-flag fixture");
    patch_5_4_0_forbidden_frame_flag_round_trips::run(&env);
}

prefork_full_ui_case! {
fn patch_5_4_0_instance_group_size_tracks_world_state(env: &WowLuaEnv) {
    {
        let mut state = env.state().borrow_mut();
        state.world.instance_name = "Siege of Orgrimmar".into();
        state.world.instance_type = "raid".into();
        state.world.instance_max_players = 25;
        state.world.instance_group_size = 17;
        state.world.instance_is_dynamic = true;
    }
    let (name, max_players, group_size): (String, i32, i32) = env
        .eval("local name, _, _, _, maxPlayers, _, _, _, groupSize = GetInstanceInfo(); return name, maxPlayers, groupSize")
        .expect("read independent capacity and tuned group size");
    assert_eq!(name, "Siege of Orgrimmar");
    assert_eq!(max_players, 25);
    assert_eq!(group_size, 17);
    env.state().borrow_mut().world.instance_group_size = 22;
    let group_size: i32 = env
        .eval("return select(9, GetInstanceInfo())")
        .expect("observe changed tuning without changing capacity");
    assert_eq!(group_size, 22);
}
}

prefork_full_ui_case! {
fn patch_5_4_0_forbidden_frame_flag_round_trips(env: &WowLuaEnv) {
    env.exec(
        r#"
        local frame = CreateFrame('Frame')
        assert(not frame:IsForbidden())
        frame:SetForbidden(true)
        assert(frame:IsForbidden())
        frame:SetForbidden(false)
        assert(not frame:IsForbidden())
        "#,
    )
    .expect("observe modeled forbidden flag transitions; no access-enforcement claim");
}
}
