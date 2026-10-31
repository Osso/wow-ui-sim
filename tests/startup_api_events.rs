//! Smoke tests for startup-surface stubs added to unblock Blizzard addon loading.

#[path = "startup_api_stubs/common.rs"]
mod startup_api_common;

use startup_api_common::*;

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_source_events_register_and_dispatch() {
    let env = env();
    for event in [
        "GUILD_PREFERRED_PLAY_SETTINGS_UPDATED",
        "HIDDEN_GROUP_BUFFS_CHANGED",
        "PET_STATS_UPDATE",
        "SHARD_TRANSFER",
        "SHARD_TRANSFER_IMMINENT",
    ] {
        env.exec(&format!(
            "received = nil; receiver = CreateFrame('Frame'); \
             receiver:RegisterEvent('{event}'); \
             receiver:SetScript('OnEvent', function(_, name, value) \
             received = name .. ':' .. value end)"
        ))
        .unwrap();
        env.fire_event_with_args(event, &[env.lua_string("delivered")])
            .unwrap();
        assert_eq!(
            env.eval::<String>("return received").unwrap(),
            format!("{event}:delivered")
        );
        env.exec("receiver:UnregisterAllEvents()").unwrap();
    }
}

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_source_events_reject_unknown_and_empty_names() {
    let env = env();
    env.exec(
        "local frame = CreateFrame('Frame'); \
         assert(not pcall(frame.RegisterEvent, frame, 'WOWFOREVER_INVENTED_EVENT')); \
         assert(not pcall(frame.RegisterEvent, frame, ''))",
    )
    .unwrap();
}

#[test]
fn map_util_helpers_exist_in_shared_bootstrap() {
    let env = env();
    let (displayable_map_id_type, map_type_zone_callable, parent_info_callable, cache_match): (
        String,
        bool,
        bool,
        bool,
    ) = env
        .eval(
            r#"
            local displayableMapID = MapUtil.GetDisplayableMapForPlayer()
            return type(displayableMapID),
                   pcall(function() return MapUtil.IsMapTypeZone(1) end),
                   pcall(function() return MapUtil.GetMapParentInfo(1, Enum.UIMapType.Zone) end),
                   MapUtil.IsChildMapCached(1, 1) == MapUtil.IsChildMap(1, 1)
            "#,
        )
        .expect("MapUtil fallback helpers should be callable");
    assert_eq!(displayable_map_id_type, "number");
    assert!(map_type_zone_callable);
    assert!(parent_info_callable);
    assert!(cache_match);
}

#[test]
fn get_icon_for_role_enum_returns_expected_role_atlases() {
    let env = env();
    let (tank, healer_disabled, damage): (String, String, String) = env
        .eval(
            r#"
            return GetIconForRoleEnum(Enum.LFGRole.Tank, false),
                   GetIconForRoleEnum(Enum.LFGRole.Healer, true),
                   GetIconForRoleEnum(Enum.LFGRole.Damage, false)
            "#,
        )
        .expect("role icon helper should be callable");
    assert_eq!(tank, "UI-LFG-RoleIcon-Tank");
    assert_eq!(healer_disabled, "UI-LFG-RoleIcon-Healer-Disabled");
    assert_eq!(damage, "UI-LFG-RoleIcon-DPS");
}

#[test]
fn event_util_helpers_defer_until_matching_startup_events_fire() {
    let env = env();
    env.exec(
        r#"
        EventUtilCalls = {
            variablesLoaded = 0,
            allEvents = 0,
            lateVariablesLoaded = 0,
        }

        EventUtil.ContinueOnVariablesLoaded(function()
            EventUtilCalls.variablesLoaded = EventUtilCalls.variablesLoaded + 1
        end)

        EventUtil.ContinueAfterAllEvents(function()
            EventUtilCalls.allEvents = EventUtilCalls.allEvents + 1
        end, "VARIABLES_LOADED", "PLAYER_ENTERING_WORLD", "FIRST_FRAME_RENDERED")
        "#,
    )
    .expect("EventUtil helpers should register callbacks");

    let (before_variables_loaded, before_all_events): (i32, i32) = env
        .eval("return EventUtilCalls.variablesLoaded, EventUtilCalls.allEvents")
        .expect("EventUtil callback counts should be readable");
    assert_eq!(before_variables_loaded, 0);
    assert_eq!(before_all_events, 0);

    env.fire_event("VARIABLES_LOADED")
        .expect("VARIABLES_LOADED should dispatch");
    let (after_variables_loaded, after_partial_events): (i32, i32) = env
        .eval("return EventUtilCalls.variablesLoaded, EventUtilCalls.allEvents")
        .expect("VARIABLES_LOADED should update EventUtil callback state");
    assert_eq!(after_variables_loaded, 1);
    assert_eq!(after_partial_events, 0);

    env.exec(
        r#"
        EventUtil.ContinueOnVariablesLoaded(function()
            EventUtilCalls.lateVariablesLoaded = EventUtilCalls.lateVariablesLoaded + 1
        end)
        "#,
    )
    .expect("ContinueOnVariablesLoaded should run immediately after VARIABLES_LOADED");
    let late_variables_loaded: i32 = env
        .eval("return EventUtilCalls.lateVariablesLoaded")
        .expect("late VARIABLES_LOADED callback count should be readable");
    assert_eq!(late_variables_loaded, 1);

    env.fire_event_with_args(
        "PLAYER_ENTERING_WORLD",
        &[rilua::Val::Bool(true), rilua::Val::Bool(false)],
    )
    .expect("PLAYER_ENTERING_WORLD should dispatch");
    let after_player_entering_world: i32 = env
        .eval("return EventUtilCalls.allEvents")
        .expect("EventUtil all-events count should stay readable");
    assert_eq!(after_player_entering_world, 0);

    env.fire_event("FIRST_FRAME_RENDERED")
        .expect("FIRST_FRAME_RENDERED should dispatch");
    let after_first_frame_rendered: i32 = env
        .eval("return EventUtilCalls.allEvents")
        .expect("EventUtil all-events callback should fire after the last event");
    assert_eq!(after_first_frame_rendered, 1);
}

#[test]
fn event_util_register_once_can_capture_zero_or_more_required_args() {
    let env = env();
    let (ok, handle_type, registered, unregister_type): (bool, String, bool, String) = env
        .eval(
            r#"
            local originalCreateFrame = CreateFrame
            CreateFrame = nil

            local ok, handle = pcall(function()
                return EventUtil.RegisterOnceFrameEventAndCallback(
                    "ADDON_LOADED",
                    function() end,
                    "Blizzard_PlayerSpells"
                )
            end)

            CreateFrame = originalCreateFrame

            return ok,
                   type(handle),
                   ok and handle.registered == true or false,
                   ok and type(handle.Unregister) or "nil"
            "#,
        )
        .expect("EventUtil.RegisterOnceFrameEventAndCallback should be callable");

    assert!(ok, "register-once helper should not fail when packing args");
    assert_eq!(handle_type, "table");
    assert!(registered);
    assert_eq!(unregister_type, "function");
}

#[test]
fn contribution_collector_namespace_exists_with_load_safe_defaults() {
    let env = env();
    let (namespace_type, close_type, state, percent, appearance_type, color_type): (
        String,
        String,
        i32,
        i32,
        String,
        String,
    ) = env
        .eval(
            r#"
            local state, percent = C_ContributionCollector.GetState(42)
            local appearance = C_ContributionCollector.GetContributionAppearance(42, state)
            return type(C_ContributionCollector), type(C_ContributionCollector.Close), state, percent, type(appearance), type(appearance.stateColor)
            "#,
        )
        .expect("ContributionCollector startup stub should be callable");

    assert_eq!(namespace_type, "table");
    assert_eq!(close_type, "function");
    assert_eq!(state, 0);
    assert_eq!(percent, 0);
    assert_eq!(appearance_type, "table");
    assert_eq!(color_type, "table");
}

#[test]
fn region_helpers_return_us_region_defaults() {
    let env = env();
    let (region, region_name): (i32, String) = env
        .eval("return GetCurrentRegion(), GetCurrentRegionName()")
        .expect("region helpers should be callable");

    assert_eq!(region, 1);
    assert_eq!(region_name, "US");
}

// Registration/dispatch only: these tests do not exercise native mouse or voice producers.
#[test]
#[cfg(feature = "client-wowforever")]
fn forever_known_startup_global_region_mouse_down_dispatches_documented_payload() {
    let env = env();
    env.exec(
        r#"
        region = CreateFrame("Frame")
        listener = CreateFrame("Frame")
        mouseCalls = 0
        assert(listener:RegisterEvent("GLOBAL_REGION_MOUSE_DOWN"))
        assert(listener:IsEventRegistered("GLOBAL_REGION_MOUSE_DOWN"))
        listener:SetScript("OnEvent", function(self, event, ...)
            mouseCalls = mouseCalls + 1
            mouseSelf = self == listener
            mouseEvent = event
            mouseArgCount = select('#', ...)
            local receivedRegion, button = ...
            mouseRegionMatches = rawequal(receivedRegion, region)
            mouseButton = button
        end)
        "#,
    )
    .unwrap();
    let region = env.eval::<rilua::Val>("return region").unwrap();
    let args = [region, env.lua_string("LeftButton")];
    env.fire_event_with_args("GLOBAL_REGION_MOUSE_DOWN", &args)
        .unwrap();
    // Assert outside protected OnEvent dispatch so callback failures cannot pass silently.
    env.exec(
        r#"
        assert(mouseCalls == 1)
        assert(mouseSelf and mouseRegionMatches)
        assert(mouseEvent == "GLOBAL_REGION_MOUSE_DOWN")
        assert(mouseArgCount == 2 and mouseButton == "LeftButton")
        listener:UnregisterEvent("GLOBAL_REGION_MOUSE_DOWN")
        assert(not listener:IsEventRegistered("GLOBAL_REGION_MOUSE_DOWN"))
        "#,
    )
    .unwrap();
    env.fire_event_with_args("GLOBAL_REGION_MOUSE_DOWN", &args)
        .unwrap();
    assert_eq!(env.eval::<i32>("return mouseCalls").unwrap(), 1);
}

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_known_startup_active_voice_provider_dispatches_documented_payload() {
    let env = env();
    env.exec(
        r#"
        listener = CreateFrame("Frame")
        providerEvents = {}
        assert(listener:RegisterEvent("VOICE_CHAT_ACTIVE_VOICE_PROVIDER_CHANGED"))
        assert(listener:IsEventRegistered("VOICE_CHAT_ACTIVE_VOICE_PROVIDER_CHANGED"))
        listener:SetScript("OnEvent", function(self, event, ...)
            table.insert(providerEvents, {
                sameSelf = self == listener,
                event = event,
                count = select('#', ...),
                providerID = ...,
            })
        end)
        "#,
    )
    .unwrap();
    // Cached VoiceChatDocumentation defines Legacy=0 and Discord=1.
    for provider_id in [0.0, 1.0] {
        env.fire_event_with_args(
            "VOICE_CHAT_ACTIVE_VOICE_PROVIDER_CHANGED",
            &[rilua::Val::Num(provider_id)],
        )
        .unwrap();
    }
    env.exec(
        r#"
        assert(#providerEvents == 2)
        for index, received in ipairs(providerEvents) do
            assert(received.sameSelf)
            assert(received.event == "VOICE_CHAT_ACTIVE_VOICE_PROVIDER_CHANGED")
            assert(received.count == 1 and received.providerID == index - 1)
        end
        listener:UnregisterEvent("VOICE_CHAT_ACTIVE_VOICE_PROVIDER_CHANGED")
        assert(not listener:IsEventRegistered("VOICE_CHAT_ACTIVE_VOICE_PROVIDER_CHANGED"))
        "#,
    )
    .unwrap();
    env.fire_event_with_args(
        "VOICE_CHAT_ACTIVE_VOICE_PROVIDER_CHANGED",
        &[rilua::Val::Num(0.0)],
    )
    .unwrap();
    assert_eq!(env.eval::<i32>("return #providerEvents").unwrap(), 2);
}

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_known_startup_names_register_query_and_unregister() {
    let env = env();
    env.exec(
        r#"
        local frame = CreateFrame("Frame")
        for _, event in ipairs({
            "ALERT_AGE_VERIFICATION_RESTRICTED",
            "GLOBAL_REGION_MOUSE_DOWN",
            "PLAYER_PVP_FLAG_CHANGED",
            "VOICE_CHAT_ACTIVE_VOICE_PROVIDER_CHANGED",
            "VOICE_CHAT_VOICE_PROVIDER_CHANGED",
            -- 70532 VoiceChatDocumentation.lua:1229, :1408; Audio.lua registers both.
            "VOICE_CHAT_DISCORD_SETTINGS_UPDATED",
            "VOICE_CHAT_VOICE_PROVIDERS_AVAILABLE_CHANGED",
        }) do
            assert(not frame:IsEventRegistered(event), event)
            assert(frame:RegisterEvent(event), event)
            assert(frame:IsEventRegistered(event), event)
            assert(frame:UnregisterEvent(event), event)
            assert(not frame:IsEventRegistered(event), event)
        end
        "#,
    )
    .unwrap();
}

#[test]
#[cfg(not(feature = "client-wowforever"))]
fn forever_known_startup_names_preserve_other_profile_registration_policy() {
    let env = env();
    let permissive_classic = cfg!(any(
        feature = "client-wrath",
        feature = "client-mists",
        feature = "client-era",
        feature = "client-anniversary"
    ));
    // Retail 12.1.5 adds the age-verification event; the other four remain Forever-only.
    for event in [
        "ALERT_AGE_VERIFICATION_RESTRICTED",
        "GLOBAL_REGION_MOUSE_DOWN",
        "PLAYER_PVP_FLAG_CHANGED",
        "VOICE_CHAT_ACTIVE_VOICE_PROVIDER_CHANGED",
        "VOICE_CHAT_VOICE_PROVIDER_CHANGED",
    ] {
        let (accepted, registered, after_unregister): (bool, bool, bool) = env
            .eval(&format!(
                r#"
                local frame = CreateFrame("Frame")
                local accepted = pcall(frame.RegisterEvent, frame, "{event}")
                local registered = frame:IsEventRegistered("{event}")
                frame:UnregisterEvent("{event}")
                return accepted, registered, frame:IsEventRegistered("{event}")
                "#
            ))
            .unwrap();
        let retail_age_verification =
            cfg!(feature = "retail-12-1-5") && event == "ALERT_AGE_VERIFICATION_RESTRICTED";
        let expected = permissive_classic || retail_age_verification;
        assert_eq!(accepted, expected, "{event}: registration policy");
        assert_eq!(registered, expected, "{event}: listener state");
        assert!(
            !after_unregister,
            "{event}: listener state after unregister"
        );
    }
}

#[test]
#[cfg(any(feature = "client-wowforever", feature = "retail-12-0-0"))]
fn forever_known_startup_catalog_preserves_strict_unknown_name_rejection() {
    let env = env();
    env.exec(
        r#"
        local frame = CreateFrame("Frame")
        for _, event in ipairs({
            "ALERT_AGE_VERIFICATION_RESTRICTED_INVENTED",
            "GLOBAL_REGION_MOUSE_DOWN_INVENTED",
            "PLAYER_PVP_FLAG_CHANGED_INVENTED",
            "VOICE_CHAT_ACTIVE_VOICE_PROVIDER_CHANGED_INVENTED",
            "VOICE_CHAT_VOICE_PROVIDER_CHANGED_INVENTED",
            "VOICE_CHAT_VOICE_PROVIDERS_AVAILABLE_CHANGED_INVENTED",
            "WOWFOREVER_INVENTED_EVENT",
            "",
        }) do
            local ok, err = pcall(frame.RegisterEvent, frame, event)
            assert(not ok and string.find(err, "Attempt to register unknown event", 1, true))
            assert(not frame:IsEventRegistered(event))
        end
        "#,
    )
    .unwrap();
}
