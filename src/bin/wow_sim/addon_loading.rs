//! Blizzard and third-party addon loading with timing/summary.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use wow_ui_sim::client_profile;
use wow_ui_sim::loader::{
    LoadResult, LoadTiming, StartupAddonLoadKind, discover_blizzard_startup_addons_for_screen,
    load_startup_addon, startup_bootstrap_eligible,
};
use wow_ui_sim::logging;
use wow_ui_sim::lua_api::state::LuaErrorRecord;
use wow_ui_sim::lua_api::{AddonInfo, WowLuaEnv};
use wow_ui_sim::saved_variables::SavedVariablesManager;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

mod blizzard_dependencies;
mod cache_summary;
use blizzard_dependencies::load_required_blizzard_dependencies_for_addons;
use cache_summary::{format_cache_info, print_cache_stats};
mod enable_state;
use enable_state::{addon_enable_overrides, addon_enabled, dependency_aware_enable_overrides};
mod lua_file_profile;
use lua_file_profile::print_slowest_lua_files;
mod slowest_addons;
use slowest_addons::print_slowest_addons;
mod warning_report;
use warning_report::print_addon_warnings;

pub const TEST_ADDONS_PATH: &str = "./Interface/TestAddOns";

/// Addon names that are test-only and should not be loaded in GUI mode.
pub const TEST_ADDONS: &[&str] = &["Wowless", "WowlessData", "WowBehaviorTest", "WowDiscovery"];

pub fn load_edit_mode_cache(
    env: &WowLuaEnv,
    saved_vars: Option<&SavedVariablesManager>,
    snapshot_layout: Option<&str>,
) {
    let Some(saved_vars) = saved_vars else {
        return;
    };

    let active_spec_index = env.state().borrow().player.active_spec_index;
    match env.loader_env().with_state(|state| {
        saved_vars.load_edit_mode_cache(state, active_spec_index, snapshot_layout)
    }) {
        Ok(true) => logging::println_elapsed("Loaded EditMode layout cache from WTF"),
        Ok(false) => {}
        Err(error) => logging::println_elapsed(&format!(
            "Failed to load EditMode layout cache from WTF: {error}"
        )),
    }
}

/// Load Blizzard SharedXML and base UI addons (auto-discovered, dependency-sorted).
pub fn load_blizzard_addons(
    env: &WowLuaEnv,
    saved_vars: &mut Option<SavedVariablesManager>,
    screen: ScreenKind,
) {
    let addons_dir = blizzard_ui_addons_dir_or_exit();
    let verbose = std::env::var("WOW_SIM_VERBOSE").is_ok();
    let blizzard_start = std::time::Instant::now();
    let mut total_timing = LoadTiming::default();

    env.gc_stop();

    // Wrath ships a top-level FrameXML.toc that pre-dates the retail Blizzard_*
    // split. Load it as a synthetic addon before iterating Blizzard_* dirs so
    // global UI templates (UIParent, GameTooltip, etc.) register first.
    if let Some(framexml_toc) = client_profile::blizzard_ui_framexml_toc() {
        load_one_blizzard_addon(
            env,
            "FrameXML",
            &framexml_toc,
            StartupAddonLoadKind::Full,
            saved_vars,
            verbose,
            &mut total_timing,
        );
    }

    let addons = discover_blizzard_startup_addons_for_screen(&addons_dir, screen);
    logging::println_elapsed(&format!("Loading {} Blizzard addons...", addons.len()));

    for addon in &addons {
        load_one_blizzard_addon(
            env,
            &addon.name,
            &addon.toc_path,
            addon.kind,
            saved_vars,
            verbose,
            &mut total_timing,
        );
        if addon.name == "Blizzard_EnvironmentCleanup" {
            env.restore_post_cleanup_globals();
        }
    }

    env.sync_string_metatable_to_global_string();
    print_blizzard_summary(blizzard_start.elapsed(), &total_timing);
}

fn blizzard_ui_addons_dir_or_exit() -> PathBuf {
    let profile = client_profile::ACTIVE;
    if let Some(addons_dir) = wow_ui_sim::blizzard_ui_sync::cached_blizzard_ui_addons_path() {
        return addons_dir;
    }

    let addons_dir = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
        .unwrap_or_else(|_| client_profile::blizzard_ui_addons_dir());
    eprintln!(
        "FATAL: {} is missing or stale: Blizzard UI source not set up for {:?} profile.\n\
         Run from the repo root:\n\
         \t./scripts/setup-blizzard-ui.sh {}",
        addons_dir.display(),
        profile,
        profile.subdir().to_lowercase(),
    );
    std::process::exit(1);
}

fn load_one_blizzard_addon(
    env: &WowLuaEnv,
    name: &str,
    toc_path: &Path,
    kind: StartupAddonLoadKind,
    saved_vars: &mut Option<SavedVariablesManager>,
    verbose: bool,
    timing: &mut LoadTiming,
) {
    let result = load_startup_addon(&env.loader_env(), toc_path, kind, saved_vars.as_mut());
    match result {
        Ok(r) => record_blizzard_addon_success(env, name, kind, verbose, timing, r),
        Err(e) => println!("{} failed: {}", name, e),
    }
}

fn record_blizzard_addon_success(
    env: &WowLuaEnv,
    name: &str,
    kind: StartupAddonLoadKind,
    verbose: bool,
    timing: &mut LoadTiming,
    result: LoadResult,
) {
    print_verbose_blizzard_status(name, verbose, &result);
    print_load_diagnostics(&result);
    if kind == StartupAddonLoadKind::Full {
        fire_addon_loaded(env, name);
    }
    timing.accumulate(&result.timing);
}

fn print_verbose_blizzard_status(name: &str, verbose: bool, result: &LoadResult) {
    if !verbose {
        return;
    }
    let t = &result.timing;
    println!(
        "{} loaded: {} Lua, {} XML, {} warnings, {} nil observations, {} missing requirements ({:.1?}: xmlproc={:.1?} exec_lua={:.1?} lifecycle={:.1?} layers={:.1?} lua={:.1?} [compile={:.1?} call={:.1?}] frames={})",
        name,
        result.lua_files,
        result.xml_files,
        result.warnings.len(),
        result.nil_symbol_observations.len(),
        result.missing_requirements.len(),
        t.total(),
        t.xml_process_time,
        t.frame_exec_lua_time,
        t.frame_lifecycle_time,
        t.frame_layer_children_time,
        t.lua_exec_time,
        t.lua_compile_time,
        t.lua_call_time,
        t.frame_count
    );
}

fn print_load_diagnostics(result: &LoadResult) {
    if std::env::var("WOW_SIM_DEBUG_NIL_GLOBALS").is_err() {
        return;
    }
    for warning in &result.warnings {
        println!("  [failure] {warning}");
    }
    for observation in &result.nil_symbol_observations {
        println!("  [nil-observation] {observation}");
    }
    for requirement in &result.missing_requirements {
        println!("  [missing-requirement] {requirement}");
    }
}

fn fire_addon_loaded(env: &WowLuaEnv, name: &str) {
    if let Err(e) = env.fire_event_with_args("ADDON_LOADED", &[env.lua_string(name)]) {
        logging::println_elapsed(&format!("Error firing ADDON_LOADED for {name}: {e}"));
    }
}

fn print_blizzard_frame_detail(t: &LoadTiming) {
    println!(
        "  frame breakdown: setup={:.2?} finalize={:.2?} ({} frames)",
        t.xml_frame_setup_time, t.xml_frame_finalize_time, t.frame_count
    );
    println!(
        "  setup: code_build={:.2?} exec_lua={:.2?} props={:.2?}",
        t.frame_code_build_time, t.frame_exec_lua_time, t.frame_apply_props_time
    );
    println!(
        "  finalize: layers={:.2?} ({} tex, {} fs) anim={:.2?} button={:.2?} lifecycle={:.2?} ({} fires)",
        t.frame_layer_children_time,
        t.texture_count,
        t.fontstring_count,
        t.frame_anim_time,
        t.frame_button_time,
        t.frame_lifecycle_time,
        t.lifecycle_fire_count
    );
}

fn print_blizzard_summary(elapsed: std::time::Duration, t: &LoadTiming) {
    let cache_info = format_cache_info(t);
    logging::println_elapsed(&format!(
        "Blizzard addons loaded in {elapsed:.2?} (io={:.2?} xml={:.2?} xmlproc={:.2?} frames⊂xmlproc={:.2?} lua={:.2?} [compile={:.2?} call={:.2?}] gc=deferred{cache_info})",
        t.io_time,
        t.xml_parse_time,
        t.xml_process_time,
        t.xml_frame_create_time,
        t.lua_exec_time,
        t.lua_compile_time,
        t.lua_call_time
    ));
    print_blizzard_frame_detail(t);
    if let Some(report) = wow_ui_sim::loader::fast_create_frame_profile_report() {
        println!("  {report}");
    }
    if let Some(report) = wow_ui_sim::loader::fast_create_frame_profile_body_report() {
        println!("  {report}");
    }
}

/// Scan, load, and register third-party addons; print summary.
pub fn load_third_party_addons(
    skip_addons: bool,
    is_test: bool,
    env: &WowLuaEnv,
    saved_vars: &mut Option<SavedVariablesManager>,
    screen: ScreenKind,
) {
    if skip_addons && !is_test {
        logging::println_elapsed("Addon loading disabled");
        return;
    }

    let exclude = if is_test { &[][..] } else { TEST_ADDONS };
    let addon_paths = wow_ui_sim::paths::default_addons_paths();
    let mut addons = scan_addon_paths(&addon_paths, exclude, screen);
    if is_test {
        let test_addons_path = PathBuf::from(TEST_ADDONS_PATH);
        addons.extend(scan_addons(&test_addons_path, &[], screen));
    }
    load_required_blizzard_dependencies_for_addons(env, saved_vars, screen, &addons);
    wow_ui_sim::loader::sort_addons_by_dependencies(&mut addons);
    if skip_addons {
        addons.retain(|(name, _)| TEST_ADDONS.iter().any(|t| t == name));
    }
    if addons.is_empty() {
        return;
    }

    logging::println_elapsed(&format!("Loading {} addons...", addons.len()));
    let enable_overrides = merged_addon_enable_overrides(env, saved_vars);
    let effective_enable_overrides =
        dependency_aware_enable_overrides(&addons, enable_overrides.as_ref());
    let mut stats = LoadStats::default();
    load_discovered_addons(
        env,
        &addons,
        saved_vars,
        effective_enable_overrides.as_ref(),
        &mut stats,
    );
    print_load_summary(&addons, &stats);
}

fn merged_addon_enable_overrides(
    env: &WowLuaEnv,
    saved_vars: &mut Option<SavedVariablesManager>,
) -> Option<HashMap<String, bool>> {
    let mut overrides = addon_enable_overrides(saved_vars.as_ref()).unwrap_or_default();

    if let Some(saved_vars) = saved_vars.as_mut() {
        match wow_ui_sim::server_snapshot_import::load_addon_enable_overrides(env, saved_vars) {
            Ok(snapshot_overrides) => overrides.extend(snapshot_overrides),
            Err(error) => logging::println_elapsed(&format!(
                "ServerSnapshot addon enable import failed: {error}"
            )),
        }
    }

    (!overrides.is_empty()).then_some(overrides)
}

fn is_addon_loaded(env: &WowLuaEnv, name: &str) -> bool {
    env.state()
        .borrow()
        .addons
        .iter()
        .any(|addon| addon.folder_name == name && addon.loaded)
}

pub fn scan_addon_paths(
    base_paths: &[PathBuf],
    exclude: &[&str],
    screen: ScreenKind,
) -> Vec<(String, PathBuf)> {
    let mut addons = Vec::new();
    let mut seen = HashSet::new();

    for base_path in base_paths {
        for (name, toc_path) in scan_addons(base_path, exclude, screen) {
            if seen.insert(name.clone()) {
                addons.push((name, toc_path));
            }
        }
    }

    addons
}

pub fn scan_addons(
    base_path: &Path,
    exclude: &[&str],
    screen: ScreenKind,
) -> Vec<(String, PathBuf)> {
    let mut addons = Vec::new();
    let Ok(entries) = std::fs::read_dir(base_path) else {
        return addons;
    };
    for entry in entries.flatten() {
        if let Some(addon) = scanned_addon(entry.path(), exclude, screen) {
            addons.push(addon);
        }
    }
    addons.sort_by(|(left, _), (right, _)| left.cmp(right));
    addons
}

fn scanned_addon(path: PathBuf, exclude: &[&str], screen: ScreenKind) -> Option<(String, PathBuf)> {
    if !path.is_dir() {
        return None;
    }
    let name = path.file_name()?.to_str()?.to_string();
    if should_skip_addon_dir(&name, exclude) {
        return None;
    }
    let toc_path = loadable_toc_path(&path, screen)?;
    Some((name, toc_path))
}

fn should_skip_addon_dir(name: &str, exclude: &[&str]) -> bool {
    name.starts_with('.') || name == "BlizzardUI" || exclude.contains(&name)
}

fn loadable_toc_path(path: &Path, screen: ScreenKind) -> Option<PathBuf> {
    let toc_path = wow_ui_sim::loader::find_toc_file(path)?;
    let toc = TocFile::from_file(&toc_path).ok()?;
    let supports_screen = toc.allows_screen(screen);
    let supported_game_type = !toc.is_ptr_only() && !toc.is_game_type_restricted();
    let supported_interface = load_out_of_date_addons()
        || toc.supports_interface_version(wow_ui_sim::toc::ACTIVE_INTERFACE_VERSION);
    let startup_loadable = !toc.is_load_on_demand() || startup_bootstrap_eligible(&toc);
    (supports_screen && supported_game_type && supported_interface && startup_loadable)
        .then_some(toc_path)
}

fn load_out_of_date_addons() -> bool {
    std::env::var_os("WOW_SIM_LOAD_OUT_OF_DATE_ADDONS").is_some_and(|value| {
        let value = value.to_string_lossy();
        value == "1" || value.eq_ignore_ascii_case("true")
    })
}

/// Accumulated statistics from loading addons.
#[derive(Default)]
struct LoadStats {
    total_lua: usize,
    total_xml: usize,
    total_warnings: usize,
    total_timing: LoadTiming,
    success_count: usize,
    fail_count: usize,
    addon_timings: Vec<AddonTiming>,
}

#[derive(Clone)]
struct AddonTiming {
    name: String,
    timing: LoadTiming,
}

struct AddonMetadata {
    title: String,
    notes: String,
    metadata: HashMap<String, String>,
    load_on_demand: bool,
    default_enabled: bool,
    dependencies: Vec<String>,
    use_secure_env: bool,
}

fn parse_addon_metadata(name: &str, toc_path: &Path) -> AddonMetadata {
    let Some(toc) = TocFile::from_file(toc_path).ok() else {
        return AddonMetadata {
            title: name.to_string(),
            notes: String::new(),
            metadata: HashMap::new(),
            load_on_demand: false,
            default_enabled: true,
            dependencies: Vec::new(),
            use_secure_env: false,
        };
    };

    AddonMetadata {
        title: toc
            .metadata
            .get("Title")
            .cloned()
            .unwrap_or_else(|| name.to_string()),
        notes: toc.metadata.get("Notes").cloned().unwrap_or_default(),
        metadata: toc.metadata.clone(),
        load_on_demand: toc.is_load_on_demand(),
        default_enabled: toc.default_enabled(),
        dependencies: toc.dependencies(),
        use_secure_env: toc.is_secure_env(),
    }
}

fn load_discovered_addons(
    env: &WowLuaEnv,
    addons: &[(String, PathBuf)],
    saved_vars: &mut Option<SavedVariablesManager>,
    enable_overrides: Option<&HashMap<String, bool>>,
    stats: &mut LoadStats,
) {
    register_discovered_addons(env, addons, enable_overrides);
    for (name, toc_path) in addons {
        load_registered_single_addon(env, name, toc_path, saved_vars, stats);
    }
}

fn register_discovered_addons(
    env: &WowLuaEnv,
    addons: &[(String, PathBuf)],
    enable_overrides: Option<&HashMap<String, bool>>,
) {
    for (name, toc_path) in addons {
        let metadata = parse_addon_metadata(name, toc_path);
        let enabled = addon_enabled(name, &metadata, enable_overrides);
        register_or_update_addon(env, name, metadata, enabled);
    }
}

fn register_or_update_addon(env: &WowLuaEnv, name: &str, metadata: AddonMetadata, enabled: bool) {
    let mut state = env.state().borrow_mut();
    if let Some(addon) = state
        .addons
        .iter_mut()
        .find(|addon| addon.folder_name == name)
    {
        addon.title = metadata.title;
        addon.notes = metadata.notes;
        addon.enabled = enabled;
        addon.load_on_demand = metadata.load_on_demand;
        addon.use_secure_env = metadata.use_secure_env;
        addon.dependencies = metadata.dependencies;
        addon.metadata = metadata.metadata;
        addon.default_enabled = metadata.default_enabled;
        return;
    }

    state.addons.push(AddonInfo {
        folder_name: name.to_string(),
        title: metadata.title,
        notes: metadata.notes,
        enabled,
        loaded: false,
        load_on_demand: metadata.load_on_demand,
        bootstrap_loaded: false,
        use_secure_env: metadata.use_secure_env,
        dependencies: metadata.dependencies,
        metadata: metadata.metadata,
        default_enabled: metadata.default_enabled,
        ..Default::default()
    });
}

fn load_registered_single_addon(
    env: &WowLuaEnv,
    name: &str,
    toc_path: &Path,
    saved_vars: &mut Option<SavedVariablesManager>,
    stats: &mut LoadStats,
) {
    if !should_load_registered_addon(env, name) {
        return;
    }

    if let Some(message) = missing_required_dependency_error(env, name) {
        record_addon_load_error(env, name, message);
        stats.fail_count += 1;
        return;
    }

    print_verbose_addon_start(name);
    let toc = match TocFile::from_file(toc_path) {
        Ok(toc) => toc,
        Err(e) => {
            println!("✗ {} failed: {}", name, e);
            stats.fail_count += 1;
            return;
        }
    };
    let kind = if startup_bootstrap_eligible(&toc) {
        StartupAddonLoadKind::BootstrapOnly
    } else {
        StartupAddonLoadKind::Full
    };
    let result = load_startup_addon(&env.loader_env(), toc_path, kind, saved_vars.as_mut());
    match result {
        Ok(r) => {
            if kind == StartupAddonLoadKind::Full {
                mark_addon_loaded(env, name, &r);
                fire_addon_loaded(env, name);
            }
            record_addon_success(name, &r, stats);
        }
        Err(e) => {
            println!("✗ {} failed: {}", name, e);
            stats.fail_count += 1;
        }
    }
}

fn should_load_registered_addon(env: &WowLuaEnv, name: &str) -> bool {
    env.state()
        .borrow()
        .addons
        .iter()
        .find(|addon| addon.folder_name == name)
        .is_some_and(|addon| addon.enabled && !addon.loaded)
}

fn missing_required_dependency_error(env: &WowLuaEnv, name: &str) -> Option<String> {
    let state = env.state().borrow();
    let addon = state
        .addons
        .iter()
        .find(|addon| addon.folder_name == name)?;
    let missing: Vec<_> = addon
        .dependencies
        .iter()
        .filter(|dependency| {
            !state
                .addons
                .iter()
                .any(|addon| addon.folder_name == **dependency)
        })
        .cloned()
        .collect();

    (!missing.is_empty()).then(|| {
        format!(
            "{name} missing required TOC dependencies: {}",
            missing.join(", ")
        )
    })
}

fn record_addon_load_error(env: &WowLuaEnv, name: &str, message: String) {
    let mut state = env.state().borrow_mut();
    state.lua_errors.push(message.clone());
    state.lua_error_records.push(LuaErrorRecord {
        message: message.clone(),
        addon_name: Some(name.to_string()),
    });
    *state.lua_error_counts.entry(message).or_insert(0) += 1;
}

fn print_verbose_addon_start(name: &str) {
    if std::env::var("WOW_SIM_VERBOSE").is_ok() {
        eprintln!("→ {name} loading...");
    }
}

fn mark_addon_loaded(env: &WowLuaEnv, name: &str, r: &LoadResult) {
    let t = r.timing.total().as_secs_f64();
    let mut s = env.state().borrow_mut();
    if let Some(a) = s.addons.iter_mut().find(|a| a.folder_name == name) {
        a.loaded = true;
        a.load_time_secs = t;
    }
}

fn record_addon_success(name: &str, r: &LoadResult, stats: &mut LoadStats) {
    print_verbose_addon_status(name, r);
    stats.addon_timings.push(AddonTiming {
        name: name.to_string(),
        timing: r.timing.clone(),
    });
    print_addon_warnings(name, r);
    stats.total_lua += r.lua_files;
    stats.total_xml += r.xml_files;
    stats.total_warnings += r.warnings.len();
    stats.total_timing.accumulate(&r.timing);
    stats.success_count += 1;
}

fn print_verbose_addon_status(name: &str, r: &LoadResult) {
    if std::env::var("WOW_SIM_VERBOSE").is_err() {
        return;
    }
    let status = if r.warnings.is_empty() { "✓" } else { "⚠" };
    let t = &r.timing;
    println!(
        "{} {} loaded: {} Lua, {} XML, {} warnings, {} nil observations, {} missing requirements ({:.1?} total: io={:.1?} xml={:.1?} xmlproc={:.1?} frames⊂xmlproc={:.1?} setup⊂frames={:.1?} finalize⊂frames={:.1?} lua={:.1?} [compile={:.1?} call={:.1?}] sv={:.1?})",
        status,
        name,
        r.lua_files,
        r.xml_files,
        r.warnings.len(),
        r.nil_symbol_observations.len(),
        r.missing_requirements.len(),
        t.total(),
        t.io_time,
        t.xml_parse_time,
        t.xml_process_time,
        t.xml_frame_create_time,
        t.xml_frame_setup_time,
        t.xml_frame_finalize_time,
        t.lua_exec_time,
        t.lua_compile_time,
        t.lua_call_time,
        t.saved_vars_time
    );
}

fn print_load_summary(addons: &[(String, PathBuf)], stats: &LoadStats) {
    println!("\n=== Summary ===");
    println!("Loaded: {}/{} addons", stats.success_count, addons.len());
    println!("Failed: {}", stats.fail_count);
    println!(
        "Total: {} Lua files, {} XML files, {} warnings",
        stats.total_lua, stats.total_xml, stats.total_warnings
    );
    print_timing_breakdown(&stats.total_timing);
    print_cache_stats(&stats.total_timing);
    print_slowest_addons(&stats.addon_timings);
    print_slowest_lua_files();
}

fn print_lua_timing_detail(t: &LoadTiming, pct: &dyn Fn(std::time::Duration) -> f64) {
    println!(
        "  Lua exec:   {:.2?} ({:.1}%)",
        t.lua_exec_time,
        pct(t.lua_exec_time)
    );
    println!(
        "    compile:  {:.2?} ({:.1}%, subset of Lua exec)",
        t.lua_compile_time,
        pct(t.lua_compile_time)
    );
    println!(
        "    call:     {:.2?} ({:.1}%, subset of Lua exec)",
        t.lua_call_time,
        pct(t.lua_call_time)
    );
    println!(
        "  SavedVars:  {:.2?} ({:.1}%)",
        t.saved_vars_time,
        pct(t.saved_vars_time)
    );
}

fn print_timing_breakdown(t: &LoadTiming) {
    let total_time = t.total();
    if total_time.is_zero() {
        return;
    }
    let pct = |d: std::time::Duration| 100.0 * d.as_secs_f64() / total_time.as_secs_f64();
    println!("Total time: {:.2?}", total_time);
    println!("  IO:         {:.2?} ({:.1}%)", t.io_time, pct(t.io_time));
    println!(
        "  XML parse:  {:.2?} ({:.1}%)",
        t.xml_parse_time,
        pct(t.xml_parse_time)
    );
    println!(
        "  XML proc:   {:.2?} ({:.1}%)",
        t.xml_process_time,
        pct(t.xml_process_time)
    );
    print_frame_timing_detail(t, &pct);
    print_lua_timing_detail(t, &pct);
}

fn print_frame_timing_detail(t: &LoadTiming, pct: &dyn Fn(std::time::Duration) -> f64) {
    println!(
        "  XML frames: {:.2?} ({:.1}%, subset of XML proc)",
        t.xml_frame_create_time,
        pct(t.xml_frame_create_time)
    );
    println!(
        "    setup:  {:.2?}  (code_build={:.2?} exec_lua={:.2?} props={:.2?})",
        t.xml_frame_setup_time,
        t.frame_code_build_time,
        t.frame_exec_lua_time,
        t.frame_apply_props_time
    );
    println!(
        "    finalize: {:.2?}  (layers={:.2?} anim={:.2?} button={:.2?} lifecycle={:.2?})",
        t.xml_frame_finalize_time,
        t.frame_layer_children_time,
        t.frame_anim_time,
        t.frame_button_time,
        t.frame_lifecycle_time
    );
    println!(
        "    {} frames, {} textures, {} fontstrings, {} lifecycle fires",
        t.frame_count, t.texture_count, t.fontstring_count, t.lifecycle_fire_count
    );
}

#[cfg(test)]
mod tests;
