//! Addon loader - loads addons from TOC files.

mod addon;
mod addon_order;
mod addon_saved_variables;
pub(crate) mod button;
pub(crate) mod bytecode;
pub(crate) mod bytecode_cache;
pub(crate) mod chunk_cache;
mod error;
pub(crate) mod helpers;
pub(crate) mod helpers_anim;
mod load_addon_trace;
pub(crate) mod lua_file;
pub(crate) mod precompiled;
pub(crate) mod stack_taint;
mod xml_file;
mod xml_fontstring;
mod xml_frame;
mod xml_frame_codegen;
pub(crate) mod xml_frame_extras;
pub(crate) mod xml_layer_batch;
mod xml_lifecycle;
mod xml_texture;

use crate::lua_api::LoaderEnv;
pub use crate::lua_api::state::{
    LoadDiagnosticAttribution, LoadDiagnostics, MissingRequirement, MissingRequirementKind,
    NilSymbolEnvironment, NilSymbolObservation, NilSymbolObservationKind,
};
use crate::saved_variables::SavedVariablesManager;
use crate::screen::ScreenKind;
use crate::toc::TocFile;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

pub(crate) use addon::{append_pending_nested_addon_diagnostics, begin_addon_load};
pub use addon_order::sort_addons_by_dependencies;
use addon_order::{topological_sort_addons, topological_sort_addons_with_extra_dependencies};
pub use error::LoadError;
pub(crate) use load_addon_trace::{
    LoadAddonTraceOrigin, enter_xml_load_addon_context, runtime_load_addon_origin, trace_load_addon,
};
pub use xml_frame::create_frame_from_xml;
pub use xml_frame::{fast_create_frame_profile_body_report, fast_create_frame_profile_report};

#[doc(hidden)]
pub fn enter_bytecode_cache_parent_bypass_mode() {
    bytecode_cache::enter_parent_bypass_mode();
}

#[doc(hidden)]
pub fn enter_bytecode_cache_read_only_mode() {
    bytecode_cache::enter_read_only_mode();
}

#[doc(hidden)]
pub fn release_prefork_parent_bytecode_cache_memory() -> Result<usize, String> {
    bytecode_cache::release_prefork_parent_memory()
}

const ENV_PROFILE_LUA_FILES: &str = "WOW_SIM_PROFILE_LUA_FILES";

#[derive(Debug, Clone)]
pub struct LuaFileTiming {
    pub chunk_name: String,
    pub compile_time: Duration,
    pub call_time: Duration,
    pub cache_hits: u32,
    pub cache_misses: u32,
    pub cache_lookup_misses: u32,
    pub cache_replay_failures: u32,
}

impl LuaFileTiming {
    pub fn total_time(&self) -> Duration {
        self.compile_time + self.call_time
    }
}

fn lua_file_timing_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var_os(ENV_PROFILE_LUA_FILES).is_some_and(|value| {
            let value = value.to_string_lossy();
            value != "0" && !value.eq_ignore_ascii_case("false")
        })
    })
}

fn lua_file_timings() -> &'static Mutex<Vec<LuaFileTiming>> {
    static TIMINGS: OnceLock<Mutex<Vec<LuaFileTiming>>> = OnceLock::new();
    TIMINGS.get_or_init(|| Mutex::new(Vec::new()))
}

pub(crate) fn record_lua_file_timing(timing: LuaFileTiming) {
    if !lua_file_timing_enabled() {
        return;
    }
    if let Ok(mut timings) = lua_file_timings().lock() {
        timings.push(timing);
    }
}

pub fn lua_file_timings_snapshot() -> Vec<LuaFileTiming> {
    if !lua_file_timing_enabled() {
        return Vec::new();
    }
    lua_file_timings()
        .lock()
        .map(|timings| timings.clone())
        .unwrap_or_default()
}

/// Addons that ship the shared XML template library which Blizzard's other
/// addons inherit from (`ButtonFrameTemplate`, `PortraitFrameBaseTemplate`,
/// `UIPanelButtonTemplate`, …). In real WoW these are part of the FrameXML
/// core and load before any addon, so addons reference their templates
/// without declaring an explicit `## Dependencies:` line. When the full
/// Blizzard set is force-loaded for testing (`discover_all_blizzard_addons`),
/// we promote these to `LoadFirst` so dependency-free LoadOnDemand addons
/// that alphabetically precede them (e.g. `Blizzard_AlliedRacesUI`) do not
/// instantiate frames against an empty template registry.
const FOUNDATIONAL_LOAD_FIRST_ADDONS: &[&str] = &[
    "Blizzard_SharedXMLBase",
    "Blizzard_SharedXML",
    "Blizzard_SharedXMLGame",
    "Blizzard_MoneyFrame",
];

fn promote_foundational_addons_to_load_first(addons: &mut HashMap<String, (PathBuf, TocFile)>) {
    for name in FOUNDATIONAL_LOAD_FIRST_ADDONS {
        if let Some((_, toc)) = addons.get_mut(*name) {
            toc.metadata
                .insert("LoadFirst".to_string(), "1".to_string());
        }
    }
}

fn implicit_blizzard_startup_dependencies() -> HashMap<String, Vec<String>> {
    HashMap::from([
        (
            "Blizzard_UIPanels_Game".to_string(),
            vec!["Blizzard_MoneyFrame".to_string()],
        ),
        (
            "Blizzard_DeprecatedCombatLog".to_string(),
            vec!["Blizzard_CombatLogBase".to_string()],
        ),
        (
            "Blizzard_ObjectiveTracker".to_string(),
            vec!["Blizzard_POIButton".to_string()],
        ),
        (
            "Blizzard_Game".to_string(),
            vec![
                "Blizzard_TimeManager".to_string(),
                "Blizzard_CooldownBroadcaster".to_string(),
                "Blizzard_BoostTutorial".to_string(),
                "Blizzard_CombatLog".to_string(),
            ],
        ),
        (
            "Blizzard_RaidUI".to_string(),
            vec!["Blizzard_RaidFrame".to_string()],
        ),
        ("Blizzard_MacroUI".to_string(), vec![]),
        ("Blizzard_TrainerUI".to_string(), vec![]),
        ("Blizzard_AchievementUI".to_string(), vec![]),
        ("Blizzard_Transmog".to_string(), vec![]),
        ("Blizzard_ArdenwealdGardening".to_string(), vec![]),
        // Blizzard_SharedXML.toc declares Blizzard_Narration; the legacy
        // Blizzard_SharedXML_Mainline.toc that the loader prefers does not, so
        // NarrationSliderMixin is still nil when Blizzard_EditMode builds its
        // slider and MinimalSlider.lua:237 raises on SetNarrationValueFormatter.
        // Restoring the edge is narrower than switching that addon to its bare
        // TOC, which was measured to abort the whole Tutorial subsystem.
        (
            "Blizzard_SharedXML".to_string(),
            vec!["Blizzard_Narration".to_string()],
        ),
        // The 12.1.0 Blizzard_UIParentPanelManager TOC declares
        // Blizzard_ManagedFrameSystem and Blizzard_GameMenuEsc only; the
        // `## LoadWith: Blizzard_UIParent` of earlier builds is gone. Its Lua
        // still uses UIParent at file scope (UIPanelLayoutFrame.lua:11 parents
        // a frame to it, UpdateUIPanelPositions.lua:1 sets its
        // OnAttributeChanged script), and the eager-discovery order pinned by
        // tests/blizzard_core_frame_lane.rs keeps UIParent first, so the edge
        // is explicit rather than left to the sort's alphabetical tie-break:
        // the Narration edge above moves UIParent later and would otherwise
        // flip the pair.
        (
            "Blizzard_UIParentPanelManager".to_string(),
            vec!["Blizzard_UIParent".to_string()],
        ),
    ])
}

/// TOC suffix matching the active client profile (e.g. `_Mainline` for retail/PTR,
/// `_Vanilla` for era/anniversary).
fn active_profile_toc_suffix() -> &'static str {
    match crate::client_profile::ACTIVE {
        crate::client_profile::ClientProfile::Retail
        | crate::client_profile::ClientProfile::Ptr => "_Mainline",
        crate::client_profile::ClientProfile::Wrath => "_Wrath",
        crate::client_profile::ClientProfile::Mists => "_Mists",
        crate::client_profile::ClientProfile::Era
        | crate::client_profile::ClientProfile::Anniversary => "_Vanilla",
    }
}

/// TOC suffixes that target other client flavors and should be skipped when
/// scanning for an addon's compatible flavor toc.
fn other_profile_toc_suffixes() -> &'static [&'static str] {
    match crate::client_profile::ACTIVE {
        crate::client_profile::ClientProfile::Retail
        | crate::client_profile::ClientProfile::Ptr => {
            &["_Cata", "_Wrath", "_TBC", "_Vanilla", "_Mists"]
        }
        crate::client_profile::ClientProfile::Wrath => {
            &["_Cata", "_Mainline", "_TBC", "_Vanilla", "_Mists"]
        }
        crate::client_profile::ClientProfile::Mists => {
            &["_Cata", "_Wrath", "_TBC", "_Vanilla", "_Mainline"]
        }
        crate::client_profile::ClientProfile::Era
        | crate::client_profile::ClientProfile::Anniversary => {
            &["_Cata", "_Wrath", "_TBC", "_Mists", "_Mainline"]
        }
    }
}

/// Scan `addon_dir` for a .toc file matching the folder name that doesn't
/// carry an excluded-flavor suffix (used as a last-resort fallback in
/// `find_toc_file` for case mismatches and unusual flavor suffixes).
///
/// WoW only loads TOCs whose stem is the folder name plus an optional
/// `_`/`-` flavor suffix. Renaming a folder (e.g. `MyAddon.disabled`) must
/// disable the addon -- its `MyAddon.toc` no longer matches the folder name.
fn scan_for_compatible_flavor_toc(addon_dir: &Path) -> Option<PathBuf> {
    let exclude_suffixes = other_profile_toc_suffixes();
    let folder_name = addon_dir.file_name()?.to_str()?.to_ascii_lowercase();
    let entries = std::fs::read_dir(addon_dir).ok()?;
    entries.flatten().find_map(|entry| {
        let path = entry.path();
        if path.extension().map(|e| e == "toc").unwrap_or(false) {
            let name = path.file_name()?.to_str()?;
            let stem = path.file_stem()?.to_str()?.to_ascii_lowercase();
            if !toc_stem_matches_folder(&stem, &folder_name) {
                return None;
            }
            if !exclude_suffixes.iter().any(|s| name.contains(s)) {
                return Some(path);
            }
        }
        None
    })
}

/// Whether a lowercased TOC stem names the lowercased addon folder: an exact
/// match or the folder name followed by a `_`/`-` flavor suffix.
fn toc_stem_matches_folder(stem: &str, folder_name: &str) -> bool {
    let Some(rest) = stem.strip_prefix(folder_name) else {
        return false;
    };
    rest.is_empty() || rest.starts_with('_') || rest.starts_with('-')
}

/// Addons whose bare `<addon>.toc` supersedes the flavor-suffixed variant.
///
/// The 12.x TOC format annotates entries inline with `[AllowLoadGameType ...]`
/// so one file can serve every flavor, which makes the per-flavor TOCs beside
/// it legacy. `Blizzard_FrameXML.toc` carries 88 such annotations and lists 105
/// files against `Blizzard_FrameXML_Mainline.toc`'s 86; taking the flavor
/// variant drops the Lua for PVPHonorSystem, PVPUITemplates, EquipmentFlyout,
/// GhostFrame and SharedPetBattleTemplates, whose XML siblings carry no
/// `<Script file=...>` include to pull them back in. Six startup errors trace to
/// that. The five entries the flavor TOC has and the bare one lacks
/// (ColorPickerFrame, RaidWarning, SplashFrame, UIErrorsFrame) moved into
/// addons of their own, so nothing is lost by preferring the bare file.
///
/// `Blizzard_SettingsDefinitions_Frame` is the same shape: its bare TOC lists
/// `CombatAudioAlertConstants.lua` (which `CombatAudioAlertManager:OnLoad`
/// reads) and uses `[Family]\` paths, while `_Mainline.toc` predates the split
/// that moved `AudioAssist` and `Subtitles` into
/// `Blizzard_SettingsDefinitions_Shared` and still lists them, so with the
/// flavor TOC `Subtitles.lua` runs twice and registers the `movieSubtitle`
/// setting twice.
///
/// Deliberately not generalised: of the sixteen addons shipping both variants,
/// most have a *fuller* flavor TOC — `Blizzard_UIParent.toc` lists one file
/// against its variant's six — so a blanket flip would lose far more than it
/// fixes.
fn prefers_bare_toc(addon_name: &str) -> bool {
    matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    ) && matches!(
        addon_name,
        "Blizzard_FrameXML" | "Blizzard_SettingsDefinitions_Frame"
    )
}

fn profile_specific_fallback_toc(addon_name: &str) -> Option<String> {
    match crate::client_profile::ACTIVE {
        crate::client_profile::ClientProfile::Mists if addon_name == "Blizzard_GameMenu" => {
            Some(format!("{addon_name}_Mainline.toc"))
        }
        crate::client_profile::ClientProfile::Mists
            if addon_name == "Blizzard_UIParentPanelManager" =>
        {
            Some(format!("{addon_name}_Classic.toc"))
        }
        _ => None,
    }
}

/// Find the TOC file for an addon directory.
///
/// Picks the variant matching the active client profile (e.g. `_Mists.toc`
/// under `client-mists`, `_Wrath.toc` under `client-wrath`, `_Mainline.toc`
/// under retail). Falls back to the bare `<addon>.toc`, then any compatible
/// flavor toc.
pub fn find_toc_file(addon_dir: &Path) -> Option<PathBuf> {
    let addon_name = addon_dir.file_name()?.to_str()?;
    if let Some(fallback) = profile_specific_fallback_toc(addon_name) {
        let toc_path = addon_dir.join(fallback);
        if toc_path.exists() {
            return Some(toc_path);
        }
    }
    let toc_variants = if prefers_bare_toc(addon_name) {
        [
            format!("{}.toc", addon_name),
            format!("{}{}.toc", addon_name, active_profile_toc_suffix()),
        ]
    } else {
        [
            format!("{}{}.toc", addon_name, active_profile_toc_suffix()),
            format!("{}.toc", addon_name),
        ]
    };
    for variant in &toc_variants {
        let toc_path = addon_dir.join(variant);
        if toc_path.exists() {
            return Some(toc_path);
        }
    }
    scan_for_compatible_flavor_toc(addon_dir)
}

/// Resolve an XML script/include path with the same addon-root fallback and
/// case-insensitive semantics used by runtime XML loading.
pub fn resolve_xml_include_path(xml_path: &Path, addon_root: &Path, file: &str) -> PathBuf {
    let xml_dir = xml_path.parent().unwrap_or(addon_root);
    helpers::resolve_path_with_fallback(xml_dir, addon_root, file)
}

/// Result of loading an addon.
#[derive(Debug)]
pub struct LoadResult {
    /// Addon name
    pub name: String,
    /// Number of Lua files loaded
    pub lua_files: usize,
    /// Number of XML files loaded
    pub xml_files: usize,
    /// Time breakdown
    pub timing: LoadTiming,
    /// Actual loader, XML, Lua, or runtime failures encountered during loading.
    pub warnings: Vec<String>,
    /// Regular-global nil accesses retained as non-fatal diagnostics.
    pub nil_symbol_observations: Vec<NilSymbolObservation>,
    /// Missing `C_*` namespaces and members retained as strict API requirements.
    pub missing_requirements: Vec<MissingRequirement>,
}

impl LoadResult {
    pub fn diagnostics(&self) -> LoadDiagnostics {
        LoadDiagnostics {
            warnings: self.warnings.clone(),
            nil_symbol_observations: self.nil_symbol_observations.clone(),
            missing_requirements: self.missing_requirements.clone(),
        }
    }

    pub(crate) fn extend_diagnostics(&mut self, diagnostics: LoadDiagnostics) {
        self.warnings.extend(diagnostics.warnings);
        self.nil_symbol_observations
            .extend(diagnostics.nil_symbol_observations);
        self.missing_requirements
            .extend(diagnostics.missing_requirements);
    }
}

/// Timing breakdown for addon loading.
#[derive(Debug, Default, Clone)]
pub struct LoadTiming {
    /// Time reading files from disk
    pub io_time: Duration,
    /// Time parsing XML
    pub xml_parse_time: Duration,
    /// Time processing parsed XML elements (excludes raw parse time)
    pub xml_process_time: Duration,
    /// Time creating/configuring frames from XML
    pub xml_frame_create_time: Duration,
    /// Time in the initial frame creation/setup phase (subset of xml_frame_create_time)
    pub xml_frame_setup_time: Duration,
    /// Time in child creation/finalization (subset of xml_frame_create_time)
    pub xml_frame_finalize_time: Duration,
    /// Time executing CreateFrame Lua code (subset of setup)
    pub frame_exec_lua_time: Duration,
    /// Time applying XML properties in Rust (subset of setup)
    pub frame_apply_props_time: Duration,
    /// Time creating layer children (textures/fontstrings, subset of finalize)
    pub frame_layer_children_time: Duration,
    /// Time firing OnLoad/OnShow lifecycle scripts (subset of finalize)
    pub frame_lifecycle_time: Duration,
    /// Number of frames created
    pub frame_count: u32,
    /// Number of OnLoad/OnShow fires
    pub lifecycle_fire_count: u32,
    /// Number of textures created
    pub texture_count: u32,
    /// Number of fontstrings created
    pub fontstring_count: u32,
    /// Time building Lua code strings (template chain, mixins, etc., subset of setup)
    pub frame_code_build_time: Duration,
    /// Time in animation groups (subset of finalize)
    pub frame_anim_time: Duration,
    /// Time in button textures+text (subset of finalize)
    pub frame_button_time: Duration,
    /// Time compiling Lua chunks into functions (source compile or bytecode load)
    pub lua_compile_time: Duration,
    /// Time preparing and calling compiled Lua functions
    pub lua_call_time: Duration,
    /// Time executing Lua (compile + call)
    pub lua_exec_time: Duration,
    /// Time loading SavedVariables
    pub saved_vars_time: Duration,
    /// Number of Lua files loaded from bytecode cache
    pub cache_hits: u32,
    /// Number of Lua files compiled from source (cache miss)
    pub cache_misses: u32,
    /// Number of Lua bytecode cache misses with no matching entry.
    pub cache_lookup_misses: u32,
    /// Number of Lua bytecode cache entries that failed to reload.
    pub cache_replay_failures: u32,
    /// Number of Lua bytecode cache entries written to disk.
    pub cache_store_successes: u32,
    /// Number of Lua bytecode cache entries that failed to write.
    pub cache_store_failures: u32,
}

impl LoadTiming {
    pub fn total(&self) -> Duration {
        self.independently_counted_time()
    }

    pub(crate) fn independently_counted_time(&self) -> Duration {
        self.io_time
            + self.xml_parse_time
            + self.xml_process_time
            + self.lua_exec_time
            + self.saved_vars_time
    }

    /// Add another timing's fields into this one.
    pub fn accumulate(&mut self, other: &LoadTiming) {
        self.io_time += other.io_time;
        self.xml_parse_time += other.xml_parse_time;
        self.xml_process_time += other.xml_process_time;
        self.xml_frame_create_time += other.xml_frame_create_time;
        self.xml_frame_setup_time += other.xml_frame_setup_time;
        self.xml_frame_finalize_time += other.xml_frame_finalize_time;
        self.frame_exec_lua_time += other.frame_exec_lua_time;
        self.frame_apply_props_time += other.frame_apply_props_time;
        self.frame_layer_children_time += other.frame_layer_children_time;
        self.frame_lifecycle_time += other.frame_lifecycle_time;
        self.frame_count += other.frame_count;
        self.lifecycle_fire_count += other.lifecycle_fire_count;
        self.texture_count += other.texture_count;
        self.fontstring_count += other.fontstring_count;
        self.frame_code_build_time += other.frame_code_build_time;
        self.frame_anim_time += other.frame_anim_time;
        self.frame_button_time += other.frame_button_time;
        self.lua_compile_time += other.lua_compile_time;
        self.lua_call_time += other.lua_call_time;
        self.lua_exec_time += other.lua_exec_time;
        self.saved_vars_time += other.saved_vars_time;
        self.cache_hits += other.cache_hits;
        self.cache_misses += other.cache_misses;
        self.cache_lookup_misses += other.cache_lookup_misses;
        self.cache_replay_failures += other.cache_replay_failures;
        self.cache_store_successes += other.cache_store_successes;
        self.cache_store_failures += other.cache_store_failures;
    }
}

/// Load an addon from its TOC file.
pub fn load_addon(env: &LoaderEnv<'_>, toc_path: &Path) -> Result<LoadResult, LoadError> {
    load_addon_path(env, toc_path, None)
}

/// Load an addon from its TOC file with saved variables support.
pub fn load_addon_with_saved_vars(
    env: &LoaderEnv<'_>,
    toc_path: &Path,
    saved_vars_mgr: &mut SavedVariablesManager,
) -> Result<LoadResult, LoadError> {
    load_addon_path(env, toc_path, Some(saved_vars_mgr))
}

fn load_addon_path(
    env: &LoaderEnv<'_>,
    toc_path: &Path,
    saved_vars_mgr: Option<&mut SavedVariablesManager>,
) -> Result<LoadResult, LoadError> {
    let addon_name = toc_path
        .parent()
        .and_then(|dir| dir.file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("Unknown");
    trace_load_addon(LoadAddonTraceOrigin::Toc, format!("begin {addon_name}"));
    trace_load_addon(
        LoadAddonTraceOrigin::Toc,
        format!("toc {}", toc_path.display()),
    );
    let toc = TocFile::from_file(toc_path)?;
    #[cfg(feature = "client-mists")]
    if let Ok(Some(result)) =
        crate::mists::character_frame_preload::ensure_before_addon(env, &toc, toc_path)
    {
        trace_load_result_diagnostics(LoadAddonTraceOrigin::Toc, &result.name, &result);
    }
    trace_load_addon(LoadAddonTraceOrigin::Toc, format!("files {addon_name}"));
    let result = addon::load_addon_internal(env, &toc, saved_vars_mgr)?;
    trace_load_result_diagnostics(LoadAddonTraceOrigin::Toc, addon_name, &result);
    trace_load_addon(LoadAddonTraceOrigin::Toc, format!("loaded {addon_name}"));
    Ok(result)
}

pub(crate) fn trace_load_result_diagnostics(
    origin: LoadAddonTraceOrigin,
    addon_name: &str,
    result: &LoadResult,
) {
    for warning in &result.warnings {
        trace_load_addon(origin, format!("warning {addon_name}: {warning}"));
    }
    for observation in &result.nil_symbol_observations {
        trace_load_addon(
            origin,
            format!("nil observation {addon_name}: {observation}"),
        );
    }
    for requirement in &result.missing_requirements {
        trace_load_addon(
            origin,
            format!("missing requirement {addon_name}: {requirement}"),
        );
    }
}

/// Load an addon from a parsed TOC.
pub fn load_addon_from_toc(env: &LoaderEnv<'_>, toc: &TocFile) -> Result<LoadResult, LoadError> {
    addon::load_addon_internal(env, toc, None)
}

/// Load an addon from a parsed TOC with saved variables support.
pub fn load_addon_from_toc_with_saved_vars(
    env: &LoaderEnv<'_>,
    toc: &TocFile,
    saved_vars_mgr: &mut SavedVariablesManager,
) -> Result<LoadResult, LoadError> {
    addon::load_addon_internal(env, toc, Some(saved_vars_mgr))
}

/// Discover all Blizzard addons in a BlizzardUI directory, topologically sorted by dependencies.
///
/// Scans for `Blizzard_*` subdirectories, parses their TOC files, filters out `LoadOnDemand`
/// addons, and returns the remaining addons in dependency order.
pub fn discover_blizzard_addons(blizzard_ui_dir: &Path) -> Vec<(String, PathBuf)> {
    discover_blizzard_addons_for_screen(blizzard_ui_dir, ScreenKind::Game)
}

/// Discover every Blizzard addon directory in a BlizzardUI tree, including LoadOnDemand addons.
///
/// This includes every parseable `Blizzard_*` directory regardless of screen restrictions
/// or `LoadOnDemand`, plus the transitive `## Dependencies:` closure of those roots. This
/// admits hard non-Blizzard dependencies (for example `middleclass`) without eagerly loading
/// unrelated non-Blizzard directories. Foundational shared XML addons
/// (`Blizzard_SharedXMLBase`, `Blizzard_SharedXML`, `Blizzard_SharedXMLGame`) are emitted first
/// via the eager LoadFirst pass so dependency-free LoadOnDemand frames that inherit their
/// templates resolve against a fully-registered template chain.
pub fn discover_all_blizzard_addons(blizzard_ui_dir: &Path) -> Vec<(String, PathBuf)> {
    let entries = match std::fs::read_dir(blizzard_ui_dir) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    let mut toc_map: HashMap<String, (PathBuf, TocFile)> = HashMap::new();
    let mut unparsed: Vec<(String, PathBuf)> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(toc_path) = find_toc_file(&path) else {
            continue;
        };
        match TocFile::from_file(&toc_path) {
            Ok(toc) => {
                toc_map.insert(name.to_string(), (toc_path, toc));
            }
            Err(_) if name.starts_with("Blizzard_") => unparsed.push((name.to_string(), toc_path)),
            Err(_) => {}
        }
    }

    retain_blizzard_roots_and_required_dependencies(&mut toc_map);
    promote_foundational_addons_to_load_first(&mut toc_map);
    let mut sorted = topological_sort_addons(toc_map);
    sorted.extend(unparsed);
    sorted
}

/// Discover Blizzard addons for a specific screen mode, topologically sorted by dependencies.
pub fn discover_blizzard_addons_for_screen(
    blizzard_ui_dir: &Path,
    screen: ScreenKind,
) -> Vec<(String, PathBuf)> {
    let Some((mut addons, mut lod_pool)) =
        discover_blizzard_addon_toc_pools_for_screen(blizzard_ui_dir, screen)
    else {
        return Vec::new();
    };

    let extra_dependencies = implicit_blizzard_startup_dependencies();
    // Pull LOD addons that are required by non-LOD addons or implicit startup deps.
    pull_required_dependency_addons(&mut addons, &mut lod_pool, &extra_dependencies);

    promote_foundational_addons_to_load_first(&mut addons);
    topological_sort_addons_with_extra_dependencies(addons, &extra_dependencies)
}

/// Discover the explicit dependency closure for one or more Blizzard addons.
///
/// The returned list is ordered the same way as `discover_blizzard_addons_for_screen`,
/// but filtered to the requested roots and everything they require via
/// `## Dependencies:` and `## OptionalDeps:`.
/// Roots or dependencies not present in the screen-allowed Blizzard TOC set are ignored.
pub fn discover_blizzard_addon_closure_for_screen(
    blizzard_ui_dir: &Path,
    screen: ScreenKind,
    roots: &[&str],
) -> Vec<(String, PathBuf)> {
    let Some((addons, lod_pool)) =
        discover_blizzard_addon_toc_pools_for_screen(blizzard_ui_dir, screen)
    else {
        return Vec::new();
    };
    let toc_map: HashMap<String, (PathBuf, TocFile)> = addons.into_iter().chain(lod_pool).collect();
    let wanted = collect_declared_dependency_closure(&toc_map, roots);
    let filtered: HashMap<String, (PathBuf, TocFile)> = toc_map
        .into_iter()
        .filter(|(name, _)| wanted.contains(name))
        .collect();
    topological_sort_addons(filtered)
}

/// Per-addon override entry for Blizzard addon closure discovery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlizzardAddonOverride<'a> {
    pub addon: &'a str,
    pub extra_roots: &'a [&'a str],
}

/// Discover the explicit dependency closure for one or more Blizzard addons.
///
/// `overrides` adds extra non-TOC roots before the dependency walk runs.
/// Use it for shared templates, startup-order assumptions, and implicit
/// addons that Blizzard code reaches outside TOC metadata.
pub fn discover_blizzard_addon_closure_for_screen_with_overrides(
    blizzard_ui_dir: &Path,
    screen: ScreenKind,
    roots: &[&str],
    overrides: &[BlizzardAddonOverride<'_>],
) -> Vec<(String, PathBuf)> {
    let Some((addons, lod_pool)) =
        discover_blizzard_addon_toc_pools_for_screen(blizzard_ui_dir, screen)
    else {
        return Vec::new();
    };
    let toc_map: HashMap<String, (PathBuf, TocFile)> = addons.into_iter().chain(lod_pool).collect();
    let extra_roots_by_addon = build_extra_roots_map(overrides);
    let wanted =
        collect_declared_dependency_closure_with_overrides(&toc_map, roots, &extra_roots_by_addon);
    let extra_dependencies = build_extra_dependency_map(overrides);
    let filtered: HashMap<String, (PathBuf, TocFile)> = toc_map
        .into_iter()
        .filter(|(name, _)| wanted.contains(name))
        .collect();
    topological_sort_addons_with_extra_dependencies(filtered, &extra_dependencies)
}

pub(crate) fn collect_declared_dependency_closure(
    toc_map: &HashMap<String, (PathBuf, TocFile)>,
    roots: &[&str],
) -> HashSet<String> {
    let extra_roots_by_addon: HashMap<&str, Vec<&str>> = HashMap::new();
    collect_declared_dependency_closure_with_overrides(toc_map, roots, &extra_roots_by_addon)
}

fn collect_declared_dependency_closure_with_overrides(
    toc_map: &HashMap<String, (PathBuf, TocFile)>,
    roots: &[&str],
    extra_roots_by_addon: &HashMap<&str, Vec<&str>>,
) -> HashSet<String> {
    let mut wanted = HashSet::new();
    let mut pending: Vec<String> = roots.iter().map(|name| (*name).to_string()).collect();
    let mut queued: HashSet<String> = pending.iter().cloned().collect();

    while let Some(name) = pending.pop() {
        if !wanted.insert(name.clone()) {
            continue;
        }

        if let Some(extra_roots) = extra_roots_by_addon.get(name.as_str()) {
            for extra_root in extra_roots {
                queue_pending_addon((*extra_root).to_string(), &mut pending, &mut queued);
            }
        }

        let Some((_, toc)) = toc_map.get(&name) else {
            continue;
        };

        for dep in toc.dependencies().into_iter().chain(toc.optional_deps()) {
            if toc_map.contains_key(&dep) {
                queue_pending_addon(dep, &mut pending, &mut queued);
            }
        }
    }

    wanted
}

fn queue_pending_addon(name: String, pending: &mut Vec<String>, queued: &mut HashSet<String>) {
    if queued.insert(name.clone()) {
        pending.push(name);
    }
}

fn retain_blizzard_roots_and_required_dependencies(
    toc_map: &mut HashMap<String, (PathBuf, TocFile)>,
) {
    let mut pending: Vec<String> = toc_map
        .keys()
        .filter(|name| name.starts_with("Blizzard_"))
        .cloned()
        .collect();
    let mut retained: HashSet<String> = pending.iter().cloned().collect();

    while let Some(name) = pending.pop() {
        let Some((_, toc)) = toc_map.get(&name) else {
            continue;
        };
        for dependency in toc.dependencies() {
            if toc_map.contains_key(&dependency) && retained.insert(dependency.clone()) {
                pending.push(dependency);
            }
        }
    }

    toc_map.retain(|name, _| retained.contains(name));
}

fn build_extra_roots_map<'a>(
    overrides: &'a [BlizzardAddonOverride<'a>],
) -> HashMap<&'a str, Vec<&'a str>> {
    let mut map: HashMap<&'a str, Vec<&'a str>> = HashMap::new();
    for override_entry in overrides {
        map.entry(override_entry.addon)
            .or_default()
            .extend(override_entry.extra_roots.iter().copied());
    }
    map
}

fn build_extra_dependency_map(
    overrides: &[BlizzardAddonOverride<'_>],
) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for override_entry in overrides {
        map.entry(override_entry.addon.to_string())
            .or_default()
            .extend(
                override_entry
                    .extra_roots
                    .iter()
                    .map(|extra_root| (*extra_root).to_string()),
            );
    }
    map
}

fn discover_blizzard_addon_toc_pools_for_screen(
    blizzard_ui_dir: &Path,
    screen: ScreenKind,
) -> Option<(
    HashMap<String, (PathBuf, TocFile)>,
    HashMap<String, (PathBuf, TocFile)>,
)> {
    let entries = std::fs::read_dir(blizzard_ui_dir).ok()?;
    let mut addons: HashMap<String, (PathBuf, TocFile)> = HashMap::new();
    let mut dependency_pool: HashMap<String, (PathBuf, TocFile)> = HashMap::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let dir_name = path.file_name().unwrap().to_str().unwrap().to_string();
        let Some(toc_path) = find_toc_file(&path) else {
            continue;
        };
        let Ok(toc) = TocFile::from_file(&toc_path) else {
            continue;
        };
        if !toc.allows_screen(screen) || toc.is_ptr_only() || toc.is_game_type_restricted() {
            continue;
        }

        let is_blizzard_root = dir_name.starts_with("Blizzard_")
            && !excluded_addons_for_screen(screen).contains(&dir_name.as_str())
            && !excluded_addons_for_active_profile().contains(&dir_name.as_str());
        if is_blizzard_root && !toc.is_load_on_demand() {
            addons.insert(dir_name, (toc_path, toc));
        } else {
            dependency_pool.insert(dir_name, (toc_path, toc));
        }
    }

    Some((addons, dependency_pool))
}

fn excluded_addons_for_screen(screen: ScreenKind) -> &'static [&'static str] {
    match screen {
        ScreenKind::Game | ScreenKind::CharacterSelect | ScreenKind::CharacterCreate => &[],
        ScreenKind::Login => &[
            "Blizzard_CharacterCreate",
            "Blizzard_CharacterCustomize",
            "Blizzard_TimerunningCharacterCreate",
        ],
    }
}

/// Whether runtime loading excludes this addon for the active client profile.
pub fn is_addon_excluded_for_active_profile(addon_name: &str) -> bool {
    excluded_addons_for_active_profile().contains(&addon_name)
}

/// Whether runtime loading skips this TOC entry for the active client profile.
pub fn is_addon_toc_file_excluded_for_active_profile(toc: &TocFile, file: &Path) -> bool {
    addon::should_skip_addon_file(toc, file)
}

/// Whether runtime loading visits this TOC entry in any environment pass.
pub fn is_addon_toc_file_loaded_for_active_profile(
    folder_name: &str,
    toc: &TocFile,
    index: usize,
) -> bool {
    let loads_normally = toc.file_allows_environment(index, toc.is_secure_env());
    let replays_securely = addon::is_secure_replay_library_addon(folder_name)
        && !toc.is_secure_env()
        && toc.file_use_secure_env(index).is_none()
        && toc.file_allows_environment(index, true);
    loads_normally || replays_securely
}

fn excluded_addons_for_active_profile() -> &'static [&'static str] {
    match crate::client_profile::ACTIVE {
        crate::client_profile::ClientProfile::Mists => &["Blizzard_Deprecated_ArenaUI"],
        _ => &[],
    }
}

/// Recursively pull hard dependencies into the main set when required by Blizzard roots.
///
/// The candidate pool contains LoadOnDemand Blizzard addons and every eligible non-Blizzard
/// TOC, so unrelated non-Blizzard directories remain excluded unless a retained root requires
/// them through `## Dependencies:`.
fn pull_required_dependency_addons(
    addons: &mut HashMap<String, (PathBuf, TocFile)>,
    lod_pool: &mut HashMap<String, (PathBuf, TocFile)>,
    extra_dependencies: &HashMap<String, Vec<String>>,
) {
    let mut needed: Vec<String> = addons
        .values()
        .flat_map(|(_, toc)| toc.dependencies())
        .chain(
            addons
                .keys()
                .flat_map(|name| extra_dependencies.get(name).into_iter().flatten().cloned()),
        )
        .chain(
            extra_dependencies
                .keys()
                .filter(|name| lod_pool.contains_key(*name))
                .cloned(),
        )
        .filter(|dep| lod_pool.contains_key(dep))
        .collect();

    while let Some(name) = needed.pop() {
        if addons.contains_key(&name) {
            continue;
        }
        if let Some((toc_path, toc)) = lod_pool.remove(&name) {
            // This LOD addon may itself depend on other LOD addons
            for dep in toc.dependencies() {
                if lod_pool.contains_key(&dep) {
                    needed.push(dep);
                }
            }
            addons.insert(name, (toc_path, toc));
        }
    }
}

#[cfg(test)]
mod tests;
