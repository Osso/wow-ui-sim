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
    #[cfg(feature = "client-ptr")]
    "Blizzard_GamepadSmartNavigation",
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
    "Blizzard_GameMenuEsc",
    "Blizzard_StoreUI",
    "Blizzard_MicroMenu",
    "Blizzard_ManagedFrameSystem",
    "Blizzard_UIParentUtil",
    "Blizzard_EditMode",
    #[cfg(feature = "client-ptr")]
    "Blizzard_StatusTrackingBar",
    "Blizzard_GarrisonBase",
    "Blizzard_GameTooltip",
    #[cfg(feature = "client-ptr")]
    "Blizzard_StaticPopup",
    "Blizzard_UIParentPanelManager",
    #[cfg(feature = "client-ptr")]
    "Blizzard_GamepadSharedUtility",
    "Blizzard_Settings_Shared",
    "Blizzard_SettingsDefinitions_Shared",
    "Blizzard_SettingsDefinitions_Frame",
    "Blizzard_FrameXMLUtil",
    "Blizzard_ItemButton",
    "Blizzard_QuickKeybind",
    #[cfg(feature = "client-ptr")]
    "Blizzard_RaidWarning",
    "Blizzard_FrameXML",
    "Blizzard_UIPanels_Game",
    // PTR bag button OnLoad needs container and item-button helpers above.
    #[cfg(feature = "client-ptr")]
    "Blizzard_MainMenuBarBagButtons",
    "Blizzard_MapCanvasSecureUtil",
    "Blizzard_MapCanvas",
    "Blizzard_SharedMapDataProviders",
    "Blizzard_WorldMap",
    "Blizzard_PingUI",
    #[cfg(feature = "client-ptr")]
    "Blizzard_SpellTypeIconOverlay",
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

    if !common::load_non_retail_game_startup_inventory(&env) {
        for addon_name in ACTION_BAR_ADDONS {
            common::load_required_blizzard_addon(&env, &blizzard_ui_dir(), addon_name);
        }
    }

    env.apply_post_load_workarounds();
    common::supply_watched_reputation(&env);
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
    wrath_action_case!(right_bars_hidden);

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
    wrath_action_case!(main_visible);

    let env = env_with_action_bar();

    let visible: bool = env
        .eval("return MainActionBar ~= nil and MainActionBar:IsVisible()")
        .unwrap();
    assert!(visible, "MainActionBar should be visible after startup");
}

#[test]
fn test_main_action_bar_end_caps_visible_after_startup() {
    wrath_action_case!(endcaps);

    let env = env_with_action_bar();

    // PTR and Forever (Camelot/MainMenuBarEndCaps.xml:8-12) nest the atlas in a Texture child.
    let endcap_texture_access = if cfg!(any(feature = "client-ptr", feature = "client-wowforever"))
    {
        ".Texture"
    } else {
        ""
    };
    let script = format!(
        r#"
        if not MainActionBar or not MainActionBar.EndCaps then
            return false, "", ""
        end
        local left = MainActionBar.EndCaps.LeftEndCap{endcap_texture_access}
        local right = MainActionBar.EndCaps.RightEndCap{endcap_texture_access}
        return MainActionBar.EndCaps:IsShown(),
               (left and left.GetAtlas and left:GetAtlas()) or "",
               (right and right.GetAtlas and right:GetAtlas()) or ""
        "#
    );
    let (shown, left_atlas, right_atlas): (bool, String, String) = env.eval(&script).unwrap();

    assert!(shown, "MainActionBar.EndCaps should be shown after startup");
    assert_eq!(left_atlas, "ui-hud-actionbar-gryphon-left");
    assert_eq!(right_atlas, "ui-hud-actionbar-gryphon-right");
}

#[test]
fn test_action_buttons_visible_after_startup() {
    wrath_action_case!(buttons_visible);

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
    wrath_action_case!(showgrid);

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
    wrath_action_case!(button_size);

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
    wrath_action_case!(bootstrap_twice);

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
    wrath_action_case!(main_size);

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

post_wrath_ui_fixture! {
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
}

/// Standard status-bar geometry (Mainline/StatusTrackingBarConstants.lua, SIZE_ADJUSTMENT 6).
/// BarPriorities Experience 4 > Reputation 1, and Shared/StatusTrackingManager.lua:61-71 gives
/// the highest priority to the first (main) container.
#[cfg(not(feature = "client-wowforever"))]
const STATUS_BAR_GEOMETRY: &str = "local BAR_OFFSET_Y, BAR_W, BAR_H = 5, 565, 11\n\
    local MAIN_BAR_KEY, SECONDARY_BAR_KEY = 'Experience', 'Reputation'";

/// Camelot/StatusTrackingBarConstants.lua:1-7: containers 1192x17, SIZE_ADJUSTMENT 3, so
/// bars are 1189x14 at BOTTOMLEFT (1, 3 - 1) (Mainline/StatusTrackingManagerOverrides.lua:46-55).
/// Camelot BarPriorities (:21-28) rank Reputation 2 over Experience 0, so the main container
/// shows Reputation and the secondary shows Experience.
#[cfg(feature = "client-wowforever")]
const STATUS_BAR_GEOMETRY: &str = "local BAR_OFFSET_Y, BAR_W, BAR_H = 2, 1189, 14\n\
    local MAIN_BAR_KEY, SECONDARY_BAR_KEY = 'Reputation', 'Experience'";

#[cfg(not(feature = "client-wowforever"))]
const STATUS_CONTAINER_LAYOUT_CHECKS: &str = r#"
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

"#;

/// Camelot stacks status containers on MainActionBar (ACTION_BARS_RELATIVE_TO_BASE_POSITIONING,
/// Camelot/EditModePresetLayoutConstants.lua:3; EditModeUtil.lua:9-21 order
/// MainActionBar, Secondary, Main). Shared/EditModeManager.lua:692-715 places Secondary at
/// 45 + BOTTOM_ACTION_BARS_SPACER_Y 4 = 49 and Main at 49 + 17 +
/// PRIMARY_AND_SECONDARY_STATUS_TRACKING_BAR_SPACER_Y -1 = 65 (constants :31-32).
#[cfg(feature = "client-wowforever")]
const STATUS_CONTAINER_LAYOUT_CHECKS: &str = r#"
            if not has_point(manager, "BOTTOM", UIParent, "BOTTOM", 0, 0, 0.1) then
                return "manager_anchor_mismatch"
            end
            if not approx(managerRect.w, 1192, 0.1) or not approx(managerRect.h, 17, 0.1) then
                return "manager_size=" .. tostring(managerRect.w) .. "x" .. tostring(managerRect.h)
            end

            if main:GetNumPoints() ~= 1 or secondary:GetNumPoints() ~= 1 then
                return "container_points=" .. tostring(main:GetNumPoints()) .. "," .. tostring(secondary:GetNumPoints())
            end
            if not has_point(secondary, "BOTTOMLEFT", MainActionBar, "BOTTOMLEFT", 0, 49, 0.1) then
                return "secondary_anchor_mismatch:" .. points_debug(secondary)
            end
            if not has_point(main, "BOTTOMLEFT", MainActionBar, "BOTTOMLEFT", 0, 65, 0.1) then
                return "main_anchor_mismatch:" .. points_debug(main)
            end
            if not approx(mainRect.w, 1192, 0.1) or not approx(mainRect.h, 17, 0.1) then
                return "main_size=" .. tostring(mainRect.w) .. "x" .. tostring(mainRect.h)
            end
            if not approx(secondaryRect.w, 1192, 0.1) or not approx(secondaryRect.h, 17, 0.1) then
                return "secondary_size=" .. tostring(secondaryRect.w) .. "x" .. tostring(secondaryRect.h)
            end
            local barL = MainActionBar:GetLeft()
            if not approx(mainRect.l, barL, 0.1) or not approx(secondaryRect.l, barL, 0.1) then
                return "container_x_mismatch"
            end
            if not approx(mainRect.b, secondaryRect.t - 1, 0.1) then
                return "container_stack_gap=" .. tostring(mainRect.b - secondaryRect.t)
            end
"#;

/// Retail and Forever start with no reputation rows; the host supplies them
/// (docs/specs/modern-reputation-host-state.md, docs/admin-api/reputation.md).
/// Other profiles keep their seeded legacy fixture.
fn supply_watched_reputation_row(env: &WowLuaEnv) {
    if cfg!(any(feature = "client-retail", feature = "client-wowforever")) {
        env.exec(
            r#"A_Admin.SetReputationFactions({
                { factionID = 72, name = "Stormwind", reaction = 5,
                  currentReactionThreshold = 3000, nextReactionThreshold = 9000,
                  currentStanding = 4500 },
            })"#,
        )
        .expect("reputation row input should be accepted");
    }
    env.exec("C_Reputation.SetWatchedFactionByID(72); StatusTrackingBarManager:UpdateBarsShown()")
        .expect("original status tracking update should run");
    settle_status_bar_fades(env);
}

#[test]
fn original_status_manager_uses_distinct_host_supplied_reputation_rows() {
    let env = env_with_action_bar();
    env.exec(
        r#"A_Admin.SetReputationFactions({
            { factionID = 72, name = "", reaction = 5,
              currentReactionThreshold = 0, nextReactionThreshold = 1000,
              currentStanding = 250, isWatched = true },
        }); StatusTrackingBarManager:UpdateBarsShown()"#,
    )
    .expect("first supplied row should reach the original status manager");
    settle_status_bar_fades(&env);
    let without_name = status_manager_shows_reputation(&env);

    env.exec(
        r#"A_Admin.SetReputationFactions({
            { factionID = 2570, name = "Hallowfall Arathi", reaction = 6,
              currentReactionThreshold = 1000, nextReactionThreshold = 5000,
              currentStanding = 3000, isWatched = true },
        }); StatusTrackingBarManager:UpdateBarsShown()"#,
    )
    .expect("second supplied row should reach the original status manager");
    settle_status_bar_fades(&env);
    let with_name = status_manager_shows_reputation(&env);

    assert_eq!((without_name, with_name), (false, true));
}

fn status_manager_shows_reputation(env: &WowLuaEnv) -> bool {
    env.eval(
        r#"local manager = StatusTrackingBarManager
        local reputation = StatusTrackingBarInfo.BarsEnum.Reputation
        for _, bar in ipairs(manager.shownBarIndices) do
            if bar == reputation then return true end
        end
        return false"#,
    )
    .expect("read selected status manager output")
}

/// Moving XP from the main to the secondary container fades both out first, then
/// `onFinishedAnimating` re-runs UpdateBarsShown (Shared/StatusTrackingManager.lua:41-44, 74-85).
fn settle_status_bar_fades(env: &WowLuaEnv) {
    for _ in 0..12 {
        env.fire_on_update(0.25).expect("OnUpdate tick should run");
    }
}

#[test]
fn test_status_tracking_xp_and_reputation_bars_layout_locked() {
    wrath_action_case!(status_layout);

    let env = env_with_action_bar();
    supply_watched_reputation_row(&env);
    let result: String = env
        .eval(
            &r#"
            local EPS = 0.75
            __BAR_GEOMETRY__

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
            __CONTAINER_LAYOUT_CHECKS__

            local barsEnum = StatusTrackingBarInfo and StatusTrackingBarInfo.BarsEnum
            if not barsEnum then
                return "bars_enum_missing"
            end
            local mainIndex = barsEnum[MAIN_BAR_KEY]
            local secondaryIndex = barsEnum[SECONDARY_BAR_KEY]
            if not mainIndex or not secondaryIndex then
                return "xp_or_rep_index_missing"
            end

            if main.shownBarIndex ~= mainIndex then
                return "main_shown_bar=" .. tostring(main.shownBarIndex)
            end
            if secondary.shownBarIndex ~= secondaryIndex then
                return "secondary_shown_bar=" .. tostring(secondary.shownBarIndex)
            end

            local mainBar = main.bars and main.bars[mainIndex]
            local secondaryBar = secondary.bars and secondary.bars[secondaryIndex]
            if not mainBar then
                return "main_exp_bar_missing"
            end
            if not secondaryBar then
                return "secondary_rep_bar_missing"
            end
            if not mainBar:IsShown() then
                return "main_exp_bar_hidden"
            end
            if not secondaryBar:IsShown() then
                return "secondary_rep_bar_hidden"
            end

            local mainBarRect, mainBarErr = rect(mainBar, "main_exp_bar")
            if not mainBarRect then return mainBarErr end
            local secondaryBarRect, secondaryBarErr = rect(secondaryBar, "secondary_rep_bar")
            if not secondaryBarRect then return secondaryBarErr end

            if mainBar:GetNumPoints() ~= 1 then
                return "main_exp_points=" .. tostring(mainBar:GetNumPoints())
            end
            if secondaryBar:GetNumPoints() ~= 1 then
                return "secondary_rep_points=" .. tostring(secondaryBar:GetNumPoints())
            end
            if not has_point(mainBar, "BOTTOMLEFT", main, "BOTTOMLEFT", 1, BAR_OFFSET_Y, 0.1) then
                return "main_exp_anchor_mismatch"
            end
            if not has_point(secondaryBar, "BOTTOMLEFT", secondary, "BOTTOMLEFT", 1, BAR_OFFSET_Y, 0.1) then
                return "secondary_rep_anchor_mismatch"
            end
            if not approx(mainBarRect.w, BAR_W, 0.1) or not approx(mainBarRect.h, BAR_H, 0.1) then
                return "main_exp_size=" .. tostring(mainBarRect.w) .. "x" .. tostring(mainBarRect.h)
            end
            if not approx(secondaryBarRect.w, BAR_W, 0.1) or not approx(secondaryBarRect.h, BAR_H, 0.1) then
                return "secondary_rep_size=" .. tostring(secondaryBarRect.w) .. "x" .. tostring(secondaryBarRect.h)
            end

            if not mainBar.StatusBar or not secondaryBar.StatusBar then
                return "xp_or_rep_statusbar_missing"
            end
            local mainStatusRect, mainStatusErr = rect(mainBar.StatusBar, "main_exp_status")
            if not mainStatusRect then return mainStatusErr end
            local repStatusRect, repStatusErr = rect(secondaryBar.StatusBar, "secondary_rep_status")
            if not repStatusRect then return repStatusErr end
            if mainBar.StatusBar:GetNumPoints() ~= 1 or secondaryBar.StatusBar:GetNumPoints() ~= 1 then
                return "status_points=" .. tostring(mainBar.StatusBar:GetNumPoints()) .. "," .. tostring(secondaryBar.StatusBar:GetNumPoints())
            end
            if not has_point(mainBar.StatusBar, "RIGHT", mainBar, "RIGHT", 0, 0, 0.1) then
                return "main_exp_status_anchor_mismatch"
            end
            if not has_point(secondaryBar.StatusBar, "RIGHT", secondaryBar, "RIGHT", 0, 0, 0.1) then
                return "secondary_rep_status_anchor_mismatch"
            end
            if not approx(mainStatusRect.w, mainBarRect.w, 0.1) or not approx(mainStatusRect.h, mainBarRect.h, 0.1) then
                return "main_status_size_mismatch"
            end
            if not approx(repStatusRect.w, secondaryBarRect.w, 0.1) or not approx(repStatusRect.h, secondaryBarRect.h, 0.1) then
                return "rep_status_size_mismatch"
            end

            return "ok"
        "#
            .replace("__BAR_GEOMETRY__", STATUS_BAR_GEOMETRY)
            .replace("__CONTAINER_LAYOUT_CHECKS__", STATUS_CONTAINER_LAYOUT_CHECKS),
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "XP/Reputation status tracking bars should remain layout-locked: {result}"
    );
}

#[test]
fn test_bag_bar_layout_locked() {
    wrath_action_case!(bag_layout);

    let env = env_with_action_bar();
    if cfg!(any(feature = "client-retail", feature = "client-ptr")) {
        let (loaded, applied): (bool, bool) = env
            .eval("return BagsBar ~= nil, BagsBar ~= nil and BagsBar.systemInfo ~= nil")
            .unwrap();
        assert_eq!(
            loaded,
            cfg!(feature = "client-ptr"),
            "bag fixture load stage"
        );
        assert_eq!(
            applied,
            cfg!(feature = "client-ptr"),
            "bag preset application stage"
        );
    }
    // Retail loads this system below, AFTER startup. OnSystemLoad only registers it
    // (Shared/EditModeSystemTemplates.lua:3-30); RegisterSystemFrame only inserts it
    // (Retail Shared/EditModeManager.lua:210-212; PTR:220-222). Retail therefore checks
    // the XML anchor BEFORE the next layout update, not the startup steady state.
    // PTR loads it before startup and applies Standard's preset offset (10).
    let result: String = env
        .eval(
            &r#"
            local EPS = 0.75
            local expectedAnchor = $BAG_ANCHOR

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
            if not toggleRect then
                local width, height = toggle:GetSize()
                local parent = toggle:GetParent()
                local parentName = parent and parent:GetName() or "nil"
                return toggleErr .. ":barLayout=" .. type(bar.Layout)
                    .. ":size=" .. tostring(width) .. "x" .. tostring(height)
                    .. ":initial=" .. tostring(bar.bagBarExpandToggleInitialWidth)
                    .. "x" .. tostring(bar.bagBarExpandToggleInitialHeight)
                    .. ":parent=" .. parentName .. ":points=" .. points_debug(toggle)
            end
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

            local point, relName, relPoint, offsetX, offsetY = unpack(expectedAnchor)
            local relFrame = _G[relName]
            if not has_point(bar, point, relFrame, relPoint, offsetX, offsetY, 0.1) then
                return "bags_bar_anchor_mismatch:" .. points_debug(bar)
            end
            local relRect, relErr = rect(relFrame, "bags_bar_anchor_target")
            if not relRect then return relErr end
            local function point_xy(r, name)
                local x = name:find("LEFT") and r.l or name:find("RIGHT") and r.r or (r.l + r.r) / 2
                local y = name:find("TOP") and r.t or name:find("BOTTOM") and r.b or (r.b + r.t) / 2
                return x, y
            end
            local barX, barY = point_xy(barRect, point)
            local relX, relY = point_xy(relRect, relPoint)
            local wantX, wantY = relX + offsetX, relY + offsetY
            -- EditModeSystemTemplate is clampedToScreen (Shared/EditModeSystemTemplates.xml:4).
            if bar:IsClampedToScreen() then
                local dx, dy = barX - barRect.l, barY - barRect.b
                local maxL, maxB = UIParent:GetRight() - barRect.w, UIParent:GetTop() - barRect.h
                wantX = math.max(0, math.min(wantX - dx, maxL)) + dx
                wantY = math.max(0, math.min(wantY - dy, maxB)) + dy
            end
            if not approx(barX, wantX, 0.1) or not approx(barY, wantY, 0.1) then
                return "bags_bar_point=" .. tostring(barX) .. "," .. tostring(barY)
                    .. ",anchor_point=" .. tostring(wantX) .. "," .. tostring(wantY)
            end

            __BAG_CHAIN_CHECKS__

            return "ok"
        "#
            .replace("$BAG_ANCHOR", bag_bar_expected_anchor())
            .replace("__BAG_CHAIN_CHECKS__", BAG_CHAIN_CHECKS),
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "Bags bar and bag-slot chain should remain layout-locked: {result}"
    );
}

/// Standard bag chain: 208x47 bar, expand toggle and 30px bag slots
/// (Mainline/MainMenuBarBagButtons.xml).
#[cfg(not(feature = "client-wowforever"))]
const BAG_CHAIN_CHECKS: &str = r#"
            if not approx(barRect.w, 208, 0.1) or not approx(barRect.h, 47, 0.1) then
                return "bags_bar_size=" .. tostring(barRect.w) .. "x" .. tostring(barRect.h)
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

"#;

/// Camelot/MainMenuBarBagButtons.xml:51-92: 45px slots, bagPadding 2, hidden expand toggle,
/// 33x45 KeyRing at the left end. Shared/BagsBar.lua:43-59 sums all seven buttons plus six
/// paddings: 6*45 + 33 + 6*2 = 315.
#[cfg(feature = "client-wowforever")]
const BAG_CHAIN_CHECKS: &str = r#"
            if not approx(barRect.w, 315, 0.1) or not approx(barRect.h, 45, 0.1) then
                return "bags_bar_size=" .. tostring(barRect.w) .. "x" .. tostring(barRect.h)
            end
            local keyRing = KeyRingButton
            if not keyRing then return "keyring_missing" end
            if not backpack:IsShown() then return "backpack_hidden" end
            if toggle:IsShown() then return "toggle_shown" end
            if not bag0:IsShown() or not bag1:IsShown() or not bag2:IsShown() or not bag3:IsShown() then
                return "one_or_more_bag_slots_hidden"
            end
            if not reagent:IsShown() then return "reagent_slot_hidden" end
            local chain = {
                { backpack, "backpack", bar, "RIGHT", 0, 45 },
                { bag0, "bag0", backpack, "LEFT", -2, 45 },
                { bag1, "bag1", bag0, "LEFT", -2, 45 },
                { bag2, "bag2", bag1, "LEFT", -2, 45 },
                { bag3, "bag3", bag2, "LEFT", -2, 45 },
                { reagent, "reagent", bag3, "LEFT", -2, 45 },
                { keyRing, "keyring", reagent, "LEFT", 0, 33 },
            }
            for _, link in ipairs(chain) do
                local frame, tag, rel, relPoint, x, width = unpack(link)
                if not has_point(frame, "RIGHT", rel, relPoint, x, 0, 0.1) then
                    return tag .. "_anchor_mismatch:" .. points_debug(frame)
                end
                local r, err = rect(frame, tag)
                if not r then return err end
                if not approx(r.w, width, 0.1) or not approx(r.h, 45, 0.1) then
                    return tag .. "_size=" .. tostring(r.w) .. "x" .. tostring(r.h)
                end
            end
            -- GetBagBarLength counts a padding before KeyRing, but its XML anchor x is 0.
            local keyRect = rect(keyRing, "keyring")
            if not approx(keyRect.l - barRect.l, 2, 0.1) then
                return "keyring_left_gap=" .. tostring(keyRect.l - barRect.l)
            end
"#;

/// BagsBar anchor at this test's stage, as a Lua `{point, relativeTo, relativePoint, x, y}`.
/// Retail checks the XML anchor (Mainline/MainMenuBarBagButtons.xml:15) before the next
/// layout update; PTR applies Standard's preset offset 10
/// (Standard/EditModePresetLayoutConstants.lua:38-42). Forever loads it at startup and applies
/// the Camelot preset (Camelot/EditModePresetLayoutConstants.lua:45-49).
fn bag_bar_expected_anchor() -> &'static str {
    if cfg!(feature = "client-ptr") {
        r#"{ "TOPRIGHT", "MicroButtonAndBagsBar", "TOPRIGHT", 0, 10 }"#
    } else if cfg!(feature = "client-wowforever") {
        r#"{ "BOTTOMLEFT", "MicroMenuContainer", "BOTTOMRIGHT", 7, -4 }"#
    } else {
        r#"{ "TOPRIGHT", "MicroButtonAndBagsBar", "TOPRIGHT", 0, 0 }"#
    }
}

/// Both pinned Standard presets use offsetY=10 (Retail constants:26; PTR:42).
/// Mainline/EditModePresetLayouts.lua:578 Retail / :598 PTR feeds that value to
/// Shared/EditModeSystemTemplates.lua:350-375 ApplySystemAnchor on both profiles.
#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn test_bag_bar_applies_selected_edit_mode_preset() {
    let env = env_with_action_bar();
    if !env.eval::<bool>("return BagsBar ~= nil").unwrap() {
        common::load_required_blizzard_addon(
            &env,
            &blizzard_ui_dir(),
            "Blizzard_MainMenuBarBagButtons",
        );
    }
    env.apply_post_load_workarounds();
    env.exec(
        r#"
        local manager = EditModeManagerFrame
        assert(manager:IsActiveLayoutPreset(), "fixture must select a preset, not a saved layout")
        assert(manager.layoutInfo.activeLayout == 1, "fixture must select the Modern preset")
        assert(#C_EditMode.GetLayouts().layouts == 0, "fixture must not load a saved cache")
        assert(BagsBar.systemInfo.anchorInfo.offsetY == 10, "selected preset must supply Y=10")
        -- Displace the frame first so a no-op ApplySystemAnchor cannot pass.
        BagsBar:ClearAllPointsBase()
        BagsBar:SetPointBase("TOPRIGHT", MicroButtonAndBagsBar, "TOPRIGHT", 0, -37)
        BagsBar:ApplySystemAnchor()
        "#,
    )
    .unwrap();
    let (point, relative, relative_point, x, y, scale): (String, bool, String, f64, f64, f64) = env
        .eval(
            r#"
        local point, relative, relativePoint, x, y = BagsBar:GetPoint(1)
        return point, relative == MicroButtonAndBagsBar, relativePoint, x, y, BagsBar:GetScale()
        "#,
        )
        .unwrap();
    assert_eq!(point, "TOPRIGHT");
    assert!(relative);
    assert_eq!(relative_point, "TOPRIGHT");
    assert_eq!(x, 0.0);
    assert!(
        (y * scale - 10.0).abs() < 0.1,
        "applied preset Y={y}, scale={scale}"
    );
}
