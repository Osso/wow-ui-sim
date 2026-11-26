use crate::common;

use super::support::{drain_test_errors, frame_is_shown, install_test_error_handler, setup_env};

#[test]
fn keybind_m_opens_world_map() {
    test_timeout! {
        let env = setup_env();
        env.send_key_press("M", None).expect("M keybind failed");
        assert!(
            frame_is_shown(&env, "WorldMapFrame"),
            "WorldMapFrame should be shown after pressing M"
        );
    }
}

#[test]
fn world_map_floor_dropdown_hidden_without_subzone_or_map_group() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);
        env.state().borrow_mut().world.sub_zone_name.clear();

        env.send_key_press("M", None).expect("M keybind failed");

        let result: String = env
            .eval(
                r#"
                if not (WorldMapFrame and WorldMapFrame:IsShown()) then
                    return "world_map_not_open"
                end

                local floorDropdown
                for _, frame in ipairs(WorldMapFrame.overlayFrames or {}) do
                    if type(frame.RefreshMenu) == "function" then
                        floorDropdown = frame
                        break
                    end
                end

                if not floorDropdown then
                    return "missing_floor_dropdown"
                end

                local mapID = WorldMapFrame:GetMapID()
                local groupID = C_Map.GetMapGroupID(mapID)
                local members = C_Map.GetMapGroupMembersInfo(groupID)
                local memberCount = 0
                if type(members) == "table" then
                    for _ in ipairs(members) do
                        memberCount = memberCount + 1
                    end
                end

                if floorDropdown:IsShown() then
                    return string.format(
                        "shown:subzone=%s:groupID=%s:groupType=%s:membersType=%s:members=%d",
                        tostring(GetSubZoneText()),
                        tostring(groupID),
                        type(groupID),
                        type(members),
                        memberCount
                    )
                end

                return "ok"
            "#,
            )
            .unwrap();

        let errors = drain_test_errors(&env);
        assert!(
            errors.is_empty(),
            "Opening world map with no subzone produced {} Lua error(s):\n{}",
            errors.len(),
            errors.join("\n"),
        );
        assert_eq!(
            result,
            "ok",
            "World map floor dropdown should stay hidden when there is no subzone or map group: {result}"
        );
    }
}

#[test]
fn keybind_m_toggles_world_map_without_errors() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);

        env.send_key_press("M", None).expect("first M keybind failed");

        let open_errors = drain_test_errors(&env);
        assert!(
            open_errors.is_empty(),
            "Opening world map produced {} Lua error(s):\n{}",
            open_errors.len(),
            open_errors.join("\n"),
        );
        assert!(
            frame_is_shown(&env, "WorldMapFrame"),
            "WorldMapFrame should be shown after first M press"
        );

        env.send_key_press("M", None).expect("second M keybind failed");

        let close_errors = drain_test_errors(&env);
        assert!(
            close_errors.is_empty(),
            "Closing world map produced {} Lua error(s):\n{}",
            close_errors.len(),
            close_errors.join("\n"),
        );
        assert!(
            !frame_is_shown(&env, "WorldMapFrame"),
            "WorldMapFrame should be hidden after second M press"
        );
    }
}

#[test]
fn world_map_title_text_is_non_empty_after_opening() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);

        env.send_key_press("M", None).expect("M keybind failed");

        let result: String = env
            .eval(
                r#"
                if not (WorldMapFrame and WorldMapFrame:IsShown()) then
                    return "world_map_not_open"
                end

                local legacyTitle = WorldMapFrame.mapTitle
                if legacyTitle then
                    local legacyText = legacyTitle:GetText()
                    if type(legacyText) ~= "string" or legacyText == "" then
                        return "empty_legacy_world_map_title"
                    end
                    return "ok"
                end

                local titleText = WorldMapFrame.BorderFrame
                    and WorldMapFrame.BorderFrame.TitleContainer
                    and WorldMapFrame.BorderFrame.TitleContainer.TitleText
                if not titleText then
                    return "missing_border_frame_title_text"
                end

                local actual = titleText:GetText()
                if type(actual) ~= "string" or actual == "" then
                    return "empty_border_frame_title_text"
                end

                return "stale_name_border_frame_title_text"
            "#,
            )
            .unwrap();

        assert!(
            result == "ok" || result == "stale_name_border_frame_title_text",
            "World map opening should produce a non-empty title on the live title widget even if the plan name is stale: {result}"
        );
    }
}

#[test]
fn world_map_exploration_pin_has_visible_overlay_textures_after_opening() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);

        env.send_key_press("M", None).expect("M keybind failed");

        let result: String = env
            .eval(
                r#"
                if not (WorldMapFrame and WorldMapFrame:IsShown()) then
                    return "world_map_not_open"
                end

                local pin = WorldMapFrame:EnumeratePinsByTemplate("MapExplorationPinTemplate")()
                if not pin then
                    return "missing_exploration_pin"
                end

                local fogPin = WorldMapFrame:EnumeratePinsByTemplate("FogOfWarPinTemplate")()
                if not fogPin then
                    return "missing_fog_pin"
                end

                if fogPin:IsShown() then
                    return string.format(
                        "fog_pin_should_be_hidden:type=%s:map=%s:bg=%s:mask=%s",
                        tostring(fogPin:GetObjectType()),
                        tostring(fogPin.GetUiMapID and fogPin:GetUiMapID()),
                        tostring(fogPin:GetFogOfWarBackgroundAtlas()),
                        tostring(fogPin:GetFogOfWarMaskAtlas())
                    )
                end

                if fogPin:GetFogOfWarBackgroundAtlas() or fogPin:GetFogOfWarMaskAtlas() then
                    return string.format(
                        "fog_pin_should_not_have_assets:bg=%s:mask=%s",
                        tostring(fogPin:GetFogOfWarBackgroundAtlas()),
                        tostring(fogPin:GetFogOfWarMaskAtlas())
                    )
                end

                local width, height = pin:GetSize()
                if width == 0 or height == 0 then
                    return string.format("zero_size:%s:%s", tostring(width), tostring(height))
                end

                local textureCount = pin.overlayTexturePool and pin.overlayTexturePool:GetNumActive() or 0
                if textureCount == 0 then
                    local mapID = WorldMapFrame:GetMapID()
                    local explored = C_MapExplorationInfo.GetExploredMapTextures(mapID)
                    local exploredCount = explored and #explored or 0
                    local layerIndex = pin.layerIndex
                    local currentLayer = WorldMapFrame:GetCanvasContainer() and WorldMapFrame:GetCanvasContainer():GetCurrentLayerIndex()
                    return string.format(
                        "no_overlay_textures:map=%s:explored=%s:pinLayer=%s:currentLayer=%s",
                        tostring(mapID),
                        tostring(exploredCount),
                        tostring(layerIndex),
                        tostring(currentLayer)
                    )
                end

                local visible = 0
                for texture in pin.overlayTexturePool:EnumerateActive() do
                    if texture:IsShown() then
                        visible = visible + 1
                    end
                end

                if visible == 0 then
                    return string.format("all_overlays_hidden:alpha=%s", tostring(pin:GetAlpha()))
                end

                return "ok"
            "#,
            )
            .unwrap();

        let errors = drain_test_errors(&env);

        assert_eq!(
            result,
            "ok",
            "World map exploration should create a visible exploration overlay pin after opening: {result}"
        );
        assert!(
            errors.is_empty(),
            "World map exploration test produced {} Lua error(s):\n{}",
            errors.len(),
            errors.join("\n"),
        );
    }
}

#[test]
fn world_map_current_map_keeps_fog_of_war_pin_hidden_without_fog_data() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);

        env.send_key_press("M", None).expect("M keybind failed");

        let result: String = env
            .eval(
                r#"
                if not (WorldMapFrame and WorldMapFrame:IsShown()) then
                    return "world_map_not_open"
                end

                local fogPin = WorldMapFrame:EnumeratePinsByTemplate("FogOfWarPinTemplate")()
                if not fogPin then
                    return "missing_fog_pin"
                end

                if fogPin:IsShown() then
                    return string.format(
                        "fog_pin_visible:map=%s:bg=%s:mask=%s",
                        tostring(fogPin.GetUiMapID and fogPin:GetUiMapID()),
                        tostring(fogPin:GetFogOfWarBackgroundAtlas()),
                        tostring(fogPin:GetFogOfWarMaskAtlas())
                    )
                end

                if fogPin:GetFogOfWarBackgroundAtlas() or fogPin:GetFogOfWarMaskAtlas() then
                    return string.format(
                        "fog_pin_has_assets:bg=%s:mask=%s",
                        tostring(fogPin:GetFogOfWarBackgroundAtlas()),
                        tostring(fogPin:GetFogOfWarMaskAtlas())
                    )
                end

                return "ok"
            "#,
            )
            .unwrap();

        let errors = drain_test_errors(&env);

        assert_eq!(
            result,
            "ok",
            "Current world map should leave the fog pin hidden when no fog DB row exists: {result}"
        );
        assert!(
            errors.is_empty(),
            "World map fog visibility test produced {} Lua error(s):\n{}",
            errors.len(),
            errors.join("\n"),
        );
    }
}

#[test]
fn world_map_fog_of_war_pin_matches_canvas_size_on_first_open() {
    test_timeout! {
        let env = setup_env();
        env.apply_post_event_workarounds();

        env.send_key_press("M", None).expect("M keybind failed");

        let (fog_width, fog_height, explored_width, explored_height, expected_width, expected_height): (f64, f64, f64, f64, f64, f64) = env
            .eval(
                r#"
                if not (WorldMapFrame and WorldMapFrame:IsShown()) then
                    error("world map not open")
                end

                local fogPin = WorldMapFrame:EnumeratePinsByTemplate("FogOfWarPinTemplate")()
                assert(fogPin, "missing fog pin")

                local explorationPin = WorldMapFrame:EnumeratePinsByTemplate("MapExplorationPinTemplate")()
                assert(explorationPin, "missing exploration pin")

                local expectedWidth = WorldMapFrame:DenormalizeHorizontalSize(1.0)
                local expectedHeight = WorldMapFrame:DenormalizeVerticalSize(1.0)

                return fogPin:GetWidth(), fogPin:GetHeight(),
                    explorationPin:GetWidth(), explorationPin:GetHeight(),
                    expectedWidth, expectedHeight
            "#,
            )
            .unwrap();

        assert!(
            (fog_width - expected_width).abs() < 0.001,
            "Fog pin width should match the full canvas width on first open: fog_width={fog_width} expected_width={expected_width}"
        );
        assert!(
            (fog_height - expected_height).abs() < 0.001,
            "Fog pin height should match the full canvas height on first open: fog_height={fog_height} expected_height={expected_height}"
        );
        assert!(
            (explored_width - expected_width).abs() < 0.001,
            "Exploration pin width should match the full canvas width on first open: explored_width={explored_width} expected_width={expected_width}"
        );
        assert!(
            (explored_height - expected_height).abs() < 0.001,
            "Exploration pin height should match the full canvas height on first open: explored_height={explored_height} expected_height={expected_height}"
        );
    }
}

#[test]
fn world_map_fog_of_war_pin_resizes_on_canvas_size_changed() {
    test_timeout! {
        let env = setup_env();
        env.apply_post_event_workarounds();

        env.send_key_press("M", None).expect("M keybind failed");

        let (fog_width, fog_height, expected_width, expected_height): (f64, f64, f64, f64) = env
            .eval(
                r#"
                if not (WorldMapFrame and WorldMapFrame:IsShown()) then
                    error("world map not open")
                end

                local fogPin = WorldMapFrame:EnumeratePinsByTemplate("FogOfWarPinTemplate")()
                assert(fogPin, "missing fog pin")

                fogPin:SetSize(128, 96)
                fogPin:OnCanvasSizeChanged()

                local expectedWidth = WorldMapFrame:DenormalizeHorizontalSize(1.0)
                local expectedHeight = WorldMapFrame:DenormalizeVerticalSize(1.0)

                return fogPin:GetWidth(), fogPin:GetHeight(), expectedWidth, expectedHeight
            "#,
            )
            .unwrap();

        assert!(
            (fog_width - expected_width).abs() < 0.001,
            "Fog pin width should refresh when the canvas size changes: fog_width={fog_width} expected_width={expected_width}"
        );
        assert!(
            (fog_height - expected_height).abs() < 0.001,
            "Fog pin height should refresh when the canvas size changes: fog_height={fog_height} expected_height={expected_height}"
        );
    }
}

#[test]
fn world_map_registers_fog_of_war_pin_template_as_fog_of_war_frame() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);

        env.send_key_press("M", None).expect("M keybind failed");

        let template_type: String = env
            .eval(
                r#"
                local info = C_XMLUtil.GetTemplateInfo("FogOfWarPinTemplate")
                assert(info, "missing FogOfWarPinTemplate")
                return info.type
            "#,
            )
            .unwrap();

        let errors = drain_test_errors(&env);

        assert_eq!(template_type, "FogOfWarFrame");
        assert!(
            errors.is_empty(),
            "World map template test produced {} Lua error(s):\n{}",
            errors.len(),
            errors.join("\n"),
        );
    }
}

#[test]
fn world_map_events_tab_click_and_zone_switch_without_errors() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);

        env.send_key_press("M", None).expect("M keybind failed");

        let events_tab_id = {
            let state = env.state().borrow();
            let quest_map_id = state
                .widgets
                .get_id_by_name("QuestMapFrame")
                .expect("QuestMapFrame should exist after opening the world map");
            state
                .widgets
                .get(quest_map_id)
                .and_then(|frame| frame.children_keys.get("EventsTab").copied())
                .expect("QuestMapFrame.EventsTab should exist after opening the world map")
        };

        env.send_click(events_tab_id)
            .expect("clicking QuestMapFrame.EventsTab failed");

        let result: String = env
            .eval(
                r#"
                if not (WorldMapFrame and WorldMapFrame:IsShown()) then
                    return "world_map_not_open"
                end

                if not (QuestMapFrame and QuestMapFrame.EventsTab and QuestMapFrame.EventsTab:IsShown()) then
                    return "events_tab_not_shown"
                end

                if QuestMapFrame.displayMode ~= QuestLogDisplayMode.Events then
                    return "events_tab_not_selected"
                end

                C_Map.SetMapForQuestLog(1)

                if WorldMapFrame:GetMapID() ~= 1 then
                    return "quest_log_map_not_switched"
                end

                ToggleWorldMap()

                if WorldMapFrame:IsShown() then
                    return "world_map_not_closed"
                end

                return "ok"
            "#,
            )
            .unwrap();

        let errors = drain_test_errors(&env);
        assert!(
            errors.is_empty(),
            "World map events tab flow produced {} Lua error(s):\n{}",
            errors.len(),
            errors.join("\n"),
        );
        assert_eq!(
            result,
            "ok",
            "World map events tab flow should open, switch to events, change zone, and close: {result}"
        );
    }
}

#[test]
fn quest_log_validate_tabs_shows_events_tab_when_scheduler_can_show_events() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);

        env.send_key_press("M", None).expect("M keybind failed");

        let result: String = env
            .eval(
                r#"
                if not (WorldMapFrame and WorldMapFrame:IsShown()) then
                    return "world_map_not_open"
                end

                if not (QuestMapFrame and QuestMapFrame.EventsTab) then
                    return "events_tab_missing"
                end

                C_EventScheduler._state.canShowEvents = true
                QuestMapFrame.EventsTab:Hide()
                QuestMapFrame:ValidateTabs()

                if not C_EventScheduler.CanShowEvents() then
                    return "scheduler_cannot_show_events"
                end

                if not QuestMapFrame.EventsTab:IsShown() then
                    return "events_tab_not_shown"
                end

                return "ok"
            "#,
            )
            .unwrap();

        let errors = drain_test_errors(&env);
        assert!(
            errors.is_empty(),
            "Quest log ValidateTabs flow produced {} Lua error(s):\n{}",
            errors.len(),
            errors.join("\n"),
        );
        assert_eq!(
            result,
            "ok",
            "Quest log ValidateTabs should show the Events tab when C_EventScheduler.CanShowEvents() is true: {result}"
        );
    }
}
