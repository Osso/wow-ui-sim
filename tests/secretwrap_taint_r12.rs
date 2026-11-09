#![cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn tainted_secretwrap_returns_secret_values() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local function addon()
            local a, b = secretwrap(42, 'payload')
            assert(issecretvalue(a) and issecretvalue(b))
            return a, b
        end
        debug.setobjecttaint(addon, 'R12Addon')
        local a, b = addon()
        assert(secretunwrap(a) == 42 and secretunwrap(b) == 'payload')
        local wrapped = secretwrap('trusted')
        assert(issecretvalue(wrapped) and secretunwrap(wrapped) == 'trusted')
    "#,
    )
    .unwrap();
}

#[test]
fn tainted_table_security_reports_and_does_not_mutate() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local events = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('ADDON_ACTION_FORBIDDEN')
        listener:SetScript('OnEvent', function(_, _, source, name)
            events[#events + 1] = {source, name}
        end)
        local t = {key = 'value'}
        local function addon()
            assert(select('#', settablesecurity(t, 0)) == 0)
            assert(select('#', settablesecurity(nil, 'invalid')) == 0)
        end
        debug.setobjecttaint(addon, 'R12Addon')
        addon()
        assert(#events == 2)
        for _, event in ipairs(events) do
            assert(event[1] == 'R12Addon' and event[2] == 'settablesecurity()')
        end
        local function read() assert(t.key == 'value') end
        debug.setobjecttaint(read, 'R12Reader')
        read()
    "#,
    )
    .unwrap();
}
