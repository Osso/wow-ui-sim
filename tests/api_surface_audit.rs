#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]
//! Documented API surface against the runtime-registered surface after full
//! Blizzard UI startup. Contract: `docs/specs/api-surface-audit.md`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use rilua::LuaApiMut;
use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;

const SECURE_PUBLICATION_CSV: &str = "data/secure-publication/retail-12-1-5-70688.csv";
const TEMPORARY_WORKAROUNDS: &str = "src/lua_api/workarounds/temporary";
const CATEGORIES: [&str; 2] = ["intentionally-unsupported", "not-yet"];

/// Sandbox for `Blizzard_APIDocumentationGenerated/*.lua`: each file only
/// builds a local table and passes it to `APIDocumentation:AddDocumentationTable`.
/// `ScriptObject` tables document widget methods, not global functions.
const DOC_COLLECTOR: &str = r#"
APIDocumentation = { names = {} }
function APIDocumentation:AddDocumentationTable(doc)
    if doc.Type ~= "System" then return end
    local prefix = doc.Namespace and (doc.Namespace .. ".") or ""
    for _, entry in ipairs(doc.Functions or {}) do
        if entry.Type == "Function" then
            table.insert(self.names, prefix .. entry.Name)
        end
    end
end
"#;

/// Raw lookup only: namespace `__index` fallbacks record nil access and must
/// not count. Simulator-embedded Lua closures are reported as `lua <name>`.
const RUNTIME_PROBE: &str = r#"
local report = {}
for _, name in ipairs(__api_surface_names) do
    local namespace, member = name:match("^([^.]+)%.(.+)$")
    local value
    if namespace then
        local container = rawget(_G, namespace)
        value = type(container) == "table" and rawget(container, member) or nil
    else
        value = rawget(_G, name)
    end
    if type(value) ~= "function" then
        table.insert(report, "missing " .. name)
    elseif debug.getinfo(value, "S").source == "=(string)" then
        table.insert(report, "lua " .. name)
    end
end
__api_surface_names = nil
return table.concat(report, "\n")
"#;

fn repo_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn read_text(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn documentation_files() -> Vec<PathBuf> {
    let dir = wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("profile Blizzard UI cache")
        .join("Blizzard_APIDocumentationGenerated");
    let entries = std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
    let mut files: Vec<PathBuf> = entries
        .map(|entry| entry.expect("documentation entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "lua"))
        .collect();
    files.sort();
    assert!(
        !files.is_empty(),
        "no documentation files in {}",
        dir.display()
    );
    files
}

fn documented_functions() -> BTreeSet<String> {
    let mut lua = rilua::Lua::new().expect("documentation sandbox");
    lua.exec(DOC_COLLECTOR).expect("documentation collector");
    for path in documentation_files() {
        let source = std::fs::read(&path).expect("documentation file");
        let chunk = format!("@{}", path.display());
        lua.exec_bytes(&source, &chunk)
            .unwrap_or_else(|error| panic!("load {}: {error}", path.display()));
    }
    lua.exec("ApiSurfaceNames = table.concat(APIDocumentation.names, '\\n')")
        .expect("join documented names");
    let joined: String = lua
        .global("ApiSurfaceNames")
        .expect("documented names string");
    joined.lines().map(str::to_owned).collect()
}

/// Retail only: `_G`/namespace rows published to the native Game environment.
/// Glue-only rows do not exist in the Game environment this test inspects.
fn secure_publication_functions() -> BTreeSet<String> {
    let text = read_text(&repo_path(SECURE_PUBLICATION_CSV));
    let mut lines = text.lines();
    assert_eq!(lines.next(), Some("namespace,member,flag,context,evidence"));
    lines
        .map(|line| line.split(',').collect::<Vec<_>>())
        .filter(|fields| fields[3] != "Glue")
        .map(|fields| match fields[0] {
            "_G" => fields[1].to_owned(),
            namespace => format!("{namespace}.{}", fields[1]),
        })
        .collect()
}

fn documented_surface() -> BTreeSet<String> {
    let mut names = documented_functions();
    if ACTIVE == ClientProfile::Retail {
        names.extend(secure_publication_functions());
    }
    names
}

fn lua_string_list(names: &BTreeSet<String>) -> String {
    let quoted: Vec<String> = names.iter().map(|name| format!("{name:?}")).collect();
    format!("__api_surface_names = {{ {} }}", quoted.join(", "))
}

/// Lua definitions (`function X.Name(` / `Name = function`) in temporary workarounds.
fn temporary_workaround_definitions() -> String {
    let dir = repo_path(TEMPORARY_WORKAROUNDS);
    let entries = std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
    let mut paths: Vec<PathBuf> = entries
        .map(|entry| entry.expect("workaround").path())
        .collect();
    paths.sort();
    paths
        .iter()
        .filter(|path| {
            path.extension()
                .is_some_and(|ext| ext == "rs" || ext == "lua")
        })
        .map(|path| read_text(path))
        .collect::<Vec<_>>()
        .join("\n")
}

fn declares_member(line: &str, member: &str) -> bool {
    line.match_indices("function ").any(|(start, keyword)| {
        let declared = line[start + keyword.len()..]
            .split('(')
            .next()
            .unwrap_or_default();
        declared.rsplit(['.', ':']).next() == Some(member)
    })
}

fn assigns_member(line: &str, member: &str) -> bool {
    let assigned = format!("{member} = function");
    line.match_indices(&assigned).any(|(start, _)| {
        let before = line[..start].chars().next_back();
        !before.is_some_and(|c| c.is_alphanumeric() || c == '_')
    })
}

fn defines_lua_function(source: &str, name: &str) -> bool {
    let member = name.rsplit('.').next().expect("member name");
    source
        .lines()
        .any(|line| assigns_member(line, member) || declares_member(line, member))
}

/// Documented names without a raw runtime function, plus names whose runtime
/// function is simulator-embedded Lua defined in a temporary workaround.
fn runtime_gaps(env: &WowLuaEnv, documented: &BTreeSet<String>) -> BTreeSet<String> {
    env.exec(&lua_string_list(documented))
        .expect("publish probe names");
    let report: String = env.eval(RUNTIME_PROBE).expect("runtime surface probe");
    let workarounds = temporary_workaround_definitions();
    report
        .lines()
        .filter_map(|line| match line.split_once(' ') {
            Some(("missing", name)) => Some(name.to_owned()),
            Some(("lua", name)) if defines_lua_function(&workarounds, name) => {
                Some(name.to_owned())
            }
            Some(("lua", _)) => None,
            _ => panic!("unexpected probe line {line:?}"),
        })
        .collect()
}

fn allowlist_path() -> PathBuf {
    repo_path(&format!(
        "tests/data/api_surface/{}-missing.txt",
        ACTIVE.cache_subdir()
    ))
}

/// `name # category: reason` lines, sorted and unique; `#` lines are comments.
fn read_allowlist(path: &Path) -> BTreeMap<String, String> {
    let mut entries = BTreeMap::new();
    let mut previous = String::new();
    for line in read_text(path)
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let (name, category) = line
            .split_once(" # ")
            .unwrap_or_else(|| panic!("no category: {line}"));
        let known = CATEGORIES.iter().any(|known| category.starts_with(known));
        assert!(
            known,
            "unknown category in {line:?}; use one of {CATEGORIES:?}"
        );
        assert!(
            previous.as_str() < name,
            "{} is not sorted/unique at {name}",
            path.display()
        );
        previous = name.to_owned();
        entries.insert(name.to_owned(), category.to_owned());
    }
    entries
}

fn assert_allowlist_matches(gaps: &BTreeSet<String>, path: &Path) {
    let allowlist = read_allowlist(path);
    let unlisted: Vec<String> = gaps
        .iter()
        .filter(|name| !allowlist.contains_key(*name))
        .map(|name| format!("{name} # not-yet"))
        .collect();
    let stale: Vec<&String> = allowlist
        .keys()
        .filter(|name| !gaps.contains(*name))
        .collect();
    assert!(
        unlisted.is_empty() && stale.is_empty(),
        "{}: {} documented names missing and not allowlisted:\n{}\n{} stale entries (now present or undocumented):\n{}",
        path.display(),
        unlisted.len(),
        unlisted.join("\n"),
        stale.len(),
        stale
            .iter()
            .map(|name| name.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
    );
}

prefork_full_ui_case! {
fn documented_api_surface_matches_allowlist(env: &WowLuaEnv) {
    let gaps = runtime_gaps(env, &documented_surface());
    assert_allowlist_matches(&gaps, &allowlist_path());
}
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_documented_api_surface_matches_allowlist() {
    crate::common::with_timeout(120, || {
        let env = WowLuaEnv::new().expect("create Forever Lua environment");
        crate::common::load_game_startup_inventory(&env);
        env.sync_addon_names_to_lua();
        env.apply_post_load_workarounds();
        let game = wow_ui_sim::screen::ScreenKind::Game;
        wow_ui_sim::startup::fire_startup_events_for_screen(&env, game);
        documented_api_surface_matches_allowlist::run(&env);
    });
}
