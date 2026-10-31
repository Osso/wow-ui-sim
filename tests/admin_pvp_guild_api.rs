//! Tests for A_Admin PvP and Guild API.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn test_admin_collections_pvp_method_surface_is_registered() {
    let env = env();
    let missing_method: Option<String> = env
        .eval(
            r#"
            local methods = {
                "AddTransmog",
                "RemoveTransmog",
                "AddTransmogAppearance",
                "SetTransmogForSlot",
                "CollectHeirloom",
                "UncollectHeirloom",
                "SetMountCollected",
                "SetPetCollected",
                "SetToyCollected",
                "SetCampsiteCollected",
                "SetAchievementEarned",
                "HasAchievement",
                "CollectMount",
                "UncollectMount",
                "CollectPet",
                "UncollectPet",
                "CollectToy",
                "UncollectToy",
                "CollectCampsite",
                "UncollectCampsite",
                "EarnAchievement",
                "SetPvPEnabled",
                "SetHonorLevel",
                "SetGuildInfo",
                "JoinGuild",
                "ClearGuild",
                "LeaveGuild",
            }

            for _, method in ipairs(methods) do
                if type(A_Admin[method]) ~= "function" then
                    return method
                end
            end

            return nil
        "#,
        )
        .unwrap();

    assert!(
        missing_method.is_none(),
        "missing A_Admin collections/PvP method: {missing_method:?}"
    );
}

// ============================================================================
// SetHonorLevel
// ============================================================================

#[test]
fn test_set_honor_level_unit_honor_level_returns_level() {
    let env = env();
    let level: i32 = env
        .eval(
            r#"
            A_Admin.SetHonorLevel(50)
            return UnitHonorLevel("player")
            "#,
        )
        .unwrap();
    assert_eq!(level, 50);
}

#[test]
fn test_unit_honor_returns_zero() {
    let env = env();
    let honor: i32 = env.eval(r#"return UnitHonor("player")"#).unwrap();
    assert_eq!(honor, 0);
}

#[test]
fn test_unit_power_bar_timer_info_returns_nil() {
    let env = env();
    let is_nil: bool = env
        .eval(r#"return UnitPowerBarTimerInfo("player", 1) == nil"#)
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_get_send_mail_price_returns_30() {
    let env = env();
    let price: i32 = env.eval("return GetSendMailPrice()").unwrap();
    assert_eq!(price, 30);
}

#[test]
fn test_get_web_ticket_returns_nil() {
    let env = env();
    let is_nil: bool = env.eval("return GetWebTicket() == nil").unwrap();
    assert!(is_nil);
}

#[test]
fn test_reset_cursor_does_not_error() {
    let env = env();
    env.eval::<()>("ResetCursor()").unwrap();
}

#[test]
fn test_set_honor_level_zero() {
    let env = env();
    let level: i32 = env
        .eval(
            r#"
            A_Admin.SetHonorLevel(0)
            return UnitHonorLevel("player")
            "#,
        )
        .unwrap();
    assert_eq!(level, 0);
}

#[test]
fn test_set_honor_level_max() {
    let env = env();
    let level: i32 = env
        .eval(
            r#"
            A_Admin.SetHonorLevel(500)
            return UnitHonorLevel("player")
            "#,
        )
        .unwrap();
    assert_eq!(level, 500);
}

#[test]
fn test_set_honor_level_overwrites_previous() {
    let env = env();
    let level: i32 = env
        .eval(
            r#"
            A_Admin.SetHonorLevel(10)
            A_Admin.SetHonorLevel(75)
            return UnitHonorLevel("player")
            "#,
        )
        .unwrap();
    assert_eq!(level, 75);
}

// ============================================================================
// SetGuildInfo
// ============================================================================

#[test]
fn test_set_guild_info_c_guild_get_guild_info_name() {
    let env = env();
    let guild_name: String = env
        .eval(
            r#"
            A_Admin.SetGuildInfo("Epic Guild", "Officer", 150)
            local guildName = C_Guild.GetGuildInfo("player")
            return guildName
            "#,
        )
        .unwrap();
    assert_eq!(guild_name, "Epic Guild");
}

#[test]
fn test_set_guild_info_is_in_guild_returns_true() {
    let env = env();
    let in_guild: bool = env
        .eval(
            r#"
            A_Admin.SetGuildInfo("Epic Guild", "Officer", 150)
            return IsInGuild()
            "#,
        )
        .unwrap();
    assert!(in_guild);
}

#[test]
fn test_set_guild_info_c_guild_is_in_guild_returns_true() {
    let env = env();
    let in_guild: bool = env
        .eval(
            r#"
            A_Admin.SetGuildInfo("Epic Guild", "Officer", 150)
            return C_Guild.IsInGuild()
            "#,
        )
        .unwrap();
    assert!(in_guild);
}

#[test]
fn test_set_guild_info_c_guild_get_guild_info_rank() {
    let env = env();
    let (guild_name, guild_rank): (String, String) = env
        .eval(
            r#"
            A_Admin.SetGuildInfo("Some Guild", "Guild Master", 42)
            local name, rank = C_Guild.GetGuildInfo("player")
            return name, rank
            "#,
        )
        .unwrap();
    assert_eq!(guild_name, "Some Guild");
    assert_eq!(guild_rank, "Guild Master");
}

// ============================================================================
// ClearGuild
// ============================================================================

#[test]
fn test_clear_guild_is_in_guild_returns_false() {
    let env = env();
    let in_guild: bool = env
        .eval(
            r#"
            A_Admin.SetGuildInfo("Epic Guild", "Officer", 150)
            A_Admin.ClearGuild()
            return IsInGuild()
            "#,
        )
        .unwrap();
    assert!(!in_guild);
}

#[test]
fn test_clear_guild_c_guild_is_in_guild_returns_false() {
    let env = env();
    let in_guild: bool = env
        .eval(
            r#"
            A_Admin.SetGuildInfo("Epic Guild", "Officer", 150)
            A_Admin.ClearGuild()
            return C_Guild.IsInGuild()
            "#,
        )
        .unwrap();
    assert!(!in_guild);
}

#[test]
fn test_in_guild_by_default() {
    let env = env();
    let in_guild: bool = env.eval("return IsInGuild()").unwrap();
    assert!(in_guild);
}

#[test]
fn test_default_guild_name() {
    let env = env();
    let name: String = env.eval("return (C_Guild.GetGuildInfo('player'))").unwrap();
    assert_eq!(name, "Heroes of Azeroth");
}

#[test]
fn test_legacy_get_guild_info_matches_c_guild() {
    let env = env();
    let (legacy_name, c_name): (String, String) = env
        .eval(
            r#"
            return (GetGuildInfo("player")), (C_Guild.GetGuildInfo("player"))
            "#,
        )
        .unwrap();
    assert_eq!(legacy_name, "Heroes of Azeroth");
    assert_eq!(legacy_name, c_name);
}

// ============================================================================
// GuildQuit
// ============================================================================

#[test]
fn test_guild_quit_clears_guild() {
    let env = env();
    assert!(env.eval::<bool>("return IsInGuild()").unwrap());
    env.exec("GuildQuit()").unwrap();
    assert!(!env.eval::<bool>("return IsInGuild()").unwrap());
}

#[test]
fn test_guild_quit_fires_event() {
    let env = env();
    let fired: bool = env
        .eval(
            r#"
            local fired = false
            local f = CreateFrame("Frame")
            f:RegisterEvent("PLAYER_GUILD_UPDATE")
            f:SetScript("OnEvent", function() fired = true end)
            GuildQuit()
            return fired
            "#,
        )
        .unwrap();
    assert!(fired);
}

// ============================================================================
// A_Admin.JoinGuild
// ============================================================================

#[test]
fn test_join_guild_sets_info_and_fires_event() {
    let env = env();
    env.exec("GuildQuit()").unwrap(); // start fresh
    let (name, fired): (String, bool) = env
        .eval(
            r#"
            local fired = false
            local f = CreateFrame("Frame")
            f:RegisterEvent("PLAYER_GUILD_UPDATE")
            f:SetScript("OnEvent", function() fired = true end)
            A_Admin.JoinGuild("Test Guild", "Officer", 42)
            return (C_Guild.GetGuildInfo("player")), fired
            "#,
        )
        .unwrap();
    assert_eq!(name, "Test Guild");
    assert!(fired);
}

// ============================================================================
// A_Admin.LeaveGuild
// ============================================================================

#[test]
fn test_leave_guild_clears_and_fires_event() {
    let env = env();
    let (in_guild, fired): (bool, bool) = env
        .eval(
            r#"
            local fired = false
            local f = CreateFrame("Frame")
            f:RegisterEvent("PLAYER_GUILD_UPDATE")
            f:SetScript("OnEvent", function() fired = true end)
            A_Admin.LeaveGuild()
            return IsInGuild(), fired
            "#,
        )
        .unwrap();
    assert!(!in_guild);
    assert!(fired);
}

// ============================================================================
// GuildControlGetNumRanks
// ============================================================================

#[test]
fn test_guild_control_get_num_ranks_returns_zero_when_not_in_guild() {
    let env = env();
    env.exec("A_Admin.ClearGuild()").unwrap();
    let ranks: i32 = env.eval("return GuildControlGetNumRanks()").unwrap();
    assert_eq!(ranks, 0);
}

#[test]
fn test_guild_control_set_rank_does_not_error() {
    let env = env();
    env.eval::<()>("GuildControlSetRank(1)").unwrap();
}

// ============================================================================
// GetGuildFactionGroup
// ============================================================================

#[test]
fn test_get_guild_faction_group_returns_default_faction() {
    let env = env();
    let faction: i32 = env.eval("return GetGuildFactionGroup()").unwrap();
    assert_eq!(faction, 1);
}

// ============================================================================
// GetGroupMemberCounts
// ============================================================================

// PartyUtil.lua:108-112 consumes counts including unassigned members as damage.
#[cfg(not(feature = "client-wrath"))]
#[test]
fn test_get_group_member_counts_includes_solo_player() {
    let env = env();
    let total: i32 = env
        .eval(
            r#"
            local c = GetGroupMemberCounts()
            return c.TANK + c.HEALER + c.DAMAGER + c.NOROLE
            "#,
        )
        .unwrap();
    assert_eq!(total, 1);
}

// ============================================================================
// UnitGroupRolesAssigned
// ============================================================================

#[test]
fn test_unit_group_roles_assigned_returns_none() {
    let env = env();
    let role: String = env
        .eval(r#"return UnitGroupRolesAssigned("player")"#)
        .unwrap();
    assert_eq!(role, "NONE");
    env.exec(
        r#"
        for _, unit in ipairs({"player", "party1", "nonexistent"}) do
            assert(select('#', UnitGroupRolesAssigned(unit)) == 1)
            assert(UnitGroupRolesAssigned(unit) == "NONE")
            assert(select('#', UnitGroupRolesAssignedEnum(unit)) == 1)
            assert(UnitGroupRolesAssignedEnum(unit) == -1)
        end
        assert(UnitGroupRolesAssigned(nil) == "NONE")
        assert(UnitGroupRolesAssignedEnum(nil) == -1)
        assert(not pcall(UnitGroupRolesAssigned, {}))
        assert(not pcall(UnitGroupRolesAssignedEnum, {}))
        "#,
    )
    .unwrap();
}

// ============================================================================
// GetDungeonDifficultyID
// ============================================================================

#[test]
fn test_get_dungeon_difficulty_id_returns_normal() {
    let env = env();
    let diff: i32 = env.eval("return GetDungeonDifficultyID()").unwrap();
    assert_eq!(diff, 1);
}

// ============================================================================
// RequestGuildChallengeInfo
// ============================================================================

#[test]
fn test_request_guild_challenge_info_does_not_error() {
    let env = env();
    env.eval::<()>("RequestGuildChallengeInfo()").unwrap();
}

// ============================================================================
// StoreSecureReference
// ============================================================================

#[test]
fn test_store_secure_reference_does_not_error() {
    let env = env();
    env.eval::<()>(r#"StoreSecureReference("myref", CreateFrame("Frame"))"#)
        .unwrap();
}

// ============================================================================
// GetLFGInfoServer
// ============================================================================

#[test]
fn test_get_lfg_info_server_returns_not_queued() {
    let env = env();
    let (in_party, joined, queued): (bool, bool, bool) =
        env.eval(r#"return GetLFGInfoServer(1, 0)"#).unwrap();
    assert!(!in_party);
    assert!(!joined);
    assert!(!queued);
}

// ============================================================================
// GetAvailableLocaleInfo
// ============================================================================

#[test]
fn test_get_available_locale_info_returns_enus() {
    let env = env();
    let name: String = env
        .eval("return GetAvailableLocaleInfo()[1].localeName")
        .unwrap();
    assert_eq!(name, "enUS");
}

#[test]
fn test_get_available_locale_info_returns_locale_id() {
    let env = env();
    let id: i32 = env
        .eval("return GetAvailableLocaleInfo()[1].localeId")
        .unwrap();
    assert_eq!(id, 1);
}

// ============================================================================
// Startup Surface Parity
// ============================================================================

#[test]
fn test_c_addon_profiler_check_for_performance_message_is_callable() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_AddOnProfiler.CheckForPerformanceMessage() == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_get_repair_all_cost_returns_zero_and_false() {
    let env = env();
    let (cost, can_repair): (i32, bool) = env.eval("return GetRepairAllCost()").unwrap();
    assert_eq!(cost, 0);
    assert!(!can_repair);
}

#[test]
fn test_c_ping_get_default_ping_options_returns_empty_table() {
    let env = env();
    let count: i32 = env
        .eval("local options = C_Ping.GetDefaultPingOptions(); return #options")
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_c_lfg_list_get_available_categories_returns_seeded_categories() {
    let env = env();
    let count: i32 = env
        .eval("local categories = C_LFGList.GetAvailableCategories(); return #categories")
        .unwrap();
    assert_eq!(count, 5);
}

#[test]
fn test_ready_check_globals_report_no_active_check() {
    let env = env();
    let (status_is_nil, time_left): (bool, i32) = env
        .eval(
            r#"
            return GetReadyCheckStatus("player") == nil,
                   GetReadyCheckTimeLeft()
            "#,
        )
        .unwrap();
    assert!(status_is_nil);
    assert_eq!(time_left, 0);
}

#[test]
fn test_unit_has_incoming_resurrection_returns_false() {
    let env = env();
    let has_resurrection: bool = env
        .eval(r#"return UnitHasIncomingResurrection("player")"#)
        .unwrap();
    assert!(!has_resurrection);
}
