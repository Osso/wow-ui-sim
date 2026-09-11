//! Tests for seeded data APIs: C_EncounterTimeline, C_DamageMeter, sound drivers,
//! Battle.net stubs, C_FriendList, C_LossOfControl, macros, and PlayerLocation.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[cfg(not(feature = "retail-12-1-5"))]
#[test]
fn test_c_encounter_timeline_returns_seeded_visible_event() {
    let env = env();
    let result: String = env
        .eval(
            r#"
        if not C_EncounterTimeline.IsFeatureAvailable() then
            return "feature_unavailable"
        end
        if not C_EncounterTimeline.IsFeatureEnabled() then
            return "feature_disabled"
        end

        local eventIDs = C_EncounterTimeline.GetEventList()
        if #eventIDs ~= 1 then
            return "event_count=" .. tostring(#eventIDs)
        end

        local eventID = eventIDs[1]
        local info = C_EncounterTimeline.GetEventInfo(eventID)
        if not info then
            return "missing_event_info"
        end
        if info.spellID ~= 19750 then
            return "spell_id=" .. tostring(info.spellID)
        end
        if info.spellName ~= "Flash of Light" then
            return "spell_name=" .. tostring(info.spellName)
        end

        local timer = C_EncounterTimeline.GetEventTimer(eventID)
        if not timer then
            return "missing_timer"
        end
        if timer:GetRemainingDuration() <= 0 then
            return "remaining_duration=" .. tostring(timer:GetRemainingDuration())
        end

        local track, trackSortIndex = C_EncounterTimeline.GetEventTrack(eventID)
        if track ~= Enum.EncounterTimelineTrack.Short then
            return "track=" .. tostring(track)
        end
        if trackSortIndex ~= 1 then
            return "track_sort_index=" .. tostring(trackSortIndex)
        end

        if not C_EncounterTimeline.HasActiveEvents() then
            return "missing_active_events"
        end
        if not C_EncounterTimeline.HasVisibleEvents() then
            return "missing_visible_events"
        end
        if C_EncounterTimeline.AddEditModeEvents() == nil then
            return "missing_edit_mode_duration"
        end
        C_EncounterTimeline.CancelEditModeEvents()

        return "ok"
    "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "C_EncounterTimeline should expose a seeded visible event with timer data: {result}"
    );
}

#[test]
fn test_c_damage_meter_returns_seeded_session_and_source_data() {
    let env = env();
    let result: String = env
        .eval(
            r#"
        local isAvailable, failureReason = C_DamageMeter.IsDamageMeterAvailable()
        if not isAvailable then
            return "unavailable:" .. tostring(failureReason)
        end

        local availableSessions = C_DamageMeter.GetAvailableCombatSessions()
        if #availableSessions ~= 1 then
            return "session_count=" .. tostring(#availableSessions)
        end

        local sessionInfo = availableSessions[1]
        if sessionInfo.sessionID ~= 1 then
            return "session_id=" .. tostring(sessionInfo.sessionID)
        end

        local overallSession = C_DamageMeter.GetCombatSessionFromType(
            Enum.DamageMeterSessionType.Overall,
            Enum.DamageMeterType.DamageDone
        )
        if not overallSession then
            return "missing_overall_session"
        end
        if overallSession.totalAmount <= 0 then
            return "overall_total=" .. tostring(overallSession.totalAmount)
        end
        if #overallSession.combatSources ~= 2 then
            return "source_count=" .. tostring(#overallSession.combatSources)
        end

        local topSource = overallSession.combatSources[1]
        if topSource.name ~= "Player" then
            return "top_source=" .. tostring(topSource.name)
        end
        if not topSource.isLocalPlayer then
            return "top_source_not_local"
        end

        local sourceSession = C_DamageMeter.GetCombatSessionSourceFromType(
            Enum.DamageMeterSessionType.Overall,
            Enum.DamageMeterType.DamageDone,
            topSource.sourceGUID,
            topSource.sourceCreatureID
        )
        if not sourceSession then
            return "missing_source_session"
        end
        if sourceSession.totalAmount ~= topSource.totalAmount then
            return "source_total=" .. tostring(sourceSession.totalAmount)
        end
        if #sourceSession.combatSpells == 0 then
            return "spell_count=0"
        end
        if sourceSession.combatSpells[1].spellID ~= 19750 then
            return "spell_id=" .. tostring(sourceSession.combatSpells[1].spellID)
        end

        local durationSeconds = C_DamageMeter.GetSessionDurationSeconds(Enum.DamageMeterSessionType.Overall)
        if durationSeconds <= 0 then
            return "duration=" .. tostring(durationSeconds)
        end

        return "ok"
    "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "C_DamageMeter should expose seeded session, source, and spell data: {result}"
    );
}

#[test]
fn test_c_damage_meter_current_session_returns_seeded_data() {
    let env = env();
    let result: String = env
        .eval(
            r#"
        local currentSession = C_DamageMeter.GetCombatSessionFromType(
            Enum.DamageMeterSessionType.Current,
            Enum.DamageMeterType.DamageDone
        )
        if not currentSession then
            return "missing_current_session"
        end
        if currentSession.sessionID ~= 1 then
            return "session_id=" .. tostring(currentSession.sessionID)
        end

        local topSource = currentSession.combatSources[1]
        if not topSource then
            return "missing_top_source"
        end

        local sourceSession = C_DamageMeter.GetCombatSessionSourceFromType(
            Enum.DamageMeterSessionType.Current,
            Enum.DamageMeterType.DamageDone,
            topSource.sourceGUID,
            topSource.sourceCreatureID
        )
        if not sourceSession then
            return "missing_current_source_session"
        end
        if sourceSession.totalAmount ~= topSource.totalAmount then
            return "source_total=" .. tostring(sourceSession.totalAmount)
        end

        local durationSeconds = C_DamageMeter.GetSessionDurationSeconds(Enum.DamageMeterSessionType.Current)
        if durationSeconds <= 0 then
            return "duration=" .. tostring(durationSeconds)
        end

        return "ok"
    "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "C_DamageMeter should expose seeded data for the current damage session: {result}"
    );
}

#[test]
fn test_c_damage_meter_returns_empty_sessions_for_unseeded_meter_types() {
    let env = env();
    let result: String = env.eval(r#"
        for name, damageType in pairs(Enum.DamageMeterType) do
            if damageType ~= Enum.DamageMeterType.DamageDone then
                local byId = C_DamageMeter.GetCombatSessionFromID(1, damageType)
                if type(byId) ~= "table" then
                    return "by_id_" .. name .. "=" .. type(byId)
                end
                if type(byId.combatSources) ~= "table" then
                    return "by_id_sources_" .. name .. "=" .. type(byId.combatSources)
                end
                if #byId.combatSources ~= 0 then
                    return "by_id_source_count_" .. name .. "=" .. tostring(#byId.combatSources)
                end
                if byId.totalAmount ~= 0 or byId.maxAmount ~= 0 or byId.durationSeconds ~= 0 then
                    return "by_id_totals_" .. name
                end

                local byType = C_DamageMeter.GetCombatSessionFromType(
                    Enum.DamageMeterSessionType.Overall,
                    damageType
                )
                if type(byType) ~= "table" then
                    return "by_type_" .. name .. "=" .. type(byType)
                end
                if type(byType.combatSources) ~= "table" then
                    return "by_type_sources_" .. name .. "=" .. type(byType.combatSources)
                end
                if #byType.combatSources ~= 0 then
                    return "by_type_source_count_" .. name .. "=" .. tostring(#byType.combatSources)
                end

                local sourceById = C_DamageMeter.GetCombatSessionSourceFromID(1, damageType, nil, nil)
                if type(sourceById) ~= "table" then
                    return "source_by_id_" .. name .. "=" .. type(sourceById)
                end
                if type(sourceById.combatSpells) ~= "table" then
                    return "source_by_id_spells_" .. name .. "=" .. type(sourceById.combatSpells)
                end
                if #sourceById.combatSpells ~= 0 then
                    return "source_by_id_spell_count_" .. name .. "=" .. tostring(#sourceById.combatSpells)
                end

                local sourceByType = C_DamageMeter.GetCombatSessionSourceFromType(
                    Enum.DamageMeterSessionType.Overall,
                    damageType,
                    nil,
                    nil
                )
                if type(sourceByType) ~= "table" then
                    return "source_by_type_" .. name .. "=" .. type(sourceByType)
                end
                if type(sourceByType.combatSpells) ~= "table" then
                    return "source_by_type_spells_" .. name .. "=" .. type(sourceByType.combatSpells)
                end
            end
        end

        local damageSession = C_DamageMeter.GetCombatSessionFromID(1, Enum.DamageMeterType.DamageDone)
        local topSource = damageSession.combatSources[1]
        local sourceWithoutCreatureID = C_DamageMeter.GetCombatSessionSourceFromID(
            1,
            Enum.DamageMeterType.DamageDone,
            topSource.sourceGUID,
            nil
        )
        if not sourceWithoutCreatureID or sourceWithoutCreatureID.sourceGUID ~= topSource.sourceGUID then
            return "source_without_creature_id"
        end

        return "ok"
    "#).unwrap();
    assert_eq!(
        result, "ok",
        "C_DamageMeter should return empty sessions for valid unseeded meter types: {result}"
    );
}

#[test]
fn test_sound_system_driver_functions_return_seeded_devices() {
    let env = env();
    let result: String = env
        .eval(
            r#"
        local gameOutputs = Sound_GameSystem_GetNumOutputDrivers()
        if gameOutputs ~= 1 then
            return "game_outputs=" .. tostring(gameOutputs)
        end
        if Sound_GameSystem_GetOutputDriverNameByIndex(0) ~= "Silent Output Device" then
            return "game_output_name=" .. tostring(Sound_GameSystem_GetOutputDriverNameByIndex(0))
        end

        local gameInputs = Sound_GameSystem_GetNumInputDrivers()
        if gameInputs ~= 1 then
            return "game_inputs=" .. tostring(gameInputs)
        end
        if Sound_GameSystem_GetInputDriverNameByIndex(0) ~= "Silent Input Device" then
            return "game_input_name=" .. tostring(Sound_GameSystem_GetInputDriverNameByIndex(0))
        end

        local chatOutputs = Sound_ChatSystem_GetNumOutputDrivers()
        if chatOutputs ~= 1 then
            return "chat_outputs=" .. tostring(chatOutputs)
        end
        if Sound_ChatSystem_GetOutputDriverNameByIndex(0) ~= "Silent Voice Output Device" then
            return "chat_output_name=" .. tostring(Sound_ChatSystem_GetOutputDriverNameByIndex(0))
        end

        local chatInputs = Sound_ChatSystem_GetNumInputDrivers()
        if chatInputs ~= 1 then
            return "chat_inputs=" .. tostring(chatInputs)
        end
        if Sound_ChatSystem_GetInputDriverNameByIndex(0) ~= "Silent Voice Input Device" then
            return "chat_input_name=" .. tostring(Sound_ChatSystem_GetInputDriverNameByIndex(0))
        end

        Sound_GameSystem_RestartSoundSystem()
        return "ok"
    "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "Audio driver globals should expose seeded input/output devices for settings UI: {result}"
    );
}

// ============================================================================
// Battle.net stubs
// ============================================================================

#[test]
fn test_bn_features_enabled() {
    let env = env();
    let val: bool = env.eval("return BNFeaturesEnabled()").unwrap();
    assert!(!val);
}

#[test]
fn test_bn_features_enabled_and_connected() {
    let env = env();
    let val: bool = env.eval("return BNFeaturesEnabledAndConnected()").unwrap();
    assert!(!val);
}

#[test]
fn test_bn_connected() {
    let env = env();
    let val: bool = env.eval("return BNConnected()").unwrap();
    assert!(val);
}

#[test]
fn test_bn_get_num_friends() {
    let env = env();
    let (total, online): (i32, i32) = env.eval("return BNGetNumFriends()").unwrap();
    assert_eq!(total, 0);
    assert_eq!(online, 0);
}

#[test]
fn test_bn_get_friend_info_nil() {
    let env = env();
    let is_nil: bool = env.eval("return BNGetFriendInfo(1) == nil").unwrap();
    assert!(is_nil);
}

#[test]
fn test_c_friend_list_returns_seeded_wow_friends() {
    let env = env();
    let result: String = env
        .eval(
            r#"
        if type(C_FriendList.ShowFriends) ~= "function" then
            return "missing_show_friends"
        end
        C_FriendList.ShowFriends()

        if C_FriendList.GetNumFriends() ~= 3 then
            return "friend_count=" .. tostring(C_FriendList.GetNumFriends())
        end
        if C_FriendList.GetNumOnlineFriends() ~= 2 then
            return "online_count=" .. tostring(C_FriendList.GetNumOnlineFriends())
        end
        if C_FriendList.GetNumIgnores() ~= 0 then
            return "ignore_count=" .. tostring(C_FriendList.GetNumIgnores())
        end
        if C_FriendList.GetIgnoreName(1) ~= nil then
            return "unexpected_ignore_name"
        end

        local info = C_FriendList.GetFriendInfoByIndex(1)
        if not info then
            return "missing_index_1"
        end
        if info.name ~= "Arthax" then
            return "name=" .. tostring(info.name)
        end
        if not info.connected then
            return "friend_should_be_online"
        end
        if info.level ~= 70 then
            return "level=" .. tostring(info.level)
        end
        if info.className ~= "Paladin" then
            return "class=" .. tostring(info.className)
        end
        if info.area ~= "Stormwind City" then
            return "area=" .. tostring(info.area)
        end
        if info.notes ~= "" then
            return "notes=" .. tostring(info.notes)
        end
        if info.guid ~= "Player-1-0000A001" then
            return "guid=" .. tostring(info.guid)
        end

        local by_name = C_FriendList.GetFriendInfoByName("Arthax")
        if not by_name or by_name.guid ~= info.guid then
            return "name_lookup_failed"
        end
        if not C_FriendList.IsFriend("Arthax") then
            return "is_friend_failed"
        end
        local second = C_FriendList.GetFriendInfoByIndex(2)
        if not second then
            return "missing_index_2"
        end
        if second.name ~= "Sylvara" then
            return "second_name=" .. tostring(second.name)
        end
        if not second.connected then
            return "second_friend_should_be_online"
        end
        local offline = C_FriendList.GetFriendInfoByIndex(3)
        if not offline then
            return "missing_index_3"
        end
        if offline.name ~= "Durotan" then
            return "offline_name=" .. tostring(offline.name)
        end
        if offline.connected then
            return "offline_friend_should_be_offline"
        end
        if C_FriendList.GetFriendInfoByIndex(99) ~= nil then
            return "unexpected_friend_at_99"
        end
        return "ok"
    "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "C_FriendList should expose seeded WoW friends: {result}"
    );

    let state = env.state().borrow();
    let has_friend_list_update = state
        .events
        .pending()
        .iter()
        .any(|event| event.name == "FRIENDLIST_UPDATE");
    assert!(
        has_friend_list_update,
        "C_FriendList.ShowFriends should queue FRIENDLIST_UPDATE"
    );
}

#[test]
fn test_c_loss_of_control_returns_seeded_active_event() {
    let env = env();
    let result: String = env
        .eval(
            r#"
        if C_LossOfControl.GetActiveLossOfControlDataCount() ~= 1 then
            return "global_count=" .. tostring(C_LossOfControl.GetActiveLossOfControlDataCount())
        end

        local data = C_LossOfControl.GetActiveLossOfControlData(1)
        if not data then
            return "missing_global_data"
        end
        if data.spellID ~= 408 then
            return "spell=" .. tostring(data.spellID)
        end
        if data.displayText ~= "Kidney Shot" then
            return "text=" .. tostring(data.displayText)
        end
        if data.iconTexture ~= "Interface\\Icons\\Ability_Rogue_KidneyShot" then
            return "icon=" .. tostring(data.iconTexture)
        end
        if data.displayType ~= 2 then
            return "display_type=" .. tostring(data.displayType)
        end
        if data.timeRemaining ~= 4 then
            return "time_remaining=" .. tostring(data.timeRemaining)
        end

        if C_LossOfControl.GetActiveLossOfControlData(2) ~= nil then
            return "unexpected_global_index_2"
        end

        if C_LossOfControl.GetActiveLossOfControlDataCountByUnit("player") ~= 1 then
            return "player_count=" .. tostring(C_LossOfControl.GetActiveLossOfControlDataCountByUnit("player"))
        end
        if C_LossOfControl.GetActiveLossOfControlDataByUnit("player", 1) == nil then
            return "missing_player_data"
        end
        if C_LossOfControl.GetActiveLossOfControlDuration("player", 1) ~= 4 then
            return "duration=" .. tostring(C_LossOfControl.GetActiveLossOfControlDuration("player", 1))
        end

        if C_LossOfControl.GetActiveLossOfControlDataCountByUnit("focus") ~= 0 then
            return "unexpected_focus_count"
        end
        if C_LossOfControl.GetActiveLossOfControlDataByUnit("focus", 1) ~= nil then
            return "unexpected_focus_data"
        end
        if C_LossOfControl.GetActiveLossOfControlDuration("focus", 1) ~= nil then
            return "unexpected_focus_duration"
        end

        return "ok"
    "#,
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "C_LossOfControl should expose seeded active loss-of-control data: {result}"
    );
}

#[test]
fn test_macro_apis_return_seeded_macros() {
    let env = env();
    let result: String = env
        .eval(
            r#"
        local accountCount, characterCount = GetNumMacros()
        if accountCount ~= 2 or characterCount ~= 1 then
            return "counts=" .. tostring(accountCount) .. "," .. tostring(characterCount)
        end

        local accountName, accountIcon, accountBody = GetMacroInfo(1)
        if accountName ~= "Raid Beacon" then
            return "account_name=" .. tostring(accountName)
        end
        if accountIcon ~= "Interface\\Icons\\INV_Misc_QuestionMark" then
            return "account_icon=" .. tostring(accountIcon)
        end
        if accountBody ~= "/rw Stack on star" then
            return "account_body=" .. tostring(accountBody)
        end

        local characterName, characterIcon, characterBody = GetMacroInfo(121)
        if characterName ~= "Crusader" then
            return "character_name=" .. tostring(characterName)
        end
        if characterIcon ~= "Interface\\Icons\\Spell_Holy_CrusaderAura" then
            return "character_icon=" .. tostring(characterIcon)
        end
        if characterBody ~= "/cast Crusader Aura" then
            return "character_body=" .. tostring(characterBody)
        end

        if GetMacroInfo(999) ~= nil then
            return "unexpected_macro_999"
        end
        if C_Macro.GetMacroName(1) ~= "Raid Beacon" then
            return "c_macro_name=" .. tostring(C_Macro.GetMacroName(1))
        end
        if C_Macro.GetSelectedMacroIcon(121) ~= "Interface\\Icons\\Spell_Holy_CrusaderAura" then
            return "c_macro_icon=" .. tostring(C_Macro.GetSelectedMacroIcon(121))
        end
        local cAccountCount, cCharacterCount = C_Macro.GetNumMacros()
        if cAccountCount ~= 2 or cCharacterCount ~= 1 then
            return "c_counts=" .. tostring(cAccountCount) .. "," .. tostring(cCharacterCount)
        end
        return "ok"
    "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "Macro APIs should expose seeded macros: {result}"
    );
}

#[test]
fn test_macro_icon_apis_populate_icon_tables() {
    let env = env();
    let result: String = env
        .eval(
            r#"
        local spellIcons = {}
        if GetLooseMacroIcons(spellIcons) ~= nil then
            return "loose_spell_returned_value"
        end
        if spellIcons[1] ~= "INV_Misc_QuestionMark" then
            return "loose_spell_first=" .. tostring(spellIcons[1])
        end

        local returnedSpellIcons = GetMacroIcons(spellIcons)
        if returnedSpellIcons ~= spellIcons then
            return "spell_table_not_reused"
        end
        if spellIcons[2] ~= "Spell_Holy_CrusaderAura" then
            return "macro_spell_second=" .. tostring(spellIcons[2])
        end

        local itemIcons = {}
        if GetLooseMacroItemIcons(itemIcons) ~= nil then
            return "loose_item_returned_value"
        end
        if itemIcons[1] ~= "INV_Misc_Bag_08" then
            return "loose_item_first=" .. tostring(itemIcons[1])
        end

        local returnedItemIcons = GetMacroItemIcons(itemIcons)
        if returnedItemIcons ~= itemIcons then
            return "item_table_not_reused"
        end
        if itemIcons[2] ~= "INV_Sword_04" then
            return "macro_item_second=" .. tostring(itemIcons[2])
        end
        return "ok"
    "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "Macro icon APIs should populate caller-provided icon tables: {result}"
    );
}

// ============================================================================
// PlayerLocation
// ============================================================================

#[test]
fn test_player_location_guid_uses_guid_validity_and_clear_invalidates() {
    let env = env();
    let (is_guid, is_unit, guid_matches, valid_before, valid_after): (
        bool,
        bool,
        bool,
        bool,
        bool,
    ) = env
        .eval(
            r#"
        C_PlayerInfo.GUIDIsPlayer = function(guid)
            return guid == "Player-3676-00000001"
        end
        C_AccountInfo.IsGUIDBattleNetAccountType = function()
            return false
        end

        local location = PlayerLocation:CreateFromGUID("Player-3676-00000001")
        local valid_before = location:IsValid()
        local guid_matches = location:GetGUID() == "Player-3676-00000001"
        location:Clear()

        return location:IsGUID(), location:IsUnit(), guid_matches, valid_before, location:IsValid()
    "#,
        )
        .unwrap();
    assert!(!is_guid, "Clear should remove the GUID source kind");
    assert!(
        !is_unit,
        "GUID locations should not report as unit locations"
    );
    assert!(
        guid_matches,
        "GUID locations should preserve the original GUID"
    );
    assert!(
        valid_before,
        "GUID validity should come from the GUID resolver"
    );
    assert!(!valid_after, "Cleared locations should become invalid");
}

#[test]
fn test_player_location_unit_uses_unit_validity() {
    let env = env();
    let (player_valid, pet_valid, player_is_unit, pet_is_unit): (bool, bool, bool, bool) = env
        .eval(
            r#"
        UnitIsHumanPlayer = function(unit)
            return unit == "player"
        end

        local player_location = PlayerLocation:CreateFromUnit("player")
        local pet_location = PlayerLocation:CreateFromUnit("pet")
        return player_location:IsValid(), pet_location:IsValid(), player_location:IsUnit(), pet_location:IsUnit()
    "#,
        )
        .unwrap();
    assert!(
        player_valid,
        "player unit should be valid when UnitIsHumanPlayer says so"
    );
    assert!(!pet_valid, "non-human unit locations should be invalid");
    assert!(player_is_unit);
    assert!(pet_is_unit);
}

#[test]
fn test_player_location_preserves_non_guid_source_kinds() {
    let env = env();
    let (community_valid, bnet_valid, community_kind, bnet_kind, clear_invalid): (
        bool,
        bool,
        bool,
        bool,
        bool,
    ) = env
        .eval(
            r#"
        C_Club.CanResolvePlayerLocationFromClubMessageData = function(clubID, streamID, epoch, position)
            return clubID == 7 and streamID == 11 and epoch == 13 and position == 17
        end

        local community = PlayerLocation:CreateFromCommunityChatData(7, 11, 13, 17)
        local bnet = PlayerLocation:CreateFromBattleNetID(42)
        local cleared = PlayerLocation:CreateFromVoiceID(3, 9)
        cleared:Clear()

        return community:IsValid(), bnet:IsValid(), community:IsCommunityData(), bnet:IsBattleNetID(), cleared:IsValid()
    "#,
        )
        .unwrap();
    assert!(
        community_valid,
        "community chat locations should use the resolver result"
    );
    assert!(
        bnet_valid,
        "battle.net locations should remain valid while their source kind is set"
    );
    assert!(
        community_kind,
        "community locations should preserve their source kind"
    );
    assert!(
        bnet_kind,
        "battle.net locations should preserve their source kind"
    );
    assert!(
        !clear_invalid,
        "cleared voice locations should become invalid"
    );
}
