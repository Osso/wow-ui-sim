#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]
//! Original consumer: Blizzard_CompactRaidFrames/Mainline/Blizzard_CompactRaidFrameManager.lua
//! (CRFManagerRaidIconButtonMixin world-marker tab: OnClick, UpdateRaidIcon).

use crate::common::blizzard_addon_harness as harness;

const WORLD_MARKER_TAB: &str = r#"
    local markers = CompactRaidFrameManager.displayFrame.raidMarkers
    markers.raidMarkerGroundTab:Click()
    assert(markers.activeTab == markers.raidMarkerGroundTab, "world marker tab selected")
    local byWorldMarker = {}
    for i = 1, NUM_RAID_ICONS do
        local button = markers["raidMarker" .. i]
        byWorldMarker[WORLD_RAID_MARKER_ORDER[button:GetMarker()]] = button
    end
    local function applied(marker)
        return byWorldMarker[marker].backgroundTexture:GetAtlas() == "GM-button-marker-applied"
    end
    A_Admin.SetActiveRaidMarkers({ 8, 4 })
    CompactRaidFrameManager_UpdateRaidIcons()
    assert(applied(8) and applied(4) and not applied(1), "supplied markers render applied")
    assert(IsRaidMarkerActive(8) and IsRaidMarkerActive(4) and not IsRaidMarkerActive(1))
    byWorldMarker[1]:Click("RightButton")
    assert(IsRaidMarkerActive(8) and IsRaidMarkerActive(4), "inactive marker right-click is a no-op")
    byWorldMarker[8]:Click("RightButton")
    assert(not IsRaidMarkerActive(8) and IsRaidMarkerActive(4) and not applied(8) and applied(4))
    A_Admin.SetActiveRaidMarkers({ 1, 2, 3 })
    markers.raidMarkerReset:Click("LeftButton")
    for marker = 1, 8 do
        assert(not IsRaidMarkerActive(marker) and not applied(marker), "reset clears " .. marker)
    end
    assert(not pcall(A_Admin.SetActiveRaidMarkers, { 9 }), "marker index outside 1..8 rejected")
    return true
"#;

#[test]
fn raid_manager_world_marker_buttons_follow_active_markers() {
    harness::with_blizzard_addon_startup_shape(
        &["Blizzard_CompactRaidFrames"],
        &[],
        |env, loaded| {
            assert!(
                loaded
                    .iter()
                    .any(|name| name == "Blizzard_CompactRaidFrames")
            );
            let result: bool = env
                .eval(WORLD_MARKER_TAB)
                .expect("original raid manager world-marker consumer runs");
            assert!(result);
        },
    );
}
