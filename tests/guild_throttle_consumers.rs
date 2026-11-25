#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

#[path = "common/prefork_full_ui_preload.rs"]
mod prefork_full_ui_preload;

use wow_ui_sim::lua_api::WowLuaEnv;

const GUILD_NEWS_CONSUMER_SCRIPT: &str = r#"
    A_Admin.SetManualClock(10)
    local news = CommunitiesFrame.GuildDetailsFrame.News
    local updates = 0
    news:HookScript('OnEvent', function(_, event)
        if event == 'GUILD_NEWS_UPDATE' then updates = updates + 1 end
    end)

    local function open_guild_info()
        ToggleCommunitiesFrame()
        CommunitiesFrame.GuildInfoTab:Click()
    end

    open_guild_info()
    assert(updates == 2, 'first guild-info open sends the request and sort update')
    ToggleCommunitiesFrame()
    open_guild_info()
    assert(updates == 3, 'second open inside the window only runs GuildNewsSort')
    A_Admin.AdvanceTime(900)
    ToggleCommunitiesFrame()
    open_guild_info()
    assert(updates == 5, 'request after the interval reaches the original news frame')
    return updates
"#;

fn load_communities() -> WowLuaEnv {
    let env = prefork_full_ui_preload::preload_full_game_ui()
        .expect("full Blizzard UI should load for Communities consumer tests");
    env.exec("assert(C_AddOns.IsAddOnLoaded('Blizzard_Communities'))")
        .expect("the active profile selected Blizzard_Communities from its TOC");
    env.state().borrow_mut().world.guild_name = Some("Throttle Test Guild".into());
    env
}

fn open_guild_info(env: &WowLuaEnv) {
    env.exec("ToggleCommunitiesFrame(); CommunitiesFrame.GuildInfoTab:Click()")
        .expect("the selected Communities guild-info tab should open");
}

fn close_communities(env: &WowLuaEnv) {
    env.exec("ToggleCommunitiesFrame()")
        .expect("the Communities panel should close through its toggle path");
}

fn queued_roster_updates(env: &WowLuaEnv) -> usize {
    env.state()
        .borrow()
        .events
        .pending()
        .iter()
        .filter(|event| event.name == "GUILD_ROSTER_UPDATE")
        .count()
}

#[test]
fn communities_guild_info_open_throttles_roster_request() {
    let env = load_communities();
    env.exec("A_Admin.SetManualClock(10)").unwrap();
    let before = queued_roster_updates(&env);
    open_guild_info(&env);
    close_communities(&env);
    open_guild_info(&env);
    env.exec("A_Admin.AdvanceTime(10)").unwrap();
    close_communities(&env);
    open_guild_info(&env);
    assert_eq!(queued_roster_updates(&env) - before, 2);
}

#[test]
fn communities_guild_news_open_drops_throttled_request() {
    let env = load_communities();
    let updates: i32 = env.eval(GUILD_NEWS_CONSUMER_SCRIPT).unwrap();
    assert_eq!(updates, 5);
}
