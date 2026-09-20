#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn forever_marker_search_tracks_assignments_and_search_bounds() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(GetNextAvailableRaidTargetMarkerIndex(1) == 1)
        for i = 1, 8 do
            A_Admin.SetTarget("Marked" .. i, 60, 1, true)
            SetRaidTarget("target", i)
        end
        assert(GetNextAvailableRaidTargetMarkerIndex(1) == 0)
        assert(GetNextAvailableRaidTargetMarkerIndex(8, true, true) == 0)
        SetRaidTarget("target", 0)
        assert(GetNextAvailableRaidTargetMarkerIndex(1) == 8)
        assert(GetNextAvailableRaidTargetMarkerIndex(7, true) == 0)
        assert(GetNextAvailableRaidTargetMarkerIndex(7, true, true) == 8)
        SetRaidTarget("target", 3)
        assert(GetNextAvailableRaidTargetMarkerIndex(3) == 8)
        assert(GetNextAvailableRaidTargetMarkerIndex(8, true) == 8)
    "#,
    )
    .unwrap();
}

#[test]
fn forever_marker_search_reuses_only_dead_nonfriendly_assignments() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("A_Admin.SetTarget('DeadHostile', 60, 1, true); SetRaidTarget('target', 1)")
        .unwrap();
    env.state()
        .borrow_mut()
        .current_target
        .as_mut()
        .unwrap()
        .health = 0;
    assert_eq!(
        env.eval::<i32>("return GetNextAvailableRaidTargetMarkerIndex(1)")
            .unwrap(),
        2
    );
    assert_eq!(
        env.eval::<i32>("return GetNextAvailableRaidTargetMarkerIndex(1, false, false, true)")
            .unwrap(),
        1
    );
    env.state()
        .borrow_mut()
        .current_target
        .as_mut()
        .unwrap()
        .reaction = 5;
    assert_eq!(
        env.eval::<i32>("return GetNextAvailableRaidTargetMarkerIndex(1, false, false, true)")
            .unwrap(),
        2
    );
    env.state()
        .borrow_mut()
        .current_target
        .as_mut()
        .unwrap()
        .reaction = 4;
    assert_eq!(
        env.eval::<i32>("return GetNextAvailableRaidTargetMarkerIndex(1, false, false, true)")
            .unwrap(),
        1
    );
    env.state()
        .borrow_mut()
        .current_target
        .as_mut()
        .unwrap()
        .health = 100;
    assert_eq!(
        env.eval::<i32>("return GetNextAvailableRaidTargetMarkerIndex(1, false, false, true)")
            .unwrap(),
        2
    );
    assert_eq!(
        env.eval::<i32>("return GetRaidTargetIndex('target')")
            .unwrap(),
        1
    );
}

#[test]
fn forever_marker_shared_consumer_uses_real_search_and_assignment() {
    let env = WowLuaEnv::new().unwrap();
    let source = std::fs::read_to_string(
        wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
            .unwrap()
            .join("Blizzard_GamepadActionBars/TargetActionBars/Shared.lua"),
    )
    .unwrap();
    env.exec(&format!("TestMarkerShared = (function()\n{source}\nend)()"))
        .unwrap();
    env.exec(
        r#"
        EventRegistry = {RegisterFrameEventAndCallback = function() return {} end}
        Kiosk = {IsEnabled = function() return false end}
        GamepadTargetLogic = {ConsumeModifier = function() end}
        local shownMarker
        SetRaidTargetIconTexture = function(_, marker) shownMarker = marker end
        A_Admin.SetTarget("Occupied", 60, 1, true)
        SetRaidTarget("target", 1)
        A_Admin.SetTarget("Selected", 60, 1, true)
        local bar = CreateFrame("Frame")
        function bar:SetButtonEnabled(button, enabled) button:SetEnabled(enabled) end
        local button = CreateFrame("Button", nil, bar)
        button.SpecialActionIcon = button:CreateTexture()
        button.IconOverlay = button:CreateTexture()
        TestMarkerShared.SetUpTargetMarkerButton(bar, button, 1)
        assert(shownMarker == 2, "initial marker: " .. tostring(shownMarker))
        assert(not button.IconOverlay:IsShown(), "initial overlay")
        button:GetScript("OnClick")(button, "LeftButton", true)
        assert(GetRaidTargetIndex("target") == 2, "assigned marker: " .. tostring(GetRaidTargetIndex("target")))
        assert(shownMarker == 3, "next marker: " .. tostring(shownMarker))
    "#,
    )
    .unwrap();
}

#[test]
fn forever_raid_marker_constants_match_documented_values() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local c = Constants.RaidMarkerConsts
        assert(c.MAX_RAID_TARGETS_USER == 8)
        assert(c.MAX_RAID_TARGETS_RESTRICTED == 8)
        assert(c.MAX_VALID_RAID_TARGETS == 0)
        assert(c.MAX_RAID_MARKERS == 8)
        "#,
    )
    .unwrap();
}

#[test]
fn forever_target_marker_buttons_clear_at_both_range_boundaries() {
    let env = WowLuaEnv::new().unwrap();
    let source = std::fs::read_to_string(
        wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
            .unwrap()
            .join("Blizzard_GamepadActionBars/TargetActionBars/Shared.lua"),
    )
    .unwrap();
    env.exec(&format!("TestMarkerShared = (function()\n{source}\nend)()"))
        .unwrap();
    env.exec(
        r#"
        -- Fixture isolates the documented range from unrelated marker search APIs.
        EventRegistry = {RegisterFrameEventAndCallback = function() return {} end}
        Kiosk = {IsEnabled = function() return false end}
        UnitExists = function() return true end
        GetNextAvailableRaidTargetMarkerIndex = function()
            error("range boundary must clear rather than search")
        end
        local shownMarker
        SetRaidTargetIconTexture = function(_, marker) shownMarker = marker end
        local bar = CreateFrame("Frame")
        function bar:SetButtonEnabled(button, enabled) button:SetEnabled(enabled) end
        for _, case in ipairs({{8, 1}, {1, -1}}) do
            GetRaidTargetIndex = function() return case[1] end
            local button = CreateFrame("Button", nil, bar)
            button.SpecialActionIcon = button:CreateTexture()
            button.IconOverlay = button:CreateTexture()
            TestMarkerShared.SetUpTargetMarkerButton(bar, button, case[2])
            assert(shownMarker == case[1])
            assert(button:IsEnabled())
            assert(button.IconOverlay:IsShown())
            assert(button.SpecialActionIcon:IsShown())
        end
        "#,
    )
    .unwrap();
}
