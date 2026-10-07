//! Integration tests for main action bar visibility after startup.

use crate::common;

use std::path::PathBuf;
use wow_ui_sim::lua_api::WowLuaEnv;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

/// Blizzard addons needed for the action bar, in dependency order.
const ACTION_BAR_ADDONS: &[&str] = &[
    "Blizzard_SharedXMLBase",
    "Blizzard_Colors",
    "Blizzard_SharedXML",
    "Blizzard_SharedXMLGame",
    "Blizzard_UIPanelTemplates",
    "Blizzard_FrameXMLBase",
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
    "Blizzard_ManagedFrameSystem",
    "Blizzard_GameMenuEsc",
    "Blizzard_UIParentUtil",
    "Blizzard_EditMode",
    "Blizzard_GarrisonBase",
    "Blizzard_GameTooltip",
    "Blizzard_UIParentPanelManager",
    "Blizzard_Settings_Shared",
    "Blizzard_SettingsDefinitions_Shared",
    "Blizzard_SettingsDefinitions_Frame",
    "Blizzard_FrameXMLUtil",
    "Blizzard_ItemButton",
    "Blizzard_QuickKeybind",
    "Blizzard_FrameXML",
    "Blizzard_UIPanels_Game",
    "Blizzard_MapCanvasSecureUtil",
    "Blizzard_MapCanvas",
    "Blizzard_SharedMapDataProviders",
    "Blizzard_WorldMap",
    "Blizzard_PingUI",
    "Blizzard_ActionBar",
];

fn build_action_bar_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);

    // Set addon_base_paths so runtime LoadAddOn() can find on-demand addons.
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    for addon_name in ACTION_BAR_ADDONS {
        common::load_required_blizzard_addon(&env, &blizzard_ui_dir(), addon_name);
    }

    env.apply_post_load_workarounds();
    fire_startup_events(&env);
    env
}

/// Load addons, fire startup events, and apply post-startup fixups.
fn env_with_action_bar() -> common::LockedEnv {
    common::lock_env(build_action_bar_env)
}

/// Replicate the startup event sequence from main.rs / app.rs.
fn fire_startup_events(env: &WowLuaEnv) {
    common::fire_addon_loaded(env, "WoWUISim");
    for event in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        let _ = env.fire_event(event);
    }
    common::call_global_if_present(env, "RequestTimePlayed");
    common::fire_player_entering_world(env, true, false);
    let _ = env.fire_edit_mode_layouts_updated();

    // WoW's C++ engine fires ACTIONBAR_SHOWGRID on startup to show empty slots.
    let _ = env.fire_event("ACTIONBAR_SHOWGRID");

    for event in [
        "UPDATE_BINDINGS",
        "DISPLAY_SIZE_CHANGED",
        "UI_SCALE_CHANGED",
    ] {
        let _ = env.fire_event(event);
    }
}

/// MultiBarLeft and MultiBarRight (right-side action bars) should be hidden
/// by default — they're only shown when the player enables them via
/// PROXY_SHOW_ACTIONBAR_4/5 settings (backed by GetActionBarToggles).
#[test]
fn test_right_action_bars_hidden_by_default() {
    let env = env_with_action_bar();

    let results: (bool, bool) = env
        .eval(
            r#"
            local left = MultiBarLeft and MultiBarLeft:IsVisible()
            local right = MultiBarRight and MultiBarRight:IsVisible()
            return left or false, right or false
        "#,
        )
        .unwrap();
    assert!(!results.0, "MultiBarLeft should be hidden by default");
    assert!(!results.1, "MultiBarRight should be hidden by default");
}

#[test]
fn test_main_action_bar_visible_after_startup() {
    let env = env_with_action_bar();

    let visible: bool = env
        .eval("return MainActionBar ~= nil and MainActionBar:IsVisible()")
        .unwrap();
    assert!(visible, "MainActionBar should be visible after startup");
}

#[test]
fn test_main_action_bar_end_caps_visible_after_startup() {
    let env = env_with_action_bar();

    let (shown, left_atlas, right_atlas): (bool, String, String) = env
        .eval(
            r#"
            if not MainActionBar or not MainActionBar.EndCaps then
                return false, "", ""
            end
            local left = MainActionBar.EndCaps.LeftEndCap
            local right = MainActionBar.EndCaps.RightEndCap
            return MainActionBar.EndCaps:IsShown(),
                   (left and left.GetAtlas and left:GetAtlas()) or "",
                   (right and right.GetAtlas and right:GetAtlas()) or ""
            "#,
        )
        .unwrap();

    assert!(shown, "MainActionBar.EndCaps should be shown after startup");
    assert_eq!(left_atlas, "ui-hud-actionbar-gryphon-left");
    assert_eq!(right_atlas, "ui-hud-actionbar-gryphon-right");
}

#[test]
fn test_action_buttons_visible_after_startup() {
    let env = env_with_action_bar();

    let count: i32 = env
        .eval(
            r#"
            local n = 0
            for i = 1, 12 do
                local btn = _G["ActionButton" .. i]
                if btn and btn:IsVisible() then
                    n = n + 1
                end
            end
            return n
        "#,
        )
        .unwrap();
    assert_eq!(count, 12, "All 12 ActionButtons should be visible");
}

#[test]
fn test_action_buttons_have_showgrid_attribute() {
    let env = env_with_action_bar();

    let all_have_grid: bool = env
        .eval(
            r#"
            for i = 1, 12 do
                local btn = _G["ActionButton" .. i]
                if not btn then return false end
                local grid = btn:GetAttribute("showgrid")
                if not grid or grid <= 0 then return false end
            end
            return true
        "#,
        )
        .unwrap();
    assert!(all_have_grid, "All ActionButtons should have showgrid > 0");
}

#[test]
fn test_action_button_size() {
    let env = env_with_action_bar();

    let size: (f64, f64) = env
        .eval(
            r#"
            local btn = ActionButton1
            if not btn then return 0, 0 end
            return btn:GetSize()
        "#,
        )
        .unwrap();
    assert_eq!(size, (45.0, 45.0), "ActionButton should be 45x45");
}

#[test]
fn test_action_bar_env_can_bootstrap_twice_in_same_process() {
    common::with_perf_lock(|| {
        let first = build_action_bar_env();
        let first_visible: bool = first
            .eval("return MainActionBar ~= nil and MainActionBar:IsVisible()")
            .unwrap();
        assert!(first_visible, "first action bar env should initialize");
        drop(first);

        let second = build_action_bar_env();
        let second_visible: bool = second
            .eval("return MainActionBar ~= nil and MainActionBar:IsVisible()")
            .unwrap();
        assert!(second_visible, "second action bar env should initialize");
    });
}

#[test]
fn test_main_action_bar_size() {
    let env = env_with_action_bar();

    let size: (f64, f64) = env
        .eval(
            r#"
            local bar = MainActionBar
            if not bar then return 0, 0 end
            return bar:GetSize()
        "#,
        )
        .unwrap();
    // 12 buttons (45px) + 11 gaps (2px) from the button grid layout.
    assert_eq!(
        size,
        (562.0, 45.0),
        "MainActionBar should match the button grid after layout"
    );
}

/// Verify the core EditMode enter/exit flow works without pcall.
///
/// AccountSettings:OnEditModeEnter/Exit are excluded — they call
/// Setup/Refresh on ~30 optional frames (DurabilityFrame, TargetFrame, etc.)
/// that may not be loaded in partial test environments.
#[test]
fn test_enter_exit_edit_mode_core_steps() {
    let env = env_with_action_bar();
    env.apply_post_event_workarounds();
    let failures: String = env
        .eval(
            r#"
            local emm = EditModeManagerFrame
            local fails = {}
            local function try(name, fn)
                local ok, err = pcall(fn)
                if not ok then fails[#fails+1] = name .. ": " .. tostring(err) end
            end
            emm.editModeActive = true
            try("ClearActiveChangesFlags", function() emm:ClearActiveChangesFlags() end)
            try("ShowSystemSelections", function() emm:ShowSystemSelections() end)
            try("TriggerEvent_Enter", function() EventRegistry:TriggerEvent("EditMode.Enter") end)
            emm.editModeActive = false
            try("HideSystemSelections", function() emm:HideSystemSelections() end)
            try("TriggerEvent_Exit", function() EventRegistry:TriggerEvent("EditMode.Exit") end)
            return table.concat(fails, "\n")
        "#,
        )
        .unwrap();
    assert!(
        failures.is_empty(),
        "EditMode core steps should not crash:\n{}",
        failures
    );
}

#[test]
fn test_status_tracking_xp_and_reputation_bars_layout_locked() {
    let env = env_with_action_bar();
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

            local function rect(frame, tag)
                if type(frame) ~= "table" then
                    return nil, tag .. "_missing"
                end
                local l, b, w, h = frame:GetRect()
                if not (l and b and w and h) then
                    return nil, tag .. "_missing_rect"
                end
                return { l = l, b = b, w = w, h = h, r = l + w, t = b + h }, nil
            end

            local function has_point(frame, point, rel, relPoint, x, y, eps)
                for i = 1, frame:GetNumPoints() do
                    local p, r, rp, ox, oy = frame:GetPoint(i)
                    local relMatches = (r == rel) or (r == nil and rel ~= nil and frame.GetParent and frame:GetParent() == rel)
                    if p == point and relMatches and rp == relPoint and approx(ox or 0, x, eps) and approx(oy or 0, y, eps) then
                        return true
                    end
                end
                return false
            end

            local function points_debug(frame)
                local out = {}
                for i = 1, frame:GetNumPoints() do
                    local p, r, rp, ox, oy = frame:GetPoint(i)
                    local rn = r and r:GetName() or "nil"
                    out[#out + 1] = string.format("%s->%s:%s(%.2f,%.2f)", tostring(p), rn, tostring(rp), tonumber(ox) or 0, tonumber(oy) or 0)
                end
                table.sort(out)
                return table.concat(out, " | ")
            end

            if C_Reputation and C_Reputation.SetWatchedFactionByID then
                C_Reputation.SetWatchedFactionByID(72)
            end

            local manager = StatusTrackingBarManager
            if not manager then
                return "manager_missing"
            end
            manager:UpdateBarsShown()

            local main = manager.MainStatusTrackingBarContainer
            local secondary = manager.SecondaryStatusTrackingBarContainer
            if not main then
                return "main_container_missing"
            end
            if not secondary then
                return "secondary_container_missing"
            end

            local managerRect, managerErr = rect(manager, "manager")
            if not managerRect then return managerErr end
            local mainRect, mainErr = rect(main, "main")
            if not mainRect then return mainErr end
            local secondaryRect, secondaryErr = rect(secondary, "secondary")
            if not secondaryRect then return secondaryErr end

            if not manager:IsShown() then
                return "manager_hidden"
            end
            if not main:IsShown() then
                return "main_container_hidden"
            end
            if not secondary:IsShown() then
                return "secondary_container_hidden"
            end

            if manager:GetNumPoints() ~= 1 then
                return "manager_points=" .. tostring(manager:GetNumPoints())
            end
            if not has_point(manager, "BOTTOM", UIParent, "BOTTOM", 0, 0, 0.1) then
                return "manager_anchor_mismatch"
            end
            if not approx(managerRect.w, 571, 0.1) or not approx(managerRect.h, 34, 0.1) then
                return "manager_size=" .. tostring(managerRect.w) .. "x" .. tostring(managerRect.h)
            end

            if main:GetNumPoints() ~= 1 or secondary:GetNumPoints() ~= 1 then
                return "container_points=" .. tostring(main:GetNumPoints()) .. "," .. tostring(secondary:GetNumPoints())
            end
            if not has_point(main, "BOTTOM", manager, "BOTTOM", 0, 0, 0.1) then
                return "main_anchor_mismatch:" .. points_debug(main)
            end
            if not has_point(secondary, "BOTTOM", manager, "BOTTOM", 0, 17, 0.1) then
                return "secondary_anchor_mismatch:" .. points_debug(secondary)
            end
            if not approx(mainRect.w, 571, 0.1) or not approx(mainRect.h, 17, 0.1) then
                return "main_size=" .. tostring(mainRect.w) .. "x" .. tostring(mainRect.h)
            end
            if not approx(secondaryRect.w, 571, 0.1) or not approx(secondaryRect.h, 17, 0.1) then
                return "secondary_size=" .. tostring(secondaryRect.w) .. "x" .. tostring(secondaryRect.h)
            end
            if not approx(mainRect.r, managerRect.r, 0.1) or not approx(mainRect.l, managerRect.l, 0.1) then
                return "main_x_mismatch"
            end
            if not approx(secondaryRect.r, managerRect.r, 0.1) or not approx(secondaryRect.l, managerRect.l, 0.1) then
                return "secondary_x_mismatch"
            end
            local stacked = approx(mainRect.b, secondaryRect.t, 0.1) or approx(secondaryRect.b, mainRect.t, 0.1)
            if not stacked then
                return "container_stack_gap=" .. tostring(mainRect.b - secondaryRect.t) .. "," .. tostring(secondaryRect.b - mainRect.t)
            end
            local minBottom = math.min(mainRect.b, secondaryRect.b)
            local maxTop = math.max(mainRect.t, secondaryRect.t)
            if not approx(minBottom, managerRect.b, 0.1) or not approx(maxTop, managerRect.t, 0.1) then
                return "container_vertical_span_mismatch"
            end

            local barsEnum = StatusTrackingBarInfo and StatusTrackingBarInfo.BarsEnum
            if not barsEnum then
                return "bars_enum_missing"
            end
            local expIndex = barsEnum.Experience
            local repIndex = barsEnum.Reputation
            if not expIndex or not repIndex then
                return "xp_or_rep_index_missing"
            end

            if main.shownBarIndex ~= expIndex then
                return "main_shown_bar=" .. tostring(main.shownBarIndex)
            end
            if secondary.shownBarIndex ~= repIndex then
                return "secondary_shown_bar=" .. tostring(secondary.shownBarIndex)
            end

            local mainExpBar = main.bars and main.bars[expIndex]
            local secondaryRepBar = secondary.bars and secondary.bars[repIndex]
            if not mainExpBar then
                return "main_exp_bar_missing"
            end
            if not secondaryRepBar then
                return "secondary_rep_bar_missing"
            end
            if not mainExpBar:IsShown() then
                return "main_exp_bar_hidden"
            end
            if not secondaryRepBar:IsShown() then
                return "secondary_rep_bar_hidden"
            end

            local mainExpRect, mainExpErr = rect(mainExpBar, "main_exp_bar")
            if not mainExpRect then return mainExpErr end
            local secondaryRepRect, secondaryRepErr = rect(secondaryRepBar, "secondary_rep_bar")
            if not secondaryRepRect then return secondaryRepErr end

            if mainExpBar:GetNumPoints() ~= 1 then
                return "main_exp_points=" .. tostring(mainExpBar:GetNumPoints())
            end
            if secondaryRepBar:GetNumPoints() ~= 1 then
                return "secondary_rep_points=" .. tostring(secondaryRepBar:GetNumPoints())
            end
            if not has_point(mainExpBar, "BOTTOMLEFT", main, "BOTTOMLEFT", 1, 5, 0.1) then
                return "main_exp_anchor_mismatch"
            end
            if not has_point(secondaryRepBar, "BOTTOMLEFT", secondary, "BOTTOMLEFT", 1, 5, 0.1) then
                return "secondary_rep_anchor_mismatch"
            end
            if not approx(mainExpRect.w, 565, 0.1) or not approx(mainExpRect.h, 11, 0.1) then
                return "main_exp_size=" .. tostring(mainExpRect.w) .. "x" .. tostring(mainExpRect.h)
            end
            if not approx(secondaryRepRect.w, 565, 0.1) or not approx(secondaryRepRect.h, 11, 0.1) then
                return "secondary_rep_size=" .. tostring(secondaryRepRect.w) .. "x" .. tostring(secondaryRepRect.h)
            end

            if not mainExpBar.StatusBar or not secondaryRepBar.StatusBar then
                return "xp_or_rep_statusbar_missing"
            end
            local mainStatusRect, mainStatusErr = rect(mainExpBar.StatusBar, "main_exp_status")
            if not mainStatusRect then return mainStatusErr end
            local repStatusRect, repStatusErr = rect(secondaryRepBar.StatusBar, "secondary_rep_status")
            if not repStatusRect then return repStatusErr end
            if mainExpBar.StatusBar:GetNumPoints() ~= 1 or secondaryRepBar.StatusBar:GetNumPoints() ~= 1 then
                return "status_points=" .. tostring(mainExpBar.StatusBar:GetNumPoints()) .. "," .. tostring(secondaryRepBar.StatusBar:GetNumPoints())
            end
            if not has_point(mainExpBar.StatusBar, "RIGHT", mainExpBar, "RIGHT", 0, 0, 0.1) then
                return "main_exp_status_anchor_mismatch"
            end
            if not has_point(secondaryRepBar.StatusBar, "RIGHT", secondaryRepBar, "RIGHT", 0, 0, 0.1) then
                return "secondary_rep_status_anchor_mismatch"
            end
            if not approx(mainStatusRect.w, mainExpRect.w, 0.1) or not approx(mainStatusRect.h, mainExpRect.h, 0.1) then
                return "main_status_size_mismatch"
            end
            if not approx(repStatusRect.w, secondaryRepRect.w, 0.1) or not approx(repStatusRect.h, secondaryRepRect.h, 0.1) then
                return "rep_status_size_mismatch"
            end

            return "ok"
        "#,
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "XP/Reputation status tracking bars should remain layout-locked: {result}"
    );
}

#[test]
fn test_bag_bar_layout_locked() {
    let env = env_with_action_bar();
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

            local function rect(frame, tag)
                if type(frame) ~= "table" then
                    return nil, tag .. "_missing"
                end
                local l, b, w, h = frame:GetRect()
                if not (l and b and w and h) then
                    return nil, tag .. "_missing_rect"
                end
                return { l = l, b = b, w = w, h = h, r = l + w, t = b + h }, nil
            end

            local function has_point(frame, point, rel, relPoint, x, y, eps)
                for i = 1, frame:GetNumPoints() do
                    local p, r, rp, ox, oy = frame:GetPoint(i)
                    local relMatches = (r == rel) or (r == nil and rel ~= nil and frame.GetParent and frame:GetParent() == rel)
                    if p == point and relMatches and rp == relPoint and approx(ox or 0, x, eps) and approx(oy or 0, y, eps) then
                        return true
                    end
                end
                return false
            end

            local function points_debug(frame)
                local out = {}
                for i = 1, frame:GetNumPoints() do
                    local p, r, rp, ox, oy = frame:GetPoint(i)
                    local rn = r and r:GetName() or "nil"
                    out[#out + 1] = string.format("%s->%s:%s(%.2f,%.2f)", tostring(p), rn, tostring(rp), tonumber(ox) or 0, tonumber(oy) or 0)
                end
                table.sort(out)
                return table.concat(out, " | ")
            end

            if not BagsBar then
                if UIParentLoadAddOn then
                    pcall(UIParentLoadAddOn, "Blizzard_MainMenuBarBagButtons")
                end
                if not BagsBar and C_AddOns and C_AddOns.LoadAddOn then
                    pcall(C_AddOns.LoadAddOn, "Blizzard_MainMenuBarBagButtons")
                end
            end

            local bar = BagsBar
            local microBar = MicroButtonAndBagsBar
            local backpack = MainMenuBarBackpackButton
            local toggle = BagBarExpandToggle
            local bag0 = CharacterBag0Slot
            local bag1 = CharacterBag1Slot
            local bag2 = CharacterBag2Slot
            local bag3 = CharacterBag3Slot
            local reagent = CharacterReagentBag0Slot

            if not bar then return "bags_bar_missing" end
            if not microBar then return "micro_button_and_bags_bar_missing" end
            if not backpack then return "backpack_button_missing" end
            if not toggle then return "bag_toggle_missing" end
            if not bag0 or not bag1 or not bag2 or not bag3 then return "bag_slots_missing" end
            if not reagent then return "reagent_slot_missing" end

            if MainMenuBarBagManager and MainMenuBarBagManager.SetExpandBar then
                MainMenuBarBagManager:SetExpandBar(true)
            end
            if bar.Layout then
                bar:Layout()
            end

            local barRect, barErr = rect(bar, "bags_bar")
            if not barRect then return barErr end
            local microRect, microErr = rect(microBar, "micro_bar")
            if not microRect then return microErr end
            local backpackRect, backpackErr = rect(backpack, "backpack")
            if not backpackRect then return backpackErr end
            local toggleRect, toggleErr = rect(toggle, "toggle")
            if not toggleRect then return toggleErr end
            local bag0Rect, bag0Err = rect(bag0, "bag0")
            if not bag0Rect then return bag0Err end
            local bag1Rect, bag1Err = rect(bag1, "bag1")
            if not bag1Rect then return bag1Err end
            local bag2Rect, bag2Err = rect(bag2, "bag2")
            if not bag2Rect then return bag2Err end
            local bag3Rect, bag3Err = rect(bag3, "bag3")
            if not bag3Rect then return bag3Err end
            local reagentRect, reagentErr = rect(reagent, "reagent")
            if not reagentRect then return reagentErr end

            if not has_point(bar, "TOPRIGHT", microBar, "TOPRIGHT", 0, 0, 0.1) then
                return "bags_bar_anchor_mismatch:" .. points_debug(bar)
            end
            if not approx(barRect.w, 208, 0.1) or not approx(barRect.h, 47, 0.1) then
                return "bags_bar_size=" .. tostring(barRect.w) .. "x" .. tostring(barRect.h)
            end
            if not approx(barRect.r, microRect.r, 0.1) then
                return "bags_bar_right=" .. tostring(barRect.r) .. ",micro_right=" .. tostring(microRect.r)
            end
            if not approx(barRect.t, microRect.t, 0.1) then
                return "bags_bar_top=" .. tostring(barRect.t) .. ",micro_top=" .. tostring(microRect.t)
            end

            if not backpack:IsShown() then return "backpack_hidden" end
            if not toggle:IsShown() then return "toggle_hidden" end
            if not bag0:IsShown() or not bag1:IsShown() or not bag2:IsShown() or not bag3:IsShown() then
                return "one_or_more_bag_slots_hidden"
            end
            if not reagent:IsShown() then return "reagent_slot_hidden" end

            if not has_point(backpack, "RIGHT", bar, "RIGHT", 0, 0, 0.1) then
                return "backpack_anchor_mismatch:" .. points_debug(backpack)
            end
            if not has_point(toggle, "RIGHT", backpack, "LEFT", 0, 0, 0.1) then
                return "toggle_anchor_mismatch:" .. points_debug(toggle)
            end
            if not has_point(bag0, "RIGHT", toggle, "LEFT", 0, 0, 0.1) then
                return "bag0_anchor_mismatch:" .. points_debug(bag0)
            end
            if not has_point(bag1, "RIGHT", bag0, "LEFT", 0, 0, 0.1) then
                return "bag1_anchor_mismatch:" .. points_debug(bag1)
            end
            if not has_point(bag2, "RIGHT", bag1, "LEFT", 0, 0, 0.1) then
                return "bag2_anchor_mismatch:" .. points_debug(bag2)
            end
            if not has_point(bag3, "RIGHT", bag2, "LEFT", 0, 0, 0.1) then
                return "bag3_anchor_mismatch:" .. points_debug(bag3)
            end
            if not has_point(reagent, "RIGHT", bag3, "LEFT", 0, 0, 0.1) then
                return "reagent_anchor_mismatch:" .. points_debug(reagent)
            end

            if not approx(backpackRect.w, 48, 0.1) or not approx(backpackRect.h, 48, 0.1) then
                return "backpack_size=" .. tostring(backpackRect.w) .. "x" .. tostring(backpackRect.h)
            end
            if not approx(toggleRect.w, 10, 0.1) or not approx(toggleRect.h, 16, 0.1) then
                return "toggle_size=" .. tostring(toggleRect.w) .. "x" .. tostring(toggleRect.h)
            end
            if not approx(bag0Rect.w, 30, 0.1) or not approx(bag0Rect.h, 30, 0.1) then
                return "bag0_size=" .. tostring(bag0Rect.w) .. "x" .. tostring(bag0Rect.h)
            end
            if not approx(bag1Rect.w, 30, 0.1) or not approx(bag1Rect.h, 30, 0.1) then
                return "bag1_size=" .. tostring(bag1Rect.w) .. "x" .. tostring(bag1Rect.h)
            end
            if not approx(bag2Rect.w, 30, 0.1) or not approx(bag2Rect.h, 30, 0.1) then
                return "bag2_size=" .. tostring(bag2Rect.w) .. "x" .. tostring(bag2Rect.h)
            end
            if not approx(bag3Rect.w, 30, 0.1) or not approx(bag3Rect.h, 30, 0.1) then
                return "bag3_size=" .. tostring(bag3Rect.w) .. "x" .. tostring(bag3Rect.h)
            end
            if not approx(reagentRect.w, 30, 0.1) or not approx(reagentRect.h, 30, 0.1) then
                return "reagent_size=" .. tostring(reagentRect.w) .. "x" .. tostring(reagentRect.h)
            end

            if not approx(backpackRect.l, barRect.r - backpackRect.w, 0.1) then
                return "backpack_not_at_bar_right_edge"
            end
            if not approx(toggleRect.r, backpackRect.l, 0.1) then
                return "toggle_gap_to_backpack=" .. tostring(toggleRect.r - backpackRect.l)
            end
            if not approx(bag0Rect.r, toggleRect.l, 0.1) then
                return "bag0_gap_to_toggle=" .. tostring(bag0Rect.r - toggleRect.l)
            end
            if not approx(bag1Rect.r, bag0Rect.l, 0.1) then
                return "bag1_gap_to_bag0=" .. tostring(bag1Rect.r - bag0Rect.l)
            end
            if not approx(bag2Rect.r, bag1Rect.l, 0.1) then
                return "bag2_gap_to_bag1=" .. tostring(bag2Rect.r - bag1Rect.l)
            end
            if not approx(bag3Rect.r, bag2Rect.l, 0.1) then
                return "bag3_gap_to_bag2=" .. tostring(bag3Rect.r - bag2Rect.l)
            end
            if not approx(reagentRect.r, bag3Rect.l, 0.1) then
                return "reagent_gap_to_bag3=" .. tostring(reagentRect.r - bag3Rect.l)
            end
            if not approx(reagentRect.l, barRect.l, 0.1) then
                return "leftmost_bag_not_flush_with_bar_left=" .. tostring(reagentRect.l - barRect.l)
            end

            local expectedBarWidth = backpackRect.w + toggleRect.w + bag0Rect.w + bag1Rect.w + bag2Rect.w + bag3Rect.w + reagentRect.w
            if not approx(barRect.w, expectedBarWidth, 0.1) then
                return "bags_bar_width_mismatch=" .. tostring(barRect.w) .. ",expected=" .. tostring(expectedBarWidth)
            end

            return "ok"
        "#,
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "Bags bar and bag-slot chain should remain layout-locked: {result}"
    );
}
