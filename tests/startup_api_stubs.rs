//! Smoke tests for startup-surface stubs added to unblock Blizzard addon
//! loading. Each stub returns values that reflect the simulator's reality
//! (no network, no in-game store, no premade finder, no photo sharing)
//! rather than invented placeholders.

#[path = "startup_api_stubs/common.rs"]
mod startup_api_common;

use startup_api_common::*;

type LfgWorldBnProbe = (
    String,
    bool,
    bool,
    bool,
    i32,
    f64,
    f64,
    f64,
    String,
    String,
    f64,
    f64,
    f64,
    f64,
    bool,
    i32,
    i32,
    i32,
    i32,
);

#[test]
fn clamp_and_saturate_exist_in_shared_bootstrap() {
    let env = env();
    let (clamped_low, clamped_high, saturated): (f64, f64, f64) = env
        .eval("return Clamp(-2, 0, 5), Clamp(8, 0, 5), Saturate(1.5)")
        .expect("Clamp and Saturate should be callable");
    assert_eq!(clamped_low, 0.0);
    assert_eq!(clamped_high, 5.0);
    assert_eq!(saturated, 1.0);
}

#[test]
fn fading_frame_helpers_seed_default_timers_and_can_copy_them() {
    let env = env();
    let (hidden, fade_in, hold, fade_out, copied_hold): (bool, f64, f64, f64, f64) = env
        .eval(
            r#"
            local src = CreateFrame("Frame", "CodexFadingFrameSource", UIParent)
            local dest = CreateFrame("Frame", "CodexFadingFrameDest", UIParent)
            FadingFrame_OnLoad(src)
            FadingFrame_SetFadeInTime(src, 0.25)
            FadingFrame_SetHoldTime(src, 1.5)
            FadingFrame_SetFadeOutTime(src, 0.75)
            FadingFrame_CopyTimes(src, dest)
            return not src:IsShown(), src.fadeInTime, src.holdTime, src.fadeOutTime, dest.holdTime
            "#,
        )
        .expect("fading-frame helpers should be callable");
    assert!(hidden);
    assert_eq!(fade_in, 0.25);
    assert_eq!(hold, 1.5);
    assert_eq!(fade_out, 0.75);
    assert_eq!(copied_hold, 1.5);
}

#[test]
fn get_net_stats_returns_four_zeros() {
    let env = env();
    let (bw_in, bw_out, latency_home, latency_world): (f64, f64, f64, f64) = env
        .eval("return GetNetStats()")
        .expect("GetNetStats should be callable");
    assert_eq!(bw_in, 0.0);
    assert_eq!(bw_out, 0.0);
    assert_eq!(latency_home, 0.0);
    assert_eq!(latency_world, 0.0);
}

#[test]
fn get_restricted_account_data_returns_safe_caps() {
    let env = env();
    let (level_cap, money_cap, profession_cap): (f64, f64, f64) = env
        .eval("return GetRestrictedAccountData()")
        .expect("GetRestrictedAccountData should be callable");
    assert_eq!(level_cap, 20.0);
    assert_eq!(money_cap, 0.0);
    assert_eq!(profession_cap, 0.0);
}

#[test]
fn store_frame_is_shown_returns_false() {
    let env = env();
    let shown: bool = env.eval("return StoreFrame_IsShown()").unwrap();
    assert!(!shown, "no Store UI is ever rendered in the sim");
}

#[test]
fn quest_poi_update_icons_is_callable() {
    let env = env();
    let ok: bool = env
        .eval(
            r#"
            local ok = pcall(function()
                QuestPOIUpdateIcons()
            end)
            return ok
            "#,
        )
        .expect("QuestPOIUpdateIcons smoke probe should run");
    assert!(
        ok,
        "QuestPOIUpdateIcons should be callable during QuestMap refresh"
    );
}

#[test]
fn corruption_helpers_exist_with_safe_defaults() {
    let env = env();
    let (corruption, resistance, effects_ty, effects_len): (f64, f64, String, i64) = env
        .eval(
            r#"
            local effects = GetNegativeCorruptionEffectInfo()
            return GetCorruption(),
                   GetCorruptionResistance(),
                   type(effects),
                   #effects
            "#,
        )
        .expect("corruption helpers should be callable");
    assert_eq!(corruption, 0.0);
    assert_eq!(resistance, 0.0);
    assert_eq!(effects_ty, "table");
    assert_eq!(effects_len, 0);
}

#[test]
fn quest_sort_helpers_are_callable_noops() {
    let env = env();
    let ok: bool = env
        .eval(
            r#"
            return pcall(function()
                SortQuestWatches()
                SortQuestSortTypes()
                SortQuests()
            end)
            "#,
        )
        .expect("quest sort helpers should be callable");
    assert!(
        ok,
        "quest sort helpers should stay callable during QuestMap refresh"
    );
}

#[test]
fn nameplate_size_helper_exists_as_safe_noop() {
    let env = env();
    let ok: bool = env
        .eval(
            r#"
            return pcall(function()
                C_NamePlate.SetNamePlateSize(110, 45)
            end)
            "#,
        )
        .expect("nameplate size helper should be callable");
    assert!(
        ok,
        "SetNamePlateSize should exist even though the sim renders no 3D nameplates"
    );
}

#[test]
fn instance_abandon_vote_and_shutdown_helpers_return_numeric_pairs() {
    let env = env();
    let (
        vote_duration,
        vote_time_left,
        shutdown_duration,
        shutdown_time_left,
        response_is_nil,
        response_set_ok,
        vote_count,
        can_start_vote,
        start_vote_ok,
    ): (f64, f64, f64, f64, bool, bool, f64, bool, bool) = env
        .eval(
            r#"
            local voteDuration, voteTimeLeft = C_PartyInfo.GetInstanceAbandonVoteTime()
            local shutdownDuration, shutdownTimeLeft = C_PartyInfo.GetInstanceAbandonShutdownTime()
            local response = C_PartyInfo.GetInstanceAbandonVoteResponse()
            local responseSetOk = pcall(function()
                C_PartyInfo.SetInstanceAbandonVoteResponse(true)
            end)
            local startVoteOk = pcall(function()
                C_PartyInfo.StartInstanceAbandonVote()
            end)

            return voteDuration,
                   voteTimeLeft,
                   shutdownDuration,
                   shutdownTimeLeft,
                   response == nil,
                   responseSetOk,
                   C_PartyInfo.GetNumInstanceAbandonGroupVoteResponses(),
                   C_PartyInfo.CanStartInstanceAbandonVote(),
                   startVoteOk
            "#,
        )
        .expect("instance abandon helpers should be callable");

    assert_eq!(vote_duration, 0.0);
    assert_eq!(vote_time_left, 0.0);
    assert_eq!(shutdown_duration, 0.0);
    assert_eq!(shutdown_time_left, 0.0);
    assert!(response_is_nil);
    assert!(response_set_ok);
    assert_eq!(vote_count, 0.0);
    assert!(!can_start_vote);
    assert!(start_vote_ok);
}

#[test]
fn profiling_helpers_return_elapsed_milliseconds() {
    let env = env();
    let (start_type, first, second): (String, f64, f64) = env
        .eval(
            r#"
            local startResult = debugprofilestart()
            local first = debugprofilestop()
            local second = debugprofilestop()
            return type(startResult), first, second
            "#,
        )
        .expect("debug profiling helpers should be callable");

    assert_eq!(start_type, "nil");
    assert!(first >= 0.0);
    assert!(second >= first);
}

#[test]
fn startup_party_chat_and_targeting_globals_are_callable() {
    let env = env();
    let (
        clear_focus_ok,
        focus_cleared,
        request_ok,
        remove_channel_ok,
        can_mark_player,
        raid_target_index_is_nil,
    ): (bool, bool, bool, bool, bool, bool) = env
        .eval(
            r#"
            A_Admin.SetFocus("Training Dummy", 72, 1)

            local clearFocusOk = pcall(function()
                ClearFocus()
            end)
            local requestOk = pcall(function()
                RequestGuildPartyState()
            end)
            local removeChannelOk = pcall(function()
                RemoveChatWindowChannel(1, "General")
            end)

            return clearFocusOk,
                   not UnitExists("focus"),
                   requestOk,
                   removeChannelOk,
                   CanBeRaidTarget("player"),
                   GetRaidTargetIndex("player") == nil
            "#,
        )
        .expect("party/chat/targeting startup globals should be callable");

    assert!(clear_focus_ok, "ClearFocus should exist as a normal global");
    assert!(
        focus_cleared,
        "ClearFocus should clear the simulated focus unit"
    );
    assert!(request_ok, "RequestGuildPartyState should be a safe no-op");
    assert!(
        remove_channel_ok,
        "RemoveChatWindowChannel should be callable during chat bootstrap"
    );
    assert!(
        can_mark_player,
        "existing units should remain markable by CanBeRaidTarget"
    );
    assert!(
        raid_target_index_is_nil,
        "GetRaidTargetIndex should be nil when the sim has not assigned a marker"
    );
}

#[test]
fn set_raid_target_globals_are_callable_no_ops() {
    let env = env();
    let (set_target_ok, set_icon_ok, clear_ok, invalid_unit_ok): (bool, bool, bool, bool) = env
        .eval(
            r#"
            local setTargetOk = pcall(function() SetRaidTarget("player", 5) end)
            local setIconOk = pcall(function() SetRaidTargetIcon("player", 8) end)
            local clearOk = pcall(function() SetRaidTarget("player", 0) end)
            local invalidOk = pcall(function() SetRaidTarget("notaunit", 1) end)
            return setTargetOk, setIconOk, clearOk, invalidOk
            "#,
        )
        .expect("SetRaidTarget/SetRaidTargetIcon should be registered globals");

    assert!(set_target_ok, "SetRaidTarget should accept (unit, marker)");
    assert!(
        set_icon_ok,
        "SetRaidTargetIcon should be the legacy alias of SetRaidTarget"
    );
    assert!(
        clear_ok,
        "SetRaidTarget(unit, 0) should clear without error"
    );
    assert!(
        invalid_unit_ok,
        "SetRaidTarget should silently no-op on unknown unit tokens"
    );
}

#[test]
fn startup_lfg_world_timer_and_bn_surfaces_return_empty_safe_shapes() {
    let env = env();
    let (
        queued_type,
        queued_empty,
        ready_check_in_progress,
        ready_check_is_battleground,
        elapsed_timer_count,
        elapsed_timer_id,
        elapsed_time,
        elapsed_type,
        world_pvp_status,
        world_pvp_map,
        world_pvp_queue_id,
        world_pvp_expire,
        world_pvp_average,
        world_pvp_queued,
        world_pvp_suspended,
        bn_total,
        bn_online,
        bn_favorite,
        bn_favorite_online,
    ): LfgWorldBnProbe = env
        .eval(
            r##"
            local queued = GetLFGQueuedList(1)
            local readyCheckInProgress, readyCheckIsBattleground = GetLFGReadyCheckUpdate()
            local timerID, elapsedTime, timerType = GetWorldElapsedTime(7)
            local worldPvpStatus, worldPvpMap, worldPvpQueueID, worldPvpExpire, worldPvpAverage, worldPvpQueued, worldPvpSuspended =
                GetWorldPVPQueueStatus(1)

            return type(queued),
                   next(queued) == nil,
                   readyCheckInProgress,
                   readyCheckIsBattleground,
                   select("#", GetWorldElapsedTimers()),
                   timerID,
                   elapsedTime,
                   timerType,
                   worldPvpStatus,
                   worldPvpMap,
                   worldPvpQueueID,
                   worldPvpExpire,
                   worldPvpAverage,
                   worldPvpQueued,
                   worldPvpSuspended,
                   BNGetNumFriends()
            "##,
        )
        .expect("startup LFG/world timer/BN stubs should be callable");

    assert_eq!(queued_type, "table");
    assert!(queued_empty, "no LFG queues should be seeded by default");
    assert!(!ready_check_in_progress);
    assert!(!ready_check_is_battleground);
    assert_eq!(
        elapsed_timer_count, 0,
        "no world elapsed timers should be active"
    );
    assert_eq!(elapsed_timer_id, 7.0);
    assert_eq!(elapsed_time, 0.0);
    assert_eq!(elapsed_type, 0.0);
    assert_eq!(world_pvp_status, "none");
    assert_eq!(world_pvp_map, "");
    assert_eq!(world_pvp_queue_id, 0.0);
    assert_eq!(world_pvp_expire, 0.0);
    assert_eq!(world_pvp_average, 0.0);
    assert_eq!(world_pvp_queued, 0.0);
    assert!(!world_pvp_suspended);
    assert_eq!(bn_total, 0);
    assert_eq!(bn_online, 0);
    assert_eq!(bn_favorite, 0);
    assert_eq!(bn_favorite_online, 0);
}

#[test]
fn lfd_lock_refresh_requests_are_callable_noops() {
    let env = env();
    let (player_lock_request_ok, party_lock_request_ok): (bool, bool) = env
        .eval(
            r#"
            return pcall(RequestLFDPlayerLockInfo),
                   pcall(RequestLFDPartyLockInfo)
            "#,
        )
        .expect("LFD lock refresh request probes should run");

    assert!(
        player_lock_request_ok,
        "player LFD lock refresh request should be a callable no-op"
    );
    assert!(
        party_lock_request_ok,
        "party LFD lock refresh request should be a callable no-op"
    );
}

#[test]
fn startup_resurrection_probe_defaults_false_and_is_callable() {
    let env = env();
    let (call_ok, can_resurrect): (bool, bool) = env
        .eval(
            r#"
            local ok, value = pcall(function()
                return CanHearthAndResurrectFromArea()
            end)
            return ok, value or false
            "#,
        )
        .expect("resurrection probe smoke test should run");

    assert!(
        call_ok,
        "CanHearthAndResurrectFromArea should exist during QueueStatus startup"
    );
    assert!(
        !can_resurrect,
        "the sim should default area resurrection to disabled"
    );
}

#[test]
fn startup_quest_link_and_date_helpers_are_callable() {
    let env = env();
    let (quest_link_type, quest_link, reset_time, calendar_type, calendar_year_type): (
        String,
        String,
        f64,
        String,
        String,
    ) = env
        .eval(
            r#"
            local link = GetQuestLink(80000)
            local calendar = date("*t", 0)
            return type(link),
                   link or "",
                   GetQuestResetTime(),
                   type(calendar),
                   type(calendar and calendar.year)
            "#,
        )
        .expect("quest link and date helpers should be callable");

    assert_eq!(quest_link_type, "string");
    assert!(
        quest_link.contains("|Hquest:80000|h[The Lost Expedition]|h"),
        "GetQuestLink should return a colored quest hyperlink"
    );
    assert_eq!(reset_time, 86400.0);
    assert_eq!(calendar_type, "table");
    assert_eq!(calendar_year_type, "number");
}

#[cfg(not(feature = "retail-12-0-7"))]
#[test]
fn startup_legacy_auto_complete_realms_global_matches_namespace() {
    let env = env();
    let (global_type, namespace_type, same_table, realm_count): (String, String, bool, i64) = env
        .eval(
            r#"
            local globalRealms = GetAutoCompleteRealms()
            local namespaceRealms = C_AutoComplete.GetAutoCompleteRealms()
            return type(GetAutoCompleteRealms),
                   type(C_AutoComplete.GetAutoCompleteRealms),
                   type(globalRealms) == "table" and type(namespaceRealms) == "table",
                   #globalRealms
            "#,
        )
        .expect("legacy autocomplete realm global should be callable");

    assert_eq!(global_type, "function");
    assert_eq!(namespace_type, "function");
    assert!(
        same_table,
        "global and namespaced autocomplete realm probes should both return tables"
    );
    assert_eq!(realm_count, 0);
}

#[test]
fn startup_clip_cursor_and_other_pet_probes_default_false() {
    let env = env();
    let (clip_cursor_ok, clip_cursor, other_pet_ok, other_pet): (bool, bool, bool, bool) = env
        .eval(
            r#"
            local clip_ok, clip_value = pcall(function()
                return SupportsClipCursor()
            end)
            local pet_ok, pet_value = pcall(function()
                return UnitIsOtherPlayersPet("target")
            end)
            return clip_ok, clip_value or false, pet_ok, pet_value or false
            "#,
        )
        .expect("clip-cursor and pet probe smoke tests should run");

    assert!(clip_cursor_ok, "SupportsClipCursor should be callable");
    assert!(
        !clip_cursor,
        "the sim should default clip-cursor support to disabled"
    );
    assert!(other_pet_ok, "UnitIsOtherPlayersPet should be callable");
    assert!(
        !other_pet,
        "the sim should default other-player pet detection to false"
    );
}

#[test]
fn startup_world_map_provider_list_apis_are_iterable() {
    let env = env();
    let ok: bool = env
        .eval(
            r#"
            return pcall(function()
                for _, value in ipairs(C_QuestLine.GetAvailableQuestLines(947)) do end
                for _, value in ipairs(C_QuestLine.GetForceVisibleQuests(947)) do end
                for _, value in ipairs(C_AreaPoiInfo.GetQuestHubsForMap(947)) do end
                for _, value in ipairs(C_AreaPoiInfo.GetEventsForMap(947)) do end
                for _, value in ipairs(C_AreaPoiInfo.GetDelvesForMap(947)) do end
                for _, value in ipairs(C_ResearchInfo.GetDigSitesForMap(947)) do end
                for _, value in ipairs(C_Garrison.GetGarrisonPlotsInstancesForMap(947)) do end
                for _, value in ipairs(C_EncounterJournal.GetDungeonEntrancesForMap(947)) do end
                for _, value in ipairs(C_EncounterJournal.GetEncountersOnMap(947)) do end
                for _, value in ipairs(C_Map.GetMapBannersForMap(947)) do end
                for _, value in ipairs(C_Map.GetMapLinksForMap(947)) do end
                for _, value in ipairs(C_DeathInfo.GetGraveyardsForMap(947)) do end
                for _, value in ipairs(C_ContentTracking.GetCollectableSourceTypes()) do end
                local _, trackables = C_ContentTracking.GetTrackablesOnMap(1, 947)
                for _, value in ipairs(trackables) do end
                for _, focus in ipairs(GetMouseFoci()) do end
            end)
            "#,
        )
        .expect("world-map provider iterable smoke test should run");

    assert!(
        ok,
        "world-map provider list APIs should return empty tables rather than nil"
    );
}

#[test]
fn startup_world_map_auxiliary_globals_are_callable() {
    let env = env();
    let (mouse_foci_type, mouse_foci_len, flag_positions, pvp_inactive, spell_id_type): (
        String,
        i64,
        f64,
        bool,
        String,
    ) = env
        .eval(
            r#"
            local foci = GetMouseFoci()
            local spellID = GetWorldMapActionButtonSpellInfo()
            return type(foci), #foci, GetNumBattlefieldFlagPositions(), PlayerIsPVPInactive("player"), type(spellID)
            "#,
        )
        .expect("world-map auxiliary globals should be callable");

    assert_eq!(mouse_foci_type, "table");
    assert_eq!(mouse_foci_len, 0);
    assert_eq!(flag_positions, 0.0);
    assert!(!pvp_inactive);
    assert_eq!(spell_id_type, "nil");
}

#[test]
fn startup_client_time_camera_and_channel_globals_are_callable() {
    let env = env();
    let (
        is_windows_type,
        is_windows,
        request_time_ok,
        camera_verbs_ok,
        server_channel_count,
        pvp_lifetime_hks,
        pvp_lifetime_rank,
        modified_click_default,
        modified_click_updated,
    ): (String, bool, bool, bool, i64, f64, f64, String, String) = env
        .eval(
            r#"
            local channels = { EnumerateServerChannels() }
            local lifetimeHKs, lifetimeRank = GetPVPLifetimeStats()
            local before = GetModifiedClick("SELFCAST")
            SetModifiedClick("SELFCAST", "NONE")
            local cameraVerbsOK = true
            for _, name in ipairs({
                "MoveViewOutStart", "MoveViewOutStop",
                "MoveViewInStart", "MoveViewInStop",
                "MoveViewLeftStart", "MoveViewLeftStop",
                "MoveViewRightStart", "MoveViewRightStop",
                "MoveViewUpStart", "MoveViewUpStop",
                "MoveViewDownStart", "MoveViewDownStop",
            }) do
                cameraVerbsOK = cameraVerbsOK and pcall(_G[name], 1.0, 0, true)
            end
            return type(IsWindowsClient()),
                   IsWindowsClient(),
                   pcall(RequestTimePlayed),
                   cameraVerbsOK,
                   #channels,
                   lifetimeHKs,
                   lifetimeRank,
                   before,
                   GetModifiedClick("SELFCAST")
            "#,
        )
        .expect("client/time/camera/channel globals should be callable");

    assert_eq!(is_windows_type, "boolean");
    assert!(
        !is_windows,
        "the Linux-hosted simulator should not report a Windows client"
    );
    assert!(request_time_ok, "RequestTimePlayed should be a callable no-op");
    assert!(camera_verbs_ok, "camera movement verbs should be callable no-ops");
    assert_eq!(
        server_channel_count, 0,
        "sim should not seed any server-managed chat channels"
    );
    assert_eq!(pvp_lifetime_hks, 0.0);
    assert_eq!(pvp_lifetime_rank, 0.0);
    assert_eq!(modified_click_default, "ALT");
    assert_eq!(modified_click_updated, "NONE");
}

#[test]
fn startup_pvp_match_state_defaults_to_inactive() {
    let env = env();
    let state: i32 = env
        .eval("return C_PvP.GetActiveMatchState()")
        .expect("C_PvP.GetActiveMatchState should be callable");
    assert_eq!(
        state, 0,
        "startup should default to inactive PvP match state"
    );
}

#[test]
fn is_character_newly_boosted_returns_false() {
    let env = env();
    let boosted: bool = env
        .eval("return IsCharacterNewlyBoosted()")
        .expect("IsCharacterNewlyBoosted should be callable");
    assert!(
        !boosted,
        "boosted-character help flow is not simulated, so the probe should stay false"
    );
}

#[cfg(feature = "retail-12-0-7")]
#[test]
fn autocomplete_namespace_remains_live_without_retired_native_globals() {
    let env = env();
    env.exec(r#"
        assert(rawget(_G, "GetAutoCompleteRealms") == nil)
        assert(rawget(_G, "GetAutoCompleteResults") == nil)
        assert(type(C_AutoComplete.GetAutoCompleteRealms) == "function")
        assert(#C_AutoComplete.GetAutoCompleteRealms() == 0)
    "#).unwrap();
}
