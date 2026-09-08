//! Process-level contract for the lua-errors command, not addon compatibility coverage.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::{Command, Output};

use serde_json::Value;
use tempfile::TempDir;

const FIXTURE: &str = "LuaErrorCliFixture";
const NESTED: &str = "LuaErrorCliNested";
const PROOF: &str = "LUA_ERROR_CLI_FIXTURE_LOADED";

#[test]
fn lua_error_cli_collects_file_syntax_event_and_nested_load_errors() {
    let directory = isolated_addon_root();
    let root = directory.path().join("AddOns");
    write_addon(
        &root,
        NESTED,
        "## LoadOnDemand: 1\n",
        &[(
            "nested.lua",
            "LuaErrorCliNestedRan = true; error('CLI_NESTED_FAILURE')",
        )],
    );
    write_addon(
        &root,
        FIXTURE,
        "",
        &[
            (
                "setup.lua",
                r#"
            local frame = CreateFrame('Frame')
            frame:RegisterEvent('ADDON_LOADED')
            frame:RegisterEvent('PLAYER_LOGIN')
            frame:SetScript('OnEvent', function(_, event, name)
                if event == 'ADDON_LOADED' and name == 'LuaErrorCliFixture' then
                    error('CLI_ADDON_LOADED_FAILURE')
                elseif event == 'PLAYER_LOGIN' then
                    error('CLI_PLAYER_LOGIN_FAILURE')
                end
            end)
            assert(not pcall(function() error('CLI_HANDLED_FAILURE') end))
        "#,
            ),
            ("runtime.lua", "error('CLI_FILE_FAILURE')"),
            ("cli_syntax_failure.lua", "local = broken"),
            ("load_nested.lua", "C_AddOns.LoadAddOn('LuaErrorCliNested')"),
            (
                "complete.lua",
                "LuaErrorCliLoaded = LuaErrorCliNestedRan == true",
            ),
        ],
    );
    let output = run_cli(
        &root,
        false,
        &format!("assert(LuaErrorCliLoaded); print('{PROOF}')"),
    );
    let errors = parse_errors(&output, true);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    for marker in [
        "CLI_FILE_FAILURE",
        "cli_syntax_failure.lua",
        "CLI_ADDON_LOADED_FAILURE",
        "CLI_PLAYER_LOGIN_FAILURE",
        "CLI_NESTED_FAILURE",
    ] {
        assert!(
            errors
                .iter()
                .any(|error| error["message"].as_str().unwrap().contains(marker)),
            "missing {marker}: {errors:?}"
        );
    }
    assert!(!errors.iter().any(|error| {
        error["message"]
            .as_str()
            .unwrap()
            .contains("CLI_HANDLED_FAILURE")
    }));
    assert!(
        errors
            .iter()
            .all(|error| error["count"].as_u64().is_some_and(|count| count > 0))
    );
}

#[test]
fn lua_error_cli_clean_addon_and_handled_errors_exit_successfully() {
    let directory = isolated_addon_root();
    let root = directory.path().join("AddOns");
    write_addon(
        &root,
        FIXTURE,
        "",
        &[(
            "clean.lua",
            r#"
        local ok, message = pcall(function() error('CLI_HANDLED_FILE_FAILURE') end)
        assert(not ok and message:find('CLI_HANDLED_FILE_FAILURE', 1, true))
        LuaErrorCliLoaded = true
    "#,
        )],
    );
    let output = run_cli(
        &root,
        false,
        &format!(
            r#"
        assert(LuaErrorCliLoaded)
        assert(not pcall(function() error('CLI_HANDLED_EXEC_FAILURE') end))
        print('{PROOF}')
    "#
        ),
    );
    let errors = parse_errors(&output, true);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(errors.is_empty(), "{errors:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("=== Final observed Lua error summary ===\nStatus: CLEAN\n"));
    assert!(stderr.contains("Lua errors: 0 unique, 0 occurrence(s)\n"));
    assert!(stderr.contains("Attributed owners: 0\n"));
    assert!(stderr.contains("Unattributed Lua errors: 0 occurrence(s)\n"));
}

#[test]
fn lua_error_cli_final_summary_includes_post_load_events_and_updates() {
    let directory = isolated_addon_root();
    let root = directory.path().join("AddOns");
    write_addon(
        &root,
        FIXTURE,
        "",
        &[(
            "deferred.lua",
            r#"
            seterrorhandler(function() end)
            local frame = CreateFrame('Frame')
            frame:RegisterEvent('PLAYER_LOGIN')
            frame:SetScript('OnEvent', function(self)
                self:SetScript('OnUpdate', function(updateFrame)
                    updateFrame:SetScript('OnUpdate', nil)
                    error('CLI_STARTUP_UPDATE_FAILURE')
                end)
                error('CLI_POST_LOAD_EVENT_FAILURE')
            end)
            "#,
        )],
    );
    let output = run_cli(
        &root,
        false,
        r#"
        local frame = CreateFrame('Frame')
        frame:SetScript('OnUpdate', function(self)
            self:SetScript('OnUpdate', nil)
            error('CLI_EXEC_UPDATE_FAILURE')
        end)
        "#,
    );
    let errors = parse_errors(&output, false);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(errors.len(), 3, "{errors:?}");
    for marker in [
        "CLI_POST_LOAD_EVENT_FAILURE",
        "CLI_STARTUP_UPDATE_FAILURE",
        "CLI_EXEC_UPDATE_FAILURE",
    ] {
        assert!(
            errors.iter().any(|error| {
                error["message"].as_str().unwrap().contains(marker) && error["count"] == 1
            }),
            "missing single occurrence of {marker}: {errors:?}"
        );
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let (loading, final_summary) = stderr
        .split_once("=== Final observed Lua error summary ===")
        .unwrap_or_else(|| panic!("missing final summary: {stderr}"));
    assert!(loading.contains("=== Addon loading summary (before startup events) ==="));
    assert!(loading.contains("Failed during loading: 0\n"));
    assert!(final_summary.contains("Status: FAILED\n"));
    assert!(final_summary.contains("Lua errors: 3 unique, 3 occurrence(s)\n"));
    assert!(
        final_summary.contains("Attributed owners: 1\n"),
        "{final_summary}"
    );
    assert!(
        final_summary.contains("  LuaErrorCliFixture: 2 occurrence(s)\n"),
        "{final_summary}"
    );
    // The dispatch label __BuiltIn does not supply an addon owner for this exec-created frame.
    assert!(!final_summary.contains("  __BuiltIn:"), "{final_summary}");
    assert!(
        final_summary.contains("Unattributed Lua errors: 1 occurrence(s)\n"),
        "{final_summary}"
    );
}

#[test]
fn lua_error_cli_uncaught_exec_error_is_json_failure() {
    let root = tempfile::tempdir().expect("create exec fixture directory");
    let output = run_cli(
        root.path(),
        true,
        "seterrorhandler(function() end); error('CLI_UNCAUGHT_EXEC_FAILURE')",
    );
    let errors = parse_errors(&output, false);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let matching: Vec<_> = errors
        .iter()
        .filter(|error| {
            error["message"]
                .as_str()
                .unwrap()
                .contains("CLI_UNCAUGHT_EXEC_FAILURE")
        })
        .collect();
    assert_eq!(
        matching.len(),
        1,
        "uncaught exec failure must be collected once: {errors:?}"
    );
    assert_eq!(matching[0]["count"], 1);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("=== Final observed Lua error summary ===\nStatus: FAILED\n"));
    assert!(stderr.contains("Lua errors: 1 unique, 1 occurrence(s)\n"));
    assert!(stderr.contains("Unattributed Lua errors: 1 occurrence(s)\n"));
}

fn isolated_addon_root() -> TempDir {
    let root = tempfile::tempdir().expect("create addon fixture directory");
    let mut names = BTreeSet::new();
    for path in wow_ui_sim::paths::default_addons_paths() {
        for entry in std::fs::read_dir(&path).expect("read merged addon root") {
            let entry = entry.expect("read addon directory entry");
            if entry.path().is_dir() {
                names.insert(entry.file_name().into_string().expect("UTF-8 addon name"));
            }
        }
    }
    // The CLI merges external roots. First-root disabled fixtures shadow those
    // names without editing installed addons or relying on personal enable state.
    for name in names {
        write_addon(
            &root.path().join("AddOns"),
            &name,
            "## DefaultState: disabled\n",
            &[],
        );
    }
    root
}

fn write_addon(root: &Path, name: &str, metadata: &str, files: &[(&str, &str)]) {
    let directory = root.join(name);
    std::fs::create_dir_all(&directory).expect("create addon directory");
    let mut toc = format!(
        "## Interface: {}\n## Title: {name}\n{metadata}",
        wow_ui_sim::toc::ACTIVE_INTERFACE_VERSION
    );
    for (file, source) in files {
        std::fs::write(directory.join(file), source).expect("write addon Lua fixture");
        toc.push_str(file);
        toc.push('\n');
    }
    std::fs::write(directory.join(format!("{name}.toc")), toc).expect("write addon TOC fixture");
}

fn run_cli(root: &Path, no_addons: bool, code: &str) -> Output {
    let mut command = Command::new("timeout");
    command.args(["90", env!("CARGO_BIN_EXE_wow-sim"), "--no-saved-vars"]);
    if no_addons {
        command.arg("--no-addons");
    }
    command
        .args(["--exec-lua", code, "lua-errors"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env_remove("WOW_SIM_NO_ADDONS")
        .env_remove("WOW_SIM_LOAD_OUT_OF_DATE_ADDONS")
        .env("WOW_SIM_ADDONS_PATH", root)
        .env("WOW_SIM_WTF_PATH", root)
        .env("WOW_SIM_WTF_ACCOUNT", "LuaErrorCliAccount")
        .env("WOW_SIM_WTF_REALM", "LuaErrorCliRealm")
        .env("WOW_SIM_WTF_CHARACTER", "LuaErrorCliCharacter")
        .output()
        .expect("run bounded wow-sim lua-errors process")
}

fn parse_errors(output: &Output, expect_proof: bool) -> Vec<Value> {
    let stdout = std::str::from_utf8(&output.stdout).expect("UTF-8 CLI output");
    let json = if expect_proof {
        stdout
            .strip_prefix(&format!("{PROOF}\n"))
            .unwrap_or_else(|| {
                panic!(
                    "fixture did not execute: stdout={stdout}, stderr={}",
                    String::from_utf8_lossy(&output.stderr)
                )
            })
    } else {
        stdout
    };
    serde_json::from_str(json).unwrap_or_else(|error| {
        panic!(
            "invalid error JSON: {error}; stdout={stdout}; stderr={}",
            String::from_utf8_lossy(&output.stderr)
        )
    })
}
