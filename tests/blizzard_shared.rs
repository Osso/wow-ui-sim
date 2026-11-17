#![cfg(feature = "client-retail")]
use std::path::PathBuf;
use wow_ui_sim::loader::{find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn blizzard_toc(addon: &str) -> PathBuf {
    let addon_dir = blizzard_ui_dir().join(addon);
    find_toc_file(&addon_dir).unwrap_or_else(|| {
        panic!(
            "current active-profile TOC for `{addon}` must resolve at {}",
            addon_dir.display()
        )
    })
}

#[test]
fn test_load_blizzard_shared_xml_base() {
    let toc_path = blizzard_toc("Blizzard_SharedXMLBase");
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
    assert!(
        has_create,
        "CreateFromMixins should be defined after loading"
    );
}

#[test]
fn test_mixin_functionality_after_load() {
    let toc_path = blizzard_toc("Blizzard_SharedXMLBase");
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    load_addon(&env.loader_env(), &toc_path).expect("Failed to load addon");

    let result: bool = env
        .eval(
            r#"
        local MyMixin = { value = 42, GetValue = function(self) return self.value end }
        local obj = {}
        Mixin(obj, MyMixin)
        return obj:GetValue() == 42
    "#,
        )
        .unwrap_or(false);

    assert!(
        result,
        "Mixin should work after loading Blizzard_SharedXMLBase"
    );
}

#[test]
fn test_table_util_after_load() {
    let toc_path = blizzard_toc("Blizzard_SharedXMLBase");
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

    let base_toc = blizzard_toc("Blizzard_SharedXMLBase");
    let base_result =
        load_addon(&env.loader_env(), &base_toc).expect("Failed to load SharedXMLBase");
    println!(
        "Loaded base: {} ({} Lua, {} XML, {} warnings)",
        base_result.name,
        base_result.lua_files,
        base_result.xml_files,
        base_result.warnings.len()
    );

    let toc_path = blizzard_toc("Blizzard_SharedXML");
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
    println!(
        "Total loaded: {}, Success rate: {:.1}%",
        total_loaded, success_rate
    );
}

/// Load SharedXML then Blizzard_AddOnList, returning the env.
fn env_with_addon_list() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");

    let base_toc = blizzard_toc("Blizzard_SharedXMLBase");
    if let Err(e) = load_addon(&env.loader_env(), &base_toc) {
        eprintln!("Warning: Failed to load SharedXMLBase: {}", e);
    }

    let shared_toc = blizzard_toc("Blizzard_SharedXML");
    if let Err(e) = load_addon(&env.loader_env(), &shared_toc) {
        eprintln!("Warning: Failed to load SharedXML: {}", e);
    }

    load_addon_list_panel_globals(&env);

    let addon_list_toc = blizzard_toc("Blizzard_AddOnList");
    let result =
        load_addon(&env.loader_env(), &addon_list_toc).expect("Failed to load Blizzard_AddOnList");
    println!(
        "Loaded {}: {} Lua, {} XML, {} warnings",
        result.name,
        result.lua_files,
        result.xml_files,
        result.warnings.len()
    );
    for w in &result.warnings {
        println!("  WARN: {}", w);
    }

    env
}

fn load_addon_list_panel_globals(env: &WowLuaEnv) {
    for addon in [
        "Blizzard_GameMenuEsc",
        "Blizzard_ManagedFrameSystem",
        "Blizzard_UIParentPanelManager",
    ] {
        let toc = blizzard_toc(addon);
        load_addon(&env.loader_env(), &toc)
            .unwrap_or_else(|error| panic!("Failed to load {addon}: {error}"));
    }
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
        .eval(
            r#"
        local btn = AddonList.EnableAllButton
        return btn.Left ~= nil and btn.Center ~= nil and btn.Right ~= nil
    "#,
        )
        .unwrap_or(false);
    assert!(
        has_children,
        "EnableAllButton should have Left/Center/Right child textures"
    );

    // Show the parent panel first (AddonList is hidden="true" in XML)
    env.exec("AddonList:Show()").unwrap();
    env.exec("AddonList.EnableAllButton:Show()").unwrap();

    let left_atlas: String = env
        .eval(
            r#"
        local tex = AddonList.EnableAllButton.Left
        return tex and tex:GetAtlas() or ""
    "#,
        )
        .unwrap_or_default();
    assert!(
        left_atlas.contains("128-RedButton"),
        "Left texture should have 128-RedButton atlas, got: '{}'",
        left_atlas
    );
}

// UNKNOWN (Retail 12.1.5.70783): this injects nil although the consumer says
// Group is C++-supplied; reachability of that input is not independently proved.
#[cfg(feature = "optional-tests")]
fn run_addon_list_nil_group_probe(env: &WowLuaEnv) -> (bool, String, bool, i32) {
    env.eval(
        r#"
        AddonList:Show()
        local original = C_AddOns.GetAddOnMetadata
        C_AddOns.GetAddOnMetadata = function(addon, field)
            if field == "Group" and addon == 1 then
                return nil
            end
            return original(addon, field)
        end

        local ok, err = pcall(AddonList_Update)
        return ok, tostring(err), AddonList.ScrollBox:GetDataProvider() ~= nil,
            C_AddOns.GetNumAddOns()
        "#,
    )
    .expect("AddonList nil-group regression probe should return")
}

#[cfg(feature = "optional-tests")]
#[test]
fn test_addon_list_update_tolerates_missing_group_metadata() {
    let env = env_with_addon_list();
    let (ok, message, has_data_provider, addon_count) = run_addon_list_nil_group_probe(&env);
    assert!(
        addon_count > 0,
        "nil-group probe must visit a real addon row"
    );
    assert!(
        ok,
        "AddonList_Update should tolerate nil group metadata: {message}"
    );
    assert!(
        has_data_provider,
        "AddonList should still install a data provider when one addon has nil group metadata"
    );
}
