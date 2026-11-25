#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common::blizzard_addon_harness::{blizzard_ui_dir, build_blizzard_addon_closure_env};
use wow_ui_sim::lua_api::WowLuaEnv;

fn load_communities() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let (env, loaded) = build_blizzard_addon_closure_env(&ui, &["Blizzard_Communities"], &[]);
    assert!(loaded.iter().any(|name| name == "Blizzard_Communities"));
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::settle_headless_startup(&env);
    env
}

#[test]
fn communities_guild_info_open_throttles_roster_request() {
    let env = load_communities();
    env.exec(
        r#"
        A_Admin.SetManualClock(10)
        CommunitiesGuildInfoFrame:Show()
        CommunitiesGuildInfoFrame:Hide()
        CommunitiesGuildInfoFrame:Show()
        A_Admin.AdvanceTime(9)
        CommunitiesGuildInfoFrame:Hide()
        CommunitiesGuildInfoFrame:Show()
        A_Admin.AdvanceTime(1)
        CommunitiesGuildInfoFrame:Hide()
        CommunitiesGuildInfoFrame:Show()
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
            CommunitiesGuildNewsFrame:Show()
            CommunitiesGuildNewsFrame:Hide()
            CommunitiesGuildNewsFrame:Show()
            A_Admin.AdvanceTime(899)
            CommunitiesGuildNewsFrame:Hide()
            CommunitiesGuildNewsFrame:Show()
            A_Admin.AdvanceTime(1)
            CommunitiesGuildNewsFrame:Hide()
            CommunitiesGuildNewsFrame:Show()
            return updates
            "#,
        )
        .unwrap();
    // Each open fires GuildNewsSort's update; QueryGuildNews adds one more only
    // on the first open and after the throttle window.
    assert_eq!(updates, 6);
}
