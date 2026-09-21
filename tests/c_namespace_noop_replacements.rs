use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[cfg(feature = "retail-12-0-0")]
mod combatlog_settings_tests {
    use super::{WowLuaEnv, env};

    fn read_settings(env: &WowLuaEnv) -> (bool, i64) {
        env.eval(
            "return C_CombatLog.AreFilteredEventsEnabled(), C_CombatLog.GetEntryRetentionTime()",
        )
        .unwrap()
    }

    #[test]
    fn combatlog_settings_filtered_events_roundtrip() {
        let env = env();
        env.exec(
            r#"
            for index, enabled in ipairs({true, false, false, true, true, false}) do
                C_CombatLog.SetFilteredEventsEnabled(enabled)
                assert(C_CombatLog.AreFilteredEventsEnabled() == enabled,
                    "filtered-events setting differs after write " .. index)
            end
            "#,
        )
        .unwrap();
    }

    #[test]
    fn combatlog_settings_retention_roundtrip() {
        let env = env();
        env.exec(
            r#"
            for index, retention in ipairs({120, 240, 240, 120, 120, 240}) do
                C_CombatLog.SetEntryRetentionTime(retention)
                assert(C_CombatLog.GetEntryRetentionTime() == retention,
                    "retention setting differs after write " .. index)
            end
            "#,
        )
        .unwrap();
    }

    #[test]
    fn combatlog_settings_return_arity() {
        let env = env();
        env.exec(
            r#"
            for index, retention in ipairs({120, 240}) do
                local enabled = index == 1
                assert(select('#', C_CombatLog.SetFilteredEventsEnabled(enabled)) == 0,
                    "filtered-events setter must return zero values")
                assert(select('#', C_CombatLog.SetEntryRetentionTime(retention)) == 0,
                    "retention setter must return zero values")
                assert(select('#', C_CombatLog.AreFilteredEventsEnabled()) == 1,
                    "filtered-events getter must return one value")
                assert(type(C_CombatLog.AreFilteredEventsEnabled()) == "boolean",
                    "filtered-events getter must return a boolean")
                assert(select('#', C_CombatLog.GetEntryRetentionTime()) == 1,
                    "retention getter must return one value")
                assert(type(C_CombatLog.GetEntryRetentionTime()) == "number",
                    "retention getter must return a number")
            end
            "#,
        )
        .unwrap();
    }

    #[test]
    fn combatlog_settings_are_independent() {
        let env = env();
        env.exec(
            r#"
            C_CombatLog.SetFilteredEventsEnabled(false)
            C_CombatLog.SetEntryRetentionTime(120)
            C_CombatLog.SetFilteredEventsEnabled(true)
            assert(C_CombatLog.GetEntryRetentionTime() == 120,
                "enabling filtered events changed retention")
            C_CombatLog.SetEntryRetentionTime(240)
            assert(C_CombatLog.AreFilteredEventsEnabled() == true,
                "changing retention disabled filtered events")
            C_CombatLog.SetFilteredEventsEnabled(false)
            assert(C_CombatLog.GetEntryRetentionTime() == 240,
                "disabling filtered events changed retention")
            C_CombatLog.SetEntryRetentionTime(120)
            assert(C_CombatLog.AreFilteredEventsEnabled() == false,
                "changing retention enabled filtered events")
            "#,
        )
        .unwrap();
    }

    #[test]
    fn combatlog_settings_isolate_lua_environments() {
        let first = env();
        first
            .exec(
                "C_CombatLog.SetFilteredEventsEnabled(true); C_CombatLog.SetEntryRetentionTime(120)",
            )
            .unwrap();
        let second = env();
        second
            .exec(
                "C_CombatLog.SetFilteredEventsEnabled(false); C_CombatLog.SetEntryRetentionTime(240)",
            )
            .unwrap();

        assert_eq!(read_settings(&first), (true, 120));
        assert_eq!(read_settings(&second), (false, 240));

        first
            .exec(
                "C_CombatLog.SetFilteredEventsEnabled(false); C_CombatLog.SetEntryRetentionTime(240)",
            )
            .unwrap();
        assert_eq!(read_settings(&first), (false, 240));
        assert_eq!(read_settings(&second), (false, 240));

        first
            .exec(
                "C_CombatLog.SetFilteredEventsEnabled(true); C_CombatLog.SetEntryRetentionTime(180)",
            )
            .unwrap();
        assert_eq!(read_settings(&first), (true, 180));
        assert_eq!(read_settings(&second), (false, 240));

        second
            .exec(
                "C_CombatLog.SetFilteredEventsEnabled(true); C_CombatLog.SetEntryRetentionTime(120)",
            )
            .unwrap();
        assert_eq!(read_settings(&first), (true, 180));
        assert_eq!(read_settings(&second), (true, 120));

        second
            .exec(
                "C_CombatLog.SetFilteredEventsEnabled(false); C_CombatLog.SetEntryRetentionTime(60)",
            )
            .unwrap();
        assert_eq!(read_settings(&first), (true, 180));
        assert_eq!(read_settings(&second), (false, 60));
    }
}

#[test]
fn perks_activities_remove_tracked_updates_state() {
    let env = env();
    let (removed, remaining_one, remaining_two, remaining_count, remove_count, last_removed): (
        bool,
        i64,
        i64,
        i64,
        i64,
        i64,
    ) = env
        .eval(
            r#"
            C_PerksActivities._state.trackedIDs = { 101, 202, 303 }
            C_PerksActivities._state.removeCount = 0
            C_PerksActivities._state.lastRemovedID = nil
            C_PerksActivities._state.activityInfoByID = {
                [303] = { id = 303, name = "Trading Post Task" },
            }
            C_PerksActivities._state.chatLinkByID = {
                [303] = "|cffffff00|Hperk:303|h[Trading Post Task]|h|r",
            }

            local removed = C_PerksActivities.RemoveTrackedPerksActivity("202")
            local tracked = C_PerksActivities.GetTrackedPerksActivities().trackedIDs
            local info = C_PerksActivities.GetPerksActivityInfo(303)
            local link = C_PerksActivities.GetPerksActivityChatLink(303)

            assert(info and info.id == 303 and info.name == "Trading Post Task")
            assert(link ~= nil and string.find(link, "perk:303", 1, true) ~= nil)

            return removed == true,
                tracked[1],
                tracked[2],
                #tracked,
                C_PerksActivities._state.removeCount,
                C_PerksActivities._state.lastRemovedID
            "#,
        )
        .unwrap();

    assert!(
        removed,
        "RemoveTrackedPerksActivity should report successful removal"
    );
    assert_eq!(remaining_one, 101, "first tracked id should remain");
    assert_eq!(remaining_two, 303, "second tracked id should remain");
    assert_eq!(remaining_count, 2, "one tracked id should be removed");
    assert_eq!(remove_count, 1, "remove count should increment");
    assert_eq!(last_removed, 202, "lastRemovedID should store removed id");
}

#[test]
fn store_glue_methods_are_state_backed() {
    let env = env();
    let (
        disconnect_on_logout,
        vas_ready,
        purchase_state,
        product_id,
        result_tag,
        request_count,
        update_count,
        queued_count,
        last_queued_guid,
    ): (bool, bool, i64, i64, String, i64, i64, i64, String) = env
        .eval(
            r#"
            C_StoreGlue._state.disconnectOnLogout = true
            C_StoreGlue._state.vasProductReady = true
            C_StoreGlue._state.purchaseStateByGuid = {
                ["Player-111"] = {
                    purchaseState = 3,
                    productID = 77,
                    result = "READY",
                }
            }
            C_StoreGlue._state.requestedQueueGuids = {}
            C_StoreGlue._state.requestCharacterQueueTimeCount = 0
            C_StoreGlue._state.updateVASPurchaseStatesCount = 0
            C_StoreGlue._state.lastRequestedQueueGuid = nil

            local purchaseState, productID, resultTag = C_StoreGlue.GetVASPurchaseStateInfo("Player-111")
            local firstQueueRequest = C_StoreGlue.RequestCharacterQueueTime("Player-111")
            local secondQueueRequest = C_StoreGlue.RequestCharacterQueueTime("Player-111")
            local firstUpdate = C_StoreGlue.UpdateVASPurchaseStates()
            local secondUpdate = C_StoreGlue.UpdateVASPurchaseStates()

            assert(firstQueueRequest and secondQueueRequest)
            assert(firstUpdate and secondUpdate)

            return C_StoreGlue.GetDisconnectOnLogout(),
                C_StoreGlue.GetVASProductReady(),
                purchaseState,
                productID,
                resultTag,
                C_StoreGlue._state.requestCharacterQueueTimeCount,
                C_StoreGlue._state.updateVASPurchaseStatesCount,
                #C_StoreGlue._state.requestedQueueGuids,
                C_StoreGlue._state.lastRequestedQueueGuid
            "#,
        )
        .unwrap();

    assert!(
        disconnect_on_logout,
        "disconnectOnLogout should read from _state"
    );
    assert!(vas_ready, "vasProductReady should read from _state");
    assert_eq!(
        purchase_state, 3,
        "purchase state should read from map entry"
    );
    assert_eq!(product_id, 77, "product id should read from map entry");
    assert_eq!(result_tag, "READY", "result should read from map entry");
    assert_eq!(request_count, 2, "queue request count should increment");
    assert_eq!(update_count, 2, "update count should increment");
    assert_eq!(
        queued_count, 2,
        "requested guid list should record requests"
    );
    assert_eq!(
        last_queued_guid, "Player-111",
        "last requested guid should be tracked"
    );
}

#[test]
fn video_options_set_window_size_updates_current_size() {
    let env = env();
    let (
        default_x,
        default_y,
        current_x,
        current_y,
        size_count,
        set_count,
        last_set_x,
        last_set_y,
    ): (i64, i64, i64, i64, i64, i64, i64, i64) = env
        .eval(
            r#"
            C_VideoOptions._state.defaultGameWindowSize = { x = 2560, y = 1440 }
            C_VideoOptions._state.currentGameWindowSize = { x = 1920, y = 1080 }
            C_VideoOptions._state.availableGameWindowSizes = {
                { x = 1280, y = 720 },
                { x = 1920, y = 1080 },
                { x = 2560, y = 1440 },
            }
            C_VideoOptions._state.setGameWindowSizeCount = 0
            C_VideoOptions._state.lastSetWindowSize = nil

            local defaultSize = C_VideoOptions.GetDefaultGameWindowSize(1)
            local sizes = C_VideoOptions.GetGameWindowSizes()
            local currentBefore = C_VideoOptions.GetCurrentGameWindowSize()
            assert(currentBefore.x == 1920 and currentBefore.y == 1080)
            assert(#sizes == 3)

            local changed = C_VideoOptions.SetGameWindowSize(1600, 900)
            assert(changed == true)

            local currentAfter = C_VideoOptions.GetCurrentGameWindowSize()
            local lastSet = C_VideoOptions._state.lastSetWindowSize

            return defaultSize.x,
                defaultSize.y,
                currentAfter.x,
                currentAfter.y,
                #sizes,
                C_VideoOptions._state.setGameWindowSizeCount,
                lastSet.x,
                lastSet.y
            "#,
        )
        .unwrap();

    assert_eq!(default_x, 2560, "default width should read from _state");
    assert_eq!(default_y, 1440, "default height should read from _state");
    assert_eq!(
        current_x, 1600,
        "current width should update after SetGameWindowSize"
    );
    assert_eq!(
        current_y, 900,
        "current height should update after SetGameWindowSize"
    );
    assert_eq!(
        size_count, 3,
        "GetGameWindowSizes should return configured sizes"
    );
    assert_eq!(set_count, 1, "setGameWindowSizeCount should increment");
    assert_eq!(last_set_x, 1600, "last set width should be tracked");
    assert_eq!(last_set_y, 900, "last set height should be tracked");
}

#[test]
fn video_options_official_api_surface_returns_stable_shapes() {
    let env = env();
    let (
        current_x,
        current_y,
        default_x,
        default_y,
        size_count,
        first_size_x,
        first_size_y,
        adapter_count,
        first_adapter_name,
        first_adapter_low_power,
        second_adapter_external,
        spell_density_supported,
    ): (
        i64,
        i64,
        i64,
        i64,
        i64,
        i64,
        i64,
        i64,
        String,
        bool,
        bool,
        bool,
    ) = env
        .eval(
            r#"
            C_VideoOptions._state.defaultGameWindowSize = { x = 2560, y = 1440 }
            C_VideoOptions._state.currentGameWindowSize = { x = 1920, y = 1080 }
            C_VideoOptions._state.availableGameWindowSizes = {
                { x = 1280, y = 720 },
                { x = 1920, y = 1080 },
            }
            C_VideoOptions._state.gxAdapterInfo = {
                { name = "Integrated GPU", isLowPower = true, isExternal = false },
                { name = "Dock GPU", isLowPower = false, isExternal = true },
            }

            local current = C_VideoOptions.GetCurrentGameWindowSize(0, true)
            local defaultSize = C_VideoOptions.GetDefaultGameWindowSize(1)
            local sizes = C_VideoOptions.GetGameWindowSizes(1, false)
            local adapters = C_VideoOptions.GetGxAdapterInfo()
            local spellDensitySupported = C_VideoOptions.IsSpellVisualDensitySystemSupported()

            assert(type(current) == "table" and current.x == 1920 and current.y == 1080)
            assert(type(defaultSize) == "table" and defaultSize.x == 2560 and defaultSize.y == 1440)
            assert(type(sizes) == "table" and #sizes == 2)
            assert(type(adapters) == "table" and #adapters == 2)
            assert(type(spellDensitySupported) == "boolean")

            return current.x,
                current.y,
                defaultSize.x,
                defaultSize.y,
                #sizes,
                sizes[1].x,
                sizes[1].y,
                #adapters,
                adapters[1].name,
                adapters[1].isLowPower,
                adapters[2].isExternal,
                spellDensitySupported
            "#,
        )
        .unwrap();

    assert_eq!(current_x, 1920, "current width should come from _state");
    assert_eq!(current_y, 1080, "current height should come from _state");
    assert_eq!(default_x, 2560, "default width should come from _state");
    assert_eq!(default_y, 1440, "default height should come from _state");
    assert_eq!(
        size_count, 2,
        "window sizes should preserve configured entries"
    );
    assert_eq!(
        first_size_x, 1280,
        "first window size width should match _state"
    );
    assert_eq!(
        first_size_y, 720,
        "first window size height should match _state"
    );
    assert_eq!(
        adapter_count, 2,
        "adapter list should preserve configured entries"
    );
    assert_eq!(
        first_adapter_name, "Integrated GPU",
        "adapter info should preserve names"
    );
    assert!(
        first_adapter_low_power,
        "adapter info should preserve low-power flags"
    );
    assert!(
        second_adapter_external,
        "adapter info should preserve external flags"
    );
    assert!(
        !spell_density_supported,
        "spell density support should remain disabled in the simulator"
    );
}

#[test]
fn perks_activities_monthly_accessors_return_stable_shapes() {
    let env = env();
    let (
        tag_type,
        tag_count,
        activities_type,
        activities_count,
        thresholds_type,
        thresholds_count,
        active_month,
        seconds_remaining,
        pending_type,
        pending_count,
    ): (String, i64, String, i64, String, i64, i64, i64, String, i64) = env
        .eval(
            r#"
            C_PerksActivities._state.activitiesInfo = nil
            C_PerksActivities._state.allTags = nil
            C_PerksActivities._state.pendingCompletion = nil

            local tags = C_PerksActivities.GetAllPerksActivityTags()
            local info = C_PerksActivities.GetPerksActivitiesInfo()
            local pending = C_PerksActivities.GetPerksActivitiesPendingCompletion()

            return type(tags.tagName),
                #tags.tagName,
                type(info.activities),
                #info.activities,
                type(info.thresholds),
                #info.thresholds,
                info.activePerksMonth,
                info.secondsRemaining,
                type(pending.pendingIDs),
                #pending.pendingIDs
            "#,
        )
        .unwrap();

    assert_eq!(tag_type, "table", "tagName should be a table");
    assert_eq!(tag_count, 0, "default tag list should be empty");
    assert_eq!(
        activities_type, "table",
        "activities should be a table for pairs/ipairs safety"
    );
    assert_eq!(
        activities_count, 0,
        "default activities list should be empty"
    );
    assert_eq!(
        thresholds_type, "table",
        "thresholds should be a table for pairs/ipairs safety"
    );
    assert_eq!(
        thresholds_count, 0,
        "default threshold list should be empty"
    );
    assert!(
        active_month >= 1,
        "activePerksMonth should default to a positive integer"
    );
    assert!(
        seconds_remaining >= 0,
        "secondsRemaining should default to a non-negative integer"
    );
    assert_eq!(
        pending_type, "table",
        "pendingIDs should be a table for ipairs safety"
    );
    assert_eq!(pending_count, 0, "pendingIDs should default to empty");
}

#[test]
fn perks_program_pending_chest_rewards_return_empty_table() {
    let env = env();
    let (request_ok, rewards_type, rewards_count): (bool, String, i64) = env
        .eval(
            r#"
            local requestOK = pcall(C_PerksProgram.RequestPendingChestRewards)
            local rewards = C_PerksProgram.GetPendingChestRewards()
            return requestOK, type(rewards), #rewards
            "#,
        )
        .unwrap();

    assert!(
        request_ok,
        "RequestPendingChestRewards should be callable as a no-op"
    );
    assert_eq!(
        rewards_type, "table",
        "pending chest rewards should be a table for TableIsEmpty/pairs safety"
    );
    assert_eq!(
        rewards_count, 0,
        "pending chest rewards should default empty"
    );
}

#[test]
fn encounter_journal_global_filters_and_tier_are_numeric() {
    let env = env();
    let (
        tier_type,
        class_before,
        spec_before,
        numeric_class,
        numeric_spec,
        string_class,
        nil_spec,
        invalid_class,
        non_integral_spec,
        tier_after,
        has_valid_difficulty,
    ): (String, i64, i64, i64, i64, i64, i64, i64, i64, i64, bool) = env
        .eval(
            r#"
            local tierBefore = EJ_GetCurrentTier()
            local classBefore, specBefore = EJ_GetLootFilter()

            EJ_SetLootFilter(4, 7)
            local numericClass, numericSpec = EJ_GetLootFilter()

            EJ_SetLootFilter("3", nil)
            local stringClass, nilSpec = EJ_GetLootFilter()

            EJ_SetLootFilter("invalid", 2.5)
            local invalidClass, nonIntegralSpec = EJ_GetLootFilter()

            EJ_SelectTier(11)
            local tierAfter = EJ_GetCurrentTier()

            return type(tierBefore),
                classBefore,
                specBefore,
                numericClass,
                numericSpec,
                stringClass,
                nilSpec,
                invalidClass,
                nonIntegralSpec,
                tierAfter,
                EJ_IsValidInstanceDifficulty(14)
            "#,
        )
        .unwrap();

    assert_eq!(tier_type, "number", "EJ_GetCurrentTier should be numeric");
    assert!(class_before >= 0, "default class filter should be numeric");
    assert!(spec_before >= 0, "default spec filter should be numeric");
    assert_eq!(
        numeric_class, 4,
        "numeric class filters should be preserved"
    );
    assert_eq!(numeric_spec, 7, "numeric spec filters should be preserved");
    assert_eq!(
        string_class, 3,
        "class filter should normalize numeric strings"
    );
    assert_eq!(nil_spec, 0, "spec filter should normalize nil input to 0");
    assert_eq!(
        invalid_class, 0,
        "class filter should normalize invalid strings to 0"
    );
    assert_eq!(
        non_integral_spec, 0,
        "spec filter should normalize non-integral numbers to 0"
    );
    assert_eq!(tier_after, 11, "EJ_SelectTier should update current tier");
    assert!(
        has_valid_difficulty,
        "numeric difficulties should be treated as valid"
    );
}

#[test]
fn combat_log_globals_have_stable_stub_behavior() {
    let env = env();
    let (
        reset_ok,
        add_filter_ok,
        set_entry_ok,
        clear_ok,
        set_retention_ok,
        current_entry_ok,
        num_entries_is_number,
        show_current_is_boolean,
        advance_result_is_nil_or_boolean,
        retention_time,
        event_info_ok,
        object_match,
        object_miss,
    ): (
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        f64,
        bool,
        bool,
        bool,
    ) = env
        .eval(
            r#"
            local resetOk = pcall(CombatLogResetFilter)
            local addFilterOk = pcall(CombatLogAddFilter, "anything")
            local setEntryOk = pcall(CombatLogSetCurrentEntry, 5)
            local clearOk = pcall(CombatLogClearEntries)
            local setRetentionOk = pcall(CombatLogSetRetentionTime, 120)

            local currentEntryOk, currentEntry = pcall(CombatLogGetCurrentEntry)
            local numEntries = CombatLogGetNumEntries()
            local showCurrent = CombatLogShowCurrentEntry()
            local advanceResult = CombatLogAdvanceEntry(1)
            local retentionTime = CombatLogGetRetentionTime()
            local eventInfoOk
            if select(4, GetBuildInfo()) == 16001 then
                eventInfoOk = CombatLogGetCurrentEventInfo == nil
                    and C_CombatLog.GetCurrentEventInfo == nil
            else
                eventInfoOk = pcall(CombatLogGetCurrentEventInfo)
            end

            local objectMatch = CombatLog_Object_IsA(0x21, 0x01)
            local objectMiss = CombatLog_Object_IsA(0x20, 0x01)

            return resetOk,
                addFilterOk,
                setEntryOk,
                clearOk,
                setRetentionOk,
                currentEntryOk,
                type(numEntries) == "number",
                type(showCurrent) == "boolean",
                advanceResult == nil or type(advanceResult) == "boolean",
                retentionTime,
                eventInfoOk,
                objectMatch,
                objectMiss
            "#,
        )
        .unwrap();

    assert!(reset_ok, "CombatLogResetFilter should be callable");
    assert!(add_filter_ok, "CombatLogAddFilter should be callable");
    assert!(set_entry_ok, "CombatLogSetCurrentEntry should be callable");
    assert!(clear_ok, "CombatLogClearEntries should be callable");
    assert!(
        set_retention_ok,
        "CombatLogSetRetentionTime should be callable"
    );
    assert!(
        current_entry_ok,
        "CombatLogGetCurrentEntry should stay callable through deprecated globals"
    );
    assert!(
        num_entries_is_number,
        "CombatLogGetNumEntries should return a numeric value"
    );
    assert!(
        show_current_is_boolean,
        "CombatLogShowCurrentEntry should return a boolean"
    );
    assert!(
        advance_result_is_nil_or_boolean,
        "CombatLogAdvanceEntry should return a boolean-like validity result"
    );
    assert!(
        retention_time > 0.0,
        "CombatLogGetRetentionTime should return a positive numeric retention time"
    );
    assert!(
        event_info_ok,
        "current-event publication must match the active profile"
    );
    assert!(object_match, "bitmask check should match overlapping flags");
    assert!(
        !object_miss,
        "bitmask check should fail for non-overlapping flags"
    );
}

#[test]
fn combat_log_namespaces_iterate_seeded_entries_and_messages() {
    let env = env();
    let (
        public_count_before,
        secure_count_before,
        public_current_event,
        public_show_current,
        secure_newest_valid,
        secure_entry_count,
        secure_current_event_name,
        secure_previous_valid,
        created_message_count,
        newest_message_text,
        oldest_message_text,
        public_count_after_clear,
        public_retention_after_set,
        filtered_enabled,
        message_limit,
    ): (
        i64,
        i64,
        String,
        bool,
        bool,
        i64,
        String,
        bool,
        i64,
        String,
        String,
        i64,
        f64,
        bool,
        i64,
    ) = env
        .eval(
            r#"
            local SOURCE_PLAYER_FLAGS = 1297
            local TARGET_PLAYER_FLAGS = 1298
            local TARGET_CREATURE_FLAGS = 68168

            local olderEntry = {
                1234567891,
                "SPELL_HEAL",
                false,
                "Player-1",
                "Player",
                SOURCE_PLAYER_FLAGS,
                0,
                "Player-2",
                "Target",
                TARGET_PLAYER_FLAGS,
                0,
                82326,
                "Holy Light",
                2,
                275,
                0,
                0,
                false,
            }
            local newestEntry = {
                1234567890,
                "SPELL_DAMAGE",
                false,
                "Player-1",
                "Player",
                SOURCE_PLAYER_FLAGS,
                0,
                "Creature-1",
                "Training Dummy",
                TARGET_CREATURE_FLAGS,
                0,
                19750,
                "Flash of Light",
                2,
                150,
                0,
                2,
                0,
                0,
                0,
                false,
                false,
                false,
                false,
            }

            C_CombatLogSecure._state.entries = { olderEntry, newestEntry }
            C_CombatLogSecure._state.currentIndex = nil
            C_CombatLogSecure._state.createdMessages = {}
            C_CombatLog._state.retentionTime = 300
            C_CombatLog._state.filteredEventsEnabled = false
            C_CombatLog._state.messageLimit = 300

            local publicCountBefore = C_CombatLog.GetEntryCount()
            local secureCountBefore = C_CombatLogSecure.GetEntryCount()
            local currentEventNamespace = C_CombatLog
            if select(4, GetBuildInfo()) == 16001 then
                currentEventNamespace = C_CombatLogInternal
            end
            local publicCurrentEvent = { currentEventNamespace.GetCurrentEventInfo() }
            local publicShowCurrent = C_CombatLog.ShouldShowCurrentEntry()

            local secureNewestValid = C_CombatLogSecure.SeekToNewestEntry()
            local secureEntryCount = C_CombatLogSecure.GetEntryCount()
            local secureCurrent = { C_CombatLogSecure.GetCurrentEntryInfo() }
            local securePreviousValid = C_CombatLogSecure.SeekToPreviousEntry()

            C_CombatLogSecure.CreateCombatLogMessage("newest message", 1, 0.5, 0.25, Enum.CombatLogMessageOrder.Newest)
            C_CombatLogSecure.CreateCombatLogMessage("oldest message", 0.25, 0.5, 1, Enum.CombatLogMessageOrder.Oldest)

            C_CombatLog.SetEntryRetentionTime(120)
            C_CombatLog.SetFilteredEventsEnabled(true)
            C_CombatLog.SetMessageLimit(400)
            C_CombatLog.ClearEntries()

            return publicCountBefore,
                secureCountBefore,
                publicCurrentEvent[2],
                publicShowCurrent,
                secureNewestValid,
                secureEntryCount,
                secureCurrent[2],
                securePreviousValid,
                #C_CombatLogSecure._state.createdMessages,
                C_CombatLogSecure._state.createdMessages[1].message,
                C_CombatLogSecure._state.createdMessages[2].message,
                C_CombatLog.GetEntryCount(),
                C_CombatLog.GetEntryRetentionTime(),
                C_CombatLog.AreFilteredEventsEnabled(),
                C_CombatLog.GetMessageLimit()
            "#,
        )
        .unwrap();

    assert_eq!(
        public_count_before, 2,
        "public count should read seeded entries"
    );
    assert_eq!(
        secure_count_before, 2,
        "secure count should read seeded entries"
    );
    assert_eq!(
        public_current_event, "SPELL_DAMAGE",
        "public current event should expose the newest seeded entry"
    );
    assert!(
        public_show_current,
        "public show-current should reflect a valid entry"
    );
    assert!(secure_newest_valid, "secure seek-to-newest should succeed");
    assert_eq!(
        secure_entry_count, 2,
        "secure entry count should match seeded entries"
    );
    assert_eq!(
        secure_current_event_name, "SPELL_DAMAGE",
        "secure current entry should expose the newest seeded entry"
    );
    assert!(
        secure_previous_valid,
        "secure seek-to-previous should reach the older entry"
    );
    assert_eq!(
        created_message_count, 2,
        "created messages should be recorded"
    );
    assert_eq!(
        newest_message_text, "newest message",
        "newest message should be stored"
    );
    assert_eq!(
        oldest_message_text, "oldest message",
        "oldest message should be stored"
    );
    assert_eq!(
        public_count_after_clear, 0,
        "ClearEntries should empty the combat log"
    );
    assert_eq!(
        public_retention_after_set, 120.0,
        "retention time should persist through SetEntryRetentionTime"
    );
    assert!(filtered_enabled, "filtered-events flag should persist");
    assert_eq!(message_limit, 400, "message limit should persist");
}

#[cfg(feature = "retail-12-0-0")]
mod combatlog_message_limit_tests {
    use super::{WowLuaEnv, env};

    fn read_limit(env: &WowLuaEnv) -> i64 {
        env.eval("return C_CombatLog.GetMessageLimit()").unwrap()
    }

    #[test]
    fn combatlog_message_limit_roundtrip_and_arity() {
        let env = env();
        env.exec(
            r#"
            for index, limit in ipairs({41, 42, 42, 41, 41, 42}) do
                assert(select('#', C_CombatLog.SetMessageLimit(limit)) == 0,
                    "message-limit setter must return zero values")
                assert(select('#', C_CombatLog.GetMessageLimit()) == 1,
                    "message-limit getter must return one value")
                local actual = C_CombatLog.GetMessageLimit()
                assert(type(actual) == "number",
                    "message-limit getter must return a number")
                assert(actual == limit,
                    "message limit differs after write " .. index)
            end
            "#,
        )
        .unwrap();
    }

    #[test]
    fn combatlog_message_limit_independent_of_filter_and_retention() {
        let env = env();
        env.exec(
            r#"
            C_CombatLog.SetFilteredEventsEnabled(false)
            C_CombatLog.SetEntryRetentionTime(120)
            C_CombatLog.SetMessageLimit(41)
            C_CombatLog.SetMessageLimit(42)
            assert(C_CombatLog.AreFilteredEventsEnabled() == false,
                "message-limit write changed filtered-events setting")
            assert(C_CombatLog.GetEntryRetentionTime() == 120,
                "message-limit write changed retention")

            C_CombatLog.SetFilteredEventsEnabled(true)
            assert(C_CombatLog.GetMessageLimit() == 42,
                "enabling filtered events changed message limit")
            C_CombatLog.SetEntryRetentionTime(240)
            assert(C_CombatLog.GetMessageLimit() == 42,
                "retention write changed message limit")

            C_CombatLog.SetMessageLimit(41)
            assert(C_CombatLog.AreFilteredEventsEnabled() == true,
                "message-limit write disabled filtered events")
            assert(C_CombatLog.GetEntryRetentionTime() == 240,
                "message-limit write changed updated retention")
            C_CombatLog.SetFilteredEventsEnabled(false)
            assert(C_CombatLog.GetMessageLimit() == 41,
                "disabling filtered events changed message limit")
            C_CombatLog.SetEntryRetentionTime(120)
            assert(C_CombatLog.GetMessageLimit() == 41,
                "restoring retention changed message limit")
            "#,
        )
        .unwrap();
    }

    #[test]
    fn combatlog_message_limit_isolates_lua_environments() {
        let first = env();
        first.exec("C_CombatLog.SetMessageLimit(41)").unwrap();
        let second = env();
        second.exec("C_CombatLog.SetMessageLimit(42)").unwrap();
        assert_eq!(read_limit(&first), 41);
        assert_eq!(read_limit(&second), 42);

        first.exec("C_CombatLog.SetMessageLimit(43)").unwrap();
        assert_eq!(read_limit(&first), 43);
        assert_eq!(read_limit(&second), 42);

        second.exec("C_CombatLog.SetMessageLimit(44)").unwrap();
        assert_eq!(read_limit(&first), 43);
        assert_eq!(read_limit(&second), 44);
    }
}

#[cfg(feature = "retail-12-0-0")]
mod audio_speaker_speed_tests {
    use super::{WowLuaEnv, env};

    fn read_speed(env: &WowLuaEnv) -> f64 {
        env.eval("return C_CombatAudioAlert.GetSpeakerSpeed()")
            .unwrap()
    }

    fn set_speed(env: &WowLuaEnv, speed: i64) {
        let accepted: bool = env
            .eval(&format!(
                "return C_CombatAudioAlert.SetSpeakerSpeed({speed})"
            ))
            .unwrap();
        assert!(accepted, "simulator accepted-write policy requires true");
    }

    #[test]
    fn audio_speaker_speed_explicit_and_repeated_writes() {
        let env = env();
        env.exec(
            r#"
            for index, speed in ipairs({1, 2, 2, 1, 1, 2}) do
                -- Accepted writes return true by simulator policy, not native change detection.
                assert(C_CombatAudioAlert.SetSpeakerSpeed(speed) == true,
                    "simulator must accept explicit speed write " .. index)
                assert(C_CombatAudioAlert.GetSpeakerSpeed() == speed,
                    "speaker speed differs after write " .. index)
            end
            "#,
        )
        .unwrap();
    }

    #[test]
    fn audio_speaker_speed_return_types_and_arity() {
        let env = env();
        env.exec(
            r#"
            local function single_value(expectedType, ...)
                assert(select('#', ...) == 1, "expected exactly one return value")
                local value = ...
                assert(type(value) == expectedType, "unexpected return type")
                return value
            end
            local accepted = single_value("boolean", C_CombatAudioAlert.SetSpeakerSpeed(2))
            assert(accepted == true, "simulator accepted-write policy requires true")
            local speed = single_value("number", C_CombatAudioAlert.GetSpeakerSpeed())
            assert(speed == 2, "getter must return explicitly written speed")
            "#,
        )
        .unwrap();
    }

    #[test]
    fn audio_speaker_speed_isolates_lua_environments() {
        let first = env();
        set_speed(&first, 1);
        let second = env();
        set_speed(&second, 2);
        assert_eq!(read_speed(&first), 1.0);
        assert_eq!(read_speed(&second), 2.0);

        set_speed(&first, 3);
        assert_eq!(read_speed(&first), 3.0);
        assert_eq!(read_speed(&second), 2.0);

        set_speed(&second, 4);
        assert_eq!(read_speed(&first), 3.0);
        assert_eq!(read_speed(&second), 4.0);
    }
}

#[cfg(feature = "retail-12-0-0")]
mod audio_speaker_volume_tests {
    use super::{WowLuaEnv, env};

    fn read_volume(env: &WowLuaEnv) -> f64 {
        env.eval("return C_CombatAudioAlert.GetSpeakerVolume()")
            .unwrap()
    }

    fn set_volume(env: &WowLuaEnv, volume: i64) {
        let accepted: bool = env
            .eval(&format!(
                "return C_CombatAudioAlert.SetSpeakerVolume({volume})"
            ))
            .unwrap();
        assert!(accepted, "simulator accepted-write policy requires true");
    }

    #[test]
    fn audio_speaker_volume_explicit_and_repeated_writes() {
        let env = env();
        env.exec(
            r#"
            for index, volume in ipairs({25, 75, 75, 25, 25, 75}) do
                -- Accepted writes return true by simulator policy, not native change detection.
                assert(C_CombatAudioAlert.SetSpeakerVolume(volume) == true,
                    "simulator must accept explicit volume write " .. index)
                assert(C_CombatAudioAlert.GetSpeakerVolume() == volume,
                    "speaker volume differs after write " .. index)
            end
            "#,
        )
        .unwrap();
    }

    #[test]
    fn audio_speaker_volume_arity_and_speed_independence() {
        let env = env();
        env.exec(
            r#"
            local function single_value(expectedType, ...)
                assert(select('#', ...) == 1, "expected exactly one return value")
                local value = ...
                assert(type(value) == expectedType, "unexpected return type: " .. type(value))
                return value
            end
            assert(C_CombatAudioAlert.SetSpeakerSpeed(1) == true)
            local accepted = single_value("boolean", C_CombatAudioAlert.SetSpeakerVolume(25))
            assert(accepted == true, "simulator accepted-write policy requires true")
            assert(single_value("number", C_CombatAudioAlert.GetSpeakerVolume()) == 25)
            assert(C_CombatAudioAlert.GetSpeakerSpeed() == 1,
                "volume write changed speaker speed")

            assert(C_CombatAudioAlert.SetSpeakerVolume(75) == true)
            assert(single_value("number", C_CombatAudioAlert.GetSpeakerVolume()) == 75)
            assert(C_CombatAudioAlert.GetSpeakerSpeed() == 1,
                "second volume write changed speaker speed")

            assert(C_CombatAudioAlert.SetSpeakerSpeed(2) == true)
            assert(C_CombatAudioAlert.GetSpeakerSpeed() == 2)
            assert(C_CombatAudioAlert.GetSpeakerVolume() == 75,
                "speed write changed speaker volume")

            assert(C_CombatAudioAlert.SetSpeakerVolume(25) == true)
            assert(C_CombatAudioAlert.GetSpeakerVolume() == 25)
            assert(C_CombatAudioAlert.GetSpeakerSpeed() == 2,
                "volume write changed updated speaker speed")
            "#,
        )
        .unwrap();
    }

    #[test]
    fn audio_speaker_volume_isolates_lua_environments() {
        let first = env();
        set_volume(&first, 20);
        let second = env();
        set_volume(&second, 40);
        assert_eq!(read_volume(&first), 20.0);
        assert_eq!(read_volume(&second), 40.0);

        set_volume(&first, 60);
        assert_eq!(read_volume(&first), 60.0);
        assert_eq!(read_volume(&second), 40.0);

        set_volume(&second, 80);
        assert_eq!(read_volume(&first), 60.0);
        assert_eq!(read_volume(&second), 80.0);
    }
}

#[cfg(feature = "retail-12-0-0")]
mod audio_format_setting_tests {
    use super::{WowLuaEnv, env};

    fn write_pair(env: &WowLuaEnv, health: i64, cast: i64) {
        env.exec(&format!(
            "assert(C_CombatAudioAlert.SetFormatSetting(0, 0, {health}) == true, \
             'health accepted-write policy requires true'); \
             assert(C_CombatAudioAlert.SetFormatSetting(1, 1, {cast}) == true, \
             'cast accepted-write policy requires true')"
        ))
        .unwrap();
    }

    fn read_pair(env: &WowLuaEnv) -> (f64, f64) {
        env.eval(
            "return C_CombatAudioAlert.GetFormatSetting(0, 0), \
             C_CombatAudioAlert.GetFormatSetting(1, 1)",
        )
        .unwrap()
    }

    #[test]
    fn audio_format_setting_distinct_keys_and_repeated_writes() {
        let env = env();
        env.exec(
            r#"
            local unit, alert = Enum.CombatAudioAlertUnit, Enum.CombatAudioAlertType
            assert(unit.Player == 0 and unit.Target == 1)
            assert(alert.Health == 0 and alert.Cast == 1)
            local keys = {
                {unit.Player, alert.Health}, {unit.Player, alert.Cast},
                {unit.Target, alert.Health}, {unit.Target, alert.Cast},
            }
            local expected = {}
            for index, key in ipairs(keys) do
                assert(C_CombatAudioAlert.SetFormatSetting(key[1], key[2], index) == true,
                    "simulator accepted-write policy requires true")
                expected[index] = index
            end
            for _, write in ipairs({{1, 2}, {2, 3}, {3, 1}, {4, 2}, {1, 2}, {4, 2}}) do
                local index, value = write[1], write[2]
                local key = keys[index]
                assert(C_CombatAudioAlert.SetFormatSetting(key[1], key[2], value) == true)
                expected[index] = value
                for other, otherKey in ipairs(keys) do
                    assert(C_CombatAudioAlert.GetFormatSetting(otherKey[1], otherKey[2]) == expected[other],
                        "format key changed unexpectedly: " .. other)
                end
            end
            "#,
        )
        .unwrap();
    }

    #[test]
    fn audio_format_setting_arity_and_speaker_preservation() {
        let env = env();
        env.exec(
            r#"
            local function single_value(expectedType, ...)
                assert(select('#', ...) == 1, "expected exactly one return value")
                local value = ...
                assert(type(value) == expectedType, "unexpected return type: " .. type(value))
                return value
            end
            assert(C_CombatAudioAlert.SetSpeakerSpeed(1) == true)
            assert(C_CombatAudioAlert.SetSpeakerVolume(25) == true)
            for _, value in ipairs({1, 2, 2}) do
                assert(single_value("boolean", C_CombatAudioAlert.SetFormatSetting(0, 1, value)) == true,
                    "simulator accepted-write policy requires true")
                assert(single_value("number", C_CombatAudioAlert.GetFormatSetting(0, 1)) == value)
                assert(C_CombatAudioAlert.GetSpeakerSpeed() == 1)
                assert(C_CombatAudioAlert.GetSpeakerVolume() == 25)
            end
            assert(C_CombatAudioAlert.SetSpeakerSpeed(2) == true)
            assert(C_CombatAudioAlert.SetSpeakerVolume(75) == true)
            assert(C_CombatAudioAlert.GetFormatSetting(0, 1) == 2)
            assert(C_CombatAudioAlert.SetFormatSetting(0, 1, 3) == true)
            assert(C_CombatAudioAlert.GetFormatSetting(0, 1) == 3)
            assert(C_CombatAudioAlert.GetSpeakerSpeed() == 2)
            assert(C_CombatAudioAlert.GetSpeakerVolume() == 75)
            "#,
        )
        .unwrap();
    }

    #[test]
    fn audio_format_setting_isolates_lua_environments_bidirectionally() {
        let first = env();
        write_pair(&first, 1, 2);
        let second = env();
        write_pair(&second, 3, 4);
        assert_eq!(read_pair(&first), (1.0, 2.0));
        assert_eq!(read_pair(&second), (3.0, 4.0));

        write_pair(&first, 2, 1);
        assert_eq!(read_pair(&first), (2.0, 1.0));
        assert_eq!(read_pair(&second), (3.0, 4.0));

        write_pair(&second, 4, 3);
        assert_eq!(read_pair(&first), (2.0, 1.0));
        assert_eq!(read_pair(&second), (4.0, 3.0));
    }
}

#[cfg(feature = "retail-12-0-0")]
mod custom_set_crud_tests {
    use super::{WowLuaEnv, env};

    // Accepted concrete fixtures and copy-value storage are simulator policies,
    // not native validation, defaults, limits, persistence, or event contracts.
    // Pinned 12.0.0 declares these seven APIs; current ItemUtil.lua confirms
    // appearanceID, secondaryAppearanceID, illusionID, with no slot information.
    fn fixture_env() -> WowLuaEnv {
        let env = env();
        env.exec(
            r#"
            api = C_TransmogCollection
            function item(a, s, i)
                return {appearanceID=a, secondaryAppearanceID=s, illusionID=i}
            end
            function one(kind, ...)
                assert(select('#', ...) == 1, 'expected one return')
                local value = ...
                assert(type(value) == kind, 'expected '..kind..', got '..type(value))
                return value
            end
            function zero(...)
                assert(select('#', ...) == 0, 'mutator must return zero values')
            end
            function create(name, icon, items)
                return one('number', api.NewCustomSet(name, icon, items))
            end
            function check_record(id, name, icon, expected)
                assert(select('#', api.GetCustomSetInfo(id)) == 2, 'info must return two values')
                local actualName, actualIcon = api.GetCustomSetInfo(id)
                assert(actualName == name and actualIcon == icon, 'name/icon mismatch')
                local actual = one('table', api.GetCustomSetItemTransmogInfoList(id))
                assert(#actual == #expected, 'item count mismatch')
                for index, value in ipairs(expected) do
                    for _, key in ipairs({'appearanceID', 'secondaryAppearanceID', 'illusionID'}) do
                        assert(actual[index][key] == value[key], 'item field mismatch: '..key)
                    end
                end
            end
            function has_id(id)
                local ids = one('table', api.GetCustomSets())
                for _, candidate in ipairs(ids) do
                    assert(type(candidate) == 'number')
                    if candidate == id then return true end
                end
                return false
            end
            firstItems = {item(101, 102, 103), item(111, 112, 113)}
            secondItems = {item(201, 202, 203)}
            "#,
        )
        .unwrap();
        env
    }

    #[test]
    fn custom_set_crud_creates_distinct_records_and_lists_ids() {
        let env = fixture_env();
        env.exec(
            r#"
            local first = create('Copper Dawn', 134400, firstItems)
            local second = create('Silver Dusk', 134401, secondItems)
            assert(first ~= second, 'two records require distinct IDs')
            assert(has_id(first) and has_id(second), 'created IDs must be listed')
            check_record(first, 'Copper Dawn', 134400, firstItems)
            check_record(second, 'Silver Dusk', 134401, secondItems)
            "#,
        )
        .unwrap();
    }

    #[test]
    fn custom_set_crud_replaces_renames_and_deletes_without_changing_other_record() {
        let env = fixture_env();
        env.exec(
            r#"
            local first = create('Copper Dawn', 134400, firstItems)
            local second = create('Silver Dusk', 134401, secondItems)
            assert(first ~= second)
            local replacement = {item(301, 302, 303)}
            zero(api.ModifyCustomSet(first, replacement))
            check_record(first, 'Copper Dawn', 134400, replacement)
            check_record(second, 'Silver Dusk', 134401, secondItems)
            zero(api.RenameCustomSet(first, 'Golden Noon'))
            check_record(first, 'Golden Noon', 134400, replacement)
            check_record(second, 'Silver Dusk', 134401, secondItems)
            zero(api.DeleteCustomSet(first))
            assert(not has_id(first) and has_id(second), 'delete must remove only its ID')
            check_record(second, 'Silver Dusk', 134401, secondItems)
            "#,
        )
        .unwrap();
    }

    #[test]
    fn custom_set_crud_copies_input_and_output_tables() {
        let env = fixture_env();
        env.exec(
            r#"
            local id = create('Copper Dawn', 134400, firstItems)
            firstItems[1].appearanceID = 999
            firstItems[1].secondaryAppearanceID = 998
            firstItems[1].illusionID = 997
            firstItems[2] = nil
            check_record(id, 'Copper Dawn', 134400, {item(101, 102, 103), item(111, 112, 113)})
            local replacement = {item(301, 302, 303), item(311, 312, 313)}
            zero(api.ModifyCustomSet(id, replacement))
            replacement[1].appearanceID = 888
            replacement[1].secondaryAppearanceID = 887
            replacement[1].illusionID = 886
            replacement[2] = nil
            local expected = {item(301, 302, 303), item(311, 312, 313)}
            check_record(id, 'Copper Dawn', 134400, expected)
            local output = one('table', api.GetCustomSetItemTransmogInfoList(id))
            output[1].appearanceID = 777
            output[1].secondaryAppearanceID = 776
            output[1].illusionID = 775
            output[2] = nil
            local ids = one('table', api.GetCustomSets())
            for index in pairs(ids) do ids[index] = nil end
            assert(has_id(id), 'mutating returned IDs must not remove stored records')
            check_record(id, 'Copper Dawn', 134400, expected)
            "#,
        )
        .unwrap();
    }

    #[test]
    fn custom_set_crud_isolates_distinct_environment_data() {
        let first = fixture_env();
        first
            .exec("id = create('Copper Dawn', 134400, firstItems)")
            .unwrap();
        let second = fixture_env();
        second
            .exec("id = create('Silver Dusk', 134401, secondItems)")
            .unwrap();
        first
            .exec("check_record(id, 'Copper Dawn', 134400, firstItems)")
            .unwrap();
        second
            .exec("check_record(id, 'Silver Dusk', 134401, secondItems)")
            .unwrap();
        first.exec("zero(api.RenameCustomSet(id, 'Golden Noon')); zero(api.ModifyCustomSet(id, {item(301,302,303)}))").unwrap();
        second.exec("check_record(id, 'Silver Dusk', 134401, secondItems); zero(api.RenameCustomSet(id, 'Moonrise'))").unwrap();
        first.exec("check_record(id, 'Golden Noon', 134400, {item(301,302,303)}); zero(api.DeleteCustomSet(id)); assert(not has_id(id))").unwrap();
        second
            .exec("assert(has_id(id)); check_record(id, 'Moonrise', 134401, secondItems)")
            .unwrap();
    }
}

#[cfg(any(feature = "retail-12-0-0", feature = "client-wowforever"))]
mod neighborhood_tracked_tasks_tests {
    use super::{WowLuaEnv, env};

    #[cfg(feature = "client-wowforever")]
    #[test]
    fn neighborhood_tracked_tasks_forever_tracker_consumer() {
        let env = fixture_env();
        let root = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
        let source = std::fs::read_to_string(
            root.join("Blizzard_ObjectiveTracker/Blizzard_InitiativeTasksObjectiveTracker.lua"),
        )
        .unwrap();
        env.exec(&source).unwrap();
        env.exec(
            r#"
            local tracker = InitiativeTasksObjectiveTrackerMixin
            tracker:InitModule()
            tracker:OnEvent('PLAYER_ENTERING_WORLD')
            tracker:LayoutContents()
            api.AddTrackedInitiativeTask(20)
            api.AddTrackedInitiativeTask(40)
            check_members({20, 40})
            tracker:UntrackInitiativeTask(20)
            check_members({40})
            tracker:UntrackInitiativeTask(40)
            check_members({})
            tracker:OnEvent('ZONE_CHANGED_NEW_AREA')
            tracker:LayoutContents()
        "#,
        )
        .unwrap();
    }

    // Duplicate-add/unknown-remove idempotence and detached getter copies are
    // simulator policies, not native ordering, validation, or lifecycle claims.
    fn fixture_env() -> WowLuaEnv {
        let env = env();
        env.exec(
            r#"
            api = C_NeighborhoodInitiative
            function check_members(expected)
                local result = api.GetTrackedInitiativeTasks()
                assert(type(result) == 'table', 'getter must return a table')
                local ids = result.trackedIDs
                assert(type(ids) == 'table', 'trackedIDs must be an array')
                local wanted, seen = {}, {}
                for _, id in ipairs(expected) do wanted[id] = true end
                local count = 0
                for index, id in pairs(ids) do
                    assert(type(index) == 'number' and index >= 1 and index <= #expected
                        and index == math.floor(index), 'trackedIDs must use contiguous array indices')
                    assert(type(id) == 'number', 'tracked ID must be numeric')
                    assert(wanted[id], 'unexpected tracked ID: '..tostring(id))
                    assert(not seen[id], 'duplicate tracked ID: '..tostring(id))
                    seen[id] = true
                    count = count + 1
                end
                assert(count == #expected, 'tracked membership count mismatch')
                for _, id in ipairs(expected) do
                    assert(seen[id], 'missing tracked ID: '..id)
                end
            end
            "#,
        )
        .unwrap();
        env
    }

    #[test]
    fn neighborhood_tracked_tasks_membership_and_idempotent_mutations() {
        let env = fixture_env();
        env.exec(
            r#"
            api.AddTrackedInitiativeTask(20)
            api.AddTrackedInitiativeTask(40)
            check_members({20, 40})
            api.AddTrackedInitiativeTask(20)
            check_members({20, 40})
            api.RemoveTrackedInitiativeTask(60)
            check_members({20, 40})
            api.RemoveTrackedInitiativeTask(20)
            check_members({40})
            api.RemoveTrackedInitiativeTask(20)
            check_members({40})
            "#,
        )
        .unwrap();
    }

    #[test]
    fn neighborhood_tracked_tasks_arity_and_detached_getter_array() {
        let env = fixture_env();
        env.exec(
            r#"
            local function zero(...)
                assert(select('#', ...) == 0, 'mutator must return zero values')
            end
            local function one_table(...)
                assert(select('#', ...) == 1, 'getter must return exactly one value')
                local value = ...
                assert(type(value) == 'table', 'getter must return a table')
                return value
            end
            zero(api.AddTrackedInitiativeTask(20))
            zero(api.AddTrackedInitiativeTask(40))
            local snapshot = one_table(api.GetTrackedInitiativeTasks())
            check_members({20, 40})
            snapshot.trackedIDs[1] = 80
            snapshot.trackedIDs[2] = nil
            check_members({20, 40})
            zero(api.RemoveTrackedInitiativeTask(20))
            check_members({40})
            "#,
        )
        .unwrap();
    }

    #[test]
    fn neighborhood_tracked_tasks_isolates_environments_bidirectionally() {
        let first = fixture_env();
        first
            .exec("api.AddTrackedInitiativeTask(20); api.AddTrackedInitiativeTask(40)")
            .unwrap();
        let second = fixture_env();
        second
            .exec("api.AddTrackedInitiativeTask(60); api.AddTrackedInitiativeTask(80)")
            .unwrap();
        first.exec("check_members({20, 40})").unwrap();
        second.exec("check_members({60, 80})").unwrap();
        first.exec("api.RemoveTrackedInitiativeTask(20); api.AddTrackedInitiativeTask(100); check_members({40, 100})").unwrap();
        second.exec("check_members({60, 80})").unwrap();
        second.exec("api.RemoveTrackedInitiativeTask(80); api.AddTrackedInitiativeTask(120); check_members({60, 120})").unwrap();
        first.exec("check_members({40, 100})").unwrap();
    }
}

#[cfg(feature = "retail-12-0-0")]
mod combat_text_active_unit_tests {
    use super::env;

    #[test]
    fn combat_text_active_unit_explicit_and_repeated_writes() {
        let env = env();
        env.exec(
            r#"
            for index, unit in ipairs({"player", "vehicle", "vehicle", "player", "player"}) do
                C_CombatText.SetActiveUnit(unit)
                local actual = C_CombatText.GetActiveUnit()
                assert(type(actual) == "string", "getter must return a string after write " .. index)
                assert(actual == unit, "active unit differs after write " .. index)
            end
            "#,
        )
        .unwrap();
    }

    #[test]
    fn combat_text_active_unit_return_types_and_arity() {
        let env = env();
        env.exec(
            r#"
            local function zero(...)
                assert(select('#', ...) == 0, "setter must return zero values")
            end
            local function one_string(expected, ...)
                assert(select('#', ...) == 1, "getter must return exactly one value")
                local actual = ...
                assert(type(actual) == "string", "getter must return a string after explicit write")
                assert(actual == expected, "getter must return the explicitly written token")
            end
            zero(C_CombatText.SetActiveUnit("vehicle"))
            one_string("vehicle", C_CombatText.GetActiveUnit())
            zero(C_CombatText.SetActiveUnit("player"))
            one_string("player", C_CombatText.GetActiveUnit())
            "#,
        )
        .unwrap();
    }

    #[test]
    fn combat_text_active_unit_isolates_environments_bidirectionally() {
        let first = env();
        let second = env();
        first.exec("C_CombatText.SetActiveUnit('player')").unwrap();
        second.exec("C_CombatText.SetActiveUnit('player')").unwrap();
        first.exec("C_CombatText.SetActiveUnit('vehicle')").unwrap();
        assert_eq!(
            first
                .eval::<String>("return C_CombatText.GetActiveUnit()")
                .unwrap(),
            "vehicle"
        );
        assert_eq!(
            second
                .eval::<String>("return C_CombatText.GetActiveUnit()")
                .unwrap(),
            "player"
        );

        second
            .exec("C_CombatText.SetActiveUnit('vehicle')")
            .unwrap();
        first.exec("C_CombatText.SetActiveUnit('vehicle')").unwrap();
        second.exec("C_CombatText.SetActiveUnit('player')").unwrap();
        assert_eq!(
            first
                .eval::<String>("return C_CombatText.GetActiveUnit()")
                .unwrap(),
            "vehicle"
        );
        assert_eq!(
            second
                .eval::<String>("return C_CombatText.GetActiveUnit()")
                .unwrap(),
            "player"
        );
    }
}

#[cfg(feature = "retail-12-0-0")]
mod transmog_sets_filter_tests {
    use super::env;

    #[test]
    fn transmog_sets_filter_explicit_writes_preserve_other_keys() {
        let env = env();
        env.exec(
            r#"
            -- Consumer IDs: collected, uncollected, PVE, PVP.
            local expected = {true, false, true, false}
            for index = 1, 4 do
                C_TransmogSets.SetSetsFilter(index, expected[index])
            end
            local function check_all()
                for index = 1, 4 do
                    local actual = C_TransmogSets.GetSetsFilter(index)
                    assert(type(actual) == "boolean", "filter must be boolean: " .. index)
                    assert(actual == expected[index], "filter value differs: " .. index)
                end
            end
            check_all()
            for index = 1, 4 do
                for _, value in ipairs({false, true, true, false}) do
                    C_TransmogSets.SetSetsFilter(index, value)
                    expected[index] = value
                    check_all()
                end
            end
            "#,
        )
        .unwrap();
    }

    #[test]
    fn transmog_sets_filter_return_types_and_arity() {
        let env = env();
        env.exec(
            r#"
            local function zero(...)
                assert(select('#', ...) == 0, "setter must return zero values")
            end
            local function one_boolean(expected, ...)
                assert(select('#', ...) == 1, "getter must return exactly one value")
                local actual = ...
                assert(type(actual) == "boolean", "getter must return a boolean")
                assert(actual == expected, "getter must return the explicit value")
            end
            for index = 1, 4 do
                for _, value in ipairs({true, false}) do
                    zero(C_TransmogSets.SetSetsFilter(index, value))
                    one_boolean(value, C_TransmogSets.GetSetsFilter(index))
                end
            end
            "#,
        )
        .unwrap();
    }

    #[test]
    fn transmog_sets_filter_isolates_environments_bidirectionally() {
        let first = env();
        let second = env();
        first
            .exec("for i = 1, 4 do C_TransmogSets.SetSetsFilter(i, i % 2 == 1) end")
            .unwrap();
        second
            .exec("for i = 1, 4 do C_TransmogSets.SetSetsFilter(i, i % 2 == 0) end")
            .unwrap();
        first
            .exec("for i = 1, 4 do assert(C_TransmogSets.GetSetsFilter(i) == (i % 2 == 1)) end")
            .unwrap();
        second
            .exec("for i = 1, 4 do assert(C_TransmogSets.GetSetsFilter(i) == (i % 2 == 0)) end")
            .unwrap();

        first
            .exec("for i = 1, 4 do C_TransmogSets.SetSetsFilter(i, true) end")
            .unwrap();
        second
            .exec("for i = 1, 4 do assert(C_TransmogSets.GetSetsFilter(i) == (i % 2 == 0)) end")
            .unwrap();
        second
            .exec("for i = 1, 4 do C_TransmogSets.SetSetsFilter(i, false) end")
            .unwrap();
        first
            .exec("for i = 1, 4 do assert(C_TransmogSets.GetSetsFilter(i) == true) end")
            .unwrap();
        second
            .exec("for i = 1, 4 do assert(C_TransmogSets.GetSetsFilter(i) == false) end")
            .unwrap();
    }
}

#[cfg(feature = "retail-12-0-0")]
mod outfit_situations_enabled_tests {
    use super::env;

    #[test]
    fn outfit_situations_enabled_explicit_and_repeated_writes() {
        let env = env();
        env.exec(
            r#"
            for _, enabled in ipairs({false, true, true, false, false}) do
                C_TransmogOutfitInfo.SetOutfitSituationsEnabled(enabled)
                local actual = C_TransmogOutfitInfo.GetOutfitSituationsEnabled()
                assert(type(actual) == "boolean", "getter must return a boolean")
                assert(actual == enabled, "getter must reflect explicit global toggle")
            end
            "#,
        )
        .unwrap();
    }

    #[test]
    fn outfit_situations_enabled_return_types_and_arity() {
        let env = env();
        env.exec(
            r#"
            local function zero(...)
                assert(select('#', ...) == 0, "setter must return zero values")
            end
            local function one_boolean(expected, ...)
                assert(select('#', ...) == 1, "getter must return exactly one value")
                local actual = ...
                assert(type(actual) == "boolean", "getter must return a boolean")
                assert(actual == expected, "getter must return the explicit value")
            end
            for _, enabled in ipairs({false, true}) do
                zero(C_TransmogOutfitInfo.SetOutfitSituationsEnabled(enabled))
                one_boolean(enabled, C_TransmogOutfitInfo.GetOutfitSituationsEnabled())
            end
            "#,
        )
        .unwrap();
    }

    #[test]
    fn outfit_situations_enabled_isolates_environments_bidirectionally() {
        let first = env();
        let second = env();
        first
            .exec("C_TransmogOutfitInfo.SetOutfitSituationsEnabled(false)")
            .unwrap();
        second
            .exec("C_TransmogOutfitInfo.SetOutfitSituationsEnabled(true)")
            .unwrap();
        let read = |env: &wow_ui_sim::lua_api::WowLuaEnv| {
            env.eval::<bool>("return C_TransmogOutfitInfo.GetOutfitSituationsEnabled()")
                .unwrap()
        };
        assert!(!read(&first));
        assert!(read(&second));

        first
            .exec("C_TransmogOutfitInfo.SetOutfitSituationsEnabled(true)")
            .unwrap();
        assert!(read(&first));
        assert!(read(&second));
        second
            .exec("C_TransmogOutfitInfo.SetOutfitSituationsEnabled(false)")
            .unwrap();
        assert!(read(&first));
        assert!(!read(&second));
    }
}

#[cfg(feature = "client-wowforever")]
mod forever_combat_namespace {
    use super::env;

    fn assert_epic_meter_predicate(env: &wow_ui_sim::lua_api::WowLuaEnv) {
        env.exec(
            r#"
            assert(rawget(C_CombatLog, "GetCurrentEventInfo") == nil, "raw public getter survived")
            assert(C_CombatLog.GetCurrentEventInfo == nil, "public namespace synthesized getter")
            assert(rawget(_G, "CombatLogGetCurrentEventInfo") == nil, "legacy global getter survived")
            local _, _, _, tocVersion = GetBuildInfo()
            local isRetail = tocVersion >= 120000
            -- EpicDamageMeter 8930362, Core/Constants.lua: actual capability predicate.
            local useMeterAPI = isRetail
                or (CombatLogGetCurrentEventInfo == nil
                    and C_DamageMeter ~= nil
                    and type(C_DamageMeter.GetCombatSessionFromType) == "function")
            assert(tocVersion == 16001 and useMeterAPI, "Epic meter predicate rejected current surface")
        "#,
        )
        .unwrap();
    }

    #[test]
    fn direct_environment_selects_meter_without_legacy_getter() {
        assert_epic_meter_predicate(&env());
    }

    #[test]
    fn deprecated_publisher_does_not_restore_legacy_getter() {
        crate::common::blizzard_addon_harness::with_blizzard_addon_closure(
            &["Blizzard_DeprecatedCombatLog"],
            &[],
            |env, loaded| {
                assert!(
                    loaded
                        .iter()
                        .any(|name| name == "Blizzard_DeprecatedCombatLog")
                );
                assert!(
                    env.eval::<bool>("return GetCVarBool('loadDeprecationFallbacks')")
                        .unwrap()
                );
                assert_epic_meter_predicate(env);
                env.apply_post_load_workarounds();
                assert_epic_meter_predicate(env);
            },
        );
    }

    #[test]
    fn documented_getters_share_concrete_fixture_entries() {
        let env = env();
        env.exec(
            r#"
            assert(type(C_CombatLogInternal) == "table")
            assert(type(C_CombatLogInternal.GetCurrentEventInfo) == "function")
            assert(type(C_CombatLogSecure.GetCurrentEventInfo) == "function")
            local state = C_CombatLog._state
            state.entries = {{"SPELL_DAMAGE", 19750, 150}, {"SPELL_HEAL", 19750, 275}}
            state.currentIndex = 1
            local event, spell, amount = C_CombatLogInternal.GetCurrentEventInfo()
            assert(event == "SPELL_DAMAGE" and spell == 19750 and amount == 150)
            assert(C_CombatLogSecure.SeekToNewestEntry())
            event, spell, amount = C_CombatLogSecure.GetCurrentEventInfo()
            assert(event == "SPELL_HEAL" and spell == 19750 and amount == 275)
            event, spell, amount = C_CombatLogInternal.GetCurrentEventInfo()
            assert(event == "SPELL_HEAL" and spell == 19750 and amount == 275)
            C_CombatLog.ClearEntries()
            assert(C_CombatLogInternal.GetCurrentEventInfo() == nil)
            assert(C_CombatLogSecure.GetCurrentEventInfo() == nil)
        "#,
        )
        .unwrap();
    }
}
