#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;

#[test]
fn original_minimap_uses_supplied_hybrid_state_for_layout() {
    with_blizzard_addon_startup_shape(&["Blizzard_Minimap"], &[], |env, loaded| {
        assert!(loaded.iter().any(|name| name == "Blizzard_Minimap"));
        env.exec(
            r#"
            A_Admin.SetShouldUseHybridMinimap(true)
            Minimap:OnEvent("PLAYER_ENTERING_WORLD")
            assert(C_AddOns.IsAddOnLoaded("Blizzard_HybridMinimap"))
            assert(HybridMinimap:IsEventRegistered("NEW_WMO_CHUNK"))

            A_Admin.SetShouldUseHybridMinimap(false)
            Minimap:OnEvent("PLAYER_ENTERING_WORLD")
            assert(not HybridMinimap:IsEventRegistered("NEW_WMO_CHUNK"))
            assert(not HybridMinimap:IsShown())
            "#,
        )
        .expect("the original Minimap consumer must follow supplied hybrid state");
    });
}
