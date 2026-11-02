//! Original HonorFrame regular-brawl selection with a deterministic clock.
#![cfg(feature = "client-retail")]

use std::time::Duration;
use wow_ui_sim::c_api::c_pvp::available_brawl::AvailableBrawlCache;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::PvpBrawlInfo;

fn record() -> PvpBrawlInfo {
    PvpBrawlInfo {
        brawl_id: 701,
        name: "Regular fixture".into(),
        short_description: "Short".into(),
        long_description: "Long".into(),
        can_queue: false,
        min_level: 1,
        max_level: 80,
        groups_allowed: false,
        cross_faction_allowed: true,
        time_left_until_next_change: None,
        brawl_type: 2,
        map_names: vec!["Fixture map".into()],
        includes_all_arenas: false,
        min_item_level: 123.0,
        should_hide_reward_icon: true,
    }
}

prefork_full_ui_case! {
fn available_brawl_expiry_drives_honor_frame(env: &WowLuaEnv) {
    crate::pvp_reward_consumers::load_honor_frame(env);
    env.exec("A_Admin.SetManualClock(100)").unwrap();
    env.state().borrow_mut().available_brawl = Some(AvailableBrawlCache {
        record: record(),
        received_at: Duration::from_secs(100),
        duration: Some(Duration::from_secs(10)),
    });
    env.exec(r#"
        HonorFrameBonusFrame_Update()
        local brawl = C_PvP.GetAvailableBrawlInfo()
        assert(brawl and brawl.brawlID == 701 and not brawl.canQueue)
        assert(brawl.timeLeftUntilNextChange == 10)
        assert(HonorFrame.BonusFrame.BrawlButton.Title:GetText() ~= BRAWL_CLOSED)
        A_Admin.AdvanceTime(10)
        HonorFrameBonusFrame_Update()
        brawl = C_PvP.GetAvailableBrawlInfo()
        assert(brawl and brawl.timeLeftUntilNextChange == 0)
        assert(HonorFrame.BonusFrame.BrawlButton.Title:GetText() == BRAWL_CLOSED)
        A_Admin.AdvanceTime(0.001)
        HonorFrameBonusFrame_Update()
        assert(C_PvP.GetAvailableBrawlInfo() == nil)
        assert(HonorFrame.BonusFrame.BrawlButton.Title:GetText() == BRAWL_CLOSED)
    "#).unwrap();
    env.state().borrow_mut().available_brawl = Some(AvailableBrawlCache {
        record: record(),
        received_at: Duration::from_millis(110001),
        duration: None,
    });
    env.exec(r#"
        HonorFrameBonusFrame_Update()
        local brawl = C_PvP.GetAvailableBrawlInfo()
        assert(brawl and brawl.timeLeftUntilNextChange == 0)
        A_Admin.AdvanceTime(0.001)
        HonorFrameBonusFrame_Update()
        assert(C_PvP.GetAvailableBrawlInfo() == nil)
    "#).unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        let cached = sim.available_brawl.as_mut().unwrap();
        cached.record.brawl_id = 0;
        cached.duration = Some(Duration::from_secs(10));
    }
    env.exec("HonorFrameBonusFrame_Update(); assert(C_PvP.GetAvailableBrawlInfo() == nil)").unwrap();
}
}
