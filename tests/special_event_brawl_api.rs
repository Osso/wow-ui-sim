//! API-level coverage while the original HonorFrame path awaits WM25.
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
        min_level: 20,
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

#[test]
fn special_event_brawl_query_returns_independent_integer_record() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(select('#', C_PvP.GetSpecialEventBrawlInfo()) == 1); assert(C_PvP.GetSpecialEventBrawlInfo() == nil)").unwrap();
    env.state().borrow_mut().special_event_brawl = Some(brawl());
    env.exec(
        r#"
        local info = C_PvP.GetSpecialEventBrawlInfo()
        assert(info.brawlID == 701 and info.name == 'Fixture special brawl')
        assert(info.shortDescription == 'Short' and info.longDescription == 'Long')
        assert(info.canQueue and not info.groupsAllowed and info.crossFactionAllowed)
        assert(info.minLevel == 20 and info.maxLevel == 80 and info.brawlType == 2)
        assert(info.minItemLevel == 123 and info.timeLeftUntilNextChange == 91)
        assert(not info.includesAllArenas and info.shouldHideRewardIcon)
        assert(info.mapNames[1] == 'First map' and info.mapNames[2] == 'Second map')
        assert(info.isActive == nil)
        info.mapNames[1] = 'changed'
        assert(C_PvP.GetSpecialEventBrawlInfo().mapNames[1] == 'First map')
    "#,
    )
    .unwrap();
    env.state().borrow_mut().special_event_brawl = None;
    env.exec("assert(C_PvP.GetSpecialEventBrawlInfo() == nil)")
        .unwrap();
}
