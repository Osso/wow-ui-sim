//! Unassigned prose: failed combat-log registration has no listener side effects.
#![cfg(feature = "retail-12-0-0")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn prose_combat_log_registration_errors_without_delivery() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        ProseEvents={}
        ProseFrame=CreateFrame('Frame','ProseRegistrationFrame')
        ProseFrame:RegisterEvent('PLAYER_ENTERING_WORLD')
        ProseFrame:SetScript('OnEvent',function(_,event) ProseEvents[#ProseEvents+1]=event end)
        local function addon_registration()
            assert(debug.getstacktaint() == 'ProseAddon')
            for _,event in ipairs({'COMBAT_LOG_EVENT','COMBAT_LOG_EVENT_UNFILTERED'}) do
                local ok,err=pcall(ProseFrame.RegisterEvent,ProseFrame,event)
                assert(not ok and type(err)=='string' and #err>0, event .. ' RegisterEvent succeeded')
                assert(not ProseFrame:IsEventRegistered(event))
                ok,err=pcall(ProseFrame.RegisterUnitEvent,ProseFrame,event,'player')
                assert(not ok and type(err)=='string' and #err>0, event .. ' RegisterUnitEvent succeeded')
                assert(not ProseFrame:IsEventRegistered(event))
            end
            assert(ProseFrame:IsEventRegistered('PLAYER_ENTERING_WORLD'))
        end
        debug.setobjecttaint(addon_registration,'ProseAddon')
        addon_registration()
    "#).unwrap();
    for event in [
        "COMBAT_LOG_EVENT",
        "COMBAT_LOG_EVENT_UNFILTERED",
        "PLAYER_ENTERING_WORLD",
    ] {
        env.fire_event(event).unwrap();
    }
    env.exec("assert(#ProseEvents==1 and ProseEvents[1]=='PLAYER_ENTERING_WORLD')")
        .unwrap();
}

#[test]
fn prose_callback_event_mechanism_is_separate_from_script_registration() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(C_EventUtils.IsCallbackEvent('COMBAT_LOG_EVENT'))
        assert(not C_EventUtils.IsCallbackEvent('PLAYER_ENTERING_WORLD'))
        local f=CreateFrame('Frame')
        local callback=function() end
        local ok = pcall(f.RegisterEventCallback,f,'COMBAT_LOG_EVENT',callback)
        assert(ok and f:IsEventRegistered('COMBAT_LOG_EVENT'))
    "#,
    )
    .unwrap();
}
