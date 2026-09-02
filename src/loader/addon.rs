//! Addon loading internals.

mod nil_symbol_reports;

use crate::lua_api::LoaderEnv;
use crate::lua_api::state::SimState;
use crate::saved_variables::SavedVariablesManager;
use crate::toc::TocFile;
use rilua::Val;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use super::addon_saved_variables::{
    maybe_init_saved_variables, maybe_restore_clobbered_saved_variables,
};
use super::error::LoadError;
use super::lua_file::load_lua_file;
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
}

#[derive(Clone, Copy)]
enum EnvironmentPass {
    Normal,
    SecureReplay,
}

impl EnvironmentPass {
    fn use_secure_env(self, ctx: &AddonContext<'_>) -> bool {
        match self {
            Self::Normal => ctx.use_secure_env,
            Self::SecureReplay => true,
        }
    }

    fn loads_xml(self) -> bool {
        matches!(self, Self::Normal)
    }
}

impl LoadingAddonGuard {
    pub(crate) fn addon_index(&self) -> u16 {
        self.addon_idx
    }

    pub(crate) fn commit_loaded(&self) {
        if !self.entered {
            return;
        }

        let mut state = self.state.borrow_mut();
        if let Some(addon) = state.addons.get_mut(self.addon_idx as usize) {
            addon.loaded = true;
            addon.bootstrap_loaded = true;
        }
    }
}

impl Drop for LoadingAddonGuard {
    fn drop(&mut self) {
        if !self.entered {
            return;
        }

        let mut state = self.state.borrow_mut();
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

/// Internal addon loading with optional saved variables.
pub fn load_addon_internal(
    env: &LoaderEnv<'_>,
    toc: &TocFile,
    saved_vars_mgr: Option<&mut SavedVariablesManager>,
) -> Result<LoadResult, LoadError> {
    let folder_name = toc
        .addon_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&toc.name);

    let mut result = LoadResult {
        name: toc.name.clone(),
        lua_files: 0,
        xml_files: 0,
        timing: LoadTiming::default(),
        warnings: Vec::new(),
        nil_symbol_observations: Vec::new(),
        missing_requirements: Vec::new(),
    };
    let already_loaded = env
        .state()
        .borrow()
        .addons
        .iter()
        .any(|addon| addon.folder_name == folder_name && addon.loaded);
    if already_loaded {
        return Ok(result);
    }

    let saved_vars_mgr =
        maybe_init_saved_variables(env, toc, folder_name, saved_vars_mgr, &mut result);
    let loading_guard = begin_addon_load(env, folder_name, toc);
    let ctx = build_addon_context(env, toc, folder_name)?;
    let nil_symbol_access_start = env.state().borrow().nil_symbol_accesses.len();
    let addon_name = result.name.clone();

    load_addon_files(
        env,
        toc,
        folder_name,
        &ctx,
        EnvironmentPass::Normal,
        &mut result,
    );
    load_shared_xml_list_templates(env, toc, folder_name, &ctx, &mut result);
    maybe_replay_blizzard_lua_in_secure_env(env, toc, folder_name, &ctx, &mut result);
    maybe_restore_clobbered_saved_variables(env, folder_name, saved_vars_mgr);
    apply_blizzard_post_load_patches(env, folder_name, &mut result);
    let addon_index = loading_guard.addon_index();
    append_nil_symbol_diagnostics(
        env,
        addon_index,
        &addon_name,
        nil_symbol_access_start,
        &mut result,
    );
    append_pending_nested_addon_diagnostics(env, addon_index, &mut result);
    loading_guard.commit_loaded();
    Ok(result)
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

/// Load `ListTemplates.lua` / `ListTemplates.xml` for `Blizzard_SharedXML` when
/// the TOC that was picked does not list them.
///
/// `find_toc_file` prefers `Blizzard_SharedXML_Mainline.toc`, the legacy flavor
/// variant, and that one omits the pair the bare `Blizzard_SharedXML.toc` lists
/// at lines 233-234. Nothing else defines `ListHeaderMixin`: without the file,
/// `QuestMapFrame.lua:417` raises when the world map opens and ten files that
/// inherit `ListHeaderThreeSliceTemplate` render bare headers. Flipping the
/// whole addon to the bare TOC was measured to abort the Tutorial subsystem
/// (30 -> 70 startup errors), so only the missing pair is appended, in the
/// bare TOC's order, after the picked TOC's files.
fn load_shared_xml_list_templates(
    env: &LoaderEnv<'_>,
    toc: &TocFile,
    folder_name: &str,
    ctx: &AddonContext<'_>,
    result: &mut LoadResult,
) {
    if folder_name != "Blizzard_SharedXML" {
        return;
    }
    let listed = toc.files.iter().any(|f| {
        f.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.eq_ignore_ascii_case("ListTemplates.lua"))
    });
    if listed {
        return;
    }
    for name in ["ListTemplates.lua", "ListTemplates.xml"] {
        let file = toc.addon_dir.join(name);
        if !file.is_file() {
            result
                .warnings
                .push(format!("{folder_name}: {name} missing next to the TOC"));
            continue;
        }
        load_addon_file(env, ctx, result, &file, EnvironmentPass::Normal);
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
        "Blizzard_ActionBar" => patch_action_bar_button_event_fanout(env),
        "Blizzard_PlayerSpells" => patch_playerspells_onload_backfill(env, result),
        "Blizzard_Dispatcher" => {
            crate::lua_api::workarounds::patch_dispatcher_surface_for_addon_load(env)
        }
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

fn patch_action_bar_button_event_fanout(env: &LoaderEnv<'_>) {
    crate::lua_api::workarounds::patch_action_bar_button_event_fanout_for_addon_load(env);
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
) -> LoadingAddonGuard {
    let addon_idx = resolve_addon_index(env, folder_name);
    let mut state = env.state().borrow_mut();
    let entered = !state.loading_addon_stack.contains(&addon_idx);
    if entered {
        update_addon_metadata(&mut state, addon_idx, folder_name, toc);
        state.loading_addon_stack.push(addon_idx);
        state.loading_addon_index = Some(addon_idx);
    }

    LoadingAddonGuard {
        state: Rc::clone(env.state()),
        addon_idx,
        entered,
    }
}

fn update_addon_metadata(state: &mut SimState, addon_idx: u16, folder_name: &str, toc: &TocFile) {
    let Some(addon) = state.addons.get_mut(addon_idx as usize) else {
        return;
    };

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

/// Load all Lua/XML files listed in the TOC, applying local overlay paths.
fn load_addon_files(
    env: &LoaderEnv<'_>,
    toc: &TocFile,
    folder_name: &str,
    ctx: &AddonContext,
    pass: EnvironmentPass,
    result: &mut LoadResult,
) {
    let overlay_dir = Path::new("Interface/AddOns").join(folder_name);

    for (index, (file_rel, file)) in toc.files.iter().zip(toc.file_paths()).enumerate() {
        if should_skip_addon_file(toc, file_rel) {
            continue;
        }
        if matches!(pass, EnvironmentPass::SecureReplay) && toc.file_use_secure_env(index).is_some()
        {
            continue;
        }
        if !toc.file_allows_environment(index, pass.use_secure_env(ctx)) {
            continue;
        }
        let resolved_file = resolve_addon_file_path(&overlay_dir, file_rel, file);
        let file_ctx = AddonContext {
            name: ctx.name,
            table: ctx.table,
            addon_root: ctx.addon_root,
            use_secure_env: toc
                .file_use_secure_env(index)
                .unwrap_or_else(|| pass.use_secure_env(ctx)),
            taint: ctx.taint,
        };
        if std::env::var("WOW_SIM_VERBOSE").is_ok() {
            println!("  loading file {}", file_rel.display());
        }
        load_addon_file(env, &file_ctx, result, &resolved_file, pass);
    }
}

fn maybe_replay_blizzard_lua_in_secure_env(
    env: &LoaderEnv<'_>,
    toc: &TocFile,
    folder_name: &str,
    ctx: &AddonContext,
    result: &mut LoadResult,
) {
    if !is_secure_replay_library_addon(folder_name) || ctx.use_secure_env {
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

pub(super) fn is_secure_replay_library_addon(folder_name: &str) -> bool {
    matches!(
        folder_name,
        "Blizzard_SharedXMLBase"
            | "Blizzard_SharedXML"
            | "Blizzard_FrameXMLUtil"
            | "Blizzard_CombatLogBase"
            | "Blizzard_CatalogShopSharedTemplates"
            | "Blizzard_CatalogShopSharedUtil"
            | "Blizzard_AsyncRequest"
            | "Blizzard_GameTooltip"
    )
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

fn resolve_addon_file_path(
    overlay_dir: &Path,
    file_rel: &Path,
    default_file: std::path::PathBuf,
) -> std::path::PathBuf {
    let overlay_file = overlay_dir.join(file_rel);
    if overlay_file.exists() {
        return overlay_file;
    }
    default_file
}

fn load_addon_file(
    env: &LoaderEnv<'_>,
    ctx: &AddonContext<'_>,
    result: &mut LoadResult,
    file: &std::path::Path,
    pass: EnvironmentPass,
) {
    let _nil_symbol_environment_guard = enter_nil_symbol_environment(env, ctx.use_secure_env);
    if !file.is_file() {
        result.warnings.push(format!(
            "{}: IO error: No such file or directory (os error 2)",
            file.display()
        ));
        return;
    }

    match file.extension().and_then(|ext| ext.to_str()).unwrap_or("") {
        "lua" => load_addon_lua_file(env, ctx, result, file),
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
) {
    match load_lua_file(env, file, ctx, &mut result.timing) {
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
    let load_result = load_xml_file(env, file, ctx, &mut result.timing);
    restore_secure_xml_frame_exports(env, restore_add_to_secure_env);

    match load_result {
        Ok(count) => {
            result.xml_files += 1;
            result.lua_files += count;
        }
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
        assert!(
            result
                .warnings
                .iter()
                .all(|warning| warning.contains("No such file or directory")),
            "missing TOC files should remain loader warnings: {:?}",
            result.warnings
        );
    }
}
