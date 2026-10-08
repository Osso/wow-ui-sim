//! The full cached Game preload mirrors wow-sim startup for LoD bootstrap-only addons.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn full_game_preload_publishes_lod_bootstrap_helpers_without_loading_the_addon(env: &WowLuaEnv) {
    let (before, after): ((String, bool, bool), (bool, bool, bool)) =
        {
            let before = env
                .eval(
                    r#"return type(PVPUI_LoadUI),
                        C_AddOns.IsAddOnLoaded("Blizzard_PVPUI"),
                        PVPUIFrame ~= nil"#,
                )
                .expect("probe bootstrap helper before load");
            let after = env
                .eval(
                    r#"local loaded = PVPUI_LoadUI()
                    return loaded == true, C_AddOns.IsAddOnLoaded("Blizzard_PVPUI"),
                        PVPUIFrame ~= nil"#,
                )
                .expect("load Blizzard_PVPUI through its bootstrap helper");
            (before, after)
        };
    assert_eq!(before, ("function".to_string(), false, false));
    assert_eq!(after, (true, true, true));
}
}
