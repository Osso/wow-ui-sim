use super::*;
use std::path::{Path, PathBuf};

fn write_addon(root: &Path, name: &str) -> PathBuf {
    write_addon_with_toc(
        root,
        name,
        &format!(
            "## Interface: {}\nmain.lua\n",
            wow_ui_sim::toc::ACTIVE_INTERFACE_VERSION
        ),
    )
}

fn write_addon_with_toc(root: &Path, name: &str, toc: &str) -> PathBuf {
    let addon_dir = root.join(name);
    std::fs::create_dir_all(&addon_dir).expect("create addon dir");
    let toc_path = addon_dir.join(format!("{name}.toc"));
    std::fs::write(&toc_path, toc).expect("write toc");
    std::fs::write(addon_dir.join("main.lua"), "").expect("write lua");
    toc_path
}

fn write_addon_with_lua(root: &Path, name: &str, metadata: &str, lua: &str) -> PathBuf {
    let addon_dir = root.join(name);
    std::fs::create_dir_all(&addon_dir).expect("create addon dir");
    let toc_path = addon_dir.join(format!("{name}.toc"));
    let toc = format!(
        "## Interface: {}\n{}main.lua\n",
        wow_ui_sim::toc::ACTIVE_INTERFACE_VERSION,
        metadata
    );
    std::fs::write(&toc_path, toc).expect("write toc");
    std::fs::write(addon_dir.join("main.lua"), lua).expect("write lua");
    toc_path
}

#[cfg(feature = "retail-12-1-5")]
#[test]
fn startup_bootstrap_runs_inline_without_full_load_or_event() {
    let temp = tempfile::tempdir().unwrap();
    write_addon_with_lua(
        temp.path(),
        "A",
        "",
        "order = {'A'}; loadedEvents = {}; local f = CreateFrame('Frame'); f:RegisterEvent('ADDON_LOADED'); f:SetScript('OnEvent', function(_, _, name) loadedEvents[name] = true end)",
    );
    write_addon_with_toc(
        temp.path(),
        "B",
        &format!(
            "## Interface: {}\n## LoadOnDemand: 1\nbefore.lua\nbootstrap.lua [Bootstrap]\nmain.lua\n",
            wow_ui_sim::toc::ACTIVE_INTERFACE_VERSION
        ),
    );
    std::fs::write(temp.path().join("B/bootstrap.lua"), "table.insert(order, 'B:bootstrap'); duringLoaded, duringFinished = C_AddOns.IsAddOnLoaded('B')").unwrap();
    std::fs::write(
        temp.path().join("B/before.lua"),
        "error('normal file ran at startup')",
    )
    .unwrap();
    std::fs::write(
        temp.path().join("B/main.lua"),
        "error('normal file ran at startup')",
    )
    .unwrap();
    write_addon_with_lua(temp.path(), "C", "", "table.insert(order, 'C')");
    let mut addons = scan_addons(temp.path(), &[], ScreenKind::Game);
    wow_ui_sim::loader::sort_addons_by_dependencies(&mut addons);
    let env = WowLuaEnv::new().unwrap();
    load_discovered_addons(&env, &addons, &mut None, None, &mut LoadStats::default());
    env.exec("assert(table.concat(order, ',') == 'A,B:bootstrap,C', table.concat(order, ',')); assert(duringLoaded and not duringFinished); local loaded, finished = C_AddOns.IsAddOnLoaded('B'); assert(not loaded and not finished); assert(not loadedEvents.B); assert(loadedEvents.C)").unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());

    let disabled_env = WowLuaEnv::new().unwrap();
    let disabled = HashMap::from([("B".to_string(), false)]);
    load_discovered_addons(
        &disabled_env,
        &addons,
        &mut None,
        Some(&disabled),
        &mut LoadStats::default(),
    );
    disabled_env
        .exec("assert(table.concat(order, ',') == 'A,C'); assert(not loadedEvents.B)")
        .unwrap();
}

#[test]
fn lua_errors_reports_enabled_addon_with_missing_required_dependency() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dependent_toc = write_addon_with_lua(
        temp.path(),
        "DependentAddon",
        "## Dependencies: MissingRequiredAddon\n",
        "_G.DependentAddonLoaded = true\n",
    );
    let addons = vec![("DependentAddon".to_string(), dependent_toc)];
    let env = WowLuaEnv::new().expect("create Lua env");
    let mut saved_vars = None;
    let mut stats = LoadStats::default();

    load_discovered_addons(&env, &addons, &mut saved_vars, None, &mut stats);

    let state = env.state().borrow();
    assert_eq!(state.lua_errors.len(), 1);
    assert_eq!(
        state.lua_errors[0],
        "DependentAddon missing required TOC dependencies: MissingRequiredAddon"
    );
    assert_eq!(state.lua_error_counts.get(&state.lua_errors[0]), Some(&1));
    assert_eq!(state.lua_error_records.len(), 1);
    assert_eq!(
        state.lua_error_records[0].addon_name.as_deref(),
        Some("DependentAddon")
    );
    assert!(
        !state
            .addons
            .iter()
            .find(|addon| addon.folder_name == "DependentAddon")
            .expect("registered dependent addon")
            .loaded,
        "addon with a missing required dependency should not load"
    );
    drop(state);
    let dependent_loaded: Option<bool> = env
        .eval("return DependentAddonLoaded")
        .expect("read global");
    assert_eq!(dependent_loaded, None);
}

#[test]
fn load_summary_counts_file_event_and_nested_lua_failures_once_per_addon() {
    let temp = tempfile::tempdir().unwrap();
    write_addon_with_lua(temp.path(), "Healthy", "", "healthyLoaded = true");
    write_addon_with_lua(temp.path(), "FileBroken", "", "error('file failure')");
    write_addon_with_lua(
        temp.path(),
        "EventBroken",
        "",
        "local f = CreateFrame('Frame'); f:RegisterEvent('ADDON_LOADED'); f:SetScript('OnEvent', function(_, _, name) if name == 'EventBroken' or name == 'NestedCaller' then error('event failure') end end)",
    );
    write_addon_with_lua(
        temp.path(),
        "NestedChild",
        "## LoadOnDemand: 1\n",
        "error('nested failure')",
    );
    write_addon_with_lua(
        temp.path(),
        "NestedCaller",
        "",
        "assert(C_AddOns.LoadAddOn('NestedChild'))",
    );
    write_addon_with_lua(
        temp.path(),
        "MissingDependency",
        "## Dependencies: Absent\n",
        "error('must not execute')",
    );
    let addons = scan_addons(temp.path(), &[], ScreenKind::Game);
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().addon_base_paths = vec![temp.path().to_path_buf()];
    let mut stats = LoadStats::default();
    load_discovered_addons(&env, &addons, &mut None, None, &mut stats);

    let state = env.state().borrow();
    let recorded: HashSet<_> = state
        .lua_error_records
        .iter()
        .filter_map(|record| record.addon_name.as_deref())
        .collect();
    assert!(recorded.contains("FileBroken"), "{recorded:?}");
    assert!(recorded.contains("EventBroken"), "{recorded:?}");
    assert!(recorded.contains("NestedChild"), "{recorded:?}");
    assert!(recorded.contains("MissingDependency"), "{recorded:?}");
    for name in [
        "Healthy",
        "FileBroken",
        "EventBroken",
        "NestedCaller",
        "NestedChild",
    ] {
        assert!(
            state
                .addons
                .iter()
                .any(|addon| addon.folder_name == name && addon.loaded),
            "loaded state must be preserved for {name}"
        );
    }
    assert!(
        !state
            .addons
            .iter()
            .find(|addon| addon.folder_name == "MissingDependency")
            .unwrap()
            .loaded
    );
    assert_eq!(stats.success_count, 4);
    assert_eq!(
        stats.fail_count, 4,
        "file, callback, nested child, and missing dependency must not report one failure"
    );
    assert_eq!(
        format_load_outcomes(addons.len(), &stats),
        "Loaded: 4/5 addons\nFailed: 4\nLoad failures: 1\nLoaded with Lua errors: 3"
    );
}

#[test]
fn earlier_addon_can_see_later_addon_metadata_before_later_loads() {
    let temp = tempfile::tempdir().expect("tempdir");
    let early_toc = write_addon_with_lua(
        temp.path(),
        "EarlyAddon",
        "",
        r#"
            _G.EarlyAddonSawLaterDisplay = false
            _G.LaterAddonWasLoadedDuringEarlyScan = _G.LaterAddonLoaded == true
            for i = 1, C_AddOns.GetNumAddOns() do
                if C_AddOns.GetAddOnMetadata(i, "X-BugGrabber-Display") == "LaterDisplay" then
                    _G.EarlyAddonSawLaterDisplay = true
                end
            end
        "#,
    );
    let later_toc = write_addon_with_lua(
        temp.path(),
        "LaterAddon",
        "## X-BugGrabber-Display: LaterDisplay\n",
        "_G.LaterAddonLoaded = true\n",
    );
    let addons = vec![
        ("EarlyAddon".to_string(), early_toc),
        ("LaterAddon".to_string(), later_toc),
    ];
    let env = WowLuaEnv::new().expect("create Lua env");
    let mut saved_vars = None;
    let mut stats = LoadStats::default();

    load_discovered_addons(&env, &addons, &mut saved_vars, None, &mut stats);

    let saw_later: bool = env
        .eval("return EarlyAddonSawLaterDisplay")
        .expect("read early scan result");
    let later_loaded_during_scan: bool = env
        .eval("return LaterAddonWasLoadedDuringEarlyScan")
        .expect("read early load result");
    let later_loaded_after_all: bool = env
        .eval("return LaterAddonLoaded == true")
        .expect("read later load result");

    assert!(
        saw_later,
        "metadata for all discovered addons should be available before any addon Lua runs"
    );
    assert!(
        !later_loaded_during_scan,
        "pre-registration must not execute later addon files early"
    );
    assert!(
        later_loaded_after_all,
        "later addon should still load in normal order"
    );
}

#[test]
fn scan_addon_paths_merges_roots_and_keeps_first_duplicate() {
    let temp = tempfile::tempdir().expect("tempdir");
    let sim_root = temp.path().join("sim");
    let wow_root = temp.path().join("wow");
    let sim_shared_toc = write_addon(&sim_root, "SharedAddon");
    write_addon(&sim_root, "SimulatorOnly");
    write_addon(&wow_root, "SharedAddon");
    write_addon(&wow_root, "WowOnly");

    let addons = scan_addon_paths(&[sim_root.clone(), wow_root], &[], ScreenKind::Game);
    let names: Vec<_> = addons.iter().map(|(name, _)| name.as_str()).collect();
    let shared_toc = addons
        .iter()
        .find(|(name, _)| name == "SharedAddon")
        .map(|(_, toc)| toc)
        .expect("shared addon should be present");

    assert_eq!(names, ["SharedAddon", "SimulatorOnly", "WowOnly"]);
    assert_eq!(
        shared_toc, &sim_shared_toc,
        "the first addon root should win duplicate addon names"
    );
}

#[test]
fn scan_addons_skips_out_of_date_interfaces_by_default() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_addon_with_toc(
        temp.path(),
        "CurrentAddon",
        &format!(
            "## Interface: {}\nmain.lua\n",
            wow_ui_sim::toc::ACTIVE_INTERFACE_VERSION
        ),
    );
    write_addon_with_toc(temp.path(), "OldAddon", "## Interface: 120001\nmain.lua\n");

    let addons = scan_addons(temp.path(), &[], ScreenKind::Game);
    let names: Vec<_> = addons.iter().map(|(name, _)| name.as_str()).collect();

    assert_eq!(names, ["CurrentAddon"]);
}

#[test]
#[cfg(feature = "client-mists")]
fn scan_addons_accepts_mists_interface_version() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_addon_with_toc(temp.path(), "ElvUI", "## Interface: 50504\nmain.lua\n");

    let addons = scan_addons(temp.path(), &[], ScreenKind::Game);
    let names: Vec<_> = addons.iter().map(|(name, _)| name.as_str()).collect();

    assert_eq!(names, ["ElvUI"]);
}
