//! Original PvP frame consumer of the supplied special-event brawl record.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::SpecialEventBrawlInfo;

fn brawl() -> SpecialEventBrawlInfo {
    SpecialEventBrawlInfo {
        brawl_id: 701,
        name: "Fixture special brawl".into(),
        short_description: "Short".into(),
        long_description: "Long".into(),
        can_queue: true,
        min_level: 1,
        max_level: 80,
        groups_allowed: false,
        cross_faction_allowed: true,
        time_left_until_next_change: Some(91),
        brawl_type: 2,
        map_names: vec!["First map".into(), "Second map".into()],
        includes_all_arenas: false,
        min_item_level: 123,
        should_hide_reward_icon: true,
    }
}

prefork_full_ui_case! {
fn honor_frame_special_brawl_tracks_supplied_record(env: &WowLuaEnv) {
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let toc = ui.join("Blizzard_PVPUI/Blizzard_PVPUI_Mainline.toc");
    wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    env.exec(r#"
        local source = debug.getinfo(HonorFrameBonusFrame_Update, 'S').source
        assert(source:find('Blizzard_PVPUI.lua', 1, true))
        HonorFrameBonusFrame_Update()
        assert(not HonorFrame.BonusFrame.BrawlButton2:IsShown())
        assert(select('#', C_PvP.GetSpecialEventBrawlInfo()) == 1)
        assert(C_PvP.GetSpecialEventBrawlInfo() == nil)
    "#).unwrap();
    env.state().borrow_mut().special_event_brawl = Some(brawl());
    env.exec(r#"
        HonorFrameBonusFrame_Update()
        local button = HonorFrame.BonusFrame.BrawlButton2
        assert(button:IsShown())
        assert(button.Title:GetText() == 'Fixture special brawl')
        local info = C_PvP.GetSpecialEventBrawlInfo()
        assert(info.brawlID == 701 and info.groupsAllowed == false)
        assert(info.mapNames[1] == 'First map' and info.mapNames[2] == 'Second map')
    "#).unwrap();
    env.state().borrow_mut().special_event_brawl = None;
    env.exec(r#"
        HonorFrameBonusFrame_Update()
        assert(not HonorFrame.BonusFrame.BrawlButton2:IsShown())
    "#).unwrap();
}
}
