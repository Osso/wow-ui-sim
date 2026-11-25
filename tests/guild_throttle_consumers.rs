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
    env.exec("A_Admin.SetGuildInfo('Throttle Test Guild', 'Member', 2)")
        .expect("guild context should be supplied through the admin API");
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

fn open_guild_perks(env: &WowLuaEnv) {
    env.exec(
        "A_Admin.SetGuildInfo('Perk Test Guild', 'Member', 2); A_Admin.SetGuildPerks(1459, 1, 6673, 2); ToggleCommunitiesFrame(); CommunitiesFrame.GuildInfoTab:Click()",
    )
    .expect("guild perks panel should open through the original guild tab");
    wow_ui_sim::startup::fire_one_on_update_tick(env);
}

fn guild_perk_rows(env: &WowLuaEnv) -> (bool, i32, String, String) {
    env.eval(
        r#"
        local perks = CommunitiesFrame.GuildBenefitsFrame.Perks
        local names = {}
        perks.ScrollBox:ForEachFrame(function(button)
            names[#names + 1] = button.Name:GetText()
        end)
        return perks:IsShown(), #names, names[1] or "", names[2] or ""
        "#,
    )
    .expect("original Communities guild perks rows should be available")
}

fn clear_guild_perks_and_reopen(env: &WowLuaEnv) {
    env.exec(
        "A_Admin.SetGuildPerks(); ToggleCommunitiesFrame(); ToggleCommunitiesFrame(); CommunitiesFrame.GuildInfoTab:Click()",
    )
    .expect("guild perks panel should refresh after the supplied perks are cleared");
}

fn guild_perks_panel_is_shown(env: &WowLuaEnv) -> bool {
    env.eval("return CommunitiesFrame.GuildBenefitsFrame.Perks:IsShown()")
        .expect("guild perks panel visibility should be observable")
}

#[test]
fn communities_guild_perks_panel_uses_supplied_spell_rows() {
    let env = load_communities();
    open_guild_perks(&env);
    let rows = guild_perk_rows(&env);
    assert!(rows.0, "perks panel should show when two perks are supplied");
    assert_eq!(rows.1, 2, "two supplied perks should produce two rows");
    assert!(!rows.2.is_empty(), "first perk row should have a name");
    assert_ne!(rows.2, rows.3, "perk rows should have distinct names");
    clear_guild_perks_and_reopen(&env);
    assert!(!guild_perks_panel_is_shown(&env), "zero perks should hide panel");
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
