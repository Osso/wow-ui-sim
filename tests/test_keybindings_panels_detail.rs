//! Integration tests for keybinding dispatch — core world map panel details.

use crate::prefork_full_ui_preload::{
    drain_test_errors, frame_is_shown, install_test_error_handler,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::startup::{fire_one_on_update_tick, process_pending_timers};

// ── M → ToggleWorldMap() ────────────────────────────────────────────────

pub(crate) fn keybind_m_opens_world_map(env: &WowLuaEnv) {
    env.send_key_press("M", None).expect("M keybind failed");
    assert!(
        frame_is_shown(&env, "WorldMapFrame"),
        "WorldMapFrame should be shown after pressing M"
    );
}

pub(crate) fn world_map_floor_dropdown_hidden_without_subzone_or_map_group(env: &WowLuaEnv) {
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
        result, "ok",
        "World map floor dropdown should stay hidden when there is no subzone or map group: {result}"
    );
}

pub(crate) fn world_map_tracking_defaults_follow_filter_masks(env: &WowLuaEnv) {
    env.exec(
        r#"
        local button = CreateFromMixins(WorldMapTrackingOptionsButtonMixin)
        button:BuildFilterTable()
        local off = button:GetWorldMapFilter('trivialQuests')
        local on = button:GetWorldMapFilter('showAccountCompletedQuests')
        assert(off:GetDefault() == false)
        assert(on:GetDefault() == true)
        assert(C_Minimap.GetDefaultTrackingValue(0) == true)
        assert(C_Minimap.GetDefaultTrackingValue(2097152 + 1024) == false)
    "#,
    )
    .expect("original world-map filters must observe both defaults");
}

pub(crate) fn world_map_bounty_board_reads_supplied_bounties(env: &WowLuaEnv) {
    use wow_ui_sim::lua_api::state::QuestBounty;
    {
        let mut state = env.state().borrow_mut();
        state.bounty_set_maps.insert(98701, vec![2248]);
        state.quest_log_entries.bounties_by_set.insert(
            98701,
            vec![QuestBounty {
                quest_id: 701,
                faction_id: 72,
                icon: Some(83),
                num_objectives: 2,
                turnin_requirement_text: Some("Return to camp".to_string()),
            }],
        );
    }
    env.exec(
        r#"
        WorldMapFrame:SetMapID(2248)
        local board = CreateFrame('Frame', nil, WorldMapFrame, 'WorldMapBountyBoardTemplate')
        board:Refresh()
        assert(#board.bounties == 1 and board.bounties[1].questID == 701)
        assert(board.bounties[1].factionID == 72 and board.bounties[1].icon == 83)
        assert(board.bounties[1].numObjectives == 2)
        assert(board.bounties[1].turninRequirementText == 'Return to camp')
    "#,
    )
    .expect("original bounty board must read the supplied bounty");
    env.state()
        .borrow_mut()
        .quest_log_entries
        .bounties_by_set
        .insert(98701, vec![]);
    env.exec(
        r#"
        local bounties = C_QuestLog.GetBountiesForMapID(2248)
        assert(type(bounties) == 'table' and #bounties == 0)
        assert(C_QuestLog.GetBountiesForMapID(-98701) == nil)
    "#,
    )
    .expect("resolved empty and missing map must differ");
}

pub(crate) fn world_map_threat_frame_reads_quest_log_flags(env: &WowLuaEnv) {
    let quest_id = env
        .state()
        .borrow()
        .quest_log_entries
        .entries
        .first()
        .expect("seed quest")
        .quest_id;
    env.state()
        .borrow_mut()
        .quest_log_entries
        .threat_entry_flags
        .insert(quest_id, 8);
    env.exec(
        r#"
        local frame = CreateFrame('Frame', nil, WorldMapFrame, 'WorldMapThreatFrameTemplate')
        frame:Refresh()
        assert(C_QuestLog.HasActiveThreats() == true)
    "#,
    )
    .expect("original threat frame must query the active quest");
    env.state()
        .borrow_mut()
        .quest_log_entries
        .threat_entry_flags
        .insert(quest_id, 8 | 32);
    env.exec(
        r#"
        assert(C_QuestLog.HasActiveThreats() == false)
    "#,
    )
    .expect("excluded quest flags must not count as active threats");
}

pub(crate) fn world_map_campaign_headers_refresh_on_show(env: &WowLuaEnv) {
    env.exec(
        r#"
        QuestMapFrame.initialCampaignHeadersUpdate = nil
        QuestMapFrame_OnShow(QuestMapFrame)
        assert(QuestMapFrame.initialCampaignHeadersUpdate == true)
        assert(select('#', C_QuestLog.UpdateCampaignHeaders()) == 0)
    "#,
    )
    .expect("original quest map show must refresh campaign headers");
}

pub(crate) fn world_map_party_quest_counts_follow_membership(env: &WowLuaEnv) {
    let quest_id = env.state().borrow().quest_log_entries.entries[0].quest_id;
    {
        let mut state = env.state().borrow_mut();
        state.party_group_active = true;
        assert!(!state.party_members.is_empty());
        state.quest_log_entries.party_quest_ids.insert(
            "Player-0000-00000002".to_string(),
            [quest_id].into_iter().collect(),
        );
    }
    env.exec(&format!(
        r#"
        assert(QuestUtils_GetNumPartyMembersOnQuest({quest_id}) == 1)
        assert(C_QuestLog.IsUnitOnQuest('party1', {quest_id}) == true)
        assert(C_QuestLog.IsUnitOnQuest('party1', -1) == false)
        assert(C_QuestLog.IsUnitOnQuest('nobody', {quest_id}) == false)
        assert(C_QuestLog.IsUnitOnQuest('player', {quest_id}) == true)
    "#
    ))
    .expect("original quest utility must count supplied party quest membership");
    env.state()
        .borrow_mut()
        .quest_log_entries
        .party_quest_ids
        .clear();
    env.exec(&format!(
        r#"
        assert(QuestUtils_GetNumPartyMembersOnQuest({quest_id}) == 0)
        assert(C_QuestLog.IsUnitOnQuest('party1', {quest_id}) == false)
    "#
    ))
    .expect("removing supplied membership must update the original utility");
}

pub(crate) fn keybind_m_toggles_world_map_without_errors(env: &WowLuaEnv) {
    install_test_error_handler(&env);

    env.send_key_press("M", None)
        .expect("first M keybind failed");

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

    env.send_key_press("M", None)
        .expect("second M keybind failed");

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

pub(crate) fn world_map_title_text_is_non_empty_after_opening(env: &WowLuaEnv) {
    install_test_error_handler(&env);

    env.send_key_press("M", None).expect("M keybind failed");
    process_pending_timers(&env);
    fire_one_on_update_tick(&env);

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

pub(crate) fn world_map_exploration_pin_has_visible_overlay_textures_after_opening(
    env: &WowLuaEnv,
) {
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
        result, "ok",
        "World map exploration should create a visible exploration overlay pin after opening: {result}"
    );
    assert!(
        errors.is_empty(),
        "World map exploration test produced {} Lua error(s):\n{}",
        errors.len(),
        errors.join("\n"),
    );
}

pub(crate) fn world_map_exploration_pin_converges_visible_after_onupdate_ticks(env: &WowLuaEnv) {
    install_test_error_handler(&env);

    env.send_key_press("M", None).expect("M keybind failed");
    for _ in 0..12 {
        process_pending_timers(&env);
        env.state().borrow_mut().ensure_layout_rects();
        fire_one_on_update_tick(&env);
    }

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

                local waiting = rawget(pin, "isWaitingForLoad")
                local detailLoaded = WorldMapFrame:AreDetailLayersLoaded()
                local alpha = pin:GetAlpha()
                local shown = pin:IsShown()
                local visible = pin:IsVisible()
                local textureVisibleCount = 0
                local textureActiveCount = 0
                if pin.overlayTexturePool and pin.overlayTexturePool.EnumerateActive then
                    for texture in pin.overlayTexturePool:EnumerateActive() do
                        textureActiveCount = textureActiveCount + 1
                        if texture:IsVisible() then
                            textureVisibleCount = textureVisibleCount + 1
                        end
                    end
                end

                if waiting ~= nil then
                    return string.format(
                        "pin_waiting:detailLoaded=%s:shown=%s:visible=%s:alpha=%.2f:activeTextures=%d:visibleTextures=%d",
                        tostring(detailLoaded),
                        tostring(shown),
                        tostring(visible),
                        alpha,
                        textureActiveCount,
                        textureVisibleCount
                    )
                end

                if not detailLoaded then
                    return string.format(
                        "detail_layers_not_loaded:shown=%s:visible=%s:alpha=%.2f",
                        tostring(shown),
                        tostring(visible),
                        alpha
                    )
                end

                if not shown or not visible or alpha <= 0 then
                    return string.format(
                        "pin_not_visible:shown=%s:visible=%s:alpha=%.2f:activeTextures=%d:visibleTextures=%d",
                        tostring(shown),
                        tostring(visible),
                        alpha,
                        textureActiveCount,
                        textureVisibleCount
                    )
                end

                if textureVisibleCount == 0 then
                    return string.format(
                        "no_visible_overlay_textures:active=%d:shown=%s:visible=%s:alpha=%.2f",
                        textureActiveCount,
                        tostring(shown),
                        tostring(visible),
                        alpha
                    )
                end

                return "ok"
            "#,
            )
            .unwrap();

    let errors = drain_test_errors(&env);

    assert_eq!(
        result, "ok",
        "World map exploration pin should become visible after update ticks: {result}"
    );
    assert!(
        errors.is_empty(),
        "World map exploration visibility test produced {} Lua error(s):\n{}",
        errors.len(),
        errors.join("\n"),
    );
}

pub(crate) fn world_map_exploration_pin_first_open_settles_without_reopen(env: &WowLuaEnv) {
    install_test_error_handler(&env);

    env.send_key_press("M", None).expect("M keybind failed");
    for _ in 0..4 {
        process_pending_timers(&env);
        env.state().borrow_mut().ensure_layout_rects();
        fire_one_on_update_tick(&env);
    }

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

                local waiting = rawget(pin, "isWaitingForLoad")
                local alpha = pin:GetAlpha()
                local visible = pin:IsVisible()
                local detailLoaded = WorldMapFrame:AreDetailLayersLoaded()
                if waiting ~= nil then
                    return string.format(
                        "pin_still_waiting:detailLoaded=%s:visible=%s:alpha=%.2f",
                        tostring(detailLoaded),
                        tostring(visible),
                        alpha
                    )
                end
                if not visible or alpha <= 0 then
                    return string.format(
                        "pin_not_visible:detailLoaded=%s:visible=%s:alpha=%.2f",
                        tostring(detailLoaded),
                        tostring(visible),
                        alpha
                    )
                end
                return "ok"
            "#,
        )
        .unwrap();

    let errors = drain_test_errors(&env);
    assert_eq!(
        result, "ok",
        "World map first open should settle explored pin visibility without requiring reopen: {result}"
    );
    assert!(
        errors.is_empty(),
        "World map first-open settle test produced {} Lua error(s):\n{}",
        errors.len(),
        errors.join("\n"),
    );
}

pub(crate) fn world_map_exploration_pin_recovers_when_first_overlay_fetch_is_empty(
    env: &WowLuaEnv,
) {
    install_test_error_handler(&env);

    env.eval::<bool>(
        r#"
            local original = C_MapExplorationInfo.GetExploredMapTextures
            local suppressedByMapID = {}
            C_MapExplorationInfo.GetExploredMapTextures = function(mapID)
                if type(mapID) == "number" and not suppressedByMapID[mapID] then
                    suppressedByMapID[mapID] = true
                    return {}
                end
                return original(mapID)
            end
            return true
        "#,
    )
    .unwrap();

    env.send_key_press("M", None).expect("M keybind failed");
    for _ in 0..12 {
        process_pending_timers(&env);
        env.state().borrow_mut().ensure_layout_rects();
        fire_one_on_update_tick(&env);
    }

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

                local activeTextures = 0
                local visibleTextures = 0
                if pin.overlayTexturePool and pin.overlayTexturePool.EnumerateActive then
                    for texture in pin.overlayTexturePool:EnumerateActive() do
                        activeTextures = activeTextures + 1
                        if texture:IsVisible() then
                            visibleTextures = visibleTextures + 1
                        end
                    end
                end

                local waiting = rawget(pin, "isWaitingForLoad")
                local alpha = pin:GetAlpha()
                if activeTextures == 0 or visibleTextures == 0 or waiting ~= nil or alpha <= 0 then
                    return string.format(
                        "pin_unsettled:active=%d:visible=%d:waiting=%s:alpha=%.2f",
                        activeTextures,
                        visibleTextures,
                        tostring(waiting),
                        alpha
                    )
                end
                return "ok"
            "#,
        )
        .unwrap();

    let errors = drain_test_errors(&env);
    assert_eq!(
        result, "ok",
        "World map should recover from an empty first explored-texture fetch without requiring reopen: {result}"
    );
    assert!(
        errors.is_empty(),
        "World map empty-first-fetch recovery test produced {} Lua error(s):\n{}",
        errors.len(),
        errors.join("\n"),
    );
}

pub(crate) fn world_map_registers_fog_of_war_pin_template_as_fog_of_war_frame(env: &WowLuaEnv) {
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
