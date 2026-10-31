//! Selected PartyUtil consumes live group role counts.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use wow_ui_sim::loader::discover_blizzard_addon_closure_for_screen;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

#[test]
fn group_role_counts_original_party_util_tracks_roster() {
    let env = WowLuaEnv::new().expect("environment");
    let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir();
    let addons = discover_blizzard_addon_closure_for_screen(
        &ui,
        ScreenKind::Game,
        &["Blizzard_FrameXMLUtil"],
    );
    assert!(
        addons
            .iter()
            .any(|(name, _)| name == "Blizzard_FrameXMLUtil")
    );
    let source_path = ui.join("Blizzard_FrameXMLUtil/PartyUtil.lua");
    let source = std::fs::read_to_string(&source_path).expect("selected PartyUtil source");
    env.exec_named(&source, &format!("@{}", source_path.display()))
        .expect("load original PartyUtil");
    {
        let mut state = env.state().borrow_mut();
        state.player.class_index = 1;
        state.party_members[0].class_index = 2;
        state.party_members[1].class_index = 99;
        state.party_members.truncate(2);
        assert_eq!(state.party_members.len(), 2);
        state.party_group_active = true;
        state.party_leader_index = None;
    }
    env.exec(r#"
        assert(debug.getinfo(GetGroupMemberCountsForDisplay, 'S').source:find('PartyUtil.lua', 1, true))
        assert(UnitSetRole('player', 'TANK'))
        assert(UnitSetRole('party1', 'HEALER'))
        local shown = GetGroupMemberCountsForDisplay()
        assert(shown.TANK == 1 and shown.HEALER == 1 and shown.DAMAGER == 1 and shown.NOROLE == 0, 'display group counts')
        assert(shown.Warrior == 2 and shown.Paladin == 1 and shown.Hunter == 0, 'display class counts and invalid class')
        local raw = GetGroupMemberCounts()
        assert(raw.TANK == 1 and raw.HEALER == 1 and raw.DAMAGER == 0 and raw.NOROLE == 1, 'raw group counts')
        assert(UnitSetRole('player', 'DAMAGER'))
        assert(UnitSetRole('party1', 'TANK'))
        assert(UnitSetRole('party2', 'DAMAGER'))
        shown = GetGroupMemberCountsForDisplay()
        assert(shown.TANK == 1 and shown.HEALER == 0 and shown.DAMAGER == 2 and shown.NOROLE == 0, 'changed display group counts')
    "#).expect("original PartyUtil observes assigned group roles");
    env.state().borrow_mut().party_group_active = false;
    env.exec(r#"
        local raw = GetGroupMemberCounts()
        assert(raw.TANK == 0 and raw.HEALER == 0 and raw.DAMAGER == 0 and raw.NOROLE == 0, 'solo excludes inactive roster and player')
        assert(UnitSetRole('player', 'NONE'))
        local shown = GetGroupMemberCountsForDisplay()
        assert(shown.TANK == 0 and shown.HEALER == 0 and shown.DAMAGER == 0 and shown.NOROLE == 0, 'solo unassigned display')
        assert(shown.Warrior == 0 and shown.Paladin == 0 and shown.Hunter == 0, 'solo class counts')
    "#).expect("solo role counts");
}
