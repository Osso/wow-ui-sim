//! Addon loading internals.

mod nil_symbol_reports;

use crate::lua_api::LoaderEnv;
use crate::lua_api::state::SimState;
use crate::saved_variables::SavedVariablesManager;
use crate::toc::TocFile;
use rilua::Val;
use std::cell::RefCell;
use std::collections::HashSet;
use std::path::Path;
use std::rc::Rc;

use super::addon_saved_variables::{
    maybe_init_saved_variables, maybe_restore_clobbered_saved_variables,
};
use super::error::LoadError;
use super::file_diagnostics::{
    is_recorded_xml_failure, record_file_diagnostic, uses_wrath_file_diagnostics,
};
use super::lua_file::{load_lua_file, load_secure_replay_lua_file};
use super::xml_file::load_xml_file;
use super::{LoadResult, LoadTiming};
use nil_symbol_reports::{append_nil_symbol_diagnostics, enter_nil_symbol_environment};

/// Context for loading addon files (name, private table, and addon root for path resolution).
pub struct AddonContext<'a> {
    pub name: &'a str,
    pub table: Val,
    /// Addon root directory for fallback path resolution
    pub addon_root: &'a Path,
    /// Whether this addon uses the secure Lua environment (UseSecureEnvironment: 1)
    pub use_secure_env: bool,
    /// Whether to taint code with the addon name (false for Blizzard base UI).
    pub taint: bool,
}

pub(crate) struct LoadingAddonGuard {
    state: Rc<RefCell<SimState>>,
    addon_idx: u16,
    entered: bool,
    secure_env: bool,
    previous_secure_env: Option<bool>,
}

struct LoadingEnvironmentGuard {
    state: Rc<RefCell<SimState>>,
    previous_secure_env: Option<bool>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EnvironmentPass {
    Normal,
    Bootstrap,
    Secure,
    SecureDependency,
    SecureReplay,
}

impl EnvironmentPass {
    fn use_secure_env(self, ctx: &AddonContext<'_>) -> bool {
        match self {
            Self::Normal | Self::Bootstrap => ctx.use_secure_env,
            Self::Secure | Self::SecureDependency | Self::SecureReplay => true,
        }
    }

    fn loads_xml(self) -> bool {
        !matches!(self, Self::SecureReplay)
    }

    fn skips_explicit_environment(self) -> bool {
        matches!(self, Self::SecureDependency | Self::SecureReplay)
    }

    fn is_secure_load(self) -> bool {
        matches!(self, Self::Secure | Self::SecureDependency)
    }
}

impl LoadingAddonGuard {
    pub(crate) fn addon_index(&self) -> u16 {
        self.addon_idx
    }

    pub(crate) fn commit_loaded(&self) {
        let mut state = self.state.borrow_mut();
        if let Some(addon) = state.addons.get_mut(self.addon_idx as usize) {
            if self.secure_env {
                addon.secure_loaded = true;
            } else {
                addon.loaded = true;
            }
            addon.bootstrap_loaded = true;
        }
    }
}

impl Drop for LoadingAddonGuard {
    fn drop(&mut self) {
        let mut state = self.state.borrow_mut();
        state.loading_addon_secure_env = self.previous_secure_env;
        if !self.entered {
            return;
        }

        state
            .global_publications
            .retain(|(addon_index, _)| *addon_index != self.addon_idx);
        state
            .secure_global_publications
            .retain(|(addon_index, _)| *addon_index != self.addon_idx);
        state
            .pending_nested_addon_diagnostics
            .remove(&self.addon_idx);
        let position = state
            .loading_addon_stack
            .iter()
            .rposition(|addon_idx| *addon_idx == self.addon_idx)
            .expect("active addon load must remain in the loading stack");
        state.loading_addon_stack.remove(position);
        state.loading_addon_index = state.loading_addon_stack.last().copied();
    }
}

impl Drop for LoadingEnvironmentGuard {
    fn drop(&mut self) {
        self.state.borrow_mut().loading_addon_secure_env = self.previous_secure_env;
    }
}

fn enter_loading_environment(env: &LoaderEnv<'_>, secure_env: bool) -> LoadingEnvironmentGuard {
    let previous_secure_env = env
        .state()
        .borrow_mut()
        .loading_addon_secure_env
        .replace(secure_env);
    LoadingEnvironmentGuard {
        state: Rc::clone(env.state()),
        previous_secure_env,
    }
}

impl<'a> AddonContext<'a> {
    #[cfg(test)]
    pub fn new<L>(
        _lua: L,
        name: &'a str,
        table: Val,
        addon_root: &'a Path,
        use_secure_env: bool,
        taint: bool,
    ) -> crate::Result<Self> {
        Ok(Self {
            name,
            table,
            addon_root,
            use_secure_env,
            taint,
        })
    }
}

pub(super) fn load_addon_bootstrap_internal(
    env: &LoaderEnv<'_>,
    toc: &TocFile,
) -> Result<LoadResult, LoadError> {
    let folder_name = toc
        .addon_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(&toc.name);
    let mut result = empty_load_result(toc);
    if !toc.has_bootstrap_files() || bootstrap_has_loaded(env, folder_name) {
        return Ok(result);
    }
    let guard = begin_addon_load(env, folder_name, toc)?;
    let ctx = build_addon_context(env, toc, folder_name)?;
    let start = env.state().borrow().nil_symbol_accesses.len();
    load_addon_files(
        env,
        toc,
        folder_name,
        &ctx,
        EnvironmentPass::Bootstrap,
        &mut result,
    );
    finish_bootstrap_load(env, guard.addon_index(), &toc.name, start, &mut result);
    Ok(result)
}

fn finish_bootstrap_load(
    env: &LoaderEnv<'_>,
    addon_index: u16,
    addon_name: &str,
    access_start: usize,
    result: &mut LoadResult,
) {
    append_nil_symbol_diagnostics(env, addon_index, addon_name, access_start, result);
    append_pending_nested_addon_diagnostics(env, addon_index, result);
    if let Some(addon) = env
        .state()
        .borrow_mut()
        .addons
        .get_mut(addon_index as usize)
    {
        addon.bootstrap_loaded = true;
    }
}

fn empty_load_result(toc: &TocFile) -> LoadResult {
    LoadResult {
        name: toc.name.clone(),
        lua_files: 0,
        xml_files: 0,
        timing: LoadTiming::default(),
        warnings: Vec::new(),
        nil_symbol_observations: Vec::new(),
        missing_requirements: Vec::new(),
    }
}

fn bootstrap_has_loaded(env: &LoaderEnv<'_>, folder_name: &str) -> bool {
    env.state().borrow().addons.iter().any(|addon| {
        addon.folder_name == folder_name && (addon.bootstrap_loaded || addon.is_loaded())
    })
}

/// Internal addon loading with optional saved variables.
pub fn load_addon_internal(
    env: &LoaderEnv<'_>,
    toc: &TocFile,
    saved_vars_mgr: Option<&mut SavedVariablesManager>,
) -> Result<LoadResult, LoadError> {
    let folder_name = addon_folder_name(toc);
    let pass = if toc.is_secure_env() {
        if addon_loaded_in_environment(env, folder_name, false) {
            EnvironmentPass::SecureDependency
        } else {
            EnvironmentPass::Secure
        }
    } else {
        EnvironmentPass::Normal
    };
    let mut secure_load_path = HashSet::new();
    load_addon_internal_with_pass(env, toc, saved_vars_mgr, pass, &mut secure_load_path)
}

fn addon_folder_name(toc: &TocFile) -> &str {
    toc.addon_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(&toc.name)
}

fn addon_loaded_in_environment(env: &LoaderEnv<'_>, folder_name: &str, secure: bool) -> bool {
    env.state()
        .borrow()
        .addons
        .iter()
        .find(|addon| addon.folder_name == folder_name)
        .is_some_and(|addon| addon.is_loaded_in_environment(secure))
}

fn load_addon_internal_with_pass(
    env: &LoaderEnv<'_>,
    toc: &TocFile,
    saved_vars_mgr: Option<&mut SavedVariablesManager>,
    pass: EnvironmentPass,
    secure_load_path: &mut HashSet<String>,
) -> Result<LoadResult, LoadError> {
    let folder_name = addon_folder_name(toc);
    let secure = pass.is_secure_load();
    let mut result = empty_load_result(toc);
    if addon_loaded_in_environment(env, folder_name, secure) {
        return Ok(result);
    }
    if !enter_secure_load_path(secure, secure_load_path, folder_name) {
        return Ok(result);
    }

    let load_result = load_addon_internal_body(
        env,
        toc,
        folder_name,
        saved_vars_mgr,
        pass,
        secure_load_path,
        &mut result,
    );
    leave_secure_load_path(secure, secure_load_path, folder_name);
    load_result.map(|()| result)
}

fn enter_secure_load_path(
    secure: bool,
    secure_load_path: &mut HashSet<String>,
    folder_name: &str,
) -> bool {
    !secure || secure_load_path.insert(folder_name.to_string())
}

fn leave_secure_load_path(secure: bool, secure_load_path: &mut HashSet<String>, folder_name: &str) {
    if secure {
        secure_load_path.remove(folder_name);
    }
}

fn load_addon_internal_body(
    env: &LoaderEnv<'_>,
    toc: &TocFile,
    folder_name: &str,
    saved_vars_mgr: Option<&mut SavedVariablesManager>,
    pass: EnvironmentPass,
    secure_load_path: &mut HashSet<String>,
    result: &mut LoadResult,
) -> Result<(), LoadError> {
    let saved_vars_mgr =
        prepare_saved_variables(env, toc, folder_name, saved_vars_mgr, pass, result);
    let ctx = build_addon_context(env, toc, folder_name)?;
    let loading_guard =
        begin_addon_load_in_environment(env, folder_name, toc, pass.use_secure_env(&ctx))?;
    load_secure_dependencies_for_pass(env, toc, pass, result, secure_load_path)?;
    load_addon_files_and_finish(env, toc, &ctx, pass, saved_vars_mgr, loading_guard, result);
    Ok(())
}

fn load_secure_dependencies_for_pass(
    env: &LoaderEnv<'_>,
    toc: &TocFile,
    pass: EnvironmentPass,
    result: &mut LoadResult,
    secure_load_path: &mut HashSet<String>,
) -> Result<(), LoadError> {
    if pass.is_secure_load() && uses_secure_dependency_pass() {
        load_secure_dependencies(env, toc, result, secure_load_path)?;
    }
    Ok(())
}

/// R88g.1 (Retail 70688) loads a secure addon's dependencies in secureenv. Evidence: pr46-49-decomp-answers.md, sections R88g.1.
/// Other profiles keep master's library replay; their native behavior is UNKNOWN.
fn uses_secure_dependency_pass() -> bool {
    crate::client_profile::is_active_retail_12_1_5()
}

fn load_addon_files_and_finish(
    env: &LoaderEnv<'_>,
    toc: &TocFile,
    ctx: &AddonContext<'_>,
    pass: EnvironmentPass,
    saved_vars_mgr: Option<&mut SavedVariablesManager>,
    loading_guard: LoadingAddonGuard,
    result: &mut LoadResult,
) {
    let nil_symbol_access_start = env.state().borrow().nil_symbol_accesses.len();
    let addon_name = result.name.clone();
    load_addon_files(env, toc, ctx.name, ctx, pass, result);
    apply_addon_post_load(env, toc, ctx.name, ctx, pass, saved_vars_mgr, result);
    finish_addon_load(
        env,
        loading_guard,
        &addon_name,
        nil_symbol_access_start,
        result,
    );
}

fn prepare_saved_variables<'a>(
    env: &LoaderEnv<'_>,
    toc: &TocFile,
    folder_name: &str,
    saved_vars_mgr: Option<&'a mut SavedVariablesManager>,
    pass: EnvironmentPass,
    result: &mut LoadResult,
) -> Option<&'a mut SavedVariablesManager> {
    if pass == EnvironmentPass::SecureDependency {
        return None;
    }
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Wrath {
        saved_vars_mgr
    } else {
        maybe_init_saved_variables(env, toc, folder_name, saved_vars_mgr, result)
    }
}

fn apply_addon_post_load(
    env: &LoaderEnv<'_>,
    toc: &TocFile,
    folder_name: &str,
    ctx: &AddonContext<'_>,
    pass: EnvironmentPass,
    saved_vars_mgr: Option<&mut SavedVariablesManager>,
    result: &mut LoadResult,
) {
    if pass == EnvironmentPass::Normal {
        maybe_replay_blizzard_lua_in_secure_env(env, toc, folder_name, ctx, result);
    }
    if pass == EnvironmentPass::SecureDependency {
        return;
    }
    if folder_name == "Blizzard_FrameXML" {
        load_modified_click_defaults(env, toc, result);
    }
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Wrath {
        load_conventional_bindings(env, toc, ctx, result);
        maybe_init_saved_variables(env, toc, folder_name, saved_vars_mgr, result);
    } else {
        read_conventional_binding_metadata(env, toc, folder_name, result);
        maybe_restore_clobbered_saved_variables(env, folder_name, saved_vars_mgr);
    }
    apply_blizzard_post_load_patches(env, folder_name, result);
}

fn finish_addon_load(
    env: &LoaderEnv<'_>,
    loading_guard: LoadingAddonGuard,
    addon_name: &str,
    nil_symbol_access_start: usize,
    result: &mut LoadResult,
) {
    let addon_index = loading_guard.addon_index();
    append_nil_symbol_diagnostics(
        env,
        addon_index,
        addon_name,
        nil_symbol_access_start,
        result,
    );
    append_pending_nested_addon_diagnostics(env, addon_index, result);
    loading_guard.commit_loaded();
}

fn load_secure_dependencies(
    env: &LoaderEnv<'_>,
    toc: &TocFile,
    result: &mut LoadResult,
    secure_load_path: &mut HashSet<String>,
) -> Result<(), LoadError> {
    let screen = env.state().borrow().screen_kind;
    let Some(addon_root) = toc.addon_dir.parent() else {
        return Ok(());
    };
    for (name, required) in secure_dependency_names(toc, screen) {
        load_secure_dependency(
            env,
            addon_root,
            screen,
            &name,
            required,
            result,
            secure_load_path,
        )?;
    }
    Ok(())
}

fn secure_dependency_names(
    toc: &TocFile,
    screen: crate::screen::ScreenKind,
) -> Vec<(String, bool)> {
    toc.optional_deps_for_screen(screen)
        .into_iter()
        .map(|name| (name, false))
        .chain(
            toc.dependencies_for_screen(screen)
                .into_iter()
                .map(|name| (name, true)),
        )
        .collect()
}

fn load_secure_dependency(
    env: &LoaderEnv<'_>,
    addon_root: &Path,
    screen: crate::screen::ScreenKind,
    dependency: &str,
    required: bool,
    result: &mut LoadResult,
    secure_load_path: &mut HashSet<String>,
) -> Result<(), LoadError> {
    let Some(dependency_toc) =
        read_secure_dependency_toc(addon_root, dependency, required, screen)?
    else {
        return Ok(());
    };
    load_secure_dependency_toc(
        env,
        &dependency_toc,
        dependency,
        required,
        result,
        secure_load_path,
    )
}

fn load_secure_dependency_toc(
    env: &LoaderEnv<'_>,
    dependency_toc: &TocFile,
    dependency: &str,
    required: bool,
    result: &mut LoadResult,
    secure_load_path: &mut HashSet<String>,
) -> Result<(), LoadError> {
    let folder_name = addon_folder_name(dependency_toc);
    let ordinary_loaded = addon_loaded_in_environment(env, folder_name, false);
    // R88t requires this secure request regardless of the ordinary request result. Evidence: pr46-49-decomp-answers.md, sections R88t.
    let ordinary_result = load_ordinary_dependency_if_needed(
        env,
        dependency_toc,
        dependency,
        required,
        ordinary_loaded,
        result,
        secure_load_path,
    );
    let secure_result = load_dependency_in_secure_environment(
        env,
        dependency_toc,
        ordinary_loaded,
        secure_load_path,
    );
    finish_secure_dependency_requests(result, dependency, required, ordinary_result, secure_result)
}

fn finish_secure_dependency_requests(
    result: &mut LoadResult,
    dependency: &str,
    required: bool,
    ordinary_result: Result<(), LoadError>,
    secure_result: Result<LoadResult, LoadError>,
) -> Result<(), LoadError> {
    let secure_result =
        record_dependency_result(result, dependency, required, secure_result, "secure");
    match ordinary_result {
        Ok(()) => secure_result,
        Err(ordinary_error) => {
            if let Err(secure_error) = secure_result {
                tracing::warn!(
                    "Secure dependency {dependency} request also failed after ordinary request: {secure_error}"
                );
            }
            Err(ordinary_error)
        }
    }
}

fn load_dependency_in_secure_environment(
    env: &LoaderEnv<'_>,
    toc: &TocFile,
    ordinary_loaded: bool,
    secure_load_path: &mut HashSet<String>,
) -> Result<LoadResult, LoadError> {
    let secure_pass = secure_dependency_pass(ordinary_loaded, toc.is_secure_env());
    load_addon_internal_with_pass(env, toc, None, secure_pass, secure_load_path)
}

fn load_ordinary_dependency_if_needed(
    env: &LoaderEnv<'_>,
    dependency_toc: &TocFile,
    dependency: &str,
    required: bool,
    ordinary_loaded: bool,
    result: &mut LoadResult,
    secure_load_path: &mut HashSet<String>,
) -> Result<(), LoadError> {
    if dependency_toc.is_secure_env() || ordinary_loaded {
        return Ok(());
    }
    load_dependency_ordinary_pass(
        env,
        dependency_toc,
        dependency,
        required,
        result,
        secure_load_path,
    )
}

fn secure_dependency_pass(
    ordinary_loaded: bool,
    dependency_uses_secure_env: bool,
) -> EnvironmentPass {
    if ordinary_loaded || !dependency_uses_secure_env {
        EnvironmentPass::SecureDependency
    } else {
        EnvironmentPass::Secure
    }
}

fn load_dependency_ordinary_pass(
    env: &LoaderEnv<'_>,
    toc: &TocFile,
    dependency: &str,
    required: bool,
    result: &mut LoadResult,
    secure_load_path: &mut HashSet<String>,
) -> Result<(), LoadError> {
    let normal_result =
        load_addon_internal_with_pass(env, toc, None, EnvironmentPass::Normal, secure_load_path);
    record_dependency_result(result, dependency, required, normal_result, "ordinary")
}

fn read_secure_dependency_toc(
    addon_root: &Path,
    dependency: &str,
    required: bool,
    screen: crate::screen::ScreenKind,
) -> Result<Option<TocFile>, LoadError> {
    let Some(path) = super::find_toc_file(&addon_root.join(dependency)) else {
        return Ok(None);
    };
    let toc = match TocFile::from_file(&path) {
        Ok(toc) => toc,
        Err(error) if required => return Err(LoadError::Toc(error)),
        Err(error) => {
            tracing::warn!("Cannot read optional secure dependency {dependency}: {error}");
            return Ok(None);
        }
    };
    // StoreUI's optional Plunderstorm dependency is screen-eligible in Game,
    // but must not publish its mode constants in standard Retail.
    match (
        toc.allows_screen(screen),
        toc.is_game_type_restricted(),
        required,
    ) {
        (false, _, true) => Err(LoadError::DepWrongActiveInterface(dependency.to_owned())),
        (true, true, true) => Err(LoadError::DepWrongGameType(dependency.to_owned())),
        (true, false, _) => Ok(Some(toc)),
        _ => Ok(None),
    }
}

fn record_dependency_result(
    parent: &mut LoadResult,
    dependency: &str,
    required: bool,
    result: Result<LoadResult, LoadError>,
    pass: &str,
) -> Result<(), LoadError> {
    match result {
        Ok(nested) if required => append_dependency_result(parent, nested),
        Ok(nested) => log_optional_dependency_result(parent, dependency, nested),
        Err(error) if required => return Err(error),
        Err(error) => {
            tracing::warn!("Optional secure dependency {dependency} {pass} load failed: {error}")
        }
    }
    Ok(())
}

fn append_dependency_result(parent: &mut LoadResult, dependency: LoadResult) {
    parent.timing.accumulate(&dependency.timing);
    parent.extend_diagnostics(dependency.diagnostics());
}

fn log_optional_dependency_result(parent: &mut LoadResult, dependency: &str, result: LoadResult) {
    parent.timing.accumulate(&result.timing);
    for warning in result.warnings {
        tracing::warn!("Optional secure dependency {dependency}: {warning}");
    }
    for observation in result.nil_symbol_observations {
        tracing::warn!("Optional secure dependency {dependency}: {observation}");
    }
    for requirement in result.missing_requirements {
        tracing::warn!("Optional secure dependency {dependency}: {requirement}");
    }
}

pub(crate) fn append_pending_nested_addon_diagnostics(
    env: &LoaderEnv<'_>,
    addon_index: u16,
    result: &mut LoadResult,
) {
    let pending = env
        .state()
        .borrow_mut()
        .pending_nested_addon_diagnostics
        .remove(&addon_index);
    if let Some(pending) = pending {
        result.extend_diagnostics(pending);
    }
}

/// Run hand-written workarounds that must fire after specific Blizzard addons
/// finish loading (e.g. restoring globals the addon's cleanup wiped, or
/// monkey-patching mixins that don't quite line up with our stub state).
/// Each patch is keyed by addon folder name and only fires for that addon.
fn apply_blizzard_post_load_patches(
    env: &LoaderEnv<'_>,
    folder_name: &str,
    result: &mut LoadResult,
) {
    #[cfg(feature = "client-mists")]
    crate::mists::post_load::apply_for_runtime_addon_load(env, folder_name);

    crate::lua_api::workarounds::patch_runtime_surface_for_addon_load(env, folder_name);

    match folder_name {
        "Blizzard_EnvironmentCleanup" => patch_environment_cleanup(env, result),
        "Blizzard_SharedXML" => patch_shared_xml_anim_mixins(env, result),
        "Blizzard_UIParent" => patch_uiparent_managed_frame_mixin(env, result),
        "Blizzard_GlueParent" => patch_glueparent_uiparent_attributes(env, result),
        "Blizzard_SharedMapDataProviders" => patch_unit_position_frame_mixin(env, result),
        "Blizzard_UIPanels_Game" => patch_quest_log_mixin(env, result),
        "Blizzard_FrameXMLUtil" => {
            crate::lua_api::workarounds::patch_quest_objective_defaults_for_addon_load(env);
        }
        "Blizzard_PlayerSpells" => patch_playerspells_onload_backfill(env, result),
        #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
        #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
        "Blizzard_Dispatcher" => {
            crate::lua_api::workarounds::patch_dispatcher_surface_for_addon_load(env)
        }
        #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
        "Blizzard_AchievementUI" => {
            crate::lua_api::workarounds::patch_achievement_search_preview_for_addon_load(env)
        }
        _ => {}
    }
}

fn push_patch_warning(
    result: &mut LoadResult,
    folder_name: &str,
    what: &str,
    e: &dyn std::fmt::Display,
) {
    result
        .warnings
        .push(format!("Failed to {what} for {folder_name}: {e}"));
}

fn patch_environment_cleanup(env: &LoaderEnv<'_>, result: &mut LoadResult) {
    if let Err(e) = env.restore_post_cleanup_globals() {
        push_patch_warning(
            result,
            "Blizzard_EnvironmentCleanup",
            "restore post-cleanup globals",
            &e,
        );
    }
}

fn patch_shared_xml_anim_mixins(env: &LoaderEnv<'_>, result: &mut LoadResult) {
    crate::lua_api::workarounds::patch_callback_registry_defaults(env);
    if let Err(e) = crate::lua_api::workarounds::patch_shared_xml_anim_mixins(env) {
        push_patch_warning(
            result,
            "Blizzard_SharedXML",
            "patch Blizzard_SharedXML animation mixins",
            &e,
        );
    }
}

fn patch_uiparent_managed_frame_mixin(env: &LoaderEnv<'_>, result: &mut LoadResult) {
    if let Err(e) = crate::lua_api::workarounds::patch_uiparent_managed_frame_mixin(env) {
        push_patch_warning(
            result,
            "Blizzard_UIParent",
            "patch UIParentManagedFrameMixin",
            &e,
        );
    }
}

fn patch_glueparent_uiparent_attributes(env: &LoaderEnv<'_>, result: &mut LoadResult) {
    if let Err(e) = crate::lua_api::workarounds::patch_glueparent_uiparent_attributes(env) {
        push_patch_warning(
            result,
            "Blizzard_GlueParent",
            "restore UIParent attributes after GlueParent alias",
            &e,
        );
    }
}

fn patch_unit_position_frame_mixin(env: &LoaderEnv<'_>, result: &mut LoadResult) {
    if let Err(e) = crate::lua_api::workarounds::patch_unit_position_frame_mixin(env) {
        push_patch_warning(
            result,
            "Blizzard_SharedMapDataProviders",
            "patch UnitPositionFrameMixin",
            &e,
        );
    }
}

fn patch_quest_log_mixin(env: &LoaderEnv<'_>, result: &mut LoadResult) {
    if let Err(e) = crate::lua_api::workarounds::patch_quest_log_mixin(env) {
        push_patch_warning(result, "Blizzard_UIPanels_Game", "patch QuestLogMixin", &e);
    }
}

fn patch_playerspells_onload_backfill(env: &LoaderEnv<'_>, result: &mut LoadResult) {
    if let Err(e) = crate::lua_api::workarounds::patch_playerspells_onload_backfill(env) {
        push_patch_warning(
            result,
            "Blizzard_PlayerSpells",
            "backfill PlayerSpells OnLoad state",
            &e,
        );
    }
}

fn build_addon_context<'a>(
    env: &LoaderEnv<'a>,
    toc: &'a TocFile,
    folder_name: &'a str,
) -> Result<AddonContext<'a>, LoadError> {
    let addon_table = env
        .create_addon_table()
        .map_err(|e| LoadError::Lua(e.to_string()))?;
    env.with_state(|state| {
        crate::c_api::c_addons_local_table::retain_namespace(state, folder_name, addon_table, toc);
        Ok::<(), LoadError>(())
    })?;

    Ok(AddonContext {
        name: folder_name,
        table: addon_table,
        addon_root: &toc.addon_dir,
        use_secure_env: toc.is_secure_env(),
        taint: !is_blizzard_addon(toc),
    })
}

pub(crate) fn begin_addon_load(
    env: &LoaderEnv<'_>,
    folder_name: &str,
    toc: &TocFile,
) -> Result<LoadingAddonGuard, LoadError> {
    begin_addon_load_in_environment(env, folder_name, toc, toc.is_secure_env())
}

fn begin_addon_load_in_environment(
    env: &LoaderEnv<'_>,
    folder_name: &str,
    toc: &TocFile,
    secure_env: bool,
) -> Result<LoadingAddonGuard, LoadError> {
    let addon_idx = resolve_addon_index(env, folder_name);
    let mut state = env.state().borrow_mut();
    let previous_secure_env = state.loading_addon_secure_env.replace(secure_env);
    let entered = !state.loading_addon_stack.contains(&addon_idx);
    if entered && let Err(error) = prepare_addon_load_state(&mut state, addon_idx, folder_name, toc)
    {
        state.loading_addon_secure_env = previous_secure_env;
        return Err(error);
    }

    Ok(LoadingAddonGuard {
        state: Rc::clone(env.state()),
        addon_idx,
        entered,
        secure_env,
        previous_secure_env,
    })
}

fn prepare_addon_load_state(
    state: &mut SimState,
    addon_idx: u16,
    folder_name: &str,
    toc: &TocFile,
) -> Result<(), LoadError> {
    update_addon_metadata(state, addon_idx, folder_name, toc);
    #[cfg(feature = "retail-12-0-7")]
    crate::c_api::c_ui_file_asset::register_loaded_addon_files(state, folder_name, &toc.addon_dir)
        .map_err(|error| LoadError::io_with_path(&toc.addon_dir, error))?;
    state.loading_addon_stack.push(addon_idx);
    state.loading_addon_index = Some(addon_idx);
    Ok(())
}

fn update_addon_metadata(state: &mut SimState, addon_idx: u16, folder_name: &str, toc: &TocFile) {
    let Some(addon) = state.addons.get_mut(addon_idx as usize) else {
        return;
    };

    addon.addon_dir = Some(toc.addon_dir.clone());
    addon.title = toc
        .metadata
        .get("Title")
        .cloned()
        .unwrap_or_else(|| folder_name.to_string());
    addon.notes = toc.metadata.get("Notes").cloned().unwrap_or_default();
    addon.enabled = true;
    addon.load_on_demand = toc.is_load_on_demand();
    addon.use_secure_env = toc.is_secure_env();
    addon.metadata = toc.metadata.clone();
    addon.dependencies = toc.dependencies();
    addon.default_enabled = toc.default_enabled();
}

fn is_blizzard_addon(toc: &TocFile) -> bool {
    // Blizzard UI source can come from a repo symlink or the CASC cache, so
    // path shape is not a security signal. Retail marks Blizzard UI TOCs with
    // AllowLoad, while a few secure-only Blizzard helpers only advertise
    // UseSecureEnvironment.
    toc.loads_as_blizzard_code()
}

/// Find or auto-register addon in the addon list, returning its index.
fn resolve_addon_index(env: &LoaderEnv<'_>, folder_name: &str) -> u16 {
    let mut s = env.state().borrow_mut();
    let idx = s
        .addons
        .iter()
        .position(|a| a.folder_name == folder_name)
        .unwrap_or_else(|| {
            let idx = s.addons.len();
            s.addons.push(crate::lua_api::AddonInfo {
                folder_name: folder_name.to_string(),
                title: folder_name.to_string(),
                enabled: true,
                ..Default::default()
            });
            idx
        });
    idx as u16
}

/// Load Lua/XML files from the directory selected by the TOC.
fn load_addon_files(
    env: &LoaderEnv<'_>,
    toc: &TocFile,
    folder_name: &str,
    ctx: &AddonContext,
    pass: EnvironmentPass,
    result: &mut LoadResult,
) {
    let bootstrap_loaded = bootstrap_has_loaded(env, folder_name);

    for (index, (file_rel, file)) in toc.files.iter().zip(toc.file_paths()).enumerate() {
        if !should_load_addon_file(env, toc, ctx, pass, bootstrap_loaded, index, file_rel) {
            continue;
        }
        let file_ctx = addon_file_context(toc, ctx, index, pass);
        if std::env::var("WOW_SIM_VERBOSE").is_ok() {
            println!("  loading file {}", file_rel.display());
        }
        load_addon_file(env, &file_ctx, result, &file, pass);
    }
}

fn should_load_addon_file(
    env: &LoaderEnv<'_>,
    toc: &TocFile,
    ctx: &AddonContext,
    pass: EnvironmentPass,
    bootstrap_loaded: bool,
    index: usize,
    file_rel: &Path,
) -> bool {
    if !loads_file_in_bootstrap_phase(pass, toc.file_is_bootstrap(index), bootstrap_loaded) {
        return false;
    }
    if !toc.file_allows_screen(index, env.state().borrow().screen_kind) {
        return false;
    }
    if skips_loaded_dependency_xml(env, ctx, pass, file_rel) {
        return false;
    }
    if should_skip_addon_file(toc, file_rel) {
        return false;
    }
    if pass.skips_explicit_environment() && toc.file_use_secure_env(index).is_some() {
        return false;
    }
    toc.file_allows_environment(index, pass.use_secure_env(ctx))
}

fn skips_loaded_dependency_xml(
    env: &LoaderEnv<'_>,
    ctx: &AddonContext<'_>,
    pass: EnvironmentPass,
    file_rel: &Path,
) -> bool {
    pass == EnvironmentPass::SecureDependency
        && file_rel
            .extension()
            .is_some_and(|extension| extension == "xml")
        && addon_loaded_in_environment(env, ctx.name, false)
}

fn addon_file_context<'a>(
    toc: &TocFile,
    ctx: &AddonContext<'a>,
    index: usize,
    pass: EnvironmentPass,
) -> AddonContext<'a> {
    AddonContext {
        name: ctx.name,
        table: ctx.table,
        addon_root: ctx.addon_root,
        use_secure_env: toc
            .file_use_secure_env(index)
            .unwrap_or_else(|| pass.use_secure_env(ctx)),
        taint: ctx.taint,
    }
}

fn load_modified_click_defaults(env: &LoaderEnv<'_>, toc: &TocFile, result: &mut LoadResult) {
    use crate::client_profile::ClientProfile;
    let filename = match crate::client_profile::ACTIVE {
        ClientProfile::Retail | ClientProfile::Ptr => "Bindings_Standard.xml",
        ClientProfile::WowForever => "Bindings_Camelot.xml",
        ClientProfile::Mists => "Bindings_Mists.xml",
        ClientProfile::Era | ClientProfile::Anniversary => "Bindings_Vanilla.xml",
        ClientProfile::Wrath => return,
    };
    let path = toc.addon_dir.join(filename);
    let ui = match crate::xml::parse_xml_file(&path) {
        Ok(ui) => ui,
        Err(error) => {
            result.warnings.push(format!("{}: {error}", path.display()));
            return;
        }
    };
    let install = install_modified_click_defaults(env, ui.elements);
    if let Err(error) = install {
        result.warnings.push(format!(
            "{}: installing modified-click defaults: {error}",
            path.display()
        ));
    }
}

fn install_modified_click_defaults(
    env: &LoaderEnv<'_>,
    elements: Vec<crate::xml::XmlElement>,
) -> crate::Result<()> {
    env.with_state(|state| {
        use crate::lua_api::methods::{create_string, create_table, table_set};
        let table = create_table(state);
        for element in elements {
            if let crate::xml::XmlElement::ModifiedClick(click) = element
                && let Some(action) = click.action
            {
                let value = create_string(state, click.default.as_deref().unwrap_or("NONE"));
                table_set(state, table, &action, value);
            }
        }
        table_set(
            state,
            Val::Table(state.global),
            "__wow_modified_clicks",
            table,
        );
        Ok(())
    })
}

fn load_conventional_bindings(
    env: &LoaderEnv<'_>,
    toc: &TocFile,
    ctx: &AddonContext<'_>,
    result: &mut LoadResult,
) {
    let path = toc.addon_dir.join("Bindings.xml");
    if !path.is_file() || toc.file_paths().contains(&path) {
        return;
    }
    if let Err(error) = load_xml_file(env, &path, ctx, &mut result.timing, &mut result.warnings) {
        result.warnings.push(format!("{}: {error}", path.display()));
    }
}

fn read_conventional_binding_metadata(
    env: &LoaderEnv<'_>,
    toc: &TocFile,
    folder_name: &str,
    result: &mut LoadResult,
) {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail {
        return;
    }
    let path = toc.addon_dir.join("Bindings.xml");
    if !path.is_file() || toc.file_paths().contains(&path) {
        return;
    }
    match crate::xml::parse_xml_file(&path) {
        Ok(ui) => {
            let contains_bindings = has_named_binding(&ui.elements);
            if let Some(addon) = env
                .state()
                .borrow_mut()
                .addons
                .iter_mut()
                .find(|addon| addon.folder_name == folder_name)
            {
                addon.contains_bindings |= contains_bindings;
            }
        }
        Err(error) => result.warnings.push(format!("{}: {error}", path.display())),
    }
}

fn has_named_binding(elements: &[crate::xml::XmlElement]) -> bool {
    elements.iter().any(|element| {
        matches!(element, crate::xml::XmlElement::Binding(binding) if binding.name.is_some())
    })
}

fn loads_file_in_bootstrap_phase(
    pass: EnvironmentPass,
    is_bootstrap: bool,
    bootstrap_loaded: bool,
) -> bool {
    match pass {
        EnvironmentPass::Bootstrap => is_bootstrap,
        EnvironmentPass::Secure | EnvironmentPass::SecureDependency => true,
        EnvironmentPass::Normal | EnvironmentPass::SecureReplay => {
            !is_bootstrap || !bootstrap_loaded
        }
    }
}

fn maybe_replay_blizzard_lua_in_secure_env(
    env: &LoaderEnv<'_>,
    toc: &TocFile,
    folder_name: &str,
    ctx: &AddonContext,
    result: &mut LoadResult,
) {
    if ctx.use_secure_env {
        return;
    }
    if !is_secure_replay_library_addon(folder_name)
        && !is_secure_replay_dependency(env, folder_name)
    {
        return;
    }

    load_addon_files(
        env,
        toc,
        folder_name,
        ctx,
        EnvironmentPass::SecureReplay,
        result,
    );
}

const SECURE_REPLAY_LIBRARY_ADDONS: &[&str] = &[
    "Blizzard_SharedXMLBase",
    "Blizzard_SharedXML",
    "Blizzard_FrameXMLUtil",
    "Blizzard_CombatLogBase",
    "Blizzard_CatalogShopSharedTemplates",
    "Blizzard_CatalogShopSharedUtil",
    "Blizzard_AsyncRequest",
    "Blizzard_GameTooltip",
    "Blizzard_RaidWarning",
];

pub(super) fn is_secure_replay_library_addon(folder_name: &str) -> bool {
    // Retail 12.1.5 secure loads use the shared dependency pass; preserve the
    // legacy replay path where R88g.1 does not apply. Evidence: pr46-49-decomp-answers.md, sections R88g.1.
    if crate::client_profile::is_active_retail_12_1_5() {
        return false;
    }

    // These Classic colors consume public FrameXML masks, absent in secureenv.
    if folder_name == "Blizzard_CombatLogBase"
        && matches!(
            crate::client_profile::ACTIVE,
            crate::client_profile::ClientProfile::Mists
                | crate::client_profile::ClientProfile::Era
                | crate::client_profile::ClientProfile::Anniversary
        )
    {
        return false;
    }

    SECURE_REPLAY_LIBRARY_ADDONS.contains(&folder_name)
}

fn secure_replay_dependency_names(metadata: &crate::toc::TocMetadata) -> Vec<String> {
    crate::toc::collect_metadata_lists(
        metadata,
        &[
            "Dep",
            "RequiredDep",
            "Dependencies",
            "RequiredDeps",
            "OptionalDep",
            "OptionalDeps",
        ],
    )
}

/// INFERRED (R88cj): Forever secure replay follows enabled secure TOC dependencies.
fn is_secure_replay_dependency(env: &LoaderEnv<'_>, folder_name: &str) -> bool {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::WowForever {
        return false;
    }
    let state = env.state().borrow();
    let mut pending = state
        .addons
        .iter()
        .filter(|addon| addon.enabled && addon.use_secure_env)
        .flat_map(|addon| secure_replay_dependency_names(&addon.metadata))
        .collect::<Vec<_>>();
    let mut visited = HashSet::new();
    while let Some(dependency) = pending.pop() {
        if !visited.insert(dependency.clone()) {
            continue;
        }
        if dependency == folder_name {
            return true;
        }
        if let Some(addon) = state
            .addons
            .iter()
            .find(|addon| addon.folder_name == dependency)
        {
            pending.extend(secure_replay_dependency_names(&addon.metadata));
        }
    }
    false
}

pub(super) fn should_skip_addon_file(toc: &TocFile, file_rel: &Path) -> bool {
    is_mists_collections_wardrobe_file(toc, file_rel)
}

fn is_mists_collections_wardrobe_file(toc: &TocFile, file_rel: &Path) -> bool {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Mists {
        return false;
    }
    if toc.name != "Blizzard_Collections" {
        return false;
    }

    file_rel
        .to_string_lossy()
        .replace('\\', "/")
        .contains("Wardrobe")
}

fn load_addon_file(
    env: &LoaderEnv<'_>,
    ctx: &AddonContext<'_>,
    result: &mut LoadResult,
    file: &std::path::Path,
    pass: EnvironmentPass,
) {
    let _loading_environment_guard = enter_loading_environment(env, ctx.use_secure_env);
    let _nil_symbol_environment_guard = enter_nil_symbol_environment(env, ctx.use_secure_env);
    if !file.is_file() {
        if uses_wrath_file_diagnostics() && file.extension().is_some_and(|ext| ext == "xml") {
            record_file_diagnostic(&mut result.warnings, ctx, file, "Couldn't open");
        } else {
            result.warnings.push(format!(
                "{}: IO error: No such file or directory (os error 2)",
                file.display()
            ));
        }
        return;
    }

    match file.extension().and_then(|ext| ext.to_str()).unwrap_or("") {
        "lua" => load_addon_lua_file(env, ctx, result, file, pass),
        "xml" if pass.loads_xml() => load_addon_xml_file(env, ctx, result, file),
        "xml" => {}
        _ => result
            .warnings
            .push(format!("{}: unknown file type", file.display())),
    }
}

fn load_addon_lua_file(
    env: &LoaderEnv<'_>,
    ctx: &AddonContext<'_>,
    result: &mut LoadResult,
    file: &std::path::Path,
    pass: EnvironmentPass,
) {
    let load_result = if pass == EnvironmentPass::SecureReplay {
        load_secure_replay_lua_file(env, file, ctx, &mut result.timing)
    } else {
        load_lua_file(env, file, ctx, &mut result.timing)
    };
    match load_result {
        Ok(()) => result.lua_files += 1,
        Err(error) => result
            .warnings
            .push(format!("{}: {}", file.display(), error)),
    }
    crate::lua_api::workarounds::apply_cpp_mixin_stubs_after_lua_file(env);
}

fn load_addon_xml_file(
    env: &LoaderEnv<'_>,
    ctx: &AddonContext<'_>,
    result: &mut LoadResult,
    file: &std::path::Path,
) {
    let restore_add_to_secure_env = enable_secure_xml_frame_exports(env, ctx);
    let load_result = load_xml_file(env, file, ctx, &mut result.timing, &mut result.warnings);
    restore_secure_xml_frame_exports(env, restore_add_to_secure_env);

    match load_result {
        Ok(count) => {
            result.xml_files += 1;
            result.lua_files += count;
        }
        Err(error) if is_recorded_xml_failure(&error) => {}
        Err(error) => result
            .warnings
            .push(format!("{}: {}", file.display(), error)),
    }
}

fn enable_secure_xml_frame_exports(env: &LoaderEnv<'_>, ctx: &AddonContext<'_>) -> Option<bool> {
    if !ctx.use_secure_env {
        return None;
    }

    let mut state = env.state().borrow_mut();
    let previous = state.loading_add_to_secure_env;
    state.loading_add_to_secure_env = true;
    Some(previous)
}

fn restore_secure_xml_frame_exports(env: &LoaderEnv<'_>, previous: Option<bool>) {
    let Some(previous) = previous else {
        return;
    };
    env.state().borrow_mut().loading_add_to_secure_env = previous;
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::lua_api::WowLuaEnv;
    use crate::toc::TocFile;

    use super::{build_addon_context, load_addon_internal};

    #[cfg(feature = "client-wowforever")]
    fn write_addon(root: &Path, name: &str, toc: &str, lua: &str) -> std::path::PathBuf {
        let addon_dir = root.join(name);
        std::fs::create_dir_all(&addon_dir).unwrap();
        std::fs::write(addon_dir.join(format!("{name}.toc")), toc).unwrap();
        std::fs::write(addon_dir.join("Main.lua"), lua).unwrap();
        addon_dir
    }

    #[test]
    fn blizzard_talent_ui_context_is_not_tainted() {
        let env = WowLuaEnv::new().unwrap();
        let addon_dir = Path::new("/repo/Interface/BlizzardUI/Blizzard_TalentUI");
        let toc = TocFile::parse(
            addon_dir,
            "## Title: Blizzard Talent UI\n## AllowLoad: Game\nMain.lua",
        );

        let ctx = build_addon_context(&env.loader_env(), &toc, "Blizzard_TalentUI").unwrap();

        assert!(
            !ctx.taint,
            "Blizzard_TalentUI loader context must not stamp addon taint"
        );
    }

    #[test]
    fn cached_blizzard_ui_context_is_not_tainted() {
        let env = WowLuaEnv::new().unwrap();
        let addon_dir = Path::new("/tmp/wow-ui-sim/blizzard-ui/Blizzard_Menu");
        let toc = TocFile::parse(
            addon_dir,
            "## Title: Blizzard Menu\n## AllowLoad: Both\nMenu.lua",
        );

        let ctx = build_addon_context(&env.loader_env(), &toc, "Blizzard_Menu").unwrap();

        assert!(
            !ctx.taint,
            "CASC-cached Blizzard UI source must not stamp addon taint"
        );
    }

    #[test]
    fn secure_environment_context_is_not_tainted() {
        let env = WowLuaEnv::new().unwrap();
        let addon_dir = Path::new("/tmp/wow-ui-sim/blizzard-ui/Blizzard_ClassTrialSecure");
        let toc = TocFile::parse(
            addon_dir,
            "## Title: Blizzard Class Trial Secure\n## UseSecureEnvironment: 1\nSecure.lua",
        );

        let ctx =
            build_addon_context(&env.loader_env(), &toc, "Blizzard_ClassTrialSecure").unwrap();

        assert!(
            !ctx.taint,
            "secure-environment Blizzard helpers must not stamp addon taint"
        );
    }

    #[test]
    fn blizzard_folder_context_is_not_tainted_without_allow_load() {
        let env = WowLuaEnv::new().unwrap();
        let addon_dir = Path::new("/tmp/wow-ui-sim/blizzard-ui/Blizzard_PersonalResourceDisplay");
        let toc = TocFile::parse(
            addon_dir,
            "## Title: Blizzard_PersonalResourceDisplay\n## AllowLoadGameType: mainline\nMain.lua",
        );

        let ctx = build_addon_context(&env.loader_env(), &toc, "Blizzard_PersonalResourceDisplay")
            .unwrap();

        assert!(
            !ctx.taint,
            "internal Blizzard_* addon folders must not stamp addon taint"
        );
    }

    #[test]
    fn path_alone_does_not_make_addon_untainted() {
        let env = WowLuaEnv::new().unwrap();
        let addon_dir = Path::new("/tmp/wow-ui-sim/blizzard-ui/TestAddon");
        let toc = TocFile::parse(addon_dir, "## Title: TestAddon\nMain.lua");

        let ctx = build_addon_context(&env.loader_env(), &toc, "TestAddon").unwrap();

        assert!(
            ctx.taint,
            "paths are not trusted as Blizzard taint semantics"
        );
    }

    #[test]
    fn third_party_addon_context_is_tainted() {
        let env = WowLuaEnv::new().unwrap();
        let addon_dir = Path::new("/repo/Interface/AddOns/TestAddon");
        let toc = TocFile::parse(addon_dir, "## Title: Test Addon\nMain.lua");

        let ctx = build_addon_context(&env.loader_env(), &toc, "TestAddon").unwrap();

        assert!(ctx.taint, "third-party addon code must be taint stamped");
    }

    #[cfg(feature = "client-wowforever")]
    #[test]
    fn secure_addon_toc_dependency_replays_into_secure_environment() {
        let env = WowLuaEnv::new().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let provider_dir = write_addon(
            temp.path(),
            "ReplayProvider",
            "## AllowLoad: Game\nMain.lua\n",
            "ReplayProviderValue = function() return 'from-provider' end",
        );
        write_addon(
            temp.path(),
            "ReplayConsumer",
            "## UseSecureEnvironment: 1\n## OptionalDep: ReplayProvider\nMain.lua\n",
            "",
        );
        env.scan_and_register_addons(temp.path());
        let toc = TocFile::from_file(&provider_dir.join("ReplayProvider.toc")).unwrap();

        load_addon_internal(&env.loader_env(), &toc, None).unwrap();

        let replayed: (String, String) = env
            .eval("return type(__secureenv.ReplayProviderValue), __secureenv.ReplayProviderValue()")
            .unwrap();
        assert_eq!(replayed, ("function".into(), "from-provider".into()));
    }

    #[test]
    fn missing_toc_files_warn_without_calling_lua_error_handler() {
        let env = WowLuaEnv::new().unwrap();
        env.exec(
            r#"
            __missing_file_error_count = 0
            seterrorhandler(function(message)
                __missing_file_error_count = __missing_file_error_count + 1
                __missing_file_last_error = message
            end)
            "#,
        )
        .unwrap();

        let temp = tempfile::tempdir().unwrap();
        let addon_dir = temp.path().join("MissingFiles");
        std::fs::create_dir(&addon_dir).unwrap();
        let toc = TocFile::parse(
            &addon_dir,
            "## Title: Missing Files\nMissing.lua\nMissing.xml\n",
        );

        let result = load_addon_internal(&env.loader_env(), &toc, None).unwrap();
        let error_count: i32 = env.eval("return __missing_file_error_count").unwrap();

        assert_eq!(error_count, 0);
        assert_eq!(result.lua_files, 0);
        assert_eq!(result.xml_files, 0);
        assert_eq!(result.warnings.len(), 2);
        let lua_warning = format!(
            "{}: IO error: No such file or directory (os error 2)",
            addon_dir.join("Missing.lua").display()
        );
        let xml_warning = if super::uses_wrath_file_diagnostics() {
            "Couldn't open Interface\\AddOns\\MissingFiles\\Missing.xml".to_string()
        } else {
            format!(
                "{}: IO error: No such file or directory (os error 2)",
                addon_dir.join("Missing.xml").display()
            )
        };
        assert_eq!(result.warnings, [lua_warning, xml_warning]);
    }
}
