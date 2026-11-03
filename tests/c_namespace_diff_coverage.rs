use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

fn diff_c_namespaces_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("wow-client-diff")
        .join(file_name)
}

fn read_diff_namespaces(file_name: &str) -> BTreeSet<String> {
    fs::read_to_string(diff_c_namespaces_path(file_name))
        .unwrap_or_else(|_| panic!("failed to read {file_name}"))
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

#[test]
fn diff_c_namespaces_missing_file_excludes_seeded_namespace_stubs() {
    let missing = read_diff_namespaces("diff_c_namespaces_missing.txt");
    let expected_absent = [
        "C_AccountServices",
        "C_ArrowCalloutManager",
        "C_EncounterEvents",
        "C_PrototypeDialog",
        "C_Reincarnation",
        "C_TableUtil",
        "C_TradeSkillUI",
        "C_Tutorial",
    ];

    for namespace in expected_absent {
        assert!(
            !missing.contains(namespace),
            "{namespace} should not remain in diff_c_namespaces_missing.txt"
        );
    }
}

#[test]
fn diff_c_namespaces_extra_file_excludes_documented_blizzard_namespaces() {
    let extra = read_diff_namespaces("diff_c_namespaces_extra.txt");
    let expected_absent = [
        "C_CinematicList",
        "C_CombatLogSecure",
        "C_Console",
        "C_GMTicketInfo",
        "C_Guild",
        "C_Login",
        "C_MacOptions",
        "C_PingSecure",
        "C_PrivateAuras",
        "C_UnitAurasPrivate",
        "C_Who",
    ];

    for namespace in expected_absent {
        assert!(
            !extra.contains(namespace),
            "{namespace} should not remain in diff_c_namespaces_extra.txt"
        );
    }
}

#[test]
// C_TableUtil is documented only by Era, Mists and Anniversary, and the pinned
// GMTicketInfoDocumentation declares events only, so neither has a function to
// require here (APIs are published per generated docs).
fn seeded_missing_c_namespace_tables_exist_with_representative_methods() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local expectations = {
                { "C_AccountServices", "SaveAccountData" },
                { "C_ArrowCalloutManager", "ShowCallout" },
                { "C_EncounterEvents", "GetEventList" },
                { "C_PrototypeDialog", "SelectOption" },
                { "C_Reincarnation", "StartReincarnation" },
                { "C_TradeSkillUI", "GetTradeSkillLine" },
                { "C_Tutorial", "GetTutorialStatus" },
            }

            for _, expectation in ipairs(expectations) do
                local namespace = _G[expectation[1]]
                if type(namespace) ~= "table" then
                    return expectation[1] .. ":missing_namespace"
                end
                if type(namespace[expectation[2]]) ~= "function" then
                    return expectation[1] .. ":missing_method:" .. expectation[2]
                end
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "seeded namespace tables should exist with representative methods"
    );
}

#[test]
// No profile's generated docs or UI declare C_Console, C_Guild, C_PrivateAuras
// or C_Who; the documented forms are ConsoleGetAllCommands, IsInGuild and
// C_FriendList.SendWho, so those namespaces have no method to require here.
fn documented_extra_c_namespace_tables_exist_with_representative_methods() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local expectations = {
                { "C_CinematicList", "GetUICinematicList" },
                { "C_CombatLogSecure", "GetEntryCount" },
                { "C_Login", "GetState" },
                { "C_MacOptions", "GetGameBundleName" },
                { "C_UnitAurasPrivate", "GetAllPrivateAuras" },
            }

            for _, expectation in ipairs(expectations) do
                local namespace = _G[expectation[1]]
                if type(namespace) ~= "table" then
                    return expectation[1] .. ":missing_namespace"
                end
                if type(namespace[expectation[2]]) ~= "function" then
                    return expectation[1] .. ":missing_method:" .. expectation[2]
                end
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "documented Blizzard namespaces should exist with representative methods"
    );

    // Retail 12.1.5 exposes this namespace only in the secure environment.
    // See docs/specs/ping-secure-publication.md and PingManagerSecureDocumentation.lua:3-6.
    let ping: String = env
        .eval(
            if cfg!(all(feature = "profile-retail", feature = "retail-12-1-5")) {
                "return type(__secureenv.C_PingSecure and __secureenv.C_PingSecure.CreateFrame)"
            } else {
                "return type(C_PingSecure and C_PingSecure.CreateFrame)"
            },
        )
        .unwrap();
    assert_eq!(ping, "function", "C_PingSecure.CreateFrame");
}
