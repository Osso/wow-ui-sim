#[test]
fn frame_helper_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");
    let workaround =
        include_str!("../src/lua_api/workarounds/temporary/frame_helper_defaults.rs");

    for symbol in [
        "__wow_install_frame_helpers",
        "__wow_original_CreateFrame",
        "AddDataProvider",
        "RemoveDataProvider",
    ] {
        assert!(
            !bootstrap.contains(symbol),
            "{symbol} must live in the explicit temporary frame-helper workaround boundary"
        );
        assert!(
            workaround.contains(symbol),
            "{symbol} should remain discoverable in the explicit temporary frame-helper workaround"
        );
    }
}
