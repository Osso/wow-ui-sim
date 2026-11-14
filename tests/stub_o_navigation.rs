//! Original startup-navigation consumers and separate bootstrap retirement boundary.
#![cfg(all(feature = "optional-tests", feature = "client-retail"))]
use crate::common::blizzard_addon_harness::load_blizzard_addon_closure_into_env;
use wow_ui_sim::lua_api::WowLuaEnv;

fn nav_env(roots: &[&str]) -> WowLuaEnv {
    // Existing fixture loads original panel owner/dependencies + startup events.
    // This is fixture-local, not new blanket WowLuaEnv loading.
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let env = crate::common::blizzard_addon_harness::new_blizzard_addon_env(&ui);
    wow_ui_sim::xml::register_intrinsic_templates();
    crate::common::panel_fixtures::load_panel_addons(&env);
    let loaded = load_blizzard_addon_closure_into_env(&env, &ui, roots, &[]);
    for root in roots {
        assert!(loaded.iter().any(|s| s == root));
    }
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::fire_startup_events(&env);
    env.exec(
        r#"
        function NavigationOriginalSource(fn, suffix)
            local source = debug.getinfo(fn, 'S').source:gsub('\\', '/')
            assert(source:find(suffix, 1, true), source)
        end
    "#,
    )
    .unwrap();
    env
}

#[test]
fn navigation_has_no_synthetic_bootstrap_functions() {
    let env = WowLuaEnv::new().unwrap();
    for symbol in [
        "OpenAllBags",
        "ToggleLFDParentFrame",
        "UpdateRaidAndPartyFrames",
    ] {
        assert!(
            env.eval::<bool>(&format!("return rawget(_G, {symbol:?}) == nil"))
                .unwrap(),
            "{symbol} synthetic publication"
        );
    }
}

#[test]
fn original_tutorial_consumer_opens_all_modeled_bags() {
    let env = nav_env(&["Blizzard_UIPanels_Game", "Blizzard_TutorialManager"]);
    env.exec(r#"
        NavigationOriginalSource(OpenAllBags, 'Blizzard_UIPanels_Game/Mainline/ContainerFrame.lua')
        NavigationOriginalSource(TutorialHelper.OpenAllBags, 'Blizzard_TutorialManager/Blizzard_TutorialHelper.lua')
        SetCVar('combinedBags', '0')
        CloseAllBags()
        assert(not IsAnyBagOpen(), 'fixture failed to close original bags')
        local populated = 0
        for i=0,NUM_TOTAL_BAG_FRAMES do
            if C_Container.GetContainerNumSlots(i) > 0 then populated=populated+1 end
        end
        assert(populated >= 2, 'need existing modeled backpack and equipped bag; do not invent API results')
        FRAME_THAT_OPENED_BAGS = nil
        -- Original unmodified tutorial consumer calls global OpenAllBags.
        TutorialHelper:OpenAllBags()
        assert(FRAME_THAT_OPENED_BAGS == nil)
        assert(IsBagOpen(Enum.BagIndex.Backpack))
        for i=1,NUM_TOTAL_BAG_FRAMES do
            if C_Container.GetContainerNumSlots(i) > 0 then
                assert(IsBagOpen(i), 'modeled equipped bag did not open: '..i)
            end
        end
        CloseAllBags()
    "#).unwrap();
}

#[test]
fn original_lfd_link_consumer_respects_toggle_lock_and_selects_lfd_panel() {
    let env = nav_env(&["Blizzard_UIPanels_Game", "Blizzard_GroupFinder"]);
    env.exec(
        r#"
        NavigationOriginalSource(ToggleLFDParentFrame, 'Blizzard_GroupFinder/Mainline/PVEFrame.lua')
        assert(LinkUtil.IsLinkHandlerRegistered(LinkTypes.GroupFinderUI))
        assert(LinkTypes.GroupFinderUI == 'lfd')
        HideUIPanel(PVEFrame)
        -- Actual original Shared/ItemRefHandlersShared.lua handler calls the
        -- helper; do not replace the helper or PVEFrame_ToggleFrame with a spy.
        DISALLOW_FRAME_TOGGLING = true
        assert(LinkUtil.ProcessLink('lfd', 'Group Finder', {}) == LinkProcessorResponse.Handled)
        assert(not PVEFrame:IsShown(), 'locked original consumer must not open PVE')
        DISALLOW_FRAME_TOGGLING = nil
        assert(UnitFactionGroup('player') ~= 'Neutral', 'fixture needs eligible seeded faction')
        assert(not Kiosk.IsEnabled())
        assert(C_LFGInfo.CanPlayerUseGroupFinder())
        LinkUtil.ProcessLink('lfd', 'Group Finder', {})
        assert(PVEFrame:IsShown() and GroupFinderFrame:IsShown() and LFDParentFrame:IsShown())
        LinkUtil.ProcessLink('lfd', 'Group Finder', {})
        assert(not PVEFrame:IsShown())
    "#,
    )
    .unwrap();
}

#[test]
fn original_group_roster_consumer_refreshes_compact_manager_from_group_model() {
    let env = nav_env(&[
        "Blizzard_RaidFrame",
        "Blizzard_CompactRaidFrames",
        "Blizzard_Game",
    ]);
    env.exec(r#"
        NavigationOriginalSource(UpdateRaidAndPartyFrames, 'Blizzard_RaidFrame/Mainline/RaidFrame.lua')
        NavigationOriginalSource(GameEvent.HandleGroupRosterUpdate, 'Blizzard_Game/Shared/EventImplementation.lua')
        assert(not EditModeManagerFrame:AreRaidFramesForcedShown())
        assert(not EditModeManagerFrame:ArePartyFramesForcedShown())
        assert(not C_Commentator.IsSpectating())
        assert(not C_GameRules.IsGameRuleActive(Enum.GameRule.CompactRaidFrameManagerDisabled))
    "#).unwrap();
    // Same real inputs as party_raid_probes.rs: no mocked IsInGroup/IsInRaid.
    env.state().borrow_mut().party_group_active = false;
    env.exec(
        r#"
        assert(not IsInGroup())
        CompactRaidFrameManager:Show()
        GameEvent.HandleGroupRosterUpdate(nil, 'GROUP_ROSTER_UPDATE')
        assert(not CompactRaidFrameManager:IsShown())
    "#,
    )
    .unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.party_group_active = true;
        let member = state
            .party_members
            .first()
            .expect("existing seeded party roster")
            .clone();
        while state.party_members.len() < 6 {
            state.party_members.push(member.clone());
        }
    }
    env.exec(
        r#"
        assert(IsInGroup() and IsInRaid())
        GameEvent.HandleGroupRosterUpdate(nil, 'GROUP_ROSTER_UPDATE')
        assert(CompactRaidFrameManager:IsShown())
    "#,
    )
    .unwrap();
}
