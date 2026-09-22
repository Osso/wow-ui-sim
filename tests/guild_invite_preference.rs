#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

const ROUND_TRIP: &str = r#"
    assert(GetAutoDeclineGuildInvites() == false)
    assert(select('#', SetAutoDeclineGuildInvites(true)) == 0)
    assert(GetAutoDeclineGuildInvites() == true)
    SetAutoDeclineGuildInvites(false)
    assert(GetAutoDeclineGuildInvites() == false)
"#;

const DEFAULT_ARGUMENT: &str = r#"
    SetAutoDeclineGuildInvites(true)
    SetAutoDeclineGuildInvites()
    assert(GetAutoDeclineGuildInvites() == false)
    SetAutoDeclineGuildInvites(true)
    SetAutoDeclineGuildInvites(nil)
    assert(GetAutoDeclineGuildInvites() == false)
"#;

#[test]
fn guild_preference_round_trip() {
    WowLuaEnv::new().unwrap().exec(ROUND_TRIP).unwrap();
}

#[test]
fn guild_preference_default_argument() {
    WowLuaEnv::new().unwrap().exec(DEFAULT_ARGUMENT).unwrap();
}

#[test]
fn guild_preference_existing_state_isolation_and_bootstrap() {
    let first = WowLuaEnv::new().unwrap();
    let second = WowLuaEnv::new().unwrap();
    first.state().borrow_mut().auto_decline_guild_invites = true;
    first
        .exec(
            r#"
        assert(GetAutoDeclineGuildInvites() == true)
        SetAutoDeclineGuildInvites(false)
        assert(GetAutoDeclineNeighborhoodInvites() == false)
        assert(GetAllowRecentAlliesSeeLocation() == true)
        SetAutoDeclineNeighborhoodInvites(true)
        SetAllowRecentAlliesSeeLocation(false)
        assert(GetAutoDeclineGuildInvites() == false)
        SetAutoDeclineGuildInvites(true)
    "#,
        )
        .unwrap();
    assert!(first.state().borrow().auto_decline_guild_invites);
    first.loader_env().restore_post_cleanup_globals().unwrap();
    first
        .exec("assert(GetAutoDeclineGuildInvites() == true); SetAutoDeclineGuildInvites(false)")
        .unwrap();
    assert!(!first.state().borrow().auto_decline_guild_invites);
    second
        .exec("assert(GetAutoDeclineGuildInvites() == false)")
        .unwrap();
}

#[test]
fn guild_preference_argument_and_secret_boundaries() {
    WowLuaEnv::new()
        .unwrap()
        .exec(
            r#"
        SetAutoDeclineGuildInvites(true)
        for _, value in ipairs({0, 1, 'false', {}, function() end}) do
            assert(not pcall(SetAutoDeclineGuildInvites, value))
            assert(GetAutoDeclineGuildInvites() == true)
        end
        SetAutoDeclineGuildInvites(secretwrap(false))
        assert(GetAutoDeclineGuildInvites() == false)
        assert(not pcall(SetAutoDeclineGuildInvites, secretwrap('true')))
        assert(GetAutoDeclineGuildInvites() == false)
        local secretTrue = secretwrap(true)
        local secretNil = secretwrap(nil)
        local function addon()
            local ok, err = pcall(SetAutoDeclineGuildInvites, secretTrue)
            assert(not ok and err:find('untainted caller', 1, true))
            assert(not pcall(SetAutoDeclineGuildInvites, secretNil))
            assert(GetAutoDeclineGuildInvites() == false)
            SetAutoDeclineGuildInvites(true)
            assert(GetAutoDeclineGuildInvites() == true)
            SetAutoDeclineGuildInvites()
            assert(GetAutoDeclineGuildInvites() == false)
        end
        debug.setobjecttaint(addon, 'GuildPreferenceProbe')
        addon()
    "#,
        )
        .unwrap();
}

#[test]
fn guild_preference_account_ui_save_load_and_hook() {
    WowLuaEnv::new()
        .unwrap()
        .exec(
            r#"
        local syncData = {blockGuildInvites = {special = {}}}
        local hooked = {}
        hooksecurefunc('SetAutoDeclineGuildInvites', function()
            hooked[#hooked + 1] = GetAutoDeclineGuildInvites()
        end)
        SetAutoDeclineGuildInvites(true)
        syncData.blockGuildInvites.special.blockGuildInvites = GetAutoDeclineGuildInvites()
        SetAutoDeclineGuildInvites(false)
        SetAutoDeclineGuildInvites(syncData.blockGuildInvites.special.blockGuildInvites)
        assert(GetAutoDeclineGuildInvites() == true)
        assert(#hooked == 3 and hooked[1] == true and hooked[2] == false and hooked[3] == true)
    "#,
        )
        .unwrap();
}
