#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]
//! `GetCurrentRegionName` reads the region supplied by `A_Admin.SetCurrentRegion`.
//! Original consumer: `ClubTicketUtil.FormatTicket` (Blizzard_Communities/
//! CommunitiesTicketManagerDialog.lua:70-90) embeds the region in invite links.

use wow_ui_sim::lua_api::WowLuaEnv;

const FORMAT_TICKET: &str = r#"
C_AddOns.LoadAddOn("Blizzard_Communities")
local character = ClubTicketUtil.FormatTicket({ clubType = Enum.ClubType.Character }, "T1")
local battleNet = ClubTicketUtil.FormatTicket({ clubType = Enum.ClubType.BattleNet }, "T2")
return GetCurrentRegion(), GetCurrentRegionName(), character, battleNet
"#;

fn ticket_for_region(env: &WowLuaEnv, id: i32, name: &str) -> (i32, String, String, String) {
    env.exec(&format!("A_Admin.SetCurrentRegion({id}, '{name}')"))
        .expect("the region input should be accepted");
    env.eval(FORMAT_TICKET)
        .expect("the original ticket formatter should run")
}

fn assert_region_tickets_follow_input(env: &WowLuaEnv) {
    // Character links carry the region; Battle.net links switch host only for CN.
    let faction: String = env
        .eval("return (UnitFactionGroup('player'))")
        .expect("the player faction should be readable");
    assert_eq!(
        ticket_for_region(env, 1, "US"),
        (
            1,
            "US".to_string(),
            format!("https://www.worldofwarcraft.com/invite/T1?region=US&faction={faction}"),
            "https://blizzard.com/invite/T2".to_string(),
        )
    );
    assert_eq!(
        ticket_for_region(env, 5, "CN"),
        (
            5,
            "CN".to_string(),
            format!("https://www.wowchina.com/invite/T1?region=CN&faction={faction}"),
            "https://blizzardgames.cn/invite/T2".to_string(),
        )
    );
}

#[cfg(feature = "client-retail")]
prefork_full_ui_case! {
fn club_ticket_links_follow_the_supplied_region(env: &WowLuaEnv) {
    assert_region_tickets_follow_input(env);
}
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_club_ticket_links_follow_the_supplied_region() {
    crate::common::with_timeout(120, || {
        let env = WowLuaEnv::new().expect("create Forever Lua environment");
        crate::common::load_game_startup_inventory(&env);
        env.sync_addon_names_to_lua();
        env.apply_post_load_workarounds();
        let game = wow_ui_sim::screen::ScreenKind::Game;
        wow_ui_sim::startup::fire_startup_events_for_screen(&env, game);
        assert_region_tickets_follow_input(&env);
    });
}
