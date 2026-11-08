#![cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn tainted_secretwrap_and_table_security_follow_native_restriction() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local events = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('ADDON_ACTION_FORBIDDEN')
        listener:SetScript('OnEvent', function(_, _, source, name)
            events[#events + 1] = {source, name}
        end)
        local t = {key = 'value'}
        local function addon()
            local a, b = secretwrap(42, 'payload')
            assert(issecretvalue(a) and issecretvalue(b))
            assert(select('#', settablesecurity(t, 0)) == 0)
        end
        debug.setobjecttaint(addon, 'R12Addon')
        addon()
        assert(#events == 1 and events[1][1] == 'R12Addon')
        assert(events[1][2] == 'settablesecurity()')
        local function read() assert(t.key == 'value') end
        debug.setobjecttaint(read, 'R12Reader')
        read()
        local wrapped = secretwrap('trusted')
        assert(issecretvalue(wrapped) and secretunwrap(wrapped) == 'trusted')
    "#).unwrap();
}
