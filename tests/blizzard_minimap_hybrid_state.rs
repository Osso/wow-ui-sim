#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common::blizzard_addon_harness::{blizzard_ui_dir, with_blizzard_addon_startup_shape};
use wow_ui_sim::loader::{find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;

fn load_hybrid_minimap_addon(env: &WowLuaEnv) {
    let ui_dir = blizzard_ui_dir();
    // SharedMapPoiTemplates.lua owns ClearCachedActivitiesForPlayer, which
    // MapCanvasMixin:OnMapChanged calls since e75454cb6a retired the wrapper copy.
    for addon in [
        "Blizzard_MapCanvas",
        "Blizzard_SharedMapDataProviders",
        "Blizzard_HybridMinimap",
    ] {
        let toc = find_toc_file(&ui_dir.join(addon)).expect("Blizzard TOC should resolve");
        load_addon(&env.loader_env(), &toc).expect("required minimap addon should load");
    }
}

#[test]
fn original_minimap_uses_supplied_hybrid_state_for_layout() {
    with_blizzard_addon_startup_shape(&["Blizzard_Minimap"], &[], |env, loaded| {
        assert!(loaded.iter().any(|name| name == "Blizzard_Minimap"));
        load_hybrid_minimap_addon(env);
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
