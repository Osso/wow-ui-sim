#![cfg(feature = "client-retail")]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;

const TRACKING_MENU_ASSERTIONS: &str = r#"
    local tracking = MinimapCluster.Tracking.Button
    assert(tracking, "Blizzard_Minimap tracking button is missing")
    assert(C_Minimap.GetNumTrackingTypes() >= 2, "tracking rows are empty")

    local firstFilter = C_Minimap.GetTrackingFilter(1)
    local secondFilter = C_Minimap.GetTrackingFilter(2)
    assert(firstFilter.filterID ~= secondFilter.filterID, "tracking rows share a filter")

    local first = C_Minimap.GetTrackingInfo(1)
    local second = C_Minimap.GetTrackingInfo(2)
    assert(first and second and first.name ~= second.name, "tracking rows are not distinct")
    assert(type(first.texture) == "number" and type(second.texture) == "number", "tracking texture is missing")

    A_Admin.SetMinimapTrackingMask(1)
    A_Admin.SetPlayerClass(1)
    C_Minimap.SetTracking(2, true)
    tracking:OnEvent("VARIABLES_LOADED")
    tracking:GenerateMenu()

    assert(first.active == true and C_Minimap.GetTrackingInfo(2).active == true, "tracked state is missing")
    local selected = MenuUtil.GetSelections(tracking.menuDescription)
    assert(#selected == 2, "tracking menu selection count is wrong")
    assert(selected[1].data.index == 1 and selected[2].data.index == 2, "tracking menu rows are wrong")
"#;

#[test]
fn original_minimap_tracking_menu_uses_tracking_rows_and_state() {
    with_blizzard_addon_closure(
        &["Blizzard_Menu", "Blizzard_Minimap"],
        &[],
        |env, loaded| {
            assert!(loaded.iter().any(|name| name == "Blizzard_Minimap"));
            env.exec(TRACKING_MENU_ASSERTIONS)
            .expect("Blizzard_Minimap/Mainline/Minimap.lua tracking menu should consume modeled rows and state");
        },
    );
}
