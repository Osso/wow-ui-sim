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

#[test]
fn addon_load_failures_remain_diagnostic_without_persistence() {
    for (name, metadata, lua, expected_error) in [
        ("BrokenLua", "", "error('file failure')", "file failure"),
        ("MissingLua", "", "", "missing.lua"),
        ("BrokenXml", "", "", "broken.xml"),
        ("UnreadableToc", "", "", "TOC"),
        (
            "BrokenEvent",
            "",
            "local f = CreateFrame('Frame'); f:RegisterEvent('ADDON_LOADED'); \
             f:SetScript('OnEvent', function() error('event failure') end)",
            "event failure",
        ),
        (
            "MissingDependency",
            "## Dependencies: MissingRequiredAddon\n",
            "error('dependency must block execution')",
            "MissingRequiredAddon",
        ),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let toc = write_addon_with_lua(temp.path(), name, metadata, lua);
        match name {
            "MissingLua" => {
                let content = std::fs::read_to_string(&toc).unwrap();
                std::fs::write(&toc, format!("{content}missing.lua\n")).unwrap();
            }
            "BrokenXml" => {
                let content = std::fs::read_to_string(&toc).unwrap();
                std::fs::write(&toc, format!("{content}broken.xml\n")).unwrap();
                std::fs::write(toc.parent().unwrap().join("broken.xml"), "<Ui><Frame></Ui>")
                    .unwrap();
            }
            "UnreadableToc" => std::fs::write(&toc, [0xff]).unwrap(),
            _ => {}
        }
        let addons = scan_addons(temp.path(), &[], ScreenKind::Game);
        assert_eq!(
            addons.len(),
            1,
            "selected input must remain observable: {name}"
        );
        let mut env = WowLuaEnv::new().unwrap();
        let mut stats = LoadStats::default();
        load_discovered_addons(&env, &addons, &mut None, None, &mut stats);

        if name == "BrokenEvent" {
            let state = env.state().borrow();
            assert!(
                state.lua_error_records.iter().any(|record| {
                    record.addon_name.as_deref() == Some(name)
                        && record.message.contains(expected_error)
                }),
                "event failure must retain its handler owner: {:?}",
                state.lua_error_records
            );
        }
        assert!(stats.fail_count > 0 || stats.total_warnings > 0, "{name}");
        let diagnostics = format!("{:?}", env.state().borrow().lua_errors);
        if !diagnostics.contains(expected_error) {
            assert!(
                stats.total_warnings > 0 || stats.load_failed_addons.contains(name),
                "{name}: {diagnostics}"
            );
        }
        if name == "BrokenEvent" {
            let previous_owner = env
                .state()
                .borrow()
                .addons
                .iter()
                .position(|addon| addon.folder_name == "__BuiltIn")
                .expect("built-in owner") as u16;
            env.state().borrow_mut().executing_addon_index = Some(previous_owner);
            env.loader_env()
                .fire_event_with_args("ADDON_LOADED", &[env.lua_string(name)])
                .unwrap();
            let state = env.state().borrow();
            assert_eq!(state.executing_addon_index, Some(previous_owner));
            assert_eq!(
                state
                    .lua_error_records
                    .last()
                    .unwrap()
                    .addon_name
                    .as_deref(),
                Some(name)
            );
            drop(state);
            env.state().borrow_mut().executing_addon_index = None;
        }
        env.finalize_session(&mut None).unwrap();
    }
}

#[test]
fn reload_recovers_failed_addon_without_losing_healthy_saved_variables() {
    let temp = tempfile::tempdir().unwrap();
    let broken_toc = write_addon_with_lua(
        temp.path(),
        "BrokenOwner",
        "## SavedVariables: BrokenDB\n",
        "BrokenDB = { value = 'unsafe default' }; error('owner initialization failed')",
    );
    let healthy_toc = write_addon_with_lua(
        temp.path(),
        "HealthyOwner",
        "## SavedVariables: HealthyDB\n",
        "HealthyDB = HealthyDB or {}",
    );
    let storage = temp.path().join("saved");
    std::fs::create_dir(&storage).unwrap();
    let preceding = b"BrokenDB = { value = 'preceding valid save' }\n";
    std::fs::write(storage.join("BrokenOwner.lua"), preceding).unwrap();
    std::fs::write(
        storage.join("HealthyOwner.lua"),
        "HealthyDB = { value = 1 }\n",
    )
    .unwrap();
    let addons = vec![
        ("BrokenOwner".into(), broken_toc.clone()),
        ("HealthyOwner".into(), healthy_toc),
    ];
    let mut env = WowLuaEnv::new().unwrap();
    let mut manager = Some(SavedVariablesManager::with_storage_dir(storage.clone()));
    load_discovered_addons(&env, &addons, &mut manager, None, &mut LoadStats::default());
    env.exec("HealthyDB.value = 'healthy unsaved edit'")
        .unwrap();
    let error = env.finalize_session(&mut manager).unwrap_err().to_string();
    assert!(error.contains("owner initialization failed"), "{error}");
    assert_eq!(
        std::fs::read(storage.join("BrokenOwner.lua")).unwrap(),
        preceding
    );
    // Wrath restores SavedVariables after addon files; both profiles have them at ADDON_LOADED.
    std::fs::write(
        broken_toc.parent().unwrap().join("main.lua"),
        "local f = CreateFrame('Frame'); f:RegisterEvent('ADDON_LOADED'); \
         f:SetScript('OnEvent', function(_, _, name) \
             if name == 'BrokenOwner' then \
                 assert(BrokenDB.value == 'preceding valid save'); \
                 BrokenDB.value = 'recovered edit' \
             end \
         end)",
    )
    .unwrap();
    env.set_reload_loader(move |env, manager| {
        load_discovered_addons(env, &addons, manager, None, &mut LoadStats::default());
        Ok(())
    });
    env.exec("ReloadUI()").unwrap();
    assert!(env.service_reload(&mut manager).unwrap());
    env.exec(
        "assert(BrokenDB.value == 'recovered edit' and HealthyDB.value == 'healthy unsaved edit')",
    )
    .unwrap();
    assert_eq!(
        std::fs::read(storage.join("BrokenOwner.lua")).unwrap(),
        preceding
    );
    env.finalize_session(&mut manager).unwrap();
    let mut restored = rilua::Lua::new().unwrap();
    restored
        .exec(&std::fs::read_to_string(storage.join("BrokenOwner.lua")).unwrap())
        .unwrap();
    restored
        .exec(&std::fs::read_to_string(storage.join("HealthyOwner.lua")).unwrap())
        .unwrap();
    restored.exec("assert(BrokenDB.value == 'recovered edit' and HealthyDB.value == 'healthy unsaved edit')").unwrap();
}

#[test]
fn reload_rejects_missing_failed_input_and_retries_its_repaired_source() {
    let temp = tempfile::tempdir().unwrap();
    let toc = write_addon_with_lua(
        temp.path(),
        "InputOwner",
        "## SavedVariables: InputDB\n",
        "InputDB = InputDB or { value = 'unsafe default' }",
    );
    let config = wow_ui_sim::saved_variables::WtfConfig::new(
        temp.path().join("WTF"),
        "Account",
        "Realm",
        "Hero",
    );
    let source = config.account_saved_vars_file("InputOwner");
    std::fs::create_dir_all(source.parent().unwrap()).unwrap();
    std::fs::write(&source, "InputDB = { invalid").unwrap();
    let storage = temp.path().join("saved");
    std::fs::create_dir(&storage).unwrap();
    let preceding = b"InputDB = { value = 'stale local data' }\n";
    std::fs::write(storage.join("InputOwner.lua"), preceding).unwrap();
    let mut manager = SavedVariablesManager::with_storage_dir(storage.clone());
    manager.set_wtf_config(config);
    let mut manager = Some(manager);
    let mut env = WowLuaEnv::new().unwrap();
    let addons = vec![("InputOwner".into(), toc)];
    load_discovered_addons(&env, &addons, &mut manager, None, &mut LoadStats::default());
    env.set_reload_loader(move |env, manager| {
        load_discovered_addons(env, &addons, manager, None, &mut LoadStats::default());
        Ok(())
    });
    std::fs::remove_file(&source).unwrap();
    env.exec("ReloadUI()").unwrap();
    let error = env.service_reload(&mut manager).unwrap_err().to_string();
    assert!(error.contains("input is missing"), "{error}");
    assert_eq!(
        std::fs::read(storage.join("InputOwner.lua")).unwrap(),
        preceding
    );
    std::fs::write(&source, "InputDB = { value = 'repaired original' }\n").unwrap();
    env.exec("ReloadUI()").unwrap();
    assert!(env.service_reload(&mut manager).unwrap());
    env.exec("assert(InputDB.value == 'repaired original')")
        .unwrap();
    env.finalize_session(&mut manager).unwrap();
    assert_eq!(
        std::fs::read_to_string(source).unwrap(),
        "InputDB = { value = 'repaired original' }\n"
    );
}

#[test]
fn optional_import_failures_do_not_block_registered_saved_variables() {
    let temp = tempfile::tempdir().unwrap();
    let config = wow_ui_sim::saved_variables::WtfConfig::new(
        temp.path().join("WTF"),
        "Account",
        "Realm",
        "Hero",
    );
    std::fs::create_dir_all(config.edit_mode_account_cache_file()).unwrap();
    let snapshot = config.account_saved_vars_file("ServerSnapshot");
    std::fs::create_dir_all(snapshot.parent().unwrap()).unwrap();
    let invalid_snapshot = b"ServerSnapshotDB = { invalid";
    std::fs::write(&snapshot, invalid_snapshot).unwrap();
    let storage = temp.path().join("saved");
    let mut manager = SavedVariablesManager::with_storage_dir(storage.clone());
    manager.set_wtf_config(config);
    let mut env = WowLuaEnv::new().unwrap();
    env.loader_env()
        .with_state(|state| manager.init_for_addon(state, "Healthy", &["HealthyDB".into()], &[]))
        .unwrap();
    env.exec("HealthyDB = { value = 'healthy edit' }").unwrap();
    load_edit_mode_cache(&env, Some(&manager), None);
    let mut manager = Some(manager);
    let _ = merged_addon_enable_overrides(&env, &mut manager);
    env.finalize_session(&mut manager).unwrap();
    let mut restored = rilua::Lua::new().unwrap();
    restored
        .exec(&std::fs::read_to_string(storage.join("Healthy.lua")).unwrap())
        .unwrap();
    restored
        .exec("assert(HealthyDB.value == 'healthy edit')")
        .unwrap();
    assert_eq!(std::fs::read(snapshot).unwrap(), invalid_snapshot);
    assert!(!storage.join("ServerSnapshot.lua").exists());
}

#[test]
fn optional_nil_probes_and_disabled_addons_allow_finalization() {
    let temp = tempfile::tempdir().unwrap();
    write_addon_with_lua(
        temp.path(),
        "OptionalProbe",
        "",
        "OptionalProbePassed = MissingOptionalGlobal == nil",
    );
    write_addon_with_lua(
        temp.path(),
        "DisabledBroken",
        "## Dependencies: Blizzard_MissingDependency\n",
        "error('disabled files must not execute')",
    );
    let addons = scan_addons(temp.path(), &[], ScreenKind::Game);
    let mut env = WowLuaEnv::new().unwrap();
    let overrides = HashMap::from([("DisabledBroken".to_string(), false)]);
    let mut stats = LoadStats::default();
    load_discovered_addons(&env, &addons, &mut None, Some(&overrides), &mut stats);

    assert!(env.eval::<bool>("return OptionalProbePassed").unwrap());
    assert_eq!(stats.success_count, 1);
    assert_eq!(stats.fail_count, 0);
    env.finalize_session(&mut None).unwrap();
}

#[test]
#[cfg(feature = "client-retail")]
fn retail_third_party_game_type_headers_select_startup_addons() {
    let temp = tempfile::tempdir().unwrap();
    for (name, value, expected) in [
        ("Bracketed", "camelot [AllowLoadGameType standard]", true),
        ("Unknown", "imaginary, wowlabs", true),
        ("CaseComma", "CLASSIC, MaInLiNe", true),
        ("Other", "classic, plunderstorm", false),
    ] {
        write_addon_with_lua(
            temp.path(),
            name,
            &format!("## AllowLoadGameType: {value}\n"),
            "GameTypeProbeLoaded = true",
        );
        assert_eq!(
            loadable_toc_path(&temp.path().join(name), ScreenKind::Game).is_some(),
            expected,
            "{name}"
        );
    }
    let toc = loadable_toc_path(&temp.path().join("Bracketed"), ScreenKind::Game).unwrap();
    let env = WowLuaEnv::new().unwrap();
    wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    assert!(env.eval::<bool>("return GameTypeProbeLoaded").unwrap());
}

#[test]
fn third_party_lod_metadata_precedes_startup_and_explicit_load_runs_once() {
    let temp = tempfile::tempdir().unwrap();
    write_addon_with_lua(
        temp.path(),
        "Blizzard_LodMetadataDependency",
        "## LoadOnDemand: 1\n",
        "LodDependencyRuns = (LodDependencyRuns or 0) + 1",
    );
    let lazy_toc = write_addon_with_lua(
        temp.path(),
        "LazyMetadata",
        "## LoadOnDemand: 1\n## Title: Deferred settings\n## Notes: Metadata before code\n## Dependencies: Blizzard_LodMetadataDependency\n",
        "assert(LodDependencyRuns == 1); LazyMetadataRuns = (LazyMetadataRuns or 0) + 1",
    );
    let mut toc = std::fs::read_to_string(&lazy_toc).unwrap();
    toc.push_str("panel.xml\n");
    std::fs::write(&lazy_toc, toc).unwrap();
    std::fs::write(
        lazy_toc.parent().unwrap().join("panel.xml"),
        r#"<Ui><Frame name="LazyMetadataPanel"><Scripts><OnLoad>LazyMetadataXmlRuns = (LazyMetadataXmlRuns or 0) + 1</OnLoad></Scripts></Frame></Ui>"#,
    )
    .unwrap();
    write_addon_with_lua(
        temp.path(),
        "A_MetadataObserver",
        "",
        r#"
        EagerMetadataRuns = (EagerMetadataRuns or 0) + 1
        assert(C_AddOns.IsAddOnLoadOnDemand('LazyMetadata'))
        assert(C_AddOns.IsAddOnLoadOnDemand('Blizzard_LodMetadataDependency'))
        local name, title, notes, enabled = C_AddOns.GetAddOnInfo('LazyMetadata')
        assert(name == 'LazyMetadata' and title == 'Deferred settings')
        assert(notes == 'Metadata before code' and enabled)
        assert(C_AddOns.GetAddOnEnableState('LazyMetadata') == 2)
        assert(C_AddOns.GetAddOnDependencies('LazyMetadata') == 'Blizzard_LodMetadataDependency')
        local loading, loaded = C_AddOns.IsAddOnLoaded('LazyMetadata')
        assert(not loading and not loaded)
        assert(LazyMetadataRuns == nil and LazyMetadataPanel == nil)
        assert(LodDependencyRuns == nil)
        MetadataAddonCount = C_AddOns.GetNumAddOns()
        "#,
    );
    let addons = scan_addons(temp.path(), &[], ScreenKind::Game);
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().addon_base_paths = vec![temp.path().to_path_buf()];
    let mut stats = LoadStats::default();
    load_discovered_addons(&env, &addons, &mut None, None, &mut stats);
    env.exec(
        r#"
        assert(EagerMetadataRuns == 1)
        assert(LazyMetadataRuns == nil and LazyMetadataXmlRuns == nil)
        assert(LazyMetadataPanel == nil and LodDependencyRuns == nil)
        assert(not C_AddOns.IsAddOnLoaded('Blizzard_LodMetadataDependency'))
        assert(C_AddOns.LoadAddOn('LazyMetadata'))
        local loading, loaded = C_AddOns.IsAddOnLoaded('LazyMetadata')
        assert(loading and loaded and LazyMetadataRuns == 1)
        assert(LazyMetadataXmlRuns == 1 and LazyMetadataPanel:GetName() == 'LazyMetadataPanel')
        assert(LodDependencyRuns == 1)
        assert(C_AddOns.IsAddOnLoaded('Blizzard_LodMetadataDependency'))
        assert(C_AddOns.GetNumAddOns() == MetadataAddonCount)
        local name, title, notes, enabled = C_AddOns.GetAddOnInfo('LazyMetadata')
        assert(name == 'LazyMetadata' and title == 'Deferred settings')
        assert(notes == 'Metadata before code' and enabled)
        assert(C_AddOns.IsAddOnLoadOnDemand('LazyMetadata'))
        assert(C_AddOns.GetAddOnDependencies('LazyMetadata') == 'Blizzard_LodMetadataDependency')
        assert(C_AddOns.LoadAddOn('LazyMetadata'))
        assert(LazyMetadataRuns == 1 and LazyMetadataXmlRuns == 1 and LodDependencyRuns == 1)
        assert(C_AddOns.GetNumAddOns() == MetadataAddonCount and EagerMetadataRuns == 1)
        "#,
    )
    .unwrap();
    assert_eq!(stats.success_count, 1);
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn third_party_disabled_lod_metadata_retains_enable_state_and_dependencies() {
    let temp = tempfile::tempdir().unwrap();
    write_addon_with_lua(
        temp.path(),
        "DisabledLazyMetadata",
        "## LoadOnDemand: 1\n## Dependencies: AbsentLibrary\n",
        "error('disabled LoD files must not execute')",
    );
    let addons = scan_addons(temp.path(), &[], ScreenKind::Game);
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().addon_base_paths = vec![temp.path().to_path_buf()];
    let overrides = HashMap::from([("DisabledLazyMetadata".to_string(), false)]);
    let mut stats = LoadStats::default();
    load_discovered_addons(&env, &addons, &mut None, Some(&overrides), &mut stats);
    env.exec(
        r#"
        assert(C_AddOns.IsAddOnLoadOnDemand('DisabledLazyMetadata'))
        local name, _, _, enabled, reason = C_AddOns.GetAddOnInfo('DisabledLazyMetadata')
        assert(name == 'DisabledLazyMetadata' and not enabled and reason == 'DISABLED')
        assert(C_AddOns.GetAddOnEnableState('DisabledLazyMetadata') == 0)
        assert(C_AddOns.GetAddOnDependencies('DisabledLazyMetadata') == 'AbsentLibrary')
        local before = C_AddOns.GetNumAddOns()
        local loaded, failure = C_AddOns.LoadAddOn('DisabledLazyMetadata')
        assert(not loaded and failure == 'DISABLED')
        assert(not C_AddOns.IsAddOnLoaded('DisabledLazyMetadata'))
        assert(C_AddOns.GetNumAddOns() == before)
        "#,
    )
    .unwrap();
    assert_eq!(stats.success_count, 0);
    assert_eq!(stats.fail_count, 0);
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn third_party_lod_discovery_preserves_screen_game_and_interface_filters() {
    let temp = tempfile::tempdir().unwrap();
    for (name, metadata) in [
        ("GameLazy", "## AllowLoad: Game\n"),
        ("GlueLazy", "## AllowLoad: Glue\n"),
        (
            "ExcludedLazy",
            "## ExcludeLoadGameType: mainline, classic\n",
        ),
    ] {
        write_addon_with_lua(
            temp.path(),
            name,
            &format!("## LoadOnDemand: 1\n{metadata}"),
            "error('discovery must not run Lua')",
        );
    }
    write_addon_with_toc(
        temp.path(),
        "IncompatibleLazy",
        "## Interface: 999999\n## LoadOnDemand: 1\nmain.lua\n",
    );
    // R13 (pr46-49-decomp-answers.md): insecure TOCs ignore AllowLoad and
    // retain Game. Mapping ordinary third-party AddOns to this mode is INFERRED
    // (R64 Q1); game-type and interface-version filters still apply.
    let game = scan_addons(temp.path(), &[], ScreenKind::Game);
    let glue = scan_addons(temp.path(), &[], ScreenKind::CharacterSelect);
    assert_eq!(
        game.iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        ["GameLazy", "GlueLazy"]
    );
    assert!(glue.is_empty(), "user addons must not enter Glue discovery");
}

#[cfg(feature = "retail-12-1-0")]
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
    let startup_count = load_discovered_addons(&env, &addons, &mut None, None, &mut stats);

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
    let event_errors: Vec<_> = state
        .lua_error_records
        .iter()
        .filter(|record| record.message.contains("event failure"))
        .collect();
    assert_eq!(event_errors.len(), 2, "{event_errors:?}");
    assert!(
        event_errors
            .iter()
            .all(|record| record.addon_name.as_deref() == Some("EventBroken")),
        "callbacks during another addon's load must retain their own owner: {event_errors:?}"
    );
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
        format_load_outcomes(startup_count, &stats),
        "Loaded: 4/5 addons\nFailed during loading: 4\nLoad failures: 1\nLoaded with Lua errors during loading: 3"
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
