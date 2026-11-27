use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn active_continent_name_uses_host_supplied_event_state() {
    let env = env();
    let result: (String, String, bool) = env
        .eval(
            r#"
            A_Admin.SetActiveEventContinentName("Kalimdor")
            local first = C_EventScheduler.GetActiveContinentName()
            A_Admin.SetActiveEventContinentName("Eastern Kingdoms")
            local second = C_EventScheduler.GetActiveContinentName()
            A_Admin.SetActiveEventContinentName(nil)
            return first, second, C_EventScheduler.GetActiveContinentName() == nil
            "#,
        )
        .expect("active continent query should return supplied state");

    assert_eq!(result, ("Kalimdor".into(), "Eastern Kingdoms".into(), true));
}

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn event_scheduler_defaults_to_hidden_without_events() {
    let env = env();
    let can_show: bool = env
        .eval(
            r#"
            C_EventScheduler._state.canShowEvents = nil
            C_EventScheduler._state.suppressDisplay = false
            C_EventScheduler._state.ongoingEvents = {}
            C_EventScheduler._state.scheduledEvents = {}
            return C_EventScheduler.CanShowEvents()
            "#,
        )
        .unwrap();

    assert!(
        !can_show,
        "scheduler should be hidden when no events are configured"
    );
}

#[test]
fn event_scheduler_starts_with_seeded_events_and_default_visibility() {
    let env = env();
    let (
        can_show,
        ongoing_count,
        scheduled_count,
        first_ongoing_poi,
        first_scheduled_key,
        first_start_time_is_future,
        second_has_reminder,
    ): (bool, i32, i32, i32, String, bool, bool) = env
        .eval(
            r#"
            local canShow = C_EventScheduler.CanShowEvents()
            local ongoing = C_EventScheduler._state.ongoingEvents
            local scheduled = C_EventScheduler._state.scheduledEvents
            return canShow,
                #ongoing,
                #scheduled,
                ongoing[1].areaPoiID,
                scheduled[1].eventKey,
                scheduled[1].startTime > time(),
                scheduled[2].hasReminder
            "#,
        )
        .unwrap();

    assert!(
        can_show,
        "seeded scheduler events should be visible by default"
    );
    assert_eq!(ongoing_count, 2, "expected two seeded ongoing events");
    assert_eq!(scheduled_count, 2, "expected two seeded scheduled events");
    assert_eq!(first_ongoing_poi, 1001);
    assert_eq!(first_scheduled_key, "pvp-brawl-blitz");
    assert!(
        first_start_time_is_future,
        "seeded scheduled events should be upcoming"
    );
    assert!(
        second_has_reminder,
        "seeded scheduled events should include at least one reminder example"
    );
}

#[test]
fn event_scheduler_override_flag_controls_visibility() {
    let env = env();
    let (override_true, override_false): (bool, bool) = env
        .eval(
            r#"
            C_EventScheduler._state.canShowEvents = true
            local whenTrue = C_EventScheduler.CanShowEvents()

            C_EventScheduler._state.canShowEvents = false
            local whenFalse = C_EventScheduler.CanShowEvents()

            return whenTrue, whenFalse
            "#,
        )
        .unwrap();

    assert!(
        override_true,
        "explicit true override should force visibility"
    );
    assert!(
        !override_false,
        "explicit false override should force hidden state"
    );
}

#[test]
fn event_scheduler_derives_visibility_from_event_lists() {
    let env = env();
    let (ongoing_visible, scheduled_visible, suppressed_hidden): (bool, bool, bool) = env
        .eval(
            r#"
            C_EventScheduler._state.canShowEvents = nil
            C_EventScheduler._state.suppressDisplay = false

            C_EventScheduler._state.ongoingEvents = {
                { areaPoiID = 100, rewardsClaimed = false, displayInfo = {} },
            }
            C_EventScheduler._state.scheduledEvents = {}
            local ongoingVisible = C_EventScheduler.CanShowEvents()

            C_EventScheduler._state.ongoingEvents = {}
            C_EventScheduler._state.scheduledEvents = {
                {
                    eventKey = "foo",
                    eventID = 1,
                    areaPoiID = 200,
                    startTime = 10,
                    ["endTime"] = 20,
                    duration = 10,
                    hasReminder = false,
                    rewardsClaimed = false,
                    displayInfo = {},
                },
            }
            local scheduledVisible = C_EventScheduler.CanShowEvents()

            C_EventScheduler._state.suppressDisplay = true
            local suppressedHidden = not C_EventScheduler.CanShowEvents()

            return ongoingVisible, scheduledVisible, suppressedHidden
            "#,
        )
        .unwrap();

    assert!(
        ongoing_visible,
        "ongoing event list should make scheduler visible"
    );
    assert!(
        scheduled_visible,
        "scheduled event list should make scheduler visible"
    );
    assert!(
        suppressed_hidden,
        "suppress flag should hide scheduler when no explicit override is set"
    );
}

#[test]
fn event_scheduler_request_events_repopulates_seeded_state() {
    let env = env();
    let (
        ongoing_count,
        scheduled_count,
        public_ongoing_count,
        first_public_poi,
        public_scheduled_count,
        first_public_event_key,
        has_data,
        can_show,
    ): (i32, i32, i32, i32, i32, String, bool, bool) = env
        .eval(
            r#"
            C_EventScheduler._state.canShowEvents = nil
            C_EventScheduler._state.suppressDisplay = false
            C_EventScheduler._state.ongoingEvents = {}
            C_EventScheduler._state.scheduledEvents = {}

            C_EventScheduler.RequestEvents()

            return #C_EventScheduler._state.ongoingEvents,
                #C_EventScheduler._state.scheduledEvents,
                #C_EventScheduler.GetOngoingEvents(),
                C_EventScheduler.GetOngoingEvents()[1].areaPoiID,
                #C_EventScheduler.GetScheduledEvents(),
                C_EventScheduler.GetScheduledEvents()[1].eventKey,
                C_EventScheduler.HasData(),
                C_EventScheduler.CanShowEvents()
            "#,
        )
        .unwrap();

    assert_eq!(
        ongoing_count, 2,
        "RequestEvents() should repopulate the seeded ongoing event list"
    );
    assert_eq!(
        scheduled_count, 2,
        "RequestEvents() should repopulate the seeded scheduled event list"
    );
    assert_eq!(
        public_ongoing_count, 2,
        "GetOngoingEvents() should expose the repopulated seeded ongoing event list"
    );
    assert_eq!(
        first_public_poi, 1001,
        "GetOngoingEvents() should return the seeded ongoing event records"
    );
    assert_eq!(
        public_scheduled_count, 2,
        "GetScheduledEvents() should expose the repopulated seeded scheduled event list"
    );
    assert_eq!(
        first_public_event_key, "pvp-brawl-blitz",
        "GetScheduledEvents() should return the seeded scheduled event records"
    );
    assert!(
        has_data,
        "HasData() should report true after RequestEvents() restores scheduler state"
    );
    assert!(
        can_show,
        "RequestEvents() should restore visible scheduler data"
    );
}

#[test]
fn event_scheduler_event_zone_names_follow_seeded_area_poi_ids() {
    let env = env();
    let (ongoing_zone, scheduled_zone, missing_zone): (String, String, String) = env
        .eval(
            r#"
            return C_EventScheduler.GetEventZoneName(1001),
                C_EventScheduler.GetEventZoneName(1004),
                C_EventScheduler.GetEventZoneName(999999)
            "#,
        )
        .unwrap();

    assert_eq!(
        ongoing_zone, "Warsong Gulch",
        "GetEventZoneName() should resolve seeded ongoing event POIs"
    );
    assert_eq!(
        scheduled_zone, "Darkmoon Island",
        "GetEventZoneName() should resolve seeded scheduled event POIs"
    );
    assert_eq!(
        missing_zone, "",
        "GetEventZoneName() should keep the empty-string fallback for unknown POIs"
    );
}

#[test]
fn event_scheduler_event_ui_map_ids_follow_seeded_area_poi_ids() {
    let env = env();
    let (ongoing_map, scheduled_map, missing_map): (i32, i32, i32) = env
        .eval(
            r#"
            return C_EventScheduler.GetEventUiMapID(1001),
                C_EventScheduler.GetEventUiMapID(1004),
                C_EventScheduler.GetEventUiMapID(999999)
            "#,
        )
        .unwrap();

    assert_eq!(
        ongoing_map, 8685,
        "GetEventUiMapID() should resolve seeded ongoing event POIs"
    );
    assert_eq!(
        scheduled_map, 5861,
        "GetEventUiMapID() should resolve seeded scheduled event POIs"
    );
    assert_eq!(
        missing_map, 0,
        "GetEventUiMapID() should keep the zero fallback for unknown POIs"
    );
}

#[test]
fn area_poi_info_returns_seeded_event_location_data() {
    let env = env();
    let (name, description, atlas_name, x, y, missing_info): (
        String,
        String,
        String,
        f32,
        f32,
        bool,
    ) = env
        .eval(
            r#"
            local info = C_AreaPoiInfo.GetAreaPOIInfo(8685, 1001)
            local x, y = info.position:GetXY()
            return info.name,
                info.description,
                info.atlasName,
                x,
                y,
                C_AreaPoiInfo.GetAreaPOIInfo(8685, 999999) == nil
            "#,
        )
        .unwrap();

    assert_eq!(name, "Warsong Gulch");
    assert_eq!(
        description, "Compete in the current PvP brawl.",
        "GetAreaPOIInfo() should return the seeded event description"
    );
    assert_eq!(
        atlas_name, "worldquest-icon-pvpbattle",
        "GetAreaPOIInfo() should return the seeded event atlas"
    );
    assert_eq!(
        x, 0.452,
        "GetAreaPOIInfo() should return the seeded X position"
    );
    assert_eq!(
        y, 0.641,
        "GetAreaPOIInfo() should return the seeded Y position"
    );
    assert!(
        missing_info,
        "GetAreaPOIInfo() should keep returning nil for unknown POIs"
    );
}

#[test]
fn area_poi_for_map_returns_seeded_event_poi_ids() {
    let env = env();
    let (battleground_count, battleground_first, island_count, island_first, missing_count): (
        i32,
        i32,
        i32,
        i32,
        i32,
    ) = env
        .eval(
            r#"
            local battleground = C_AreaPoiInfo.GetAreaPOIForMap(8685)
            local island = C_AreaPoiInfo.GetAreaPOIForMap(5861)
            local missing = C_AreaPoiInfo.GetAreaPOIForMap(999999)
            return #battleground,
                battleground[1],
                #island,
                island[1],
                #missing
            "#,
        )
        .unwrap();

    assert_eq!(
        battleground_count, 1,
        "GetAreaPOIForMap() should list the seeded POIs for Warsong Gulch"
    );
    assert_eq!(
        battleground_first, 1001,
        "GetAreaPOIForMap() should include the seeded Warsong Gulch event POI"
    );
    assert_eq!(
        island_count, 1,
        "GetAreaPOIForMap() should list the seeded POIs for Darkmoon Island"
    );
    assert_eq!(
        island_first, 1004,
        "GetAreaPOIForMap() should include the seeded Darkmoon Island event POI"
    );
    assert_eq!(
        missing_count, 0,
        "GetAreaPOIForMap() should keep returning an empty table for unknown maps"
    );
}

#[test]
fn seeded_event_area_poi_data_includes_cinderbrew_meadery_map() {
    let env = env();
    let (map_id, name, listed_poi_id): (i32, String, i32) = env
        .eval(
            r#"
            local mapID = C_EventScheduler.GetEventUiMapID(1002)
            local info = C_AreaPoiInfo.GetAreaPOIInfo(mapID, 1002)
            local poiList = C_AreaPoiInfo.GetAreaPOIForMap(mapID)
            return mapID, info.name, poiList[1]
            "#,
        )
        .unwrap();

    assert_eq!(
        map_id, 1980,
        "The seeded Cinderbrew event location should use the real dungeon map ID"
    );
    assert_eq!(
        name, "The Cinderbrew Meadery",
        "The Cinderbrew event location should have seeded area POI info"
    );
    assert_eq!(
        listed_poi_id, 1002,
        "The Cinderbrew event location should appear in the seeded POI list for its map"
    );
}
