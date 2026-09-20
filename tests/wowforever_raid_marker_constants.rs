#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

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
