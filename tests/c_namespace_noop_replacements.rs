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
            local eventInfoOk = pcall(CombatLogGetCurrentEventInfo)

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
        "CombatLogGetCurrentEventInfo should stay callable"
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
            local publicCurrentEvent = { C_CombatLog.GetCurrentEventInfo() }
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
            .eval(&format!("return C_CombatAudioAlert.SetSpeakerSpeed({speed})"))
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
