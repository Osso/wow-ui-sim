#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]
//! UnitIsInMyGuild / IsPlayerInGuildFromGUID over the host guild roster.

use wow_ui_sim::lua_api::WowLuaEnv;

const MEMBERSHIP: &str = r#"
    A_Admin.SetPartySize(3)
    A_Admin.ClearGuild()
    local player, first, second = UnitGUID("player"), UnitGUID("party1"), UnitGUID("party2")
    assert(first and second and first ~= second)
    assert(not UnitIsInMyGuild("player") and not IsPlayerInGuildFromGUID(player), "no guild yet")
    A_Admin.JoinGuild("Heroes", "Officer", 3)
    A_Admin.SetGuildRoster(string.format([[ [
        {"name":"Ally","guid":"%s","rankIndex":2,"online":true},
        {"name":"Unseeded","rankIndex":3,"online":false}
    ] ]], first))
    assert(UnitIsInMyGuild("player") and IsPlayerInGuildFromGUID(player), "the player is a member")
    assert(UnitIsInMyGuild("party1") and IsPlayerInGuildFromGUID(first), "roster GUID matches")
    assert(not UnitIsInMyGuild("party2") and not IsPlayerInGuildFromGUID(second), "outside roster")
    assert(not UnitIsInMyGuild("party9") and not UnitIsInMyGuild("nonsense"), "missing units")
    A_Admin.SetGuildRoster(string.format([[ [{"name":"Other","guid":"%s","rankIndex":2,"online":true}] ]], second))
    assert(not UnitIsInMyGuild("party1") and UnitIsInMyGuild("party2"), "roster replacement")
    A_Admin.LeaveGuild()
    assert(not UnitIsInMyGuild("player") and not UnitIsInMyGuild("party2"), "leaving drops membership")
"#;

#[test]
fn guild_membership_follows_guild_and_roster_inputs() {
    WowLuaEnv::new().unwrap().exec(MEMBERSHIP).unwrap();
}
