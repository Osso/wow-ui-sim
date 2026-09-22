#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

const ROUND_TRIP: &str = r#"
    assert(GetAllowRecentAlliesSeeLocation() == true)
    assert(select('#', SetAllowRecentAlliesSeeLocation(false)) == 0)
    assert(GetAllowRecentAlliesSeeLocation() == false)
    SetAllowRecentAlliesSeeLocation(true)
    assert(GetAllowRecentAlliesSeeLocation() == true)
"#;

const EVENTS: &str = r#"
    local event = 'LET_RECENT_ALLIES_SEE_LOCATION_SETTING_UPDATED'
    local calls = {}
    local first, second = CreateFrame('Frame'), CreateFrame('Frame')
    local function record(label, name, ...)
        assert(name == event and select('#', ...) == 0)
        calls[#calls + 1] = label .. ':' .. tostring(GetAllowRecentAlliesSeeLocation())
        -- Settings proxy feedback must terminate without a duplicate notification.
        SetAllowRecentAlliesSeeLocation(GetAllowRecentAlliesSeeLocation())
    end
    first:RegisterEvent(event)
    second:RegisterEvent(event)
    first:SetScript('OnEvent', function(_, ...) record('first', ...) end)
    first:HookScript('OnEvent', function(_, ...) record('hook', ...) end)
    second:SetScript('OnEvent', function(_, ...) record('second', ...) end)
    SetAllowRecentAlliesSeeLocation(true)
    assert(#calls == 0)
    SetAllowRecentAlliesSeeLocation(false)
    assert(table.concat(calls, ',') == 'first:false,hook:false,second:false')
    SetAllowRecentAlliesSeeLocation(false)
    assert(#calls == 3)
    SetAllowRecentAlliesSeeLocation(true)
    assert(table.concat(calls, ',') == 'first:false,hook:false,second:false,first:true,hook:true,second:true')
"#;

#[test]
fn location_preference_round_trip() {
    WowLuaEnv::new().unwrap().exec(ROUND_TRIP).unwrap();
}

#[test]
fn location_preference_synchronous_events_and_feedback() {
    WowLuaEnv::new().unwrap().exec(EVENTS).unwrap();
}

#[test]
fn location_preference_environment_isolation_and_bootstrap_retention() {
    let first = WowLuaEnv::new().unwrap();
    let second = WowLuaEnv::new().unwrap();
    first
        .exec("SetAllowRecentAlliesSeeLocation(false)")
        .unwrap();
    first.loader_env().restore_post_cleanup_globals().unwrap();
    first
        .exec("assert(GetAllowRecentAlliesSeeLocation() == false)")
        .unwrap();
    second
        .exec("assert(GetAllowRecentAlliesSeeLocation() == true)")
        .unwrap();
    first
        .exec("SetAllowRecentAlliesSeeLocation(true); assert(GetAllowRecentAlliesSeeLocation())")
        .unwrap();
}

#[test]
fn location_preference_validation_and_secret_caller_boundary() {
    WowLuaEnv::new()
        .unwrap()
        .exec(
            r#"
        local notifications = 0
        local frame = CreateFrame('Frame')
        frame:RegisterEvent('LET_RECENT_ALLIES_SEE_LOCATION_SETTING_UPDATED')
        frame:SetScript('OnEvent', function() notifications = notifications + 1 end)
        for _, value in ipairs({0, 1, 'false', {}, function() end}) do
            assert(not pcall(SetAllowRecentAlliesSeeLocation, value))
        end
        assert(not pcall(SetAllowRecentAlliesSeeLocation))
        assert(not pcall(SetAllowRecentAlliesSeeLocation, nil))
        assert(GetAllowRecentAlliesSeeLocation() == true and notifications == 0)
        local secretFalse = secretwrap(false)
        SetAllowRecentAlliesSeeLocation(secretFalse)
        assert(GetAllowRecentAlliesSeeLocation() == false and notifications == 1)
        local secretTrue = secretwrap(true)
        local function addon()
            local ok, err = pcall(SetAllowRecentAlliesSeeLocation, secretTrue)
            assert(not ok and err:find('untainted caller', 1, true))
            assert(GetAllowRecentAlliesSeeLocation() == false and notifications == 1)
            SetAllowRecentAlliesSeeLocation(true)
            assert(GetAllowRecentAlliesSeeLocation() == true and notifications == 2)
        end
        debug.setobjecttaint(addon, 'LocationPreferenceProbe')
        addon()
    "#,
        )
        .unwrap();
}

#[test]
fn location_preference_account_ui_save_change_load_shape() {
    WowLuaEnv::new().unwrap().exec(r#"
        local syncData = {locationVisibility = {special = {}}}
        SetAllowRecentAlliesSeeLocation(false)
        syncData.locationVisibility.special.allowRecentAlliesSeeLocation = GetAllowRecentAlliesSeeLocation()
        SetAllowRecentAlliesSeeLocation(true)
        assert(GetAllowRecentAlliesSeeLocation() == true)
        if syncData.locationVisibility.special.allowRecentAlliesSeeLocation ~= nil then
            SetAllowRecentAlliesSeeLocation(syncData.locationVisibility.special.allowRecentAlliesSeeLocation)
        end
        assert(GetAllowRecentAlliesSeeLocation() == false)
    "#).unwrap();
}
