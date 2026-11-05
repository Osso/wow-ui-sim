//! Forever runtime consumers of the supplied brawl state.
//!
//! The pinned WowForever `PvpInfoDocumentation.lua` declares
//! `C_PvP.GetActiveBrawlInfo` (:109), `GetAvailableBrawlInfo` (:255) and
//! `GetSpecialEventBrawlInfo` (:775) with the same `PvpBrawlInfo` structure as
//! Retail, and `TooltipInfoDocumentation.lua:710` declares `GetPvpBrawl`.
//!
//! `Blizzard_PVPMatch/Blizzard_PVPMatch.toc` allows `standard, camelot` and is
//! an eager root (`LoadOnDemand: 0`), so `PVPMatchUtil.lua:20` is a
//! runtime-selected consumer on Forever. `Blizzard_PVPUI/Blizzard_PVPUI.toc`
//! excludes `camelot`, so its brawl tooltip and honor-frame paths do not run on
//! this profile. `TooltipDataHandler.lua:577` maps `SetPvpBrawl` to
//! `GetPvpBrawl` and loads on Forever.

#![cfg(feature = "client-wowforever")]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::lua_api::state::{PvpBrawlInfo, SpecialEventBrawlInfo};

fn active(name: &str, brawl_id: i64, can_queue: bool) -> PvpBrawlInfo {
    PvpBrawlInfo {
        brawl_id,
        name: name.to_string(),
        short_description: "never shown".to_string(),
        long_description: "Fixture rules.".to_string(),
        can_queue,
        min_level: 10,
        max_level: 80,
        groups_allowed: true,
        cross_faction_allowed: false,
        time_left_until_next_change: Some(7200.0),
        brawl_type: 4,
        map_names: vec!["Fixture map".to_string()],
        includes_all_arenas: false,
        min_item_level: 300.0,
        should_hide_reward_icon: true,
    }
}

fn special(name: &str) -> SpecialEventBrawlInfo {
    SpecialEventBrawlInfo {
        brawl_id: 702,
        name: name.to_string(),
        short_description: "never shown".to_string(),
        long_description: "Special rules.".to_string(),
        can_queue: false,
        min_level: 10,
        max_level: 80,
        groups_allowed: false,
        cross_faction_allowed: true,
        time_left_until_next_change: None,
        brawl_type: 3,
        map_names: vec!["Arathi Basin".to_string()],
        includes_all_arenas: true,
        min_item_level: 300,
        should_hide_reward_icon: true,
    }
}

#[test]
fn forever_pvp_match_consumer_reads_the_supplied_active_brawl() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    env.exec(
        r#"
        local source = debug.getinfo(PVPMatchUtil.InSoloShuffleBrawl, 'S').source
        assert(source:find('PVPMatchUtil.lua', 1, true), source)
        assert(PVPMatchUtil.InSoloShuffleBrawl() == false, 'no active record')
        "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .active_brawl = Some(active("Solo Shuffle Brawl", 130, true));
    env.exec("assert(PVPMatchUtil.InSoloShuffleBrawl() == true, 'brawl 130')")
        .unwrap();
    env.state()
        .borrow_mut()
        .active_brawl = Some(active("Southshore Brawl", 701, true));
    env.exec("assert(PVPMatchUtil.InSoloShuffleBrawl() == false, 'brawl 701')")
        .unwrap();
}

#[test]
fn forever_declared_brawl_queries_publish_supplied_records() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    assert!(
        env.eval::<bool>(
            "return type(C_PvP.GetActiveBrawlInfo) == 'function' \
             and type(C_PvP.GetAvailableBrawlInfo) == 'function' \
             and type(C_PvP.GetSpecialEventBrawlInfo) == 'function'"
        )
        .unwrap(),
        "Forever pinned docs declare all three C_PvP brawl getters"
    );
    env.state()
        .borrow_mut()
        .active_brawl = Some(active("Active Brawl", 701, true));
    env.state().borrow_mut().special_event_brawl = Some(special("Special Brawl"));
    env.exec(
        r#"
        local active = C_PvP.GetActiveBrawlInfo()
        assert(active and active.brawlID == 701 and active.name == 'Active Brawl')
        local special = C_PvP.GetSpecialEventBrawlInfo()
        assert(special and special.brawlID == 702 and special.name == 'Special Brawl')
        assert(C_PvP.GetAvailableBrawlInfo() == nil, 'no available record')
        "#,
    )
    .unwrap();
}

#[test]
fn forever_tooltip_data_handler_routes_set_pvp_brawl_to_the_getter() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    env.state()
        .borrow_mut()
        .active_brawl = Some(active("Fixture brawl name", 130, true));
    env.exec(
        r#"
        GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
        GameTooltip:SetPvpBrawl()
        local info = GameTooltip:GetPrimaryTooltipInfo()
        assert(info and info.getterName == 'GetPvpBrawl', 'handler getter name')
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(data and data.type == Enum.TooltipDataType.PvPBrawl, 'brawl tooltip data')
        assert(data.lines[1].leftText == 'Fixture brawl name', 'record name row')
        "#,
    )
    .unwrap();
}
