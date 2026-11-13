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

#[cfg(feature = "client-retail")]
#[test]
fn retail_addon_assets_are_known_before_its_lua_runs() {
    use std::fs;
    use wow_ui_sim::loader::load_addon;
    use wow_ui_sim::lua_api::WowLuaEnv;

    let fixture = tempfile::tempdir().expect("create add-on fixture root");
    let addon_dir = fixture.path().join("EllesmereAssetProbe");
    let media = addon_dir.join("Media");
    fs::create_dir_all(&media).expect("create media directory");
    fs::write(media.join("Panel.blp"), b"fixture asset").expect("write asset fixture");
    let toc = addon_dir.join("EllesmereAssetProbe.toc");
    fs::write(&toc, "## Interface: 120105\nMain.lua\n").expect("write fixture TOC");
    fs::write(
        addon_dir.join("Main.lua"),
        r#"
        local asset = "Interface\\AddOns\\EllesmereAssetProbe\\Media\\Panel.blp"
        assert(C_UIFileAsset.IsKnownFile(asset))
        assert(C_UIFileAsset.IsLooseFile(asset))
        "#,
    )
    .expect("write asset consumer fixture");

    let env = WowLuaEnv::new().expect("create Retail Lua environment");
    let result = load_addon(&env.loader_env(), &toc).expect("load asset consumer fixture");

    assert!(
        result.warnings.is_empty(),
        "add-on assets were not registered before Lua execution: {}",
        result.warnings.join("\n")
    );
    assert_eq!(result.lua_files, 1);
}
