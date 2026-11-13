//! Integration tests for ReputationFrame ShowUIPanel layout details.

use crate::common;

use std::path::PathBuf;
use wow_ui_sim::lua_api::WowLuaEnv;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

/// Eager panel publishers in dependency order. `Blizzard_Menu` must precede
/// ReputationFrame because its filter uses `DropdownButtonMixin`.
const PANEL_ADDONS: &[&str] = &[
    "Blizzard_SharedXMLBase",
    "Blizzard_Menu",
    "Blizzard_Colors",
    "Blizzard_SharedXML",
    "Blizzard_SharedXMLGame",
    "Blizzard_UIPanelTemplates",
    "Blizzard_FrameXMLBase",
    "Blizzard_FrameEffects",
    "Blizzard_LoadLocale",
    "Blizzard_Fonts_Shared",
    "Blizzard_HelpPlate",
    "Blizzard_AccessibilityTemplates",
    "Blizzard_ObjectAPI",
    "Blizzard_UIParent",
    "Blizzard_TextStatusBar",
    "Blizzard_MoneyFrame",
    "Blizzard_POIButton",
    "Blizzard_Flyout",
    "Blizzard_StoreUI",
    "Blizzard_MicroMenu",
    "Blizzard_EditMode",
    "Blizzard_GarrisonBase",
    "Blizzard_GameTooltip",
    "Blizzard_ManagedFrameSystem",
    "Blizzard_GameMenuEsc",
    "Blizzard_UIParentPanelManager",
    "Blizzard_Settings_Shared",
    "Blizzard_SettingsDefinitions_Shared",
    "Blizzard_SettingsDefinitions_Frame",
    "Blizzard_FrameXMLUtil",
    "Blizzard_ItemButton",
    "Blizzard_QuickKeybind",
    "Blizzard_FrameXML",
    "Blizzard_UIPanels_Game",
    "Blizzard_ActionBar",
    "Blizzard_UnitFrame",
    "Blizzard_TokenUI",
];

fn setup_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    if cfg!(feature = "client-wrath") {
        common::panel_fixtures::load_wrath_panel_core(&env);
        env.apply_post_load_workarounds();
        fire_startup_events(&env);
        return env;
    }

    if !common::load_classic_game_startup_inventory(&env) {
        let ui = blizzard_ui_dir();
        for addon_name in PANEL_ADDONS {
            common::load_required_blizzard_addon(&env, &ui, addon_name);
            if *addon_name == "Blizzard_UIParentPanelManager" {
                load_ui_parent_panel_manager_global_pass(&env, &ui);
            }
        }
    }

    env.apply_post_load_workarounds();
    fire_startup_events(&env);
    env
}

/// Load the TOC entries marked `[AllowLoadEnvironment Global]` after the
/// manager's normal `UIPanelWindows.lua` pass. The regular fixture loader
/// correctly excludes those entries from its normal environment pass.
fn load_ui_parent_panel_manager_global_pass(env: &WowLuaEnv, ui: &std::path::Path) {
    for file in [
        "Shared/UIPanelLayoutFrame.lua",
        "Shared/UIParentPanelManager.lua",
        "Mainline/UIParentPanelManagerOverrides.lua",
        "Shared/UpdateUIPanelPositions.lua",
    ] {
        common::load_blizzard_addon_bootstrap(env, ui, "Blizzard_UIParentPanelManager", file);
    }
}

fn fire_startup_events(env: &WowLuaEnv) {
    common::fire_addon_loaded(env, "WoWUISim");
    for event in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        let _ = env.fire_event(event);
    }
    common::fire_player_entering_world(env, true, false);
    for event in [
        "UPDATE_BINDINGS",
        "DISPLAY_SIZE_CHANGED",
        "UI_SCALE_CHANGED",
    ] {
        let _ = env.fire_event(event);
    }
}

#[cfg(feature = "client-retail")]
#[test]
fn retail_reputation_filter_toggles_and_requires_legacy_visibility_argument() {
    test_timeout! {
        let env = setup_env();
        let result: String = env
            .eval(
                r#"
                local onShow = ReputationFrameMixin and ReputationFrameMixin.OnShow
                if type(onShow) ~= "function" then
                    return "missing_reputation_consumer"
                end
                local source = debug.getinfo(onShow, "S").source:gsub("\\", "/")
                if not source:find("Blizzard_UIPanels_Game/Mainline/ReputationFrame.lua", 1, true) then
                    return "wrong_reputation_consumer=" .. source
                end

                local menuGenerator
                local dropdown = {
                    SetupMenu = function(_, generator)
                        menuGenerator = generator
                    end,
                }
                local frame = {
                    filterDropdown = dropdown,
                    Update = function() end,
                    GetParent = function() return nil end,
                }
                local oldRegister = FrameUtil.RegisterFrameForEvents
                local oldHelpTip = HelpTip.IsShowing
                local oldPlayerExpansion = GetExpansionLevel
                local oldServerExpansion = GetServerExpansionLevel
                FrameUtil.RegisterFrameForEvents = function() end
                HelpTip.IsShowing = function() return false end
                GetExpansionLevel = function() return 12 end
                GetServerExpansionLevel = function() return 12 end
                onShow(frame)

                local isSelected
                local toggleSelection
                menuGenerator(dropdown, {
                    SetTag = function() end,
                    CreateRadio = function() end,
                    CreateDivider = function() end,
                    CreateCheckbox = function(_, _, selected, toggle)
                        isSelected = selected
                        toggleSelection = toggle
                        return { SetSelectionIgnored = function() end }
                    end,
                })
                FrameUtil.RegisterFrameForEvents = oldRegister
                HelpTip.IsShowing = oldHelpTip
                GetExpansionLevel = oldPlayerExpansion
                GetServerExpansionLevel = oldServerExpansion

                if type(isSelected) ~= "function" or type(toggleSelection) ~= "function" then
                    return "legacy_checkbox_missing"
                end
                if not isSelected() then
                    return "legacy_checkbox_should_start_selected"
                end
                toggleSelection()
                if isSelected() then
                    return "consumer_did_not_hide_legacy_reputations"
                end
                toggleSelection()
                if not isSelected() then
                    return "consumer_did_not_show_legacy_reputations"
                end

                local usage = "Usage: C_Reputation.SetLegacyReputationsShown(showLegacyReputations)"
                local missingOK, missingError = pcall(C_Reputation.SetLegacyReputationsShown)
                if missingOK then
                    return "missing_argument_was_accepted"
                end
                if not tostring(missingError):find(usage, 1, true) then
                    return "wrong_missing_argument_error=" .. tostring(missingError)
                end
                local nilOK, nilError = pcall(C_Reputation.SetLegacyReputationsShown, nil)
                if nilOK then
                    return "nil_argument_was_accepted"
                end
                if not tostring(nilError):find(usage, 1, true) then
                    return "wrong_nil_argument_error=" .. tostring(nilError)
                end
                return "ok"
                "#,
            )
            .expect("Retail reputation consumer should run");
        assert_eq!(result, "ok");
    }
}

// Era/Anniversary select `Blizzard_CharacterFrame/Vanilla/CharacterFrame.lua`,
// which does not define the Retail/Cata `CharacterFrame:RefreshDisplay` method.
#[cfg(feature = "client-retail")]
#[test]
fn retail_reputation_header_consumer_expands_all_for_zero_index() {
    test_timeout! {
        let env = setup_env();
        let result: String = env
            .eval(
                r#"
                local toggleCollapsed = ReputationHeaderMixin and ReputationHeaderMixin.ToggleCollapsed
                if type(toggleCollapsed) ~= "function" then
                    return "missing_reputation_header_consumer"
                end
                local source = debug.getinfo(toggleCollapsed, "S").source:gsub("\\", "/")
                if not source:find("Blizzard_UIPanels_Game/Mainline/ReputationFrame.lua", 1, true) then
                    return "wrong_reputation_header_consumer=" .. source
                end

                C_Reputation.CollapseAllFactionHeaders()
                local collapsedCount = C_Reputation.GetNumFactions()
                toggleCollapsed({
                    factionIndex = 0,
                    IsCollapsed = function() return true end,
                })
                return tostring(collapsedCount) .. ":" .. tostring(C_Reputation.GetNumFactions())
                "#,
            )
            .expect("Retail reputation header consumer should run");
        assert_eq!(result, "3:11");
    }
}

#[cfg(feature = "client-retail")]
#[test]
fn retail_reputation_sort_consumer_rejects_unknown_sort_type() {
    test_timeout! {
        let env = setup_env();
        let result: String = env
            .eval(
                r#"
                local onShow = ReputationFrameMixin and ReputationFrameMixin.OnShow
                if type(onShow) ~= "function" then
                    return "missing_reputation_consumer"
                end
                local source = debug.getinfo(onShow, "S").source:gsub("\\", "/")
                if not source:find("Blizzard_UIPanels_Game/Mainline/ReputationFrame.lua", 1, true) then
                    return "wrong_reputation_consumer=" .. source
                end

                local menuGenerator
                local dropdown = {
                    SetupMenu = function(_, generator)
                        menuGenerator = generator
                    end,
                }
                local frame = {
                    filterDropdown = dropdown,
                    Update = function() end,
                    GetParent = function() return nil end,
                }
                local oldRegister = FrameUtil.RegisterFrameForEvents
                local oldHelpTip = HelpTip.IsShowing
                local oldPlayerExpansion = GetExpansionLevel
                local oldServerExpansion = GetServerExpansionLevel
                FrameUtil.RegisterFrameForEvents = function() end
                HelpTip.IsShowing = function() return false end
                GetExpansionLevel = function() return 12 end
                GetServerExpansionLevel = function() return 12 end
                onShow(frame)

                local setSortType
                menuGenerator(dropdown, {
                    SetTag = function() end,
                    CreateRadio = function(_, _, _, setSelected, sortType)
                        if sortType == 0 then
                            setSortType = setSelected
                        end
                    end,
                    CreateDivider = function() end,
                    CreateCheckbox = function()
                        return { SetSelectionIgnored = function() end }
                    end,
                })
                FrameUtil.RegisterFrameForEvents = oldRegister
                HelpTip.IsShowing = oldHelpTip
                GetExpansionLevel = oldPlayerExpansion
                GetServerExpansionLevel = oldServerExpansion
                if type(setSortType) ~= "function" then
                    return "sort_radio_missing"
                end

                for _, sortType in ipairs({0, 1, 2}) do
                    setSortType(sortType)
                    if C_Reputation.GetReputationSortType() ~= sortType then
                        return "valid_sort_type_not_stored=" .. tostring(sortType)
                    end
                end
                local invalidOK = pcall(setSortType, 3)
                if invalidOK then
                    return "invalid_sort_type_was_accepted"
                end
                if C_Reputation.GetReputationSortType() ~= 2 then
                    return "invalid_sort_type_changed_state"
                end
                local usage = "Usage: C_Reputation.SetReputationSortType(sortType)"
                local missingOK, missingError = pcall(setSortType)
                if missingOK then
                    return "missing_sort_type_was_accepted"
                end
                if not tostring(missingError):find(usage, 1, true) then
                    return "wrong_missing_argument_error=" .. tostring(missingError)
                end
                return "ok"
                "#,
            )
            .expect("Retail reputation sort consumer should run");
        assert_eq!(result, "ok");
    }
}

#[cfg(feature = "client-retail")]
#[test]
fn retail_reputation_tooltip_consumer_truncates_faction_id() {
    test_timeout! {
        let env = setup_env();
        let result: String = env
            .eval(
                r#"
                local consumer = ReputationUtil and ReputationUtil.TryAppendAccountReputationLineToTooltip
                if type(consumer) ~= "function" then
                    return "missing_reputation_tooltip_consumer"
                end
                local source = debug.getinfo(consumer, "S").source:gsub("\\", "/")
                if not source:find("Blizzard_FrameXMLUtil/Mainline/ReputationUtil.lua", 1, true) then
                    return "wrong_reputation_tooltip_consumer=" .. source
                end

                GameTooltip_AddColoredLine = function(tooltip)
                    tooltip.accountWideLineCount = tooltip.accountWideLineCount + 1
                end
                local tooltip = { accountWideLineCount = 0 }
                consumer(tooltip, 2590.5)
                return tostring(tooltip.accountWideLineCount)
                "#,
            )
            .expect("Retail reputation tooltip consumer should run");
        assert_eq!(result, "1");
    }
}

#[cfg(feature = "client-retail")]
#[test]
fn retail_inactive_checkbox_consumer_updates_faction_activity() {
    test_timeout! {
        let env = setup_env();
        let result: String = env
            .eval(
                r#"
                local onClick = ReputationDetailInactiveCheckboxMixin and ReputationDetailInactiveCheckboxMixin.OnClick
                if type(onClick) ~= "function" then
                    return "missing_inactive_checkbox_consumer"
                end
                local source = debug.getinfo(onClick, "S").source:gsub("\\", "/")
                if not source:find("Blizzard_UIPanels_Game/Mainline/ReputationFrame.lua", 1, true) then
                    return "wrong_inactive_checkbox_consumer=" .. source
                end

                local faction = __wow_reputation_state.factions[2]
                faction.canSetInactive = true
                faction.reputationIndex = 114
                faction.isInactive = false
                C_Reputation.SetSelectedFaction(2)
                local checked = false
                local checkbox = { GetChecked = function() return checked end }
                PlaySound = function() end

                onClick(checkbox)
                checked = true
                onClick(checkbox)
                if C_Reputation.IsFactionActive(2) then
                    return "consumer_did_not_mark_faction_inactive"
                end
                C_Reputation.SetFactionActive(2, 0)
                if not C_Reputation.IsFactionActive(2) then
                    return "zero_was_not_truthy"
                end
                return "ok"
                "#,
            )
            .expect("Retail inactive checkbox consumer should run");
        assert_eq!(result, "ok");
    }
}

#[cfg(not(any(feature = "client-era", feature = "client-anniversary")))]
#[test]
fn show_ui_panel_locks_reputation_frame_layout() {
    test_timeout! {
        let env = setup_env();
        if cfg!(feature = "client-wrath") {
            env.exec(include_str!("fixtures/wrath_f03/reputation_layout.lua"))
                .expect("native reputation panel and scroll/detail geometry");
            return;
        }
        let result: String = env
            .eval(
                r#"
                local EPS = 0.75

                local function approx(actual, expected, eps)
                    if type(actual) ~= "number" or type(expected) ~= "number" then
                        return false
                    end
                    return math.abs(actual - expected) <= (eps or EPS)
                end

                local function rect(path, frame)
                    if type(frame) ~= "table" then
                        return nil, path .. "_missing"
                    end
                    local l, b, w, h = frame:GetRect()
                    if not (l and b and w and h) then
                        return nil, path .. "_missing_rect"
                    end
                    return { l = l, b = b, w = w, h = h, r = l + w, t = b + h }, nil
                end

                local function expect_rect(path, frame, expected_left, expected_bottom, expected_width, expected_height)
                    local r, e = rect(path, frame)
                    if not r then
                        return nil, e
                    end
                    if not approx(r.l, expected_left) then
                        return nil, path .. "_left=" .. tostring(r.l)
                    end
                    if not approx(r.b, expected_bottom) then
                        return nil, path .. "_bottom=" .. tostring(r.b)
                    end
                    if not approx(r.w, expected_width, 0.1) then
                        return nil, path .. "_width=" .. tostring(r.w)
                    end
                    if not approx(r.h, expected_height, 0.1) then
                        return nil, path .. "_height=" .. tostring(r.h)
                    end
                    return r, nil
                end

                if not CharacterFrame then
                    return "missing_character_frame"
                end
                if not ShowUIPanel then
                    return "missing_show_uipanel"
                end
                if not ReputationFrame then
                    return "missing_reputation_frame"
                end
                if PanelTemplates_SetTab then
                    PanelTemplates_SetTab(CharacterFrame, ReputationFrame:GetID())
                end
                if PaperDollFrame then
                    PaperDollFrame:Hide()
                end
                if TokenFrame then
                    TokenFrame:Hide()
                end
                ReputationFrame:Show()
                CharacterFrame.activeSubframe = "ReputationFrame"
                ShowUIPanel(CharacterFrame)
                CharacterFrame:RefreshDisplay()

                if not CharacterFrame:IsShown() then
                    return "character_not_shown"
                end
                if not ReputationFrame:IsShown() then
                    return "reputation_not_shown"
                end
                if PaperDollFrame and PaperDollFrame:IsShown() then
                    return "paperdoll_should_be_hidden"
                end

                if PanelTemplates_GetSelectedTab and PanelTemplates_GetSelectedTab(CharacterFrame) ~= 2 then
                    return "selected_tab=" .. tostring(PanelTemplates_GetSelectedTab(CharacterFrame))
                end

                local frameRect, frameErr = expect_rect("CharacterFrame", CharacterFrame, 16, 228, 400, 424)
                if not frameRect then return frameErr end

                local repRect, repErr = expect_rect("ReputationFrame", ReputationFrame, 16, 228, 400, 424)
                if not repRect then return repErr end

                local filterRect, filterErr = expect_rect(
                    "ReputationFrame.filterDropdown",
                    ReputationFrame.filterDropdown,
                    278,
                    597,
                    130,
                    25
                )
                if not filterRect then return filterErr end
                if not ReputationFrame.filterDropdown:IsShown() then
                    return "filter_dropdown_hidden"
                end

                local scrollBoxRect, scrollBoxErr = expect_rect(
                    "ReputationFrame.ScrollBox",
                    ReputationFrame.ScrollBox,
                    24,
                    234,
                    364,
                    354
                )
                if not scrollBoxRect then return scrollBoxErr end
                if not ReputationFrame.ScrollBox:IsShown() then
                    return "scroll_box_hidden"
                end

                local scrollBarRect, scrollBarErr = expect_rect(
                    "ReputationFrame.ScrollBar",
                    ReputationFrame.ScrollBar,
                    393,
                    238,
                    8,
                    348
                )
                if not scrollBarRect then return scrollBarErr end
                if not ReputationFrame.ScrollBar:IsShown() then
                    return "scroll_bar_hidden"
                end

                if not approx(scrollBarRect.l, scrollBoxRect.r + 5) then
                    return "scroll_bar_left=" .. tostring(scrollBarRect.l)
                end

                local detail = ReputationFrame.ReputationDetailFrame
                if not detail then
                    return "detail_frame_missing"
                end
                if detail:IsShown() then
                    return "detail_frame_should_start_hidden"
                end

                local detailRect, detailErr = expect_rect(
                    "ReputationFrame.ReputationDetailFrame",
                    detail,
                    416,
                    421,
                    212,
                    203
                )
                if not detailRect then return detailErr end

                detail:Show()
                if not detail:IsShown() then
                    return "detail_frame_not_shown_after_show"
                end

                local titleRect, titleErr = rect("ReputationDetailFrame.Title", detail.Title)
                if not titleRect then return titleErr end
                if not approx(titleRect.l, detailRect.l + 20) then
                    return "detail_title_left=" .. tostring(titleRect.l)
                end
                if not approx(titleRect.t, detailRect.t - 21) then
                    return "detail_title_top=" .. tostring(titleRect.t)
                end

                local atWarRect, atWarErr = expect_rect(
                    "ReputationDetailFrame.AtWarCheckbox",
                    detail.AtWarCheckbox,
                    430,
                    455,
                    26,
                    26
                )
                if not atWarRect then return atWarErr end

                local inactiveRect, inactiveErr = rect(
                    "ReputationDetailFrame.MakeInactiveCheckbox",
                    detail.MakeInactiveCheckbox
                )
                if not inactiveRect then return inactiveErr end
                if not approx(inactiveRect.w, 26, 0.1) or not approx(inactiveRect.h, 26, 0.1) then
                    return "inactive_checkbox_size=" .. tostring(inactiveRect.w) .. "x" .. tostring(inactiveRect.h)
                end

                local watchRect, watchErr = expect_rect(
                    "ReputationDetailFrame.WatchFactionCheckbox",
                    detail.WatchFactionCheckbox,
                    430,
                    432,
                    26,
                    26
                )
                if not watchRect then return watchErr end

                if not approx(inactiveRect.b, atWarRect.b) then
                    return "inactive_checkbox_bottom=" .. tostring(inactiveRect.b)
                end
                if not (inactiveRect.l > atWarRect.r + 30) then
                    return "inactive_checkbox_left=" .. tostring(inactiveRect.l)
                end
                if not approx(watchRect.l, atWarRect.l) then
                    return "watch_checkbox_left=" .. tostring(watchRect.l)
                end
                if not approx(watchRect.t, atWarRect.b + 3) then
                    return "watch_checkbox_top=" .. tostring(watchRect.t)
                end

                return "ok"
            "#,
            )
            .unwrap();
        assert_eq!(
            result, "ok",
            "ReputationFrame layout should stay fully locked after CharacterFrame shows the reputation subframe: {result}"
        );
    }
}

// The filter dropdown is only in the Mainline/Camelot ReputationFrame consumer.
retail_ptr_forever_toc_fixture! {
#[test]
fn reputation_filter_dropdown_mouse_down_materializes_menu_rows() {
    test_timeout! {
        let env = setup_env();
        let setup_result: String = env
            .eval(
                r#"
                if not CharacterFrame or not ReputationFrame or not ReputationFrame.filterDropdown then
                    return "missing_frame"
                end
                if PanelTemplates_SetTab then
                    PanelTemplates_SetTab(CharacterFrame, ReputationFrame:GetID())
                end
                return "ready"
                "#,
            )
            .unwrap();
        assert_eq!(setup_result, "ready");

        let state = env.state();
        let (reputation_id, dropdown_id) = {
            let sim = state.borrow();
            let reputation_id = sim
                .widgets
                .get_id_by_name("ReputationFrame")
                .expect("ReputationFrame should exist");
            let dropdown_id = sim
                .widgets
                .get(reputation_id)
                .and_then(|frame| frame.children_keys.get("filterDropdown"))
                .copied()
                .expect("ReputationFrame filterDropdown should exist");
            (reputation_id, dropdown_id)
        };
        env.fire_script_handler(reputation_id, "OnShow", Vec::new())
            .expect("ReputationFrame OnShow should dispatch");
        let left_button = env.lua_string("LeftButton");
        env.fire_script_handler(dropdown_id, "OnMouseDown", vec![left_button])
            .expect("reputation filter OnMouseDown should dispatch");

        let result: String = env
            .eval(
                r#"
                local dropdown = ReputationFrame.filterDropdown
                local buttons = {}
                if dropdown.menu then
                    for _, child in ipairs({ dropdown.menu:GetChildren() }) do
                        if child:GetObjectType() == "Button" then
                            buttons[#buttons + 1] = child
                        end
                    end
                end
                if #buttons ~= 4 then
                    return "button_count=" .. tostring(#buttons)
                end
                local playerName = UnitName("player")
                local expected = {"All", "Warband", playerName, "Show Legacy Reputations"}
                for index, expectedText in ipairs(expected) do
                    local button = buttons[index]
                    if not button then
                        return "missing_button_" .. tostring(index)
                    end
                    if not button:IsShown() then
                        return "hidden_button_" .. tostring(index)
                    end
                    local text = button:GetText()
                    if (text == nil or text == "") and button.fontString then
                        text = button.fontString:GetText()
                    end
                    if text ~= expectedText then
                        return "button_" .. tostring(index) .. "=" .. tostring(text)
                    end
                    if button:GetFrameStrata() ~= "FULLSCREEN_DIALOG" then
                        return "button_" .. tostring(index) .. "_strata=" .. tostring(button:GetFrameStrata())
                    end
                end

                return "ok"
                "#,
            )
            .unwrap();
        assert_eq!(result, "ok");
    }
}
}
