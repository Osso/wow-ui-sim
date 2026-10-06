#![cfg(feature = "retail-12-0-0")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn p1127_invite_preference_is_per_environment_and_roundtrips() {
    let env = WowLuaEnv::new().unwrap();
    let other = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(GetAutoDeclineNeighborhoodInvites() == false)
        SetAutoDeclineNeighborhoodInvites(true)
        assert(GetAutoDeclineNeighborhoodInvites() == true)
        assert(not pcall(SetAutoDeclineNeighborhoodInvites, 1))
        assert(GetAutoDeclineNeighborhoodInvites() == true)
        SetAutoDeclineNeighborhoodInvites()
        assert(GetAutoDeclineNeighborhoodInvites() == false)
        SetAutoDeclineNeighborhoodInvites(true)
    "#).unwrap();
    other.exec("assert(GetAutoDeclineNeighborhoodInvites() == false)").unwrap();
    env.exec("assert(GetAutoDeclineNeighborhoodInvites() == true)").unwrap();
}
