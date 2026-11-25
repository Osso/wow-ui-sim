#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

#[path = "common/prefork_full_ui_preload.rs"]
mod prefork_full_ui_preload;

use wow_ui_sim::lua_api::WowLuaEnv;

fn load_communities() -> WowLuaEnv {
    let env = prefork_full_ui_preload::preload_full_game_ui()
        .expect("full Blizzard UI should load for Communities consumer tests");
    env.exec("assert(C_AddOns.IsAddOnLoaded('Blizzard_Communities'))")
        .expect("the active profile selected Blizzard_Communities from its TOC");
    env
}

#[test]
fn communities_guild_info_open_throttles_roster_request() {
    let env = load_communities();
    env.exec(
        r#"
        A_Admin.SetManualClock(10)
        CommunitiesFrame:ToggleSubPanel(CommunitiesFrame.GuildDetailsFrame)
        CommunitiesFrame:ToggleSubPanel(CommunitiesFrame.GuildDetailsFrame)
        CommunitiesFrame:ToggleSubPanel(CommunitiesFrame.GuildDetailsFrame)
        A_Admin.AdvanceTime(9)
        CommunitiesFrame:ToggleSubPanel(CommunitiesFrame.GuildDetailsFrame)
        A_Admin.AdvanceTime(1)
        CommunitiesFrame:ToggleSubPanel(CommunitiesFrame.GuildDetailsFrame)
        "#,
    )
    .expect("open the selected Communities guild info frame repeatedly");
    let roster_updates = env
        .state()
        .borrow()
        .events
        .pending()
        .iter()
        .filter(|event| event.name == "GUILD_ROSTER_UPDATE")
        .count();
    assert_eq!(roster_updates, 2);
}

#[test]
fn communities_guild_news_open_drops_throttled_request() {
    let env = load_communities();
    let updates: i32 = env
        .eval(
            r#"
            A_Admin.SetManualClock(10)
            local updates = 0
            local listener = CreateFrame('Frame')
            listener:RegisterEvent('GUILD_NEWS_UPDATE')
            listener:SetScript('OnEvent', function() updates = updates + 1 end)
            CommunitiesFrame:ToggleSubPanel(CommunitiesFrame.GuildDetailsFrame)
            CommunitiesFrame:ToggleSubPanel(CommunitiesFrame.GuildDetailsFrame)
            CommunitiesFrame:ToggleSubPanel(CommunitiesFrame.GuildDetailsFrame)
            A_Admin.AdvanceTime(899)
            CommunitiesFrame:ToggleSubPanel(CommunitiesFrame.GuildDetailsFrame)
            A_Admin.AdvanceTime(1)
            CommunitiesFrame:ToggleSubPanel(CommunitiesFrame.GuildDetailsFrame)
            return updates
            "#,
        )
        .unwrap();
    // Each open fires GuildNewsSort's update; QueryGuildNews adds one more only
    // on the first open and after the throttle window.
    assert_eq!(updates, 5);
}
