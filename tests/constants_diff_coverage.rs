use std::fs;
use std::path::PathBuf;

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("failed to create Lua environment")
}

fn diff_constants_missing_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("wow-client-diff")
        .join("diff_constants_missing.txt")
}

fn diff_constants_wrong_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("wow-client-diff")
        .join("diff_constants_wrong.txt")
}

fn missing_constants_lua_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("lua_api")
        .join("globals")
        .join("enum_data")
        .join("missing_constants.lua")
}

fn parse_missing_constant_line(line: &str) -> Option<(String, i32)> {
    let trimmed = line.trim();
    let prefix = "if ";
    let separator = " == nil then ";
    let assignment = " = ";

    if !trimmed.starts_with(prefix) || !trimmed.ends_with(" end") {
        return None;
    }

    let separator_index = trimmed.find(separator)?;
    let name = trimmed[prefix.len()..separator_index].trim();
    let remainder = &trimmed[(separator_index + separator.len())..];
    let (assigned_name, raw_value) = remainder.split_once(assignment)?;
    if assigned_name.trim() != name {
        return None;
    }

    let value = raw_value.trim_end_matches(" end").trim().parse().ok()?;
    Some((name.to_owned(), value))
}

fn targeted_constant_entries() -> Vec<(String, i32)> {
    let prefixes = ["LE_GAME_", "LE_AUTOCOMPLETE_", "LE_LFG_"];
    fs::read_to_string(missing_constants_lua_path())
        .expect("failed to read missing_constants.lua")
        .lines()
        .filter_map(parse_missing_constant_line)
        .filter(|(name, _)| prefixes.iter().any(|prefix| name.starts_with(prefix)))
        .collect()
}

#[cfg(feature = "optional-tests")]
#[test]
fn diff_constants_wrong_is_empty_after_constant_reconciliation() {
    let contents = fs::read_to_string(diff_constants_wrong_path())
        .expect("failed to read diff_constants_wrong.txt");
    let entries: Vec<&str> = contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    assert!(
        entries.is_empty(),
        "diff_constants_wrong.txt should be empty after reconciling wrong constants, found: {entries:?}"
    );
}

#[cfg(feature = "optional-tests")]
#[test]
fn diff_constants_missing_excludes_targeted_loaded_constant_families() {
    let contents = fs::read_to_string(diff_constants_missing_path())
        .expect("failed to read diff_constants_missing.txt");
    let stale_entries: Vec<&str> = contents
        .lines()
        .map(str::trim)
        .filter(|line| {
            !line.is_empty()
                && (line.starts_with("LE_GAME_")
                    || line.starts_with("LE_AUTOCOMPLETE_")
                    || line.starts_with("LE_LFG_"))
        })
        .collect();

    assert!(
        stale_entries.is_empty(),
        "diff_constants_missing.txt should not list already loaded LE_GAME_*, LE_AUTOCOMPLETE_*, or LE_LFG_* constants: {stale_entries:?}"
    );
}

#[test]
fn targeted_missing_constant_families_resolve_at_runtime_with_expected_values() {
    let entries = targeted_constant_entries();
    assert!(
        !entries.is_empty(),
        "expected targeted constants to be present in missing_constants.lua"
    );

    let checks = entries
        .iter()
        .map(|(name, value)| format!("{{{:?}, {value}}}", name))
        .collect::<Vec<_>>()
        .join(",\n                ");
    let env = env();
    let result: String = env
        .eval(&format!(
            r#"
            local checks = {{
                {checks}
            }}

            for _, check in ipairs(checks) do
                local actual = _G[check[1]]
                if actual == nil then
                    return "missing:" .. check[1]
                end
                if actual ~= check[2] then
                    return "wrong:" .. check[1] .. ":" .. tostring(actual) .. ":" .. tostring(check[2])
                end
            end

            return "ok"
            "#
        ))
        .unwrap();

    assert_eq!(
        result, "ok",
        "all LE_GAME_*, LE_AUTOCOMPLETE_*, and LE_LFG_* constants from missing_constants.lua should resolve at runtime with the expected values"
    );
}
