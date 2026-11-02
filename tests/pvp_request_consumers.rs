//! Original PvP menu and slash command submit transport requests.
#![cfg(feature = "client-retail")]
use wow_ui_sim::c_api::c_pvp::PvpRequest;
use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-retail")]
prefork_full_ui_case! {
fn original_pvp_consumers_submit_without_changing_desired_state(env: &WowLuaEnv) {
    env.state().borrow_mut().pvp_request_transport.state = Some(5);
    env.exec(r#"
        local lookup = SlashCommandUtil.CheckAddSlashCommand
        local callback
        for i = 1, 20 do
            local name, value = debug.getupvalue(lookup, i)
            if not name then break end
            if name == 'addonTbl' then callback = value.SlashCmdList[SLASH_COMMAND.PVP] end
        end
        assert(callback, 'original PvP slash callback missing')
        assert(not GetPVPDesired())
        UnitPopupPvpEnableButtonMixin:OnClick({})
        UnitPopupPvpDisableButtonMixin:OnClick({})
        callback('')
        assert(not GetPVPDesired())
    "#).unwrap();
    assert_eq!(env.state().borrow().pvp_request_transport.submitted, vec![PvpRequest::Set(true), PvpRequest::Set(false), PvpRequest::Toggle]);
    env.state().borrow_mut().pvp_request_transport.state = Some(6);
    env.exec("UnitPopupPvpEnableButtonMixin:OnClick({})").unwrap();
    assert_eq!(env.state().borrow().pvp_request_transport.submitted.len(), 3);
    env.state().borrow_mut().pvp_request_transport.state = None;
    env.exec("UnitPopupPvpDisableButtonMixin:OnClick({})").unwrap();
    assert_eq!(env.state().borrow().pvp_request_transport.submitted.len(), 3);
    env.exec(r#"
        local reports = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('ADDON_ACTION_FORBIDDEN')
        listener:SetScript('OnEvent', function(_, _, source, name) reports[#reports+1] = source .. '|' .. name end)
        local function addon()
            assert(select('#', C_PvP.SetPVP({})) == 0)
            assert(select('#', C_PvP.TogglePVP()) == 0)
        end
        debug.setobjecttaint(addon, 'PvPFixture')
        addon()
        assert(reports[1] == 'PvPFixture|SetPVP()' and reports[2] == 'PvPFixture|TogglePVP()')
    "#).unwrap();
    assert_eq!(env.state().borrow().pvp_request_transport.submitted.len(), 3);
}
}
