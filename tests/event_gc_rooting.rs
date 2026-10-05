//! Payload roots must outlive a failed callback and its Lua error handler.
use rilua::Val;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn native_aura_event_payload_survives_collection_in_error_handler() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        errors = 0
        deliveries = 0
        seterrorhandler(function()
            errors = errors + 1
            collectgarbage('collect')
        end)
        local first = CreateFrame('Frame')
        first:RegisterEvent('UNIT_AURA')
        first:SetScript('OnEvent', function() error('audit native first listener') end)
        local second = CreateFrame('Frame')
        second:RegisterEvent('UNIT_AURA')
        second:SetScript('OnEvent', function(_, event, unit, info)
            assert(event == 'UNIT_AURA' and unit == 'player' and info.isFullUpdate == true)
            deliveries = deliveries + 1
        end)
        A_Admin.AddBuff(19750, 'Audit Aura', 136243, 60, 1)
        assert(deliveries == 1 and errors == 1)
    "#).unwrap();
}

#[test]
fn host_event_payload_survives_collection_in_error_handler() {
    assert_host_event_payload(false);
}

#[test]
fn loader_event_payload_survives_collection_in_error_handler() {
    assert_host_event_payload(true);
}

fn assert_host_event_payload(loader: bool) {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        errors = 0
        deliveries = 0
        seterrorhandler(function()
            errors = errors + 1
            collectgarbage('collect')
        end)
        local first = CreateFrame('Frame')
        first:RegisterEvent('UNIT_AURA')
        first:SetScript('OnEvent', function() error('audit first listener') end)
        local second = CreateFrame('Frame')
        second:RegisterEvent('UNIT_AURA')
        second:SetScript('OnEvent', function(_, event, unit, info)
            assert(event == 'UNIT_AURA' and unit == 'audit-unit')
            assert(info.marker == 41 and info.child.text == 'audit-child')
            deliveries = deliveries + 1
        end)
    "#).unwrap();
    let payload: Val = env.eval("return {marker=41, child={text='audit-child'}}").unwrap();
    let args = [env.lua_string("audit-unit"), payload];
    if loader {
        env.loader_env().fire_event_with_args("UNIT_AURA", &args).unwrap();
    } else {
        env.fire_event_with_args("UNIT_AURA", &args).unwrap();
    }
    env.exec("assert(deliveries == 1 and errors == 1)").unwrap();
}
