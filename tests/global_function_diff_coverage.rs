use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use wow_ui_sim::lua_api::WowLuaEnv;

const FILTERED_STANDALONE_GLOBALS: &[&str] = &[
    "GetCVarTableValue",
    "GetChatTimestampFormat",
    "GetMobileEmbeddedTexture",
    "ResolvePrefixedChannelName",
    "SetCVarTableValue",
    "SubstituteChatMessageBeforeSend",
];

const CURATED_EXTRA_GLOBALS: &[&str] = &[
    "GetContainerItemID",
    "GetContainerItemLink",
    "GetContainerNumSlots",
    "GetItemID",
    "GetPlayerAuraBySpellID",
    "GetTradeSkillTexture",
    "IsArtifactRelicItem",
    "UnitAura",
    "UnitBuff",
    "UnitDebuff",
];

const KNOWN_ADDON_LEAKS: &[&str] = &[
    "Angleur_OnLoad",
    "AllTheThings_MinimapButtonOnClick",
    "Auctionator_EditBox_OnKeyDown",
    "DejunkBindings_OpenLootables",
    "Details_OpenDefaultOptionsWindow",
    "KrowiEVU_OnAddonCompartmentClick",
    "Plumber_ToggleLandingPage",
    "UnifiedProfileManager_OnAddonCompartmentClick",
];

const KNOWN_SIMULATOR_ONLY_GLOBALS: &[&str] = &[
    "FireEvent",
    "SetScreenSize",
    "SimulatePing",
    "__hide_child",
    "__original_ipairs",
    "__original_rawget",
    "__real_getmetatable",
    "__real_setmetatable",
    "__report_script_error",
    "clock",
    "load",
    "module",
    "pow",
    "require",
    "tconcat",
];

const REPRESENTATIVE_FILTERED_GLOBALS: &[&str] = &[
    "ChatEdit_ActivateChat",
    "ChatFrame_AddMessage",
    "Chat_AddSystemMessage",
    "GetChatTimestampFormat",
    "ResolvePrefixedChannelName",
    "SubstituteChatMessageBeforeSend",
];

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("failed to create Lua environment")
}

fn diff_global_functions_missing_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("wow-client-diff")
        .join("diff_global_functions_missing.txt")
}

fn diff_global_functions_extra_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("wow-client-diff")
        .join("diff_global_functions_extra.txt")
}

fn read_missing_global_functions() -> BTreeSet<String> {
    fs::read_to_string(diff_global_functions_missing_path())
        .expect("failed to read diff_global_functions_missing.txt")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn read_extra_global_functions() -> BTreeSet<String> {
    fs::read_to_string(diff_global_functions_extra_path())
        .expect("failed to read diff_global_functions_extra.txt")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn is_filtered_startup_global(name: &str) -> bool {
    name.starts_with("ChatEdit_")
        || name.starts_with("ChatFrame_")
        || name.starts_with("Chat_")
        || FILTERED_STANDALONE_GLOBALS.contains(&name)
}

#[test]
fn diff_global_functions_extra_is_curated_to_intentional_compatibility_aliases() {
    let extra = read_extra_global_functions();
    let expected: BTreeSet<String> = CURATED_EXTRA_GLOBALS
        .iter()
        .map(|name| (*name).to_owned())
        .collect();

    assert_eq!(
        extra, expected,
        "diff_global_functions_extra.txt should only contain the curated compatibility aliases"
    );

    for leaked_name in KNOWN_ADDON_LEAKS {
        assert!(
            !extra.contains(*leaked_name),
            "{leaked_name} should not remain in the filtered extra-global diff"
        );
    }

    for simulator_only_name in KNOWN_SIMULATOR_ONLY_GLOBALS {
        assert!(
            !extra.contains(*simulator_only_name),
            "{simulator_only_name} should not remain in the curated extra-global diff"
        );
    }
}

#[test]
fn curated_extra_global_functions_exist_at_runtime() {
    let extra = read_extra_global_functions();
    let lua_list = extra
        .iter()
        .map(|name| format!("\"{}\"", name.replace('\\', "\\\\").replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(", ");

    let env = env();
    let result: String = env
        .eval(&format!(
            r#"
            local names = {{ {lua_list} }}
            local missing = {{}}

            for _, name in ipairs(names) do
                if type(_G[name]) ~= "function" then
                    table.insert(missing, name)
                end
            end

            return table.concat(missing, "\n")
            "#
        ))
        .unwrap();

    let missing: Vec<&str> = result.lines().filter(|line| !line.is_empty()).collect();
    assert!(
        missing.is_empty(),
        "curated extra-global diff should only keep runtime functions that intentionally exist: {missing:?}"
    );
}

#[test]
fn diff_global_functions_missing_is_filtered_to_blizzard_owned_startup_globals() {
    let missing = read_missing_global_functions();

    let unexpected: Vec<&str> = missing
        .iter()
        .map(String::as_str)
        .filter(|name| !is_filtered_startup_global(name))
        .collect();

    assert!(
        unexpected.is_empty(),
        "diff_global_functions_missing.txt should only contain filtered Blizzard-owned startup/API globals: {unexpected:?}"
    );

    for leaked_name in KNOWN_ADDON_LEAKS {
        assert!(
            !missing.contains(*leaked_name),
            "{leaked_name} should not remain in the filtered global-function diff"
        );
    }

    for required_name in REPRESENTATIVE_FILTERED_GLOBALS {
        assert!(
            missing.contains(*required_name),
            "{required_name} should remain tracked in the filtered global-function diff"
        );
    }
}

#[test]
fn filtered_missing_global_functions_are_still_missing_at_runtime() {
    let missing = read_missing_global_functions();
    let lua_list = missing
        .iter()
        .map(|name| format!("\"{}\"", name.replace('\\', "\\\\").replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(", ");

    let env = env();
    let result: String = env
        .eval(&format!(
            r#"
            local names = {{ {lua_list} }}
            local unexpected = {{}}

            for _, name in ipairs(names) do
                if type(_G[name]) == "function" then
                    table.insert(unexpected, name)
                end
            end

            return table.concat(unexpected, "\n")
            "#
        ))
        .unwrap();

    let unexpected: Vec<&str> = result.lines().filter(|line| !line.is_empty()).collect();
    assert!(
        unexpected.is_empty(),
        "filtered global-function diff should only track functions that are still missing at runtime: {unexpected:?}"
    );
}
