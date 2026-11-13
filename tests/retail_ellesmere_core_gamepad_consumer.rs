#[cfg(feature = "client-retail")]
#[test]
fn retail_addon_can_call_is_using_gamepad() {
    use std::fs;
    use wow_ui_sim::loader::load_addon;
    use wow_ui_sim::lua_api::WowLuaEnv;

    let addon = tempfile::tempdir().expect("create addon fixture");
    let toc = addon.path().join("EllesmereCoreProbe.toc");
    fs::write(&toc, "## Interface: 120105\nPadProbe.lua\n").expect("write fixture TOC");
    fs::write(
        addon.path().join("PadProbe.lua"),
        "local function PadNative() return IsUsingGamepad() end\nassert(type(PadNative()) == 'boolean')\n",
    )
    .expect("write consumer fixture");

    let env = WowLuaEnv::new().expect("create Retail Lua environment");
    let result = load_addon(&env.loader_env(), &toc).expect("load consumer fixture");

    assert!(
        result.warnings.is_empty(),
        "EllesmereUI's Retail input consumer failed: {}",
        result.warnings.join("\n")
    );
    assert_eq!(result.lua_files, 1);
}
