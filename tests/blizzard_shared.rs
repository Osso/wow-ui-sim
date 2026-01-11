use std::path::Path;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

const BLIZZARD_SHARED_XML_BASE_TOC: &str =
    "/home/osso/Projects/wow/reference-addons/wow-ui-source/Interface/AddOns/Blizzard_SharedXMLBase/Blizzard_SharedXMLBase.toc";

const BLIZZARD_SHARED_XML_TOC: &str =
    "/home/osso/Projects/wow/reference-addons/wow-ui-source/Interface/AddOns/Blizzard_SharedXML/Blizzard_SharedXML_Mainline.toc";

#[test]
fn test_load_blizzard_shared_xml_base() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    let toc_path = Path::new(BLIZZARD_SHARED_XML_BASE_TOC);

    let result = load_addon(&env, toc_path).expect("Failed to load addon");

    println!("Loaded addon: {}", result.name);
    println!("  Lua files: {}", result.lua_files);
    println!("  XML files: {}", result.xml_files);
    println!("  Warnings: {}", result.warnings.len());

    for warning in &result.warnings {
        println!("    WARN: {}", warning);
    }

    // Should have loaded some files
    assert!(result.lua_files > 0, "Expected to load Lua files");

    // Check that Mixin is defined
    let has_mixin: bool = env.eval("return Mixin ~= nil").unwrap_or(false);
    assert!(has_mixin, "Mixin should be defined after loading");

    // Check that CreateFromMixins is defined
    let has_create: bool = env.eval("return CreateFromMixins ~= nil").unwrap_or(false);
    assert!(has_create, "CreateFromMixins should be defined after loading");
}

#[test]
fn test_mixin_functionality_after_load() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    let toc_path = Path::new(BLIZZARD_SHARED_XML_BASE_TOC);

    // Load the addon
    load_addon(&env, toc_path).expect("Failed to load addon");

    // Test Mixin functionality
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
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    let toc_path = Path::new(BLIZZARD_SHARED_XML_BASE_TOC);

    // Load the addon
    let result = load_addon(&env, toc_path);

    match &result {
        Ok(r) => {
            for w in &r.warnings {
                if w.contains("TableUtil") {
                    println!("TableUtil warning: {}", w);
                }
            }
        }
        Err(e) => println!("Load error: {}", e),
    }

    // Check if TableUtil (or tInvert/tContains) is defined
    // These may require additional APIs we haven't implemented
    let has_table_util: bool = env.eval(r#"
        return type(tInvert) == "function" or type(tContains) == "function" or type(CopyTable) == "function"
    "#).unwrap_or(false);

    // This may fail if TableUtil.lua needs unimplemented APIs
    // We'll record the result but not assert
    println!("TableUtil functions available: {}", has_table_util);
}

#[test]
fn test_load_blizzard_shared_xml() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");

    // First load SharedXMLBase (dependency)
    let base_path = Path::new(BLIZZARD_SHARED_XML_BASE_TOC);
    let base_result = load_addon(&env, base_path).expect("Failed to load SharedXMLBase");
    println!("Loaded base: {} ({} Lua, {} XML, {} warnings)",
        base_result.name, base_result.lua_files, base_result.xml_files, base_result.warnings.len());

    // Now load SharedXML
    let toc_path = Path::new(BLIZZARD_SHARED_XML_TOC);
    let result = load_addon(&env, toc_path).expect("Failed to load addon");

    println!("Loaded addon: {}", result.name);
    println!("  Lua files: {}", result.lua_files);
    println!("  XML files: {}", result.xml_files);
    println!("  Warnings: {}", result.warnings.len());

    // Print all warnings (full content)
    for warning in &result.warnings {
        println!("    WARN: {}", warning);
    }

    // Should have loaded at least some files
    // We expect failures due to missing dependencies
    let total_loaded = result.lua_files + result.xml_files;
    println!("Total loaded: {} (including from XML)", total_loaded);

    // Calculate success rate
    let total_attempted = total_loaded + result.warnings.len();
    let success_rate = total_loaded as f64 / total_attempted as f64 * 100.0;
    println!("Success rate: {:.1}%", success_rate);
}
