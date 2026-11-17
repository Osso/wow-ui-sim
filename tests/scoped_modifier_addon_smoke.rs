use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use wow_ui_sim::loader::{find_toc_file, load_addon, sort_addons_by_dependencies};
use wow_ui_sim::lua_api::{AddonInfo, WowLuaEnv};
use wow_ui_sim::toc::TocFile;

const DEFAULT_SCOPED_MODIFIER_SMOKE_ADDON: &str = "WoWPro";
const SCOPED_MODIFIER_SMOKE_ADDON_ENV: &str = "WOW_SIM_SCOPED_MODIFIER_SMOKE_ADDON";

#[test]
fn scoped_modifier_real_addon_smoke_loads_without_lua_errors() {
    let target = scoped_modifier_smoke_addon_name();
    let addons = discover_local_addons();
    let Some(target_name) = find_addon_name(&addons, &target) else {
        eprintln!(
            "skipping ScopedModifier addon smoke: addon {target:?} not found; set {SCOPED_MODIFIER_SMOKE_ADDON_ENV} to override"
        );
        return;
    };

    let closure = addon_closure(&addons, &target_name);
    let mut load_order: Vec<_> = closure
        .iter()
        .filter_map(|name| {
            addons
                .get(name)
                .map(|addon| (name.clone(), addon.toc_path.clone()))
        })
        .collect();
    sort_addons_by_dependencies(&mut load_order);

    let env = WowLuaEnv::new().expect("env should initialize");
    register_addon_closure(&env, &addons, &closure);
    for (name, toc_path) in &load_order {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|error| panic!("{name} should load for ScopedModifier smoke: {error}"));
    }

    let lua_errors = env.state().borrow().lua_errors.clone();
    assert!(
        lua_errors.is_empty(),
        "{} ScopedModifier smoke should load without Lua errors:\n  {}",
        target_name,
        lua_errors.join("\n  ")
    );
}

fn scoped_modifier_smoke_addon_name() -> String {
    std::env::var(SCOPED_MODIFIER_SMOKE_ADDON_ENV)
        .unwrap_or_else(|_| DEFAULT_SCOPED_MODIFIER_SMOKE_ADDON.to_string())
}

#[derive(Clone)]
struct LocalAddon {
    toc_path: PathBuf,
    toc: TocFile,
}

fn discover_local_addons() -> HashMap<String, LocalAddon> {
    let mut addons = HashMap::new();
    for addon_root in wow_ui_sim::paths::default_addons_paths() {
        discover_addons_in_root(&addon_root, &mut addons);
    }
    addons
}

fn discover_addons_in_root(addon_root: &Path, addons: &mut HashMap<String, LocalAddon>) {
    let Ok(entries) = std::fs::read_dir(addon_root) else {
        return;
    };
    for entry in entries.flatten() {
        let addon_dir = entry.path();
        if !addon_dir.is_dir() {
            continue;
        }
        let Some(name) = addon_dir.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if addons.contains_key(name) {
            continue;
        }
        let Some(toc_path) = find_toc_file(&addon_dir) else {
            continue;
        };
        let Ok(toc) = TocFile::from_file(&toc_path) else {
            continue;
        };
        addons.insert(name.to_string(), LocalAddon { toc_path, toc });
    }
}

fn find_addon_name(addons: &HashMap<String, LocalAddon>, requested: &str) -> Option<String> {
    addons
        .keys()
        .find(|name| name.eq_ignore_ascii_case(requested))
        .cloned()
}

fn addon_closure(addons: &HashMap<String, LocalAddon>, root: &str) -> HashSet<String> {
    let mut closure = HashSet::new();
    collect_addon_with_dependencies(addons, root, &mut closure);
    closure
}

fn collect_addon_with_dependencies(
    addons: &HashMap<String, LocalAddon>,
    name: &str,
    closure: &mut HashSet<String>,
) {
    let Some(addon) = addons.get(name) else {
        return;
    };
    if !closure.insert(name.to_string()) {
        return;
    }
    for dependency in addon
        .toc
        .dependencies()
        .into_iter()
        .chain(addon.toc.optional_deps())
    {
        if let Some(dependency_name) = find_addon_name(addons, &dependency) {
            collect_addon_with_dependencies(addons, &dependency_name, closure);
        }
    }
}

fn register_addon_closure(
    env: &WowLuaEnv,
    addons: &HashMap<String, LocalAddon>,
    closure: &HashSet<String>,
) {
    for name in closure {
        let Some(addon) = addons.get(name) else {
            continue;
        };
        env.register_addon(AddonInfo {
            folder_name: name.clone(),
            title: addon
                .toc
                .metadata
                .get("Title")
                .cloned()
                .unwrap_or_else(|| name.clone()),
            notes: addon.toc.metadata.get("Notes").cloned().unwrap_or_default(),
            enabled: true,
            loaded: false,
            load_on_demand: addon.toc.is_load_on_demand(),
            use_secure_env: addon.toc.is_secure_env(),
            dependencies: addon.toc.dependencies(),
            metadata: addon.toc.metadata.clone(),
            default_enabled: addon.toc.default_enabled(),
            ..Default::default()
        });
    }
}
