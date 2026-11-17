//! `GetAddOnMetadata` results for a parsed fixture TOC, by addon name and by index.
//!
//! Preservation proof for the case-insensitive metadata index: the expected rows
//! are the results of the previous linear `eq_ignore_ascii_case` scan.

use std::path::Path;
use wow_ui_sim::loader::{find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;

const FIXTURE_TOC: &str = "## Title: Metadata Fixture
## Title-deDE: Metadaten-Fixture
## Notes: Fixture notes
## Notes-frFR: Notes de test
## Version: 1.2.3
## X-Website: https://example.invalid/addon
## X-Dup: first
## X-Dup: second
## X-Mixed-Case: mixed value
## OptionalDeps: OptA
## OptionalDeps: OptB
## X-Empty:
##   X-Spaced  :   spaced value   
## X-Version-Like: @project-version@
";

const FIXTURE_FIELDS: &[&str] = &[
    "Title",
    "TITLE",
    "Title-deDE",
    "title-dede",
    "Notes",
    "notes",
    "Notes-frFR",
    "Version",
    "VERSION",
    "Group",
    "X-Website",
    "x-website",
    "X-Dup",
    "X-Mixed-Case",
    "x-mixed-case",
    "X-MIXED-CASE",
    "OptionalDeps",
    "optionaldeps",
    "X-Empty",
    "X-Spaced",
    "X-Version-Like",
    "X-Missing",
    "",
    "X-Dup ",
];

const FIXTURE_EXPECTED: &[&str] = &[
    "Metadata Fixture",
    "Metadata Fixture",
    "Metadaten-Fixture",
    "Metadaten-Fixture",
    "Fixture notes",
    "Fixture notes",
    "Notes de test",
    "1.2.3",
    "1.2.3",
    "MetadataFixture",
    "https://example.invalid/addon",
    "https://example.invalid/addon",
    "second",
    "mixed value",
    "mixed value",
    "mixed value",
    "OptA,OptB",
    "OptA,OptB",
    "",
    "spaced value",
    "dev",
    "<nil>",
    "<nil>",
    "<nil>",
];

const BARE_FIELDS: &[&str] = &["Title", "Notes", "Version", "version", "X-Website"];
const BARE_EXPECTED: &[&str] = &[
    "Bare Fixture",
    "<nil>",
    "@project-version@",
    "@project-version@",
    "<nil>",
];

const QUERY_LUA: &str = r#"
local function render(value)
    if value == nil then return "<nil>" end
    return value
end
local function addon_index(name)
    for index = 1, C_AddOns.GetNumAddOns() do
        if C_AddOns.GetAddOnInfo(index) == name then return index end
    end
end
function MetadataRows(addon, fields)
    local rows, index = {}, addon_index(addon)
    for _, field in ipairs(fields) do
        local by_name = render(C_AddOns.GetAddOnMetadata(addon, field))
        local by_index = render(C_AddOns.GetAddOnMetadata(index, field))
        local legacy = render(GetAddOnMetadata(addon, field))
        rows[#rows + 1] = field .. "=" .. by_name .. "|" .. by_index .. "|" .. legacy
    end
    return table.concat(rows, "\n")
end
"#;

fn write_addon(root: &Path, name: &str, toc: &str) {
    let addon = root.join(name);
    std::fs::create_dir_all(&addon).unwrap();
    std::fs::write(addon.join(format!("{name}.toc")), toc).unwrap();
}

fn load_fixture_addon(env: &WowLuaEnv, root: &Path, name: &str, toc: &str) {
    write_addon(root, name, toc);
    let toc_path = find_toc_file(&root.join(name)).unwrap();
    let result = load_addon(&env.loader_env(), &toc_path).unwrap();
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
}

fn lua_string_list(items: &[&str]) -> String {
    let quoted: Vec<String> = items.iter().map(|item| format!("{item:?}")).collect();
    format!("{{{}}}", quoted.join(", "))
}

fn metadata_rows(env: &WowLuaEnv, addon: &str, fields: &[&str]) -> String {
    let fields = lua_string_list(fields);
    env.eval(&format!("return MetadataRows({addon:?}, {fields})"))
        .unwrap()
}

fn expected_rows(fields: &[&str], values: &[&str]) -> String {
    assert_eq!(fields.len(), values.len());
    let rows: Vec<String> = fields
        .iter()
        .zip(values)
        .map(|(field, value)| format!("{field}={value}|{value}|{value}"))
        .collect();
    rows.join("\n")
}

fn fixture_env(root: &Path) -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    load_fixture_addon(&env, root, "MetadataFixture", FIXTURE_TOC);
    load_fixture_addon(&env, root, "MetadataBare", "## Title: Bare Fixture\n");
    env.exec(QUERY_LUA).unwrap();
    env
}

#[test]
fn metadata_lookup_matches_fields_case_insensitively_by_name_and_index() {
    let root = tempfile::tempdir().unwrap();
    let env = fixture_env(root.path());
    assert_eq!(
        metadata_rows(&env, "MetadataFixture", FIXTURE_FIELDS),
        expected_rows(FIXTURE_FIELDS, FIXTURE_EXPECTED)
    );
}

#[test]
fn metadata_lookup_keeps_notes_and_version_miss_results() {
    let root = tempfile::tempdir().unwrap();
    let env = fixture_env(root.path());
    assert_eq!(
        metadata_rows(&env, "MetadataBare", BARE_FIELDS),
        expected_rows(BARE_FIELDS, BARE_EXPECTED)
    );
}

#[test]
fn metadata_lookup_for_unknown_addons_returns_only_exact_title() {
    let root = tempfile::tempdir().unwrap();
    let env = fixture_env(root.path());
    let rows: (String, bool, bool, String, bool) = env
        .eval(
            r#"return C_AddOns.GetAddOnMetadata("NoSuchAddon", "Title"),
                C_AddOns.GetAddOnMetadata("NoSuchAddon", "title") == nil,
                C_AddOns.GetAddOnMetadata("NoSuchAddon", "X-Website") == nil,
                C_AddOns.GetAddOnMetadata(9999, "Title"),
                C_AddOns.GetAddOnMetadata(9999, "X-Website") == nil"#,
        )
        .unwrap();
    assert_eq!(
        rows,
        ("NoSuchAddon".into(), true, true, String::new(), true)
    );
}

/// The previous scan iterated a `HashMap`, so case-variant duplicates such as
/// `X-Case` and `x-case` returned whichever spelling the per-map random hash
/// order visited first. The index resolves them like exact duplicates: the
/// last TOC line wins.
#[test]
fn metadata_lookup_resolves_case_variant_duplicates_to_last_toc_line() {
    let root = tempfile::tempdir().unwrap();
    let env = WowLuaEnv::new().unwrap();
    let toc = "## X-Case: upper\n## x-case: lower\n## X-CASE-ORDER: first\n## x-Case-Order: last\n";
    let names: Vec<String> = (1..=16).map(|n| format!("CaseFixture{n}")).collect();
    for name in &names {
        load_fixture_addon(&env, root.path(), name, toc);
    }
    let lookups: Vec<String> = names
        .iter()
        .map(|name| {
            env.eval::<String>(&format!(
                "return C_AddOns.GetAddOnMetadata({name:?}, 'X-Case') .. ',' .. \
                 C_AddOns.GetAddOnMetadata({name:?}, 'x-case-order')"
            ))
            .unwrap()
        })
        .collect();
    assert_eq!(lookups, vec!["lower,last".to_string(); names.len()]);
}
