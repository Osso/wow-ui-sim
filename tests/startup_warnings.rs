//! Test that loading Blizzard addons and firing startup events produces no warnings.

use crate::common;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use wow_ui_sim::loader::{discover_all_blizzard_addons, discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

#[path = "startup_warnings/diagnostics.rs"]
mod diagnostics;

use diagnostics::{
    StartupDiagnostics, collect_addon_load_diagnostics, collect_handler_and_runtime_diagnostics,
};

const STARTUP_WARNING_GAME_FOUNDATIONS: &[&str] = &[
    "Blizzard_SharedXMLBase",
    "Blizzard_Menu",
    "Blizzard_SharedXML",
    "Blizzard_SharedXMLGame",
    "Blizzard_FrameXMLBase",
    "Blizzard_UIPanelTemplates",
    "Blizzard_FrameXMLUtil",
    "Blizzard_FrameXML",
    "Blizzard_UIParent",
    "Blizzard_UIParentPanelManager",
];

const STARTUP_WARNING_GLUE_FOUNDATIONS: &[&str] = &[
    "Blizzard_GlueXMLBase",
    "Blizzard_GlueParent",
    "Blizzard_GlueMenuFrame",
    "Blizzard_GlueXML",
];

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

/// Install a Lua error handler that collects errors into `__test_errors`.
fn install_test_error_handler(env: &WowLuaEnv) {
    common::install_error_collector(env, "__test_errors");
}

/// Read collected errors from `__test_errors` and clear it.
fn drain_test_errors(env: &WowLuaEnv) -> Vec<String> {
    common::drain_string_table(env, "__test_errors")
}

fn fire(env: &WowLuaEnv, event: &str, args: &[rilua::Val]) -> Vec<String> {
    env.fire_event_with_args(event, args).ok();
    drain_test_errors(env)
}

/// Load all Blizzard addons and fire startup events, collecting failures and diagnostics.
fn load_and_startup() -> StartupDiagnostics {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);

    install_test_error_handler(&env);

    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons(&ui);
    let mut diagnostics = StartupDiagnostics::default();
    for (name, toc_path) in &addons {
        diagnostics.extend(collect_addon_load_diagnostics(&env, name, toc_path));
    }

    env.apply_post_load_workarounds();
    diagnostics.extend(collect_handler_and_runtime_diagnostics(
        &env,
        "post-load workarounds",
    ));

    wow_ui_sim::startup::fire_startup_events_headless(&env);
    diagnostics.extend(collect_handler_and_runtime_diagnostics(
        &env,
        "startup events",
    ));

    if let Err(error) = env.fire_on_update(0.016) {
        diagnostics
            .warnings
            .push(format!("[OnUpdate] FAILED: {error}"));
    }
    diagnostics.extend(collect_handler_and_runtime_diagnostics(&env, "OnUpdate"));
    diagnostics
}

#[test]
fn test_no_warnings_on_startup() {
    test_timeout! {
        let diagnostics = load_and_startup();
        let account_store_regressions: Vec<String> = diagnostics
            .warnings
            .iter()
            .filter(|warning| {
                (warning.contains("attempt to index global 'AccountStoreFrame'")
                    && warning.contains("UIParent.lua:352"))
                    || warning.contains("AccountStoreFrame:SetStoreFrontID")
            })
            .cloned()
            .collect();

        assert!(
            account_store_regressions.is_empty(),
            "Regression: AccountStore startup nil-path error reintroduced.\n\
             Matching warnings:\n  {}",
            account_store_regressions.join("\n  ")
        );
        assert!(
            diagnostics
                .nil_symbol_observations
                .iter()
                .all(|observation| !observation.attribution.addon_name.is_empty()),
            "nil observations must retain addon attribution: {:?}",
            diagnostics.nil_symbol_observations
        );
        assert!(
            diagnostics
                .missing_requirements
                .iter()
                .all(|requirement| !requirement.attribution.addon_name.is_empty()),
            "missing requirements must retain addon attribution: {:?}",
            diagnostics.missing_requirements
        );
        assert!(
            diagnostics.warnings.is_empty(),
            "Startup produced loader/runtime failures:\n  {}\n\
             Nil observations retained: {}\n\
             Missing requirements retained: {}",
            diagnostics.warnings.join("\n  "),
            diagnostics.nil_symbol_observations.len(),
            diagnostics.missing_requirements.len()
        );
    }
}

#[test]
fn test_edit_mode_layout_update_ignores_preset_layouts_during_startup() {
    test_timeout! {
        let env = load_all_addons();
        install_test_error_handler(&env);

        env.fire_edit_mode_layouts_updated().ok();
        let warnings = drain_test_errors(&env);
        let edit_mode_layout_warnings: Vec<String> = warnings
            .iter()
            .filter(|warning| {
                warning.contains("Blizzard_EditMode/Shared/EditModeManager.lua:917")
                    || warning.contains("UpdateLayoutCounts")
                    || warning.contains("attempt to perform arithmetic on field '?'")
            })
            .cloned()
            .collect();

        assert!(
            edit_mode_layout_warnings.is_empty(),
            "Edit mode layout update should ignore preset layouts when counting saved layouts:\n  {}",
            edit_mode_layout_warnings.join("\n  ")
        );
    }
}

/// Load all Blizzard addons and apply workarounds (no startup events).
fn load_all_addons() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);

    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons(&ui);
    for (name, toc_path) in &addons {
        if let Err(e) = load_addon(&env.loader_env(), toc_path) {
            eprintln!("[load {name}] FAILED: {e}");
        }
    }
    env.apply_post_load_workarounds();
    env
}

fn load_single_blizzard_addon(addon_name: &str) -> (WowLuaEnv, Vec<String>) {
    load_blizzard_addon_with_login_env(addon_name)
}

fn load_blizzard_addon_by_folder(folder_name: &str) -> (WowLuaEnv, Vec<String>) {
    load_blizzard_addon_with_login_env(folder_name)
}

fn load_blizzard_addon_with_login_env(addon_name: &str) -> (WowLuaEnv, Vec<String>) {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Login);

    let addons = addon_map_all();
    let warnings = load_blizzard_addon_with_foundations(&env, &addons, addon_name);
    (env, warnings)
}

fn addon_map_all() -> HashMap<String, PathBuf> {
    let ui = blizzard_ui_dir();
    discover_all_blizzard_addons(&ui).into_iter().collect()
}

fn load_blizzard_addon_with_foundations(
    env: &WowLuaEnv,
    addons: &HashMap<String, PathBuf>,
    addon_name: &str,
) -> Vec<String> {
    let mut loading = HashSet::new();
    let mut loaded = HashSet::new();
    let mut warnings = Vec::new();
    load_blizzard_addon_recursive(
        env,
        addons,
        addon_name,
        true,
        &mut loading,
        &mut loaded,
        &mut warnings,
    );
    warnings
}

fn load_blizzard_addon_recursive(
    env: &WowLuaEnv,
    addons: &HashMap<String, PathBuf>,
    addon_name: &str,
    required: bool,
    loading: &mut HashSet<String>,
    loaded: &mut HashSet<String>,
    warnings: &mut Vec<String>,
) {
    if already_loading_or_loaded(addon_name, loading, loaded) {
        return;
    }
    mark_addon_loading(addon_name, loading);

    let Some(toc_path) = addons.get(addon_name) else {
        if required {
            panic!("addon {addon_name} should exist");
        }
        abandon_addon_load(addon_name, loading);
        return;
    };
    let toc = parse_blizzard_addon_toc(addon_name, toc_path);
    load_startup_warning_foundations(env, addons, addon_name, loading, loaded, warnings);
    load_toc_dependencies(env, addons, addon_name, &toc, loading, loaded, warnings);
    warnings.extend(load_blizzard_addon_warnings(env, addon_name, toc_path));

    finish_addon_load(addon_name, loading, loaded);
}

fn already_loading_or_loaded(
    addon_name: &str,
    loading: &HashSet<String>,
    loaded: &HashSet<String>,
) -> bool {
    loaded.contains(addon_name) || loading.contains(addon_name)
}

fn mark_addon_loading(addon_name: &str, loading: &mut HashSet<String>) {
    loading.insert(addon_name.to_string());
}

fn abandon_addon_load(addon_name: &str, loading: &mut HashSet<String>) {
    loading.remove(addon_name);
}

fn finish_addon_load(
    addon_name: &str,
    loading: &mut HashSet<String>,
    loaded: &mut HashSet<String>,
) {
    abandon_addon_load(addon_name, loading);
    loaded.insert(addon_name.to_string());
}

fn load_blizzard_addon_warnings(env: &WowLuaEnv, addon_name: &str, toc_path: &Path) -> Vec<String> {
    let result = load_addon(&env.loader_env(), toc_path)
        .unwrap_or_else(|error| panic!("{addon_name} should load: {error}"));

    result
        .warnings
        .into_iter()
        .map(|warning| format!("[load {addon_name}] {warning}"))
        .collect()
}

fn parse_blizzard_addon_toc(addon_name: &str, toc_path: &Path) -> TocFile {
    TocFile::from_file(toc_path)
        .unwrap_or_else(|error| panic!("{addon_name} TOC should parse: {error}"))
}

fn load_startup_warning_foundations(
    env: &WowLuaEnv,
    addons: &HashMap<String, PathBuf>,
    addon_name: &str,
    loading: &mut HashSet<String>,
    loaded: &mut HashSet<String>,
    warnings: &mut Vec<String>,
) {
    for foundation in startup_warning_foundations(addon_name) {
        if foundation != addon_name && addons.contains_key(foundation) {
            load_blizzard_addon_recursive(
                env, addons, foundation, false, loading, loaded, warnings,
            );
        }
    }
}

fn load_toc_dependencies(
    env: &WowLuaEnv,
    addons: &HashMap<String, PathBuf>,
    addon_name: &str,
    toc: &TocFile,
    loading: &mut HashSet<String>,
    loaded: &mut HashSet<String>,
    warnings: &mut Vec<String>,
) {
    for dependency in toc_dependency_names(toc, addons) {
        if dependency != addon_name {
            load_blizzard_addon_recursive(
                env,
                addons,
                &dependency,
                false,
                loading,
                loaded,
                warnings,
            );
        }
    }
}

fn startup_warning_foundations(addon_name: &str) -> Vec<&'static str> {
    let foundations = if addon_name.starts_with("Blizzard_Glue") {
        STARTUP_WARNING_GLUE_FOUNDATIONS
    } else {
        STARTUP_WARNING_GAME_FOUNDATIONS
    };
    let end = foundations
        .iter()
        .position(|candidate| *candidate == addon_name)
        .unwrap_or(foundations.len());
    foundations[..end].to_vec()
}

fn toc_dependency_names(toc: &TocFile, addons: &HashMap<String, PathBuf>) -> Vec<String> {
    let mut dependencies = toc.dependencies();
    let mut seen: HashSet<String> = dependencies.iter().cloned().collect();
    for dependency in toc.optional_deps() {
        if addons.contains_key(&dependency) && seen.insert(dependency.clone()) {
            dependencies.push(dependency);
        }
    }
    dependencies
}

/// Assert that a Lua expression evaluates to true.
fn assert_lua(env: &WowLuaEnv, code: &str, msg: &str) {
    assert!(env.eval::<bool>(code).unwrap_or(false), "{msg}");
}

/// Fire startup events and process timers.
fn fire_events_and_timers(env: &WowLuaEnv) {
    common::fire_addon_loaded(env, "WoWUISim");
    for event in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        let _ = env.fire_event(event);
    }
    common::fire_player_entering_world(env, true, false);
    let _ = env.process_timers();
    let _ = env.fire_on_update(0.016);
    let _ = env.process_timers();
}

/// Audit: every `[LoadIntoEnvironment secure]` annotation in Blizzard TOCs
/// must match this explicit list.
///
/// Fails if a new annotation is introduced (we want to know about it
/// deliberately, not silently), or if an existing annotation disappears
/// (so we can track Blizzard's secure-env footprint across patches).
#[test]
fn test_secure_env_toc_annotations_are_exhaustive() {
    let ui = blizzard_ui_dir();
    let toc_paths = wow_ui_sim::loader::discover_all_blizzard_addons(&ui)
        .into_iter()
        .map(|(_, path)| path)
        .collect::<Vec<_>>();

    let mut secure_files: Vec<String> = Vec::new();
    for toc_path in &toc_paths {
        let Ok(toc) = wow_ui_sim::toc::TocFile::from_file(toc_path) else {
            continue;
        };
        for (index, file) in toc.files.iter().enumerate() {
            if toc.file_use_secure_env(index) == Some(true) {
                let addon = toc_path
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("?");
                secure_files.push(format!("{addon}/{}", file.display()));
            }
        }
    }
    secure_files.sort();

    let expected = vec![
        "Blizzard_ChatFrameBase/Shared/ChatFrameFiltersSecure.lua".to_string(),
        "Blizzard_CooldownViewer/CooldownViewerSecure.lua".to_string(),
        "Blizzard_RestrictedAddOnEnvironment/RestrictedEnvironment.lua".to_string(),
        "Blizzard_UnitFrame/Shared/TargetFrameAuraButton.lua".to_string(),
        "Blizzard_UnitFrame/Shared/TargetFrameAuraContainer.lua".to_string(),
    ];
    assert_eq!(
        secure_files, expected,
        "the set of [LoadIntoEnvironment secure] files drifted; update this test \
         only after confirming the new/removed entries actually run under secureenv"
    );
}

/// Every annotated secure file should load without warnings and its
/// downstream surface should be reachable — a smoke test that our per-file
/// TOC secureenv dispatch is wired end-to-end.
#[test]
fn test_secure_env_annotated_files_load_cleanly() {
    test_timeout! {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        env.set_screen_size(1024.0, 768.0);

        let ui = blizzard_ui_dir();
        let addons = discover_blizzard_addons(&ui);
        let mut warnings: Vec<String> = Vec::new();
        for (name, toc_path) in &addons {
            let result = load_addon(&env.loader_env(), toc_path)
                .unwrap_or_else(|e| panic!("{name} should load: {e}"));
            warnings.extend(result.warnings);
            // Stop once both secure-annotated addons have loaded.
            if name == "Blizzard_ChatFrameBase" || name == "Blizzard_RestrictedAddOnEnvironment" {
                continue;
            }
        }

        let noisy_secure = warnings
            .iter()
            .filter(|w| {
                [
                    "ChatFrameFiltersSecure.lua",
                    "CooldownViewerSecure.lua",
                    "RestrictedEnvironment.lua",
                    "TargetFrameAuraButton.lua",
                    "TargetFrameAuraContainer.lua",
                ]
                .iter()
                .any(|file| w.contains(file))
            })
            .cloned()
            .collect::<Vec<_>>();
        assert!(
            noisy_secure.is_empty(),
            "secure-annotated files should load cleanly, got:\n{}",
            noisy_secure.join("\n")
        );

        let resolved_secure_publications = warnings
            .iter()
            .filter(|warning| {
                [
                    "StoreButton_OnShow",
                    "StoreGoldButton_OnShow",
                    "StoreEditBoxWithAutoCompleteTemplate_OnTextChanged",
                    "VASCharacterSelection_ClearStoreTooltip",
                    "WoWTokenButton_OnShow",
                ]
                .iter()
                .any(|name| warning.contains(name))
            })
            .cloned()
            .collect::<Vec<_>>();
        assert!(
            resolved_secure_publications.is_empty(),
            "secure same-addon functions published by addon completion must not remain missing:\n{}",
            resolved_secure_publications.join("\n")
        );

        // Downstream surfaces exposed by secure files.
        let (
            restricted_scope_ty,
            secure_filters_delegate_ty,
            cooldown_aura_map_ty,
            cooldown_aura_map_value,
            target_aura_container_ty,
            aura_container_set_max_buffs_ty,
            aura_button_icon_ty,
            public_buff_aura_group_ty,
            public_debuff_aura_group_ty,
            forbidden_buff_aura_group_ty,
            forbidden_debuff_aura_group_ty,
        ): (
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
        ) = env
            .eval(
                r#"
                -- RESTRICTED_FUNCTIONS_SCOPE lands on the addon table, not _G,
                -- so we probe the scope via a known descendent global that
                -- Blizzard_RestrictedAddOnEnvironment registers once loaded.
                local auraMap = EssentialCooldownViewer.auraInstanceIDToItemFramesMap
                auraMap[42] = "secure-map-value"
                local targetAuras = TargetFrame.TargetFrameContent
                    .TargetFrameContentContextual.Auras
                local forbiddenTargetAuras = GetForbiddenObjectTable(targetAuras)
                local auraButton = CreateFrame(
                    "AuraButton", nil, UIParent, "TargetFrameAuraButtonTemplate"
                )
                return type(CallRestrictedClosure),
                       type(SecureTypes and SecureTypes.CreateSecureArray),
                       type(auraMap),
                       type(auraMap[42]),
                       type(targetAuras),
                       type(targetAuras.SetMaxBuffs),
                       type(auraButton.GetIcon),
                       type(targetAuras.buffAuraGroup),
                       type(targetAuras.debuffAuraGroup),
                       type(forbiddenTargetAuras.buffAuraGroup),
                       type(forbiddenTargetAuras.debuffAuraGroup)
                "#,
            )
            .expect("secure surface should be introspectable");
        assert_eq!(restricted_scope_ty, "function");
        assert_eq!(secure_filters_delegate_ty, "function");
        assert_eq!(cooldown_aura_map_ty, "table");
        assert_eq!(cooldown_aura_map_value, "string");
        assert_eq!(target_aura_container_ty, "table");
        assert_eq!(aura_container_set_max_buffs_ty, "function");
        assert_eq!(aura_button_icon_ty, "function");
        assert_eq!(public_buff_aura_group_ty, "nil");
        assert_eq!(public_debuff_aura_group_ty, "nil");
        assert_eq!(forbidden_buff_aura_group_ty, "table");
        assert_eq!(forbidden_debuff_aura_group_ty, "table");
    }
}

#[test]
fn test_restricted_addon_environment_exposes_execution_surface() {
    test_timeout! {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        env.set_screen_size(1024.0, 768.0);

        let ui = blizzard_ui_dir();
        let addons = discover_blizzard_addons(&ui);
        let mut restricted_result = None;
        for (name, toc_path) in addons {
            let result = load_addon(&env.loader_env(), &toc_path)
                .unwrap_or_else(|error| panic!("{name} should load before restricted env: {error}"));
            if name == "Blizzard_RestrictedAddOnEnvironment" {
                restricted_result = Some(result);
                break;
            }
        }
        let result = restricted_result.expect("Blizzard_RestrictedAddOnEnvironment should be in addon order");

        let warnings = result.warnings.join("\n");
        assert!(
            !warnings.contains("RestrictedExecution.lua"),
            "RestrictedExecution.lua should load cleanly:\n{warnings}"
        );
        assert!(
            !warnings.contains("SecureHoverDriver.lua"),
            "SecureHoverDriver.lua should load cleanly:\n{warnings}"
        );

        let (
            get_frame_mt_ty,
            get_frame_mt_index_ty,
            call_fn_ty,
            propagate_fn_ty,
        ): (String, String, String, String) = env
            .eval(
                r#"
                local frameMt = GetFrameMetatable()
                return type(frameMt),
                    type(frameMt and frameMt.__index),
                    type(CallRestrictedClosure),
                    type(PropagateForbiddenToReferencedFrames)
                "#,
            )
            .expect("restricted surface inspection should be callable");

        assert_eq!(get_frame_mt_ty, "table");
        assert_eq!(get_frame_mt_index_ty, "table");
        assert_eq!(call_fn_ty, "function");
        assert_eq!(propagate_fn_ty, "function");
    }
}

/// Verify that template mixin inheritance is properly applied.
///
/// ObjectiveTrackerUIWidgetContainer inherits UIWidgetContainerTemplate,
/// which inherits UIWidgetContainerNoResizeTemplate (mixin UIWidgetContainerMixin).
/// GetNumWidgetsShowing must be available on the frame.
#[test]
fn test_widget_container_mixin_applied() {
    test_timeout! {
        let env = load_all_addons();

        assert_lua(&env, "return type(UIWidgetContainerMixin) == 'table'",
            "UIWidgetContainerMixin should exist as a Lua table");
        assert_lua(&env, "return type(UIWidgetContainerMixin.GetNumWidgetsShowing) == 'function'",
            "UIWidgetContainerMixin should have GetNumWidgetsShowing");
        assert_lua(&env, "return ObjectiveTrackerUIWidgetContainer ~= nil",
            "ObjectiveTrackerUIWidgetContainer should exist");
        assert_lua(&env, "return type(ObjectiveTrackerUIWidgetContainer.GetNumWidgetsShowing) == 'function'",
            "ObjectiveTrackerUIWidgetContainer should have GetNumWidgetsShowing from UIWidgetContainerMixin");

        let result: i64 = env
            .eval("return ObjectiveTrackerUIWidgetContainer:GetNumWidgetsShowing()")
            .expect("GetNumWidgetsShowing() should not error");
        assert_eq!(result, 0, "No widgets should be showing initially");

        // Verify the method survives startup events and timer processing
        fire_events_and_timers(&env);
        assert_lua(&env, "return type(ObjectiveTrackerUIWidgetContainer.GetNumWidgetsShowing) == 'function'",
            "GetNumWidgetsShowing should still be available after startup events and timer processing");
    }
}

#[test]
fn test_load_addon_with_error_handling_loads_account_store_and_preserves_surface() {
    test_timeout! {
        let env = load_all_addons();

        let (
            ok,
            loaded_after,
            account_store_frame_ty,
            account_store_mixin_ty,
            set_storefront_ty,
            set_fullscreen_ty,
            err,
        ): (bool, bool, String, String, String, String, Option<String>) = env
            .eval(
                r#"
                local ok, err = pcall(function()
                    LoadAddOnWithErrorHandling("Blizzard_AccountStore")
                end)
                return ok,
                    C_AddOns.IsAddOnLoaded("Blizzard_AccountStore"),
                    type(AccountStoreFrame),
                    type(AccountStoreMixin),
                    type(AccountStoreFrame and AccountStoreFrame.SetStoreFrontID),
                    type(AccountStoreFrame and AccountStoreFrame.SetFullscreenMode),
                    ok and nil or tostring(err)
                "#,
            )
            .expect("LoadAddOnWithErrorHandling should be callable");

        assert!(ok, "LoadAddOnWithErrorHandling should not error: {:?}", err);
        assert!(
            loaded_after,
            "LoadAddOnWithErrorHandling should mark Blizzard_AccountStore loaded"
        );
        assert_eq!(
            account_store_frame_ty, "table",
            "AccountStoreFrame should exist after runtime load"
        );
        assert_eq!(
            account_store_mixin_ty, "table",
            "AccountStoreMixin should exist after runtime load"
        );
        assert_eq!(
            set_storefront_ty, "function",
            "AccountStoreFrame should preserve SetStoreFrontID after runtime load"
        );
        assert_eq!(
            set_fullscreen_ty, "function",
            "AccountStoreFrame should preserve SetFullscreenMode after runtime load"
        );
    }
}

#[test]
fn test_account_store_frame_exposes_mixin_methods_after_load() {
    test_timeout! {
        let (env, warnings) = load_blizzard_addon_by_folder("Blizzard_AccountStore");

        let (frame_ty, on_load_ty, set_storefront_ty, set_fullscreen_ty): (
            String,
            String,
            String,
            String,
        ) = env
            .eval(
                r#"
                return type(AccountStoreFrame),
                    type(AccountStoreFrame and AccountStoreFrame.OnLoad),
                    type(AccountStoreFrame and AccountStoreFrame.SetStoreFrontID),
                    type(AccountStoreFrame and AccountStoreFrame.SetFullscreenMode)
                "#,
            )
            .expect("AccountStoreFrame inspection should be callable");

        assert_eq!(frame_ty, "table", "AccountStoreFrame should exist after Blizzard_AccountStore load");
        assert_eq!(
            on_load_ty, "function",
            "AccountStoreFrame.OnLoad should come from AccountStoreMixin; warnings:\n  {}",
            warnings.join("\n  ")
        );
        assert_eq!(set_storefront_ty, "function");
        assert_eq!(set_fullscreen_ty, "function");
    }
}

#[test]
fn test_account_store_set_storefront_id_is_safe_after_load() {
    test_timeout! {
        let (env, warnings) = load_blizzard_addon_by_folder("Blizzard_AccountStore");

        let (ok, stored_id, err): (bool, i64, Option<String>) = env
            .eval(
                r#"
                local ok, err = pcall(function()
                    AccountStoreFrame:SetStoreFrontID(Constants.AccountStoreConsts.PlunderstormStoreFrontID)
                end)
                return ok, AccountStoreFrame.storeFrontID or 0, ok and nil or tostring(err)
                "#,
            )
            .expect("AccountStoreFrame:SetStoreFrontID should be callable");

        assert!(
            ok,
            "AccountStoreFrame:SetStoreFrontID should not error; warnings:\n  {}\nerror: {:?}",
            warnings.join("\n  "),
            err
        );
        assert_eq!(
            stored_id,
            1,
            "AccountStoreFrame:SetStoreFrontID should preserve the storefront id"
        );
    }
}

#[path = "startup_warnings/late.rs"]
mod late;
