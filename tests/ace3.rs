//! Tests for loading Ace3 library suite.

use std::path::PathBuf;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

fn ace3_path() -> PathBuf {
    PathBuf::from(env!("HOME"))
        .join("Projects/wow/reference-addons/Ace3/Ace3.toc")
}

#[test]
fn test_load_ace3() {
    if !ace3_path().exists() {
        eprintln!("Skipping: Ace3 not found at {:?}", ace3_path());
        return;
    }

    let env = WowLuaEnv::new().unwrap();
    let result = load_addon(&env.loader_env(), &ace3_path());

    match result {
        Ok(r) => {
            println!("Ace3 loaded: {} Lua files, {} XML files", r.lua_files, r.xml_files);
            if !r.warnings.is_empty() {
                println!("Warnings:");
                for w in &r.warnings {
                    println!("  {}", w);
                }
            }
            assert!(r.lua_files > 0, "Should load some Lua files");
        }
        Err(e) => {
            panic!("Failed to load Ace3: {}", e);
        }
    }
}

#[test]
fn test_libstub_from_ace3() {
    if !ace3_path().exists() {
        eprintln!("Skipping: Ace3 not found");
        return;
    }

    let env = WowLuaEnv::new().unwrap();

    // Just load LibStub
    let libstub_path = ace3_path().parent().unwrap().join("LibStub/LibStub.lua");
    let code = std::fs::read_to_string(&libstub_path).unwrap();
    env.exec(&code).expect("LibStub should load");

    // Verify LibStub works
    let exists: bool = env.eval("return LibStub ~= nil").unwrap();
    assert!(exists, "LibStub should be defined");

    // Test creating a library
    env.exec(r#"
        local lib = LibStub:NewLibrary("TestLib-1.0", 1)
        if lib then
            lib.Test = function() return "hello" end
        end
    "#).expect("Should create library");

    let result: String = env.eval(r#"
        local lib = LibStub("TestLib-1.0")
        return lib.Test()
    "#).unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_callbackhandler_from_ace3() {
    let toc_path = ace3_path();
    if !toc_path.exists() {
        eprintln!("Skipping: Ace3 not found");
        return;
    }

    let env = WowLuaEnv::new().unwrap();
    let ace3_dir = toc_path.parent().unwrap();

    // Load LibStub first
    let libstub_path = ace3_dir.join("LibStub/LibStub.lua");
    let code = std::fs::read_to_string(&libstub_path).unwrap();
    env.exec(&code).expect("LibStub should load");

    // Load CallbackHandler
    let cbh_path = ace3_dir.join("CallbackHandler-1.0/CallbackHandler-1.0.lua");
    let code = std::fs::read_to_string(&cbh_path).unwrap();
    env.exec(&code).expect("CallbackHandler should load");

    // Verify it works
    let exists: bool = env.eval("return LibStub('CallbackHandler-1.0') ~= nil").unwrap();
    assert!(exists, "CallbackHandler should be loadable via LibStub");
}

#[test]
fn test_ace_addon_loads() {
    let toc_path = ace3_path();
    if !toc_path.exists() {
        eprintln!("Skipping: Ace3 not found");
        return;
    }

    let env = WowLuaEnv::new().unwrap();
    let ace3_dir = toc_path.parent().unwrap();

    // Load LibStub
    let libstub_code = std::fs::read_to_string(ace3_dir.join("LibStub/LibStub.lua")).unwrap();
    env.exec(&libstub_code).expect("LibStub should load");

    // Load CallbackHandler
    let cbh_code = std::fs::read_to_string(ace3_dir.join("CallbackHandler-1.0/CallbackHandler-1.0.lua")).unwrap();
    env.exec(&cbh_code).expect("CallbackHandler should load");

    // Load AceAddon
    let ace_addon_code = std::fs::read_to_string(ace3_dir.join("AceAddon-3.0/AceAddon-3.0.lua")).unwrap();
    let result = env.exec(&ace_addon_code);

    match result {
        Ok(_) => {
            // Verify it works
            let exists: bool = env.eval("return LibStub('AceAddon-3.0') ~= nil").unwrap();
            assert!(exists, "AceAddon should be loadable via LibStub");
        }
        Err(e) => {
            // Print the error for debugging
            panic!("AceAddon failed to load: {}", e);
        }
    }
}

#[test]
fn test_load_details() {
    let details_path = PathBuf::from(env!("HOME"))
        .join("Projects/wow/reference-addons/Details/Details.toc");

    if !details_path.exists() {
        eprintln!("Skipping: Details not found at {:?}", details_path);
        return;
    }

    let env = WowLuaEnv::new().unwrap();
    let result = load_addon(&env.loader_env(), &details_path);

    match result {
        Ok(r) => {
            println!("Details loaded: {} Lua files, {} XML files", r.lua_files, r.xml_files);
            if !r.warnings.is_empty() {
                println!("Warnings ({}):", r.warnings.len());
                for w in r.warnings.iter().take(10) {
                    println!("  {}", w);
                }
                if r.warnings.len() > 10 {
                    println!("  ... and {} more", r.warnings.len() - 10);
                }
            }
        }
        Err(e) => {
            println!("Details failed to load: {}", e);
        }
    }
}

#[test]
fn test_load_game_menu() {
    let blizzard_ui = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("Interface/BlizzardUI");

    if !blizzard_ui.exists() {
        eprintln!("Skipping: BlizzardUI not found at {:?}", blizzard_ui);
        return;
    }

    let env = WowLuaEnv::new().unwrap();

    // First load Blizzard_SharedXML (already tested to work)
    let shared_xml_base_toc = blizzard_ui.join("Blizzard_SharedXMLBase/Blizzard_SharedXMLBase.toc");
    match load_addon(&env.loader_env(), &shared_xml_base_toc) {
        Ok(r) => println!("SharedXMLBase: {} Lua, {} XML", r.lua_files, r.xml_files),
        Err(e) => println!("SharedXMLBase failed: {}", e),
    }

    let shared_xml_toc = blizzard_ui.join("Blizzard_SharedXML/Blizzard_SharedXML_Mainline.toc");
    match load_addon(&env.loader_env(), &shared_xml_toc) {
        Ok(r) => println!("SharedXML: {} Lua, {} XML (warnings: {})", r.lua_files, r.xml_files, r.warnings.len()),
        Err(e) => println!("SharedXML failed: {}", e),
    }

    // Now try loading GameMenu
    let game_menu_toc = blizzard_ui.join("Blizzard_GameMenu/Blizzard_GameMenu_Mainline.toc");
    let result = load_addon(&env.loader_env(), &game_menu_toc);

    match result {
        Ok(r) => {
            println!("GameMenu loaded: {} Lua files, {} XML files", r.lua_files, r.xml_files);
            if !r.warnings.is_empty() {
                println!("Warnings ({}):", r.warnings.len());
                for w in &r.warnings {
                    println!("  {}", w);
                }
            }
        }
        Err(e) => {
            println!("GameMenu failed to load: {}", e);
        }
    }

    // Check if GameMenuFrame exists
    let mixin_exists: bool = env.eval("return GameMenuFrameMixin ~= nil").unwrap_or(false);
    let frame_exists: bool = env.eval("return GameMenuFrame ~= nil").unwrap_or(false);
    let main_menu_mixin: bool = env.eval("return MainMenuFrameMixin ~= nil").unwrap_or(false);

    println!("GameMenuFrameMixin exists: {}", mixin_exists);
    println!("GameMenuFrame exists: {}", frame_exists);
    println!("MainMenuFrameMixin exists: {}", main_menu_mixin);

    // Try to show the menu (if frame exists)
    if frame_exists {
        let _ = env.exec("GameMenuFrame:Show()");
        let visible: bool = env.eval("return GameMenuFrame:IsVisible()").unwrap_or(false);
        println!("GameMenuFrame visible after Show(): {}", visible);
    }

    assert!(mixin_exists, "GameMenuFrameMixin should exist");
}

#[test]
fn test_load_dbm_core() {
    let dbm_path = PathBuf::from(env!("HOME"))
        .join("Projects/wow/reference-addons/DeadlyBossMods/DBM-Core/DBM-Core_Mainline.toc");

    if !dbm_path.exists() {
        eprintln!("Skipping: DBM-Core not found at {:?}", dbm_path);
        return;
    }

    let env = WowLuaEnv::new().unwrap();
    let result = load_addon(&env.loader_env(), &dbm_path);

    match result {
        Ok(r) => {
            println!("DBM-Core loaded: {} Lua files, {} XML files", r.lua_files, r.xml_files);
            if !r.warnings.is_empty() {
                println!("Warnings ({}):", r.warnings.len());
                for w in r.warnings.iter().take(10) {
                    println!("  {}", w);
                }
                if r.warnings.len() > 10 {
                    println!("  ... and {} more", r.warnings.len() - 10);
                }
            }
        }
        Err(e) => {
            println!("DBM-Core failed to load: {}", e);
        }
    }
}

/// Load a Lua file into the env, ignoring missing files.
fn load_lua_file(env: &WowLuaEnv, path: &std::path::Path, must_succeed: bool) {
    if let Ok(code) = std::fs::read_to_string(path) {
        if must_succeed {
            env.exec(&code).unwrap_or_else(|e| panic!("{}: {}", path.display(), e));
        } else {
            let _ = env.exec(&code);
        }
    }
}

/// Load core Ace3 libraries from Details' Libs directory.
fn load_details_core_libs(env: &WowLuaEnv, libs_dir: &std::path::Path) {
    let lib_paths = [
        ("LibStub/LibStub.lua", true),
        ("CallbackHandler-1.0/CallbackHandler-1.0.lua", false),
        ("AceTimer-3.0/AceTimer-3.0.lua", false),
        ("AceSerializer-3.0/AceSerializer-3.0.lua", false),
        ("AceComm-3.0/AceComm-3.0.lua", false),
        ("LibSharedMedia-3.0/LibSharedMedia-3.0.lua", false),
        ("LibDataBroker-1.1/LibDataBroker-1.1.lua", false),
        ("LibDBIcon-1.0/LibDBIcon-1.0.lua", false),
    ];
    for (rel, must_succeed) in lib_paths {
        load_lua_file(env, &libs_dir.join(rel), must_succeed);
    }
}

/// Load a Lua file as a WoW addon chunk with (addonName, privateTable) varargs.
fn load_addon_lua_chunk(env: &WowLuaEnv, path: &std::path::Path, addon_name: &str) {
    let code = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {}", path.display(), e));
    let escaped = code.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
    let wrapper = format!(
        r#"local chunk, err = loadstring("{escaped}", "@{addon_name}/Init.lua")
        if not chunk then error("compile: " .. tostring(err)) end
        local ok, result = pcall(chunk, "{addon_name}", {{}})
        if not ok then error("runtime: " .. tostring(result)) end"#,
    );
    match env.exec(&wrapper) {
        Ok(_) => println!("{addon_name} Init.lua loaded"),
        Err(e) => println!("{addon_name} Init.lua failed: {e}"),
    }
}

#[test]
fn test_load_weakauras_init() {
    let details_path = PathBuf::from(env!("HOME"))
        .join("Projects/wow/reference-addons/Details/Details.toc");
    let weakauras_dir = PathBuf::from(env!("HOME"))
        .join("Projects/wow/reference-addons/WeakAuras2/WeakAuras");

    if !details_path.exists() || !weakauras_dir.exists() {
        eprintln!("Skipping: Details or WeakAuras not found");
        return;
    }

    let env = WowLuaEnv::new().unwrap();
    load_details_core_libs(&env, &details_path.parent().unwrap().join("Libs"));
    load_addon_lua_chunk(&env, &weakauras_dir.join("Init.lua"), "WeakAuras");

    let wa_exists: bool = env.eval("return WeakAuras ~= nil").unwrap_or(false);
    println!("WeakAuras table exists: {}", wa_exists);
    if wa_exists {
        let is_retail: bool = env.eval("return WeakAuras.IsRetail and WeakAuras.IsRetail() or false").unwrap_or(false);
        let libs_ok: bool = env.eval("return WeakAuras.IsLibsOK and WeakAuras.IsLibsOK() or false").unwrap_or(false);
        println!("IsRetail: {is_retail}, IsLibsOK: {libs_ok}");
    }
}

#[test]
fn test_load_weakauras_full() {
    let weakauras_toc = PathBuf::from(env!("HOME"))
        .join("Projects/wow/reference-addons/WeakAuras2/WeakAuras/WeakAuras.toc");

    if !weakauras_toc.exists() {
        eprintln!("Skipping: WeakAuras not found at {:?}", weakauras_toc);
        return;
    }

    let env = WowLuaEnv::new().unwrap();
    let result = load_addon(&env.loader_env(), &weakauras_toc);

    match result {
        Ok(r) => {
            println!(
                "WeakAuras loaded: {} Lua files, {} XML files",
                r.lua_files, r.xml_files
            );
            if !r.warnings.is_empty() {
                println!("Warnings ({}):", r.warnings.len());
                for w in r.warnings.iter().take(20) {
                    println!("  {}", w);
                }
                if r.warnings.len() > 20 {
                    println!("  ... and {} more", r.warnings.len() - 20);
                }
            }

            // Check if WeakAuras table exists
            let wa_exists: bool = env.eval("return WeakAuras ~= nil").unwrap_or(false);
            println!("WeakAuras table exists: {}", wa_exists);

            if wa_exists {
                let is_retail: bool = env
                    .eval("return WeakAuras.IsRetail and WeakAuras.IsRetail() or false")
                    .unwrap_or(false);
                println!("WeakAuras.IsRetail(): {}", is_retail);

                let libs_ok: bool = env
                    .eval("return WeakAuras.IsLibsOK and WeakAuras.IsLibsOK() or false")
                    .unwrap_or(false);
                println!("WeakAuras.IsLibsOK(): {}", libs_ok);
            }

        }
        Err(e) => {
            println!("WeakAuras failed to load: {}", e);
        }
    }
}

#[test]
fn test_load_plater() {
    let plater_path = PathBuf::from(env!("HOME"))
        .join("Projects/wow/reference-addons/Plater/Plater.toc");

    if !plater_path.exists() {
        eprintln!("Skipping: Plater not found at {:?}", plater_path);
        return;
    }

    let env = WowLuaEnv::new().unwrap();
    let result = load_addon(&env.loader_env(), &plater_path);

    match result {
        Ok(r) => {
            println!("Plater loaded: {} Lua files, {} XML files", r.lua_files, r.xml_files);
            if !r.warnings.is_empty() {
                println!("Warnings ({}):", r.warnings.len());
                for w in r.warnings.iter().take(15) {
                    println!("  {}", w);
                }
                if r.warnings.len() > 15 {
                    println!("  ... and {} more", r.warnings.len() - 15);
                }
            }

            // Check if Plater table exists
            let plater_exists: bool = env.eval("return Plater ~= nil").unwrap_or(false);
            println!("Plater table exists: {}", plater_exists);
        }
        Err(e) => {
            println!("Plater failed to load: {}", e);
        }
    }
}
