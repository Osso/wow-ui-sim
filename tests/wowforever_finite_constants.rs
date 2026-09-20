#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn load(env: &WowLuaEnv, path: &str) {
    let root = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    env.exec(&std::fs::read_to_string(root.join(path)).unwrap())
        .unwrap();
}

#[test]
fn forever_finite_constants_construct_minimap_filters() {
    let env = WowLuaEnv::new().unwrap();
    load(&env, "Blizzard_Minimap/Camelot/MinimapConstants.lua");
    env.exec("assert(MinimapConstants.OPTIONAL_FILTERS[8388608] == true); assert(Enum.MinimapTrackingFilter.VendorAmmo == 16777216)").unwrap();
}

#[test]
fn forever_finite_constants_publish_source_values() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(Enum.PingResult.FailedSilent == 8)
        assert(Constants.Transmog.NoTransmogID == 0)
        local names = {"SpecialPageTopBar", "Page1LeftBar", "Page1RightBar", "Page1BottomBar", "Page2TopBar", "Page2LeftBar", "Page2RightBar", "Page2BottomBar", "Page3TopBar", "Page3LeftBar", "Page3RightBar", "Page3BottomBar"}
        for index, name in ipairs(names) do assert(Enum.GamepadPossessBarOverride[name] == index, name) end
        assert(Enum.GamepadPossessBarOverrideMeta.NumValues == 12)
        assert(Enum.PingResultMeta.MaxValue == 8)
    "#).unwrap();
}
