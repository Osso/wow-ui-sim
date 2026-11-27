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
    let (us_id, us_name, us_character, us_battle_net) = ticket_for_region(env, 1, "US");
    let (eu_id, eu_name, eu_character, eu_battle_net) = ticket_for_region(env, 3, "EU");
    assert_eq!((us_id, us_name.as_str(), eu_id, eu_name.as_str()), (1, "US", 3, "EU"));
    for (ticket, region) in [
        (&us_character, "US"),
        (&us_battle_net, "US"),
        (&eu_character, "EU"),
        (&eu_battle_net, "EU"),
    ] {
        assert!(ticket.contains(region), "{ticket} should name {region}");
    }
    assert_ne!(us_character, eu_character);
    assert_ne!(us_battle_net, eu_battle_net);
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
