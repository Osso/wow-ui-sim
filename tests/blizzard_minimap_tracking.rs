#![cfg(feature = "client-retail")]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;

#[test]
fn original_minimap_tracking_menu_uses_tracking_rows_and_state() {
    with_blizzard_addon_closure(
        &["Blizzard_Menu", "Blizzard_Minimap"],
        &[],
        |env, loaded| {
            assert!(loaded.iter().any(|name| name == "Blizzard_Minimap"));
            env.exec(
            r#"
            local tracking = Minimap.Tracking.Button
            assert(tracking)
            assert(C_Minimap.GetNumTrackingTypes() >= 2)

            local firstFilter = C_Minimap.GetTrackingFilter(1)
            local secondFilter = C_Minimap.GetTrackingFilter(2)
            assert(firstFilter.filterID ~= secondFilter.filterID)

            local first = C_Minimap.GetTrackingInfo(1)
            local second = C_Minimap.GetTrackingInfo(2)
            assert(first and second and first.name ~= second.name)
            assert(type(first.texture) == "number" and type(second.texture) == "number")

            C_Minimap.SetTracking(1, true)
            tracking:OnEvent("VARIABLES_LOADED")
            tracking:GenerateMenu()

            assert(first.active == true)
            local selected = MenuUtil.GetSelections(tracking.menuDescription)
            assert(#selected == 1)
            assert(selected[1].data.index == 1)
            "#,
        )
        .expect("Blizzard_Minimap/Mainline/Minimap.lua tracking menu should consume modeled rows and state");
        },
    );
}
