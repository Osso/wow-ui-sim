#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

const ROUND_TRIP: &str = r#"
    assert(GetAutoDeclineNeighborhoodInvites() == false)
    assert(select('#', SetAutoDeclineNeighborhoodInvites(true)) == 0)
    assert(GetAutoDeclineNeighborhoodInvites() == true)
    SetAutoDeclineNeighborhoodInvites(false)
    assert(GetAutoDeclineNeighborhoodInvites() == false)
"#;

const DEFAULT_ARGUMENT: &str = r#"
    SetAutoDeclineNeighborhoodInvites(true)
    SetAutoDeclineNeighborhoodInvites()
    assert(GetAutoDeclineNeighborhoodInvites() == false)
    SetAutoDeclineNeighborhoodInvites(true)
    SetAutoDeclineNeighborhoodInvites(nil)
    assert(GetAutoDeclineNeighborhoodInvites() == false)
"#;

#[test]
fn neighborhood_preference_round_trip() {
    WowLuaEnv::new().unwrap().exec(ROUND_TRIP).unwrap();
}

#[test]
fn neighborhood_preference_default_argument() {
    WowLuaEnv::new().unwrap().exec(DEFAULT_ARGUMENT).unwrap();
}

#[test]
fn neighborhood_preference_isolation_independence_and_bootstrap() {
    let first = WowLuaEnv::new().unwrap();
    let second = WowLuaEnv::new().unwrap();
    first
        .exec(
            r#"
        assert(GetAutoDeclineGuildInvites() == false)
        SetAutoDeclineNeighborhoodInvites(true)
        assert(GetAutoDeclineGuildInvites() == false)
        assert(GetAllowRecentAlliesSeeLocation() == true)
        SetAllowRecentAlliesSeeLocation(false)
        assert(GetAutoDeclineNeighborhoodInvites() == true)
    "#,
        )
        .unwrap();
    first.loader_env().restore_post_cleanup_globals().unwrap();
    first
        .exec("assert(GetAutoDeclineNeighborhoodInvites() == true)")
        .unwrap();
    second
        .exec("assert(GetAutoDeclineNeighborhoodInvites() == false)")
        .unwrap();
    first.exec("SetAutoDeclineNeighborhoodInvites(false); assert(not GetAutoDeclineNeighborhoodInvites())").unwrap();
}

#[test]
fn neighborhood_preference_argument_and_secret_boundaries() {
    WowLuaEnv::new()
        .unwrap()
        .exec(
            r#"
        for _, value in ipairs({0, 1, 'false', {}, function() end}) do
            assert(not pcall(SetAutoDeclineNeighborhoodInvites, value))
            assert(GetAutoDeclineNeighborhoodInvites() == false)
        end
        SetAutoDeclineNeighborhoodInvites(secretwrap(true))
        assert(GetAutoDeclineNeighborhoodInvites() == true)
        assert(not pcall(SetAutoDeclineNeighborhoodInvites, secretwrap('false')))
        assert(GetAutoDeclineNeighborhoodInvites() == true)
        local secretFalse = secretwrap(false)
        local secretNil = secretwrap(nil)
        local function addon()
            local ok, err = pcall(SetAutoDeclineNeighborhoodInvites, secretFalse)
            assert(not ok and err:find('untainted caller', 1, true))
            assert(not pcall(SetAutoDeclineNeighborhoodInvites, secretNil))
            assert(GetAutoDeclineNeighborhoodInvites() == true)
            SetAutoDeclineNeighborhoodInvites(false)
            assert(GetAutoDeclineNeighborhoodInvites() == false)
            SetAutoDeclineNeighborhoodInvites(true)
            SetAutoDeclineNeighborhoodInvites()
            assert(GetAutoDeclineNeighborhoodInvites() == false)
        end
        debug.setobjecttaint(addon, 'NeighborhoodPreferenceProbe')
        addon()
    "#,
        )
        .unwrap();
}

#[test]
fn neighborhood_preference_account_ui_save_load_and_hook() {
    WowLuaEnv::new().unwrap().exec(r#"
        local syncData = {blockNeighborhoodInvites = {special = {}}}
        local hooked = {}
        hooksecurefunc('SetAutoDeclineNeighborhoodInvites', function()
            hooked[#hooked + 1] = GetAutoDeclineNeighborhoodInvites()
        end)
        SetAutoDeclineNeighborhoodInvites(true)
        syncData.blockNeighborhoodInvites.special.blockNeighborhoodInvites = GetAutoDeclineNeighborhoodInvites()
        SetAutoDeclineNeighborhoodInvites(false)
        SetAutoDeclineNeighborhoodInvites(syncData.blockNeighborhoodInvites.special.blockNeighborhoodInvites)
        assert(GetAutoDeclineNeighborhoodInvites() == true)
        assert(#hooked == 3 and hooked[1] == true and hooked[2] == false and hooked[3] == true)
    "#).unwrap();
}
