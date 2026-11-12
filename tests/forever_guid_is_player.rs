#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

const USAGE: &str = "Usage: local isPlayer = C_PlayerInfo.GUIDIsPlayer(guid)";

#[test]
fn forever_guid_is_player_classifies_parsed_player_kinds() {
    let env = WowLuaEnv::new().expect("Forever GUID environment");
    env.set_screen_mode(ScreenKind::Game);
    env.exec(
        r#"
        local query = rawget(C_PlayerInfo, 'GUIDIsPlayer')
        assert(type(query) == 'function', 'Game C_PlayerInfo installer must register GUIDIsPlayer')
        local function check(guid, expected)
            local result = query(guid)
            assert(type(result) == 'boolean' and result == expected, tostring(guid))
            assert(select('#', query(guid)) == 1, 'GUIDIsPlayer returns one value')
        end
        check('Player-1-00000001', true)
        check('Player-42-ABCDEF', true)
        check('Creature-0-0-0-0-12345-00000001', false)
        check('Item-12345-0-4000000000000000', false)
        check('BattlePet-0-000000000000', false)
        check('BNetAccount-1-0000002A', false)
        check('', false)
        check('not a GUID', false)
        check('Player-', false)
        check(901001, false)
        "#,
    )
    .expect("Forever GUID kind classification and numeric conversion");
}

#[test]
fn forever_guid_is_player_requires_a_value_and_rejects_other_types() {
    let env = WowLuaEnv::new().expect("Forever GUID environment");
    env.exec(&format!(
        r#"
        local query = rawget(C_PlayerInfo, 'GUIDIsPlayer')
        local function usage_error(...)
            local ok, message = pcall(query, ...)
            assert(not ok and type(message) == 'string', 'invalid GUID input must error')
            assert(message:find({USAGE:?}, 1, true), message)
        end
        usage_error()
        usage_error(nil)
        for _, value in ipairs({{false, true, {{}}, function() end,
            coroutine.create(function() end)}}) do
            usage_error(value)
        end
        "#
    ))
    .expect("Forever GUID input validation uses the native usage text");
}

#[test]
fn forever_guid_is_player_checks_secret_access_before_type_validation() {
    let env = WowLuaEnv::new().expect("Forever GUID environment");
    env.exec(&format!(
        r#"
        debug.settaintmode(true)
        local query = rawget(C_PlayerInfo, 'GUIDIsPlayer')
        local secretPlayer = secretwrap('Player-1-00000001')
        local secretWrongType = secretwrap(false)
        assert(issecretvalue(secretPlayer) and issecretvalue(secretWrongType))
        local function addon()
            for _, value in ipairs({{secretPlayer, secretWrongType}}) do
                local ok, message = pcall(query, value)
                assert(not ok and type(message) == 'string', 'tainted secret must error')
                assert(message:find({USAGE:?}, 1, true), message)
                assert(message:find('Secret values are only allowed during untainted execution for this argument.', 1, true), message)
            end
        end
        debug.setobjecttaint(addon, 'ForeverGuidProbe')
        addon()
        local ok = pcall(query, secretPlayer)
        assert(ok, 'untainted secret input must reach the query')
        "#
    ))
    .expect("Forever GUID secret access is checked before GUID parsing");
}
