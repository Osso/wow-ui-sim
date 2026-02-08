use std::path::PathBuf;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

fn blizzard_ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Interface/BlizzardUI")
}

fn blizzard_toc(addon: &str, toc_name: &str) -> PathBuf {
    blizzard_ui_dir().join(addon).join(toc_name)
}

#[test]
fn test_load_blizzard_shared_xml_base() {
    let toc_path = blizzard_toc("Blizzard_SharedXMLBase", "Blizzard_SharedXMLBase.toc");
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    let result = load_addon(&env.loader_env(), &toc_path).expect("Failed to load addon");

    println!("Loaded addon: {}", result.name);
    println!("  Lua files: {}", result.lua_files);
    println!("  XML files: {}", result.xml_files);
    println!("  Warnings: {}", result.warnings.len());
    for warning in &result.warnings {
        println!("    WARN: {}", warning);
    }

    assert!(result.lua_files > 0, "Expected to load Lua files");

    let has_mixin: bool = env.eval("return Mixin ~= nil").unwrap_or(false);
    assert!(has_mixin, "Mixin should be defined after loading");

    let has_create: bool = env.eval("return CreateFromMixins ~= nil").unwrap_or(false);
    assert!(has_create, "CreateFromMixins should be defined after loading");
}

#[test]
fn test_mixin_functionality_after_load() {
    let toc_path = blizzard_toc("Blizzard_SharedXMLBase", "Blizzard_SharedXMLBase.toc");
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    load_addon(&env.loader_env(), &toc_path).expect("Failed to load addon");

    let result: bool = env.eval(r#"
        local MyMixin = { value = 42, GetValue = function(self) return self.value end }
        local obj = {}
        Mixin(obj, MyMixin)
        return obj:GetValue() == 42
    "#).unwrap_or(false);

    assert!(result, "Mixin should work after loading Blizzard_SharedXMLBase");
}

#[test]
fn test_table_util_after_load() {
    let toc_path = blizzard_toc("Blizzard_SharedXMLBase", "Blizzard_SharedXMLBase.toc");
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    let result = load_addon(&env.loader_env(), &toc_path);

    if let Ok(r) = &result {
        for w in &r.warnings {
            if w.contains("TableUtil") {
                println!("TableUtil warning: {}", w);
            }
        }
    }

    let has_table_util: bool = env.eval(r#"
        return type(tInvert) == "function" or type(tContains) == "function" or type(CopyTable) == "function"
    "#).unwrap_or(false);
    println!("TableUtil functions available: {}", has_table_util);
}

#[test]
fn test_load_blizzard_shared_xml() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");

    let base_toc = blizzard_toc("Blizzard_SharedXMLBase", "Blizzard_SharedXMLBase.toc");
    let base_result = load_addon(&env.loader_env(), &base_toc).expect("Failed to load SharedXMLBase");
    println!("Loaded base: {} ({} Lua, {} XML, {} warnings)",
        base_result.name, base_result.lua_files, base_result.xml_files, base_result.warnings.len());

    let toc_path = blizzard_toc("Blizzard_SharedXML", "Blizzard_SharedXML_Mainline.toc");
    let result = load_addon(&env.loader_env(), &toc_path).expect("Failed to load addon");

    println!("Loaded addon: {}", result.name);
    println!("  Lua files: {}", result.lua_files);
    println!("  XML files: {}", result.xml_files);
    println!("  Warnings: {}", result.warnings.len());
    for warning in &result.warnings {
        println!("    WARN: {}", warning);
    }

    let total_loaded = result.lua_files + result.xml_files;
    let total_attempted = total_loaded + result.warnings.len();
    let success_rate = total_loaded as f64 / total_attempted as f64 * 100.0;
    println!("Total loaded: {}, Success rate: {:.1}%", total_loaded, success_rate);
}

/// Load SharedXML then Blizzard_AddOnList, returning the env.
fn env_with_addon_list() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");

    let base_toc = blizzard_toc("Blizzard_SharedXMLBase", "Blizzard_SharedXMLBase.toc");
    if let Err(e) = load_addon(&env.loader_env(), &base_toc) {
        eprintln!("Warning: Failed to load SharedXMLBase: {}", e);
    }

    let shared_toc = blizzard_toc("Blizzard_SharedXML", "Blizzard_SharedXML_Mainline.toc");
    if let Err(e) = load_addon(&env.loader_env(), &shared_toc) {
        eprintln!("Warning: Failed to load SharedXML: {}", e);
    }

    let addon_list_toc = blizzard_toc("Blizzard_AddOnList", "Blizzard_AddOnList.toc");
    let result = load_addon(&env.loader_env(), &addon_list_toc).expect("Failed to load Blizzard_AddOnList");
    println!(
        "Loaded {}: {} Lua, {} XML, {} warnings",
        result.name, result.lua_files, result.xml_files, result.warnings.len()
    );
    for w in &result.warnings {
        println!("  WARN: {}", w);
    }

    env
}

#[test]
fn test_addon_list_enable_all_button_has_texture() {
    let env = env_with_addon_list();

    let exists: bool = env
        .eval("return AddonList ~= nil and AddonList.EnableAllButton ~= nil")
        .unwrap_or(false);
    assert!(exists, "AddonList.EnableAllButton should exist");

    let is_button: bool = env
        .eval("return AddonList.EnableAllButton:GetObjectType() == 'Button'")
        .unwrap_or(false);
    assert!(is_button, "EnableAllButton should be a Button");

    let has_children: bool = env
        .eval(r#"
        local btn = AddonList.EnableAllButton
        return btn.Left ~= nil and btn.Center ~= nil and btn.Right ~= nil
    "#)
        .unwrap_or(false);
    assert!(
        has_children,
        "EnableAllButton should have Left/Center/Right child textures"
    );

    // Show the parent panel first (AddonList is hidden="true" in XML)
    env.exec("AddonList:Show()").unwrap();
    env.exec("AddonList.EnableAllButton:Show()").unwrap();

    let left_atlas: String = env
        .eval(r#"
        local tex = AddonList.EnableAllButton.Left
        return tex and tex:GetAtlas() or ""
    "#)
        .unwrap_or_default();
    assert!(
        left_atlas.contains("128-RedButton"),
        "Left texture should have 128-RedButton atlas, got: '{}'",
        left_atlas
    );
}
