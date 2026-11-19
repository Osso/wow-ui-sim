#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn c_party_info_invite_unit_keeps_invitation_server_mediated() {
    let env = WowLuaEnv::new().unwrap();
    let initial_roster_size = {
        let mut sim = env.state().borrow_mut();
        sim.party_group_active = true;
        sim.party_is_raid = Some(false);
        sim.party_members.len()
    };
    env.exec(
        r##"
        local observedEvents = {}
        local listener = CreateFrame("Frame")
        listener:RegisterEvent("GROUP_ROSTER_UPDATE")
        listener:RegisterEvent("PARTY_INVITE_REQUEST")
        listener:SetScript("OnEvent", function(_, event)
            observedEvents[#observedEvents + 1] = event
        end)
        local popupCalls = 0
        StaticPopup_Show = function() popupCalls = popupCalls + 1 end

        assert(type(C_PartyInfo.InviteUnit) == "function")
        assert(select("#", C_PartyInfo.InviteUnit("Target-Realm")) == 0)
        assert(not pcall(C_PartyInfo.InviteUnit))
        assert(#observedEvents == 0)
        assert(popupCalls == 0)
        "##,
    )
    .expect("documented invite call should not mutate local group state");

    let sim = env.state().borrow();
    assert!(sim.party_group_active);
    assert_eq!(sim.party_members.len(), initial_roster_size);
    assert_eq!(sim.party_is_raid, Some(false));
}
