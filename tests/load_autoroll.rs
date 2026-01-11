//! Test loading the actual AutoRoll addon.

use std::fs;
use wow_ui_sim::lua_api::WowLuaEnv;

/// Attempt to load AutoRoll.lua and see what breaks.
#[test]
fn test_load_autoroll_main() {
    let env = WowLuaEnv::new().unwrap();

    // First, set up the addon loader context (varargs)
    // In WoW, the loader passes (addonName, addonTable) as ...
    env.exec(
        r#"
        -- Simulate addon loading context
        local addonName = "AutoRoll"
        local addonTable = {}

        -- Make them available as varargs would
        _G.__addon_name = addonName
        _G.__addon_table = addonTable

        -- Override ... behavior by wrapping in a function
        function __load_addon_file(code)
            -- This won't work directly, we need a different approach
        end
        "#,
    )
    .unwrap();

    // Read the AutoRoll.lua file
    let autoroll_path = std::env::var("HOME").unwrap() + "/Projects/wow/AutoRoll/AutoRoll.lua";
    let autoroll_code = match fs::read_to_string(&autoroll_path) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Could not read AutoRoll.lua: {} (path: {})", e, autoroll_path);
            return;
        }
    };

    // The addon starts with `local addon, ns = ...`
    // We need to make ... return our values. We do this by wrapping in a function.
    let wrapped_code = format!(
        r#"
        -- Mock game APIs that AutoRoll uses
        function GetInstanceInfo()
            return "Test Instance", "none", 1, "Normal", 5, false, false, 0
        end

        function GetLootRollItemInfo(rollID)
            return nil, "Test Item", 1, 1, 1, true, true, true, 1, 1, 1, 1, true
        end

        function RollOnLoot(rollID, rollType)
            -- Mock: do nothing
        end

        -- Stub for GroupLootHistoryFrame
        GroupLootHistoryFrame = CreateFrame("Frame", "GroupLootHistoryFrame")

        -- Load the addon code by wrapping it in a function that receives varargs
        local function loadAddon(...)
            {}
        end

        -- Call with addon name and namespace table
        loadAddon("AutoRoll", {{}})
        "#,
        autoroll_code
    );

    match env.exec(&wrapped_code) {
        Ok(_) => println!("AutoRoll.lua loaded successfully!"),
        Err(e) => {
            eprintln!("Failed to load AutoRoll.lua: {}", e);
            // Don't panic - we want to see what's missing
        }
    }

    // Check if the frame was created
    let frame_exists: bool = env
        .eval("return AutoRollFrame ~= nil")
        .unwrap_or(false);

    println!("AutoRollFrame exists: {}", frame_exists);

    // Fire ADDON_LOADED to trigger initialization
    if frame_exists {
        match env.fire_event("ADDON_LOADED") {
            Ok(_) => println!("ADDON_LOADED event fired"),
            Err(e) => eprintln!("Error firing ADDON_LOADED: {}", e),
        }
    }
}

/// Test loading SlashCommand.lua
#[test]
fn test_load_autoroll_slash() {
    let env = WowLuaEnv::new().unwrap();

    // Set up context
    env.exec(
        r#"
        local _, ns = "AutoRoll", {}
        _G.ns = ns

        -- Mock functions used by slash command
        ns.print = function(...)
            print("[AutoRoll]", ...)
        end

        ns.AddOnEnabled = function() end

        ns.isInSupportedRaid = function()
            return false, nil, nil
        end

        AutoRollDB = {
            enabled = true,
            debugMode = false,
        }

        function GetInstanceInfo()
            return "Test", "none", 1, "Normal", 5, false, false, 0
        end
        "#,
    )
    .unwrap();

    // Read and load SlashCommand.lua
    let slash_path = std::env::var("HOME").unwrap() + "/Projects/wow/AutoRoll/SlashCommand.lua";
    let slash_code = match fs::read_to_string(&slash_path) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Could not read SlashCommand.lua: {}", e);
            return;
        }
    };

    // Wrap to inject varargs
    let wrapped = format!(
        r#"
        local _, ns = "AutoRoll", _G.ns
        {}
        "#,
        slash_code
    );

    match env.exec(&wrapped) {
        Ok(_) => println!("SlashCommand.lua loaded successfully!"),
        Err(e) => {
            eprintln!("Failed to load SlashCommand.lua: {}", e);
        }
    }

    // Check if slash command was registered
    let registered: bool = env
        .eval("return SlashCmdList.AUTOROLL ~= nil")
        .unwrap_or(false);

    println!("Slash command registered: {}", registered);

    // Check if SLASH_AUTOROLL1 was set
    let alias_set: bool = env
        .eval("return SLASH_AUTOROLL1 == '/autoroll'")
        .unwrap_or(false);

    println!("SLASH_AUTOROLL1 set: {}", alias_set);
}
