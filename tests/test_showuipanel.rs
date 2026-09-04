//! Integration tests for the ShowUIPanel / HideUIPanel panel slot system.
//!
//! Verifies that:
//! - UIPanelWindows is populated with panel entries after addons load
//! - ShowUIPanel shows a panel and HideUIPanel hides it
//! - Panel area attributes (left/center/right) are registered correctly

use crate::common;

use std::path::PathBuf;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

/// Blizzard addons needed for the panel system (dependency order).
const PANEL_ADDONS: &[(&str, &str)] = &[
    ("Blizzard_SharedXMLBase", "Blizzard_SharedXMLBase.toc"),
    ("Blizzard_Colors", "Blizzard_Colors.toc"),
    ("Blizzard_SharedXML", "Blizzard_SharedXML.toc"),
    (
        "Blizzard_SharedXMLGame",
        "Blizzard_SharedXMLGame.toc",
    ),
    (
        "Blizzard_UIPanelTemplates",
        "Blizzard_UIPanelTemplates_Mainline.toc",
    ),
    (
        "Blizzard_FrameXMLBase",
        "Blizzard_FrameXMLBase_Mainline.toc",
    ),
    ("Blizzard_FrameEffects", "Blizzard_FrameEffects.toc"),
    ("Blizzard_LoadLocale", "Blizzard_LoadLocale.toc"),
    ("Blizzard_Fonts_Shared", "Blizzard_Fonts_Shared.toc"),
    ("Blizzard_HelpPlate", "Blizzard_HelpPlate.toc"),
    (
        "Blizzard_AccessibilityTemplates",
        "Blizzard_AccessibilityTemplates.toc",
    ),
    ("Blizzard_ObjectAPI", "Blizzard_ObjectAPI_Mainline.toc"),
    ("Blizzard_UIParent", "Blizzard_UIParent.toc"),
    ("Blizzard_TextStatusBar", "Blizzard_TextStatusBar.toc"),
    ("Blizzard_MoneyFrame", "Blizzard_MoneyFrame_Mainline.toc"),
    ("Blizzard_POIButton", "Blizzard_POIButton.toc"),
    ("Blizzard_Flyout", "Blizzard_Flyout.toc"),
    ("Blizzard_GameMenuEsc", "Blizzard_GameMenuEsc.toc"),
    ("Blizzard_Communities", "Blizzard_Communities_Mainline.toc"),
    ("Blizzard_StoreUI", "Blizzard_StoreUI.toc"),
    ("Blizzard_MicroMenu", "Blizzard_MicroMenu_Mainline.toc"),
    ("Blizzard_ManagedFrameSystem", "Blizzard_ManagedFrameSystem_Mainline.toc"),
    ("Blizzard_EditMode", "Blizzard_EditMode.toc"),
    ("Blizzard_GarrisonBase", "Blizzard_GarrisonBase.toc"),
    ("Blizzard_GameTooltip", "Blizzard_GameTooltip_Mainline.toc"),
    ("Blizzard_StaticPopup_Game", "Blizzard_StaticPopup_Game.toc"),
    ("Blizzard_TransmogShared", "Blizzard_TransmogShared.toc"),
    (
        "Blizzard_UIParentPanelManager",
        "Blizzard_UIParentPanelManager_Mainline.toc",
    ),
    (
        "Blizzard_Settings_Shared",
        "Blizzard_Settings_Shared.toc",
    ),
    (
        "Blizzard_SettingsDefinitions_Shared",
        "Blizzard_SettingsDefinitions_Shared.toc",
    ),
    (
        "Blizzard_SettingsDefinitions_Frame",
        "Blizzard_SettingsDefinitions_Frame.toc",
    ),
    (
        "Blizzard_FrameXMLUtil",
        "Blizzard_FrameXMLUtil.toc",
    ),
    ("Blizzard_ItemButton", "Blizzard_ItemButton_Mainline.toc"),
    ("Blizzard_QuickKeybind", "Blizzard_QuickKeybind.toc"),
    ("Blizzard_FrameXML", "Blizzard_FrameXML.toc"),
    (
        "Blizzard_UIPanels_Game",
        "Blizzard_UIPanels_Game_Mainline.toc",
    ),
    ("Blizzard_ActionBar", "Blizzard_ActionBar_Mainline.toc"),
    ("Blizzard_UnitFrame", "Blizzard_UnitFrame_Mainline.toc"),
    ("Blizzard_TokenUI", "Blizzard_TokenUI.toc"),
];

fn setup_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    let ui = blizzard_ui_dir();
    for (name, toc) in PANEL_ADDONS {
        let toc_path = ui.join(name).join(toc);
        if !toc_path.exists() {
            continue;
        }
        if let Err(e) = load_addon(&env.loader_env(), &toc_path) {
            eprintln!("[load {name}] FAILED: {e}");
        }
    }

    env.apply_post_load_workarounds();
    env.exec(r#"CHARACTERFRAME_SUBFRAMES = { "PaperDollFrame", "ReputationFrame", "TokenFrame" }"#)
        .expect("character subframe fixture should restore cleanup-pruned constant");
    fire_startup_events(&env);
    env
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

fn install_test_error_handler(env: &WowLuaEnv) {
    common::install_error_collector(env, "__test_errors");
}

fn drain_test_errors(env: &WowLuaEnv) -> Vec<String> {
    common::drain_string_table(env, "__test_errors")
}

#[test]
fn uipanel_windows_has_entries() {
    test_timeout! {
        let env = setup_env();
        let count: i32 = env.eval(r#"
            local count = 0
            for _ in pairs(UIPanelWindows) do count = count + 1 end
            return count
        "#).unwrap();
        eprintln!("UIPanelWindows entries: {count}");
        assert!(count > 0, "UIPanelWindows should have entries after loading Blizzard addons");
    }
}

#[test]
fn uipanel_windows_has_area_attributes() {
    test_timeout! {
        let env = setup_env();
        // CharacterFrame is registered by Blizzard_UIPanels_Game with area info
        let has_area: bool = env.eval(r#"
            local entry = UIPanelWindows["CharacterFrame"]
            if not entry then return false end
            -- Entry should have an area field
            return entry.area ~= nil
        "#).unwrap();
        assert!(has_area, "CharacterFrame should have an area attribute in UIPanelWindows");
    }
}

#[test]
fn key_blizzard_panels_registered() {
    test_timeout! {
        let env = setup_env();
        // Core Blizzard panels loaded from UIPanelWindows.lua should all be registered.
        // LoD panels (AchievementFrame, CollectionsJournal, etc.) register when their
        // addon loads — they won't be here, and that's correct WoW behavior.
        let missing: String = env.eval(r#"
            local expected = {
                -- From Blizzard_UIParentPanelManager/Mainline/UIPanelWindows.lua
                {"CharacterFrame", "left"},
                {"FriendsFrame", "left"},
                {"GameMenuFrame", "center"},
                {"HelpFrame", "center"},
                {"ProfessionsBookFrame", "left"},
                {"PVEFrame", "left"},
                {"MailFrame", "left"},
                {"MerchantFrame", "left"},
                {"BankFrame", "left"},
                {"GossipFrame", "left"},
                {"DressUpFrame", "left"},
                {"QuestFrame", "left"},
                {"CommunitiesFrame", "left"},
            }
            local missing = {}
            for _, pair in ipairs(expected) do
                local name, area = pair[1], pair[2]
                local entry = UIPanelWindows[name]
                if not entry then
                    table.insert(missing, name .. "(not registered)")
                elseif entry.area ~= area then
                    table.insert(missing, name .. "(area=" .. tostring(entry.area) .. " expected=" .. area .. ")")
                end
            end
            return table.concat(missing, ", ")
        "#).unwrap();
        assert!(missing.is_empty(), "Missing or wrong UIPanelWindows entries: {missing}");
    }
}

prefork_full_ui_case! {
fn show_ui_panel_shows_frame(env: &WowLuaEnv) {
        let result: bool = env.eval(r#"
            if not CharacterFrame then return false end
            -- Should start hidden
            if CharacterFrame:IsShown() then return false end
            ShowUIPanel(CharacterFrame)
            return CharacterFrame:IsShown() == true
        "#).unwrap();
        assert!(result, "ShowUIPanel should show CharacterFrame");
    }
}

prefork_full_ui_case! {
fn show_ui_panel_anchors_character_frame_to_ui_parent(env: &WowLuaEnv) {
        let result: String = env
            .eval(
                r#"
                if not CharacterFrame then
                    return "missing_character_frame"
                end

                ShowUIPanel(CharacterFrame)
                if not CharacterFrame:IsShown() then
                    return "character_not_shown"
                end

                local numPoints = CharacterFrame:GetNumPoints()
                if numPoints ~= 1 then
                    return "num_points=" .. tostring(numPoints)
                end

                local point, relativeTo, relativePoint, x, y = CharacterFrame:GetPoint(1)
                if point ~= "TOPLEFT" then
                    return "point=" .. tostring(point)
                end
                if not relativeTo or relativeTo ~= UIParent then
                    return "relative_to=" .. tostring(relativeTo and relativeTo:GetName())
                end
                if relativePoint ~= "TOPLEFT" then
                    return "relative_point=" .. tostring(relativePoint)
                end
                if x ~= 16 or y ~= -116 then
                    return string.format("offsets=%s,%s", tostring(x), tostring(y))
                end

                local left, bottom, width, height = CharacterFrame:GetRect()
                if not (left and bottom and width and height) then
                    return "missing_rect"
                end

                return "ok"
            "#,
            )
            .unwrap();
        assert_eq!(
            result, "ok",
            "ShowUIPanel should anchor CharacterFrame to UIParent and resolve its rect: {result}"
        );
    }
}

prefork_full_ui_case! {
fn show_ui_panel_positions_character_frame_at_expected_rect(env: &WowLuaEnv) {
        let result: String = env
            .eval(
                r#"
                if not CharacterFrame then
                    return "missing_character_frame"
                end

                ShowUIPanel(CharacterFrame)
                if not CharacterFrame:IsShown() then
                    return "character_not_shown"
                end

                local left, bottom, width, height = CharacterFrame:GetRect()
                if not (left and bottom and width and height) then
                    return "missing_rect"
                end

                if left ~= 16 then
                    return "left=" .. tostring(left)
                end
                if bottom ~= 228 then
                    return "bottom=" .. tostring(bottom)
                end
                if width ~= 540 then
                    return "width=" .. tostring(width)
                end
                if height ~= 424 then
                    return "height=" .. tostring(height)
                end

                return "ok"
            "#,
            )
            .unwrap();
        assert_eq!(
            result, "ok",
            "ShowUIPanel should place CharacterFrame at the expected screen rect: {result}"
        );
    }
}

prefork_full_ui_case! {
fn show_ui_panel_locks_character_frame_layout(env: &WowLuaEnv) {
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

                local frame = CharacterFrame
                if not frame then
                    return "missing_character_frame"
                end

                ShowUIPanel(frame)
                if not frame:IsShown() then
                    return "character_not_shown"
                end

                local f, err = rect("CharacterFrame", frame)
                if not f then
                    return err
                end

                if not approx(f.l, 16) then return "frame_left=" .. tostring(f.l) end
                if not approx(f.b, 228) then return "frame_bottom=" .. tostring(f.b) end
                if not approx(f.w, 540, 0.1) then return "frame_width=" .. tostring(f.w) end
                if not approx(f.h, 424, 0.1) then return "frame_height=" .. tostring(f.h) end

                local closeRect, closeErr = rect("CharacterFrameCloseButton", CharacterFrameCloseButton)
                if not closeRect then return closeErr end
                if not CharacterFrameCloseButton:IsShown() then
                    return "close_button_hidden"
                end
                if not approx(closeRect.w, 24, 0.1) or not approx(closeRect.h, 24, 0.1) then
                    return "close_button_size=" .. tostring(closeRect.w) .. "x" .. tostring(closeRect.h)
                end
                if not approx(closeRect.r, f.r + 1) then return "close_button_right=" .. tostring(closeRect.r) end
                if not approx(closeRect.t, f.t) then return "close_button_top=" .. tostring(closeRect.t) end

                local tab1Rect, tab1Err = rect("CharacterFrameTab1", CharacterFrameTab1)
                if not tab1Rect then return tab1Err end
                local tab2Rect, tab2Err = rect("CharacterFrameTab2", CharacterFrameTab2)
                if not tab2Rect then return tab2Err end
                if not CharacterFrameTab1:IsShown() then return "tab1_hidden" end
                if not CharacterFrameTab2:IsShown() then return "tab2_hidden" end
                if not approx(tab1Rect.l, f.l + 11) then return "tab1_left=" .. tostring(tab1Rect.l) end
                if not approx(tab1Rect.t, f.b + 2) then return "tab1_top=" .. tostring(tab1Rect.t) end
                if not approx(tab2Rect.l, tab1Rect.r + 3) then return "tab2_left=" .. tostring(tab2Rect.l) end
                if not approx(tab2Rect.b, tab1Rect.b) then return "tab2_bottom=" .. tostring(tab2Rect.b) end

                local titleRect, titleErr = rect("CharacterFrameTitleText", CharacterFrameTitleText)
                if not titleRect then return titleErr end
                local titleText = CharacterFrameTitleText:GetText() or ""
                if titleText == "" then
                    return "title_text_empty"
                end
                if not approx(titleRect.l, f.l + 58) then return "title_left=" .. tostring(titleRect.l) end
                if not approx(titleRect.t, f.t - 6) then return "title_top=" .. tostring(titleRect.t) end

                local modelRect, modelErr = rect("CharacterModelScene", CharacterModelScene)
                if not modelRect then return modelErr end
                if not CharacterModelScene:IsShown() then
                    return "model_scene_hidden"
                end
                if not approx(modelRect.l, f.l + 52) then return "model_left=" .. tostring(modelRect.l) end
                if not approx(modelRect.b, f.b + 38) then return "model_bottom=" .. tostring(modelRect.b) end
                if not approx(modelRect.w, 231, 0.1) then return "model_width=" .. tostring(modelRect.w) end
                if not approx(modelRect.h, 320, 0.1) then return "model_height=" .. tostring(modelRect.h) end

                local paperRect, paperErr = rect("PaperDollFrame", PaperDollFrame)
                if not paperRect then return paperErr end
                if not approx(paperRect.l, f.l) or not approx(paperRect.b, f.b)
                    or not approx(paperRect.w, f.w) or not approx(paperRect.h, f.h) then
                    return "paperdoll_rect_drift"
                end

                -- PaperDollFrame expands CharacterFrame and shows the right-side stats pane.
                if not CharacterFrameInsetRight then
                    return "inset_right_missing"
                end
                if not CharacterFrameInsetRight:IsShown() then
                    return "inset_right_hidden"
                end
                if not CharacterStatsPane then
                    return "stats_pane_missing"
                end
                if not CharacterStatsPane:IsShown() then
                    return "stats_pane_hidden"
                end

                local function expect_slot(name, expected_left, expected_bottom)
                    local slot = _G[name]
                    local r, e = rect(name, slot)
                    if not r then return false, e end
                    if not slot:IsShown() then
                        return false, name .. "_hidden"
                    end
                    if not approx(r.w, 37, 0.1) or not approx(r.h, 37, 0.1) then
                        return false, name .. "_size=" .. tostring(r.w) .. "x" .. tostring(r.h)
                    end
                    if not approx(r.l, expected_left) then
                        return false, name .. "_left=" .. tostring(r.l)
                    end
                    if not approx(r.b, expected_bottom) then
                        return false, name .. "_bottom=" .. tostring(r.b)
                    end
                    return true, nil
                end

                -- Left column slots.
                local left_x = f.l + 8
                local left_rows = {
                    {"CharacterHeadSlot", 553},
                    {"CharacterNeckSlot", 512},
                    {"CharacterShoulderSlot", 471},
                    {"CharacterChestSlot", 389},
                    {"CharacterShirtSlot", 348},
                    {"CharacterTabardSlot", 307},
                    {"CharacterWristSlot", 266},
                }
                for _, row in ipairs(left_rows) do
                    local ok, e = expect_slot(row[1], left_x, row[2])
                    if not ok then return e end
                end

                -- Right-column slots stay anchored to the expanded frame's right edge.
                local right_x = f.r - 47
                local right_rows = {
                    {"CharacterHandsSlot", 553},
                    {"CharacterWaistSlot", 512},
                    {"CharacterLegsSlot", 471},
                    {"CharacterFeetSlot", 430},
                    {"CharacterFinger0Slot", 389},
                    {"CharacterFinger1Slot", 348},
                    {"CharacterTrinket0Slot", 307},
                    {"CharacterTrinket1Slot", 266},
                }
                for _, row in ipairs(right_rows) do
                    local ok, e = expect_slot(row[1], right_x, row[2])
                    if not ok then return e end
                end

                -- Weapon slots centered at the bottom of the panel.
                local ok, e = expect_slot("CharacterMainHandSlot", f.l + 130, f.b + 16)
                if not ok then return e end
                ok, e = expect_slot("CharacterSecondaryHandSlot", f.l + 172, f.b + 16)
                if not ok then return e end

                return "ok"
            "#,
            )
            .unwrap();
        assert_eq!(
            result, "ok",
            "CharacterFrame layout should stay fully locked after ShowUIPanel: {result}"
        );
    }
}

prefork_full_ui_case! {
fn show_ui_panel_reanchors_character_frame_after_reopen(env: &WowLuaEnv) {
        let result: String = env
            .eval(
                r#"
                if not CharacterFrame then
                    return "missing_character_frame"
                end

                ShowUIPanel(CharacterFrame)
                if not CharacterFrame:IsShown() then
                    return "character_not_shown_first"
                end
                HideUIPanel(CharacterFrame)
                if CharacterFrame:IsShown() then
                    return "character_not_hidden"
                end

                ShowUIPanel(CharacterFrame)
                if not CharacterFrame:IsShown() then
                    return "character_not_shown_second"
                end

                local numPoints = CharacterFrame:GetNumPoints()
                if numPoints ~= 1 then
                    return "num_points=" .. tostring(numPoints)
                end

                local point, relativeTo, relativePoint, x, y = CharacterFrame:GetPoint(1)
                if point ~= "TOPLEFT" then
                    return "point=" .. tostring(point)
                end
                if not relativeTo or relativeTo ~= UIParent then
                    return "relative_to=" .. tostring(relativeTo and relativeTo:GetName())
                end
                if relativePoint ~= "TOPLEFT" then
                    return "relative_point=" .. tostring(relativePoint)
                end
                if x ~= 16 or y ~= -116 then
                    return string.format("offsets=%s,%s", tostring(x), tostring(y))
                end

                return "ok"
            "#,
            )
            .unwrap();
        assert_eq!(
            result, "ok",
            "ShowUIPanel should restore CharacterFrame anchors after reopen: {result}"
        );
    }
}

prefork_full_ui_case! {
fn hide_ui_panel_hides_frame(env: &WowLuaEnv) {
        let result: bool = env.eval(r#"
            if not CharacterFrame then return false end
            ShowUIPanel(CharacterFrame)
            if not CharacterFrame:IsShown() then return false end
            HideUIPanel(CharacterFrame)
            return CharacterFrame:IsShown() == false
        "#).unwrap();
        assert!(result, "HideUIPanel should hide CharacterFrame after ShowUIPanel");
    }
}

#[test]
fn show_and_hide_ui_panel_toggle_registered_frame_visibility() {
    test_timeout! {
        let env = setup_env();
        let result: String = env.eval(r#"
            local panel = CreateFrame("Frame", "ShowHideUIPanelTestFrame", UIParent)
            panel:SetSize(300, 400)
            panel:Hide()
            RegisterUIPanel(panel, { area = "center", pushable = 0, whileDead = 1, allowOtherPanels = 1 })

            if panel:IsShown() then
                return "panel_started_shown"
            end

            ShowUIPanel(panel)
            if not panel:IsShown() then
                return "show_failed"
            end

            HideUIPanel(panel)
            if panel:IsShown() then
                return "hide_failed"
            end

            return "ok"
        "#).unwrap();
        assert_eq!(
            result,
            "ok",
            "ShowUIPanel/HideUIPanel should toggle visibility for a registered panel: {result}"
        );
    }
}

#[test]
fn repeated_show_ui_panel_pulse_closes_active_panel_stack() {
    test_timeout! {
        let env = setup_env();
        let result: (bool, bool, bool, bool, bool, bool) = env.eval(r#"
            local first = CreateFrame("Frame", "AttributeDispatchPanelPulseFirst", UIParent)
            first:SetSize(300, 400)
            first:Hide()
            RegisterUIPanel(first, { area = "center", pushable = 0, whileDead = 1 })

            local second = CreateFrame("Frame", "AttributeDispatchPanelPulseSecond", UIParent)
            second:SetSize(300, 400)
            second:Hide()
            RegisterUIPanel(second, { area = "center", pushable = 0, whileDead = 1 })

            ShowUIPanel(first)
            local firstShownAfterFirst = first:IsShown()
            local secondShownAfterFirst = second:IsShown()

            ShowUIPanel(second)
            local firstShownAfterSecond = first:IsShown()
            local secondShownAfterSecond = second:IsShown()

            CloseAllWindows()
            local firstShownAfterCloseAll = first:IsShown()
            local secondShownAfterCloseAll = second:IsShown()

            return firstShownAfterFirst, secondShownAfterFirst,
                firstShownAfterSecond, secondShownAfterSecond,
                firstShownAfterCloseAll, secondShownAfterCloseAll
        "#).unwrap();
        assert_eq!(
            result,
            (true, false, false, true, false, false),
            "repeated ShowUIPanel calls should replace the center panel and CloseAllWindows should hide both panels"
        );
    }
}

#[test]
fn show_ui_panel_is_function() {
    test_timeout! {
        let env = setup_env();
        let show_type: String = env.eval("return type(ShowUIPanel)").unwrap();
        assert_eq!(show_type, "function", "ShowUIPanel should be a function");
        let hide_type: String = env.eval("return type(HideUIPanel)").unwrap();
        assert_eq!(hide_type, "function", "HideUIPanel should be a function");
    }
}

#[test]
fn register_ui_panel_populates_uipanel_windows_without_overwriting() {
    test_timeout! {
        let env = setup_env();
        let result: String = env.eval(r#"
            local panel = CreateFrame("Frame", "RegisterUIPanelTestFrame", UIParent)
            panel:SetSize(300, 400)
            panel:Hide()

            RegisterUIPanel(panel, { area = "center", pushable = 0, whileDead = 1, allowOtherPanels = 1 })
            local entry = UIPanelWindows["RegisterUIPanelTestFrame"]
            if not entry then
                return "missing_entry"
            end
            if entry.area ~= "center" or entry.pushable ~= 0 or entry.whileDead ~= 1 then
                return "wrong_attributes"
            end

            RegisterUIPanel(panel, { area = "left", pushable = 3, whileDead = 0 })
            local entry_after_second_register = UIPanelWindows["RegisterUIPanelTestFrame"]
            if entry_after_second_register.area ~= "center" or entry_after_second_register.pushable ~= 0 or entry_after_second_register.whileDead ~= 1 then
                return "overwrote_existing_entry"
            end

            ShowUIPanel(panel)
            if not panel:IsShown() then
                return "show_failed"
            end

            return "ok"
        "#).unwrap();
        assert_eq!(result, "ok", "RegisterUIPanel should populate UIPanelWindows once and enable ShowUIPanel: {result}");
    }
}

#[test]
fn registered_ui_panel_closes_with_escape_stack() {
    test_timeout! {
        let env = setup_env();
        let result: String = env.eval(r#"
            local panel = CreateFrame("Frame", "EscClosableUIPanelTestFrame", UIParent)
            panel:SetSize(300, 400)
            panel:Hide()

            RegisterUIPanel(panel, { area = "center", pushable = 0, whileDead = 1 })
            if panel.editModeManuallyShown ~= nil then
                return "edit_mode_flag"
            end

            ShowUIPanel(panel)
            if not panel:IsShown() then
                return "show_failed"
            end

            CloseAllWindows()
            if panel:IsShown() then
                return "close_failed"
            end

            return "ok"
        "#).unwrap();
        assert_eq!(result, "ok", "registered UIPanels must stay in the CloseAllWindows/Escape stack: {result}");
    }
}

#[test]
fn escape_key_closes_character_frame_before_game_menu() {
    test_timeout! {
        let env = setup_env();
        let panel_state: String = env.eval(r#"
            if HelpFrame:IsShown() then
                return "help_frame_placeholder_shown"
            end

            CHARACTERFRAME_SUBFRAMES = { "PaperDollFrame", "ReputationFrame", "TokenFrame" }
            ToggleCharacter("PaperDollFrame", true)
            if not CharacterFrame:IsShown() then
                return "not_shown"
            end
            local left_panel = GetUIPanel("left")
            if left_panel ~= CharacterFrame then
                return "wrong_left_panel:" .. tostring(left_panel and left_panel:GetName())
                    .. ":manual=" .. tostring(CharacterFrame.editModeManuallyShown)
            end
            return "ok"
        "#).unwrap();
        assert_eq!(
            panel_state,
            "ok",
            "CharacterFrame should enter the active left panel slot without a visible HelpFrame placeholder preempting Escape: {panel_state}"
        );

        env.send_key_press("ESCAPE", None).unwrap();

        let result: String = env.eval(r#"
            if CharacterFrame:IsShown() then
                return "character_frame_still_shown"
            end
            if GameMenuFrame and GameMenuFrame:IsShown() then
                return "game_menu_opened"
            end
            return "ok"
        "#).unwrap();
        assert_eq!(
            result,
            "ok",
            "Escape should close CharacterFrame through ToggleGameMenu/CloseAllWindows before opening GameMenuFrame: {result}"
        );
    }
}

#[test]
fn sequential_registered_ui_panels_reenter_close_stack() {
    test_timeout! {
        let env = setup_env();
        let result: String = env.eval(r#"
            local first = CreateFrame("Frame", "SequentialUIPanelFirst", UIParent)
            first:SetSize(300, 400)
            first:Hide()
            RegisterUIPanel(first, { area = "center", pushable = 0, whileDead = 1 })

            local second = CreateFrame("Frame", "SequentialUIPanelSecond", UIParent)
            second:SetSize(300, 400)
            second:Hide()
            RegisterUIPanel(second, { area = "center", pushable = 0, whileDead = 1 })

            ShowUIPanel(first)
            if not first:IsShown() then
                return "first_show_failed"
            end

            ShowUIPanel(second)
            if not second:IsShown() then
                return "second_show_failed"
            end

            CloseAllWindows()
            if second:IsShown() then
                return "second_close_failed"
            end

            return "ok"
        "#).unwrap();
        assert_eq!(
            result, "ok",
            "sequential registered UIPanel shows must keep the active panel in the CloseAllWindows/Escape stack: {result}"
        );
    }
}

#[test]
fn startup_without_world_map_does_not_error_in_quest_map_refresh() {
    test_timeout! {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        env.set_screen_size(1024.0, 768.0);

        {
            let mut state = env.state().borrow_mut();
            state.addon_base_paths = vec![blizzard_ui_dir()];
        }

        let ui = blizzard_ui_dir();
        for (name, toc) in PANEL_ADDONS {
            let toc_path = ui.join(name).join(toc);
            if !toc_path.exists() {
                continue;
            }
            if let Err(e) = load_addon(&env.loader_env(), &toc_path) {
                eprintln!("[load {name}] FAILED: {e}");
            }
        }

        env.apply_post_load_workarounds();
        install_test_error_handler(&env);
        fire_startup_events(&env);

        let quest_map_parent_errors: Vec<String> = drain_test_errors(&env)
            .into_iter()
            .filter(|msg| msg.contains("QuestMapFrame.lua:"))
            .collect();

        assert!(
            quest_map_parent_errors.is_empty(),
            "QuestMapFrame startup refresh should tolerate a missing world map parent: {quest_map_parent_errors:#?}"
        );
    }
}
