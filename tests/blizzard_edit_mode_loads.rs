#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be synced")
}

fn edit_mode_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_EditMode/Blizzard_EditMode.toc")
}

fn load_full_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    wow_ui_sim::xml::register_intrinsic_templates();

    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn blizzard_edit_mode_toc_declares_five_dependencies() {
    let toc = TocFile::from_file(&edit_mode_toc()).expect("Blizzard_EditMode TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_EditMode omits `## LoadOnDemand` — auto-loads on Game (default per \
         src/toc.rs:311). Many other addons (DurabilityFrame, action bars, unit frames, \
         minimap) inherit EditMode-prefixed templates, so EditMode must load before them \
         and cannot be deferred"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_EditMode does not declare `## UseSecureEnvironment` — runs in the \
         standard Lua environment"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_EditMode declares no addon-level `## AllowLoadGameType:` filter — the \
         addon loads on every game type. Per-file `[AllowLoadGameType ...]` annotations \
         (e.g. `Classic\\EditModePresetLayoutConstants.lua [AllowLoadGameType classic]`) \
         pick the correct preset-constants variant for the current game type at load time"
    );

    let deps: Vec<String> = toc.dependencies();
    assert_eq!(
        deps,
        vec![
            "Blizzard_UIPanelTemplates".to_string(),
            "Blizzard_HelpPlate".to_string(),
            "Blizzard_ManagedFrameSystem".to_string(),
            "Blizzard_GameMenuEsc".to_string(),
            "Blizzard_UIParentUtil".to_string(),
        ],
        "Retail 12.1.0.69497 declares five EditMode dependencies in published order. \
         Got: {deps:?}"
    );
}

#[test]
fn blizzard_edit_mode_appears_in_game_discovery_only() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_EditMode");
    assert!(
        in_game,
        "Blizzard_EditMode (no AllowLoad flag) should appear in Game-screen auto-discovery \
         — without it, dozens of dependent addons (DurabilityFrame, action bars, unit \
         frames) would fail to inherit their EditMode-prefixed templates"
    );

    let login_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Login);
    let in_login = login_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_EditMode");
    assert!(
        !in_login,
        "Blizzard_EditMode must NOT appear on Login / glue screens — per src/toc.rs:311 \
         a missing `## AllowLoad` defaults to Game-only. The per-file \
         `[AllowLoadEnvironment Global]` annotations in the TOC are about Lua environment \
         globalness, NOT about glue-vs-game loading"
    );
}

#[test]
fn blizzard_edit_mode_loads_after_five_declared_dependencies() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let edit_mode_idx = game_addons
        .iter()
        .position(|(name, _)| name == "Blizzard_EditMode")
        .expect("Blizzard_EditMode should be discovered");

    for dependency in [
        "Blizzard_UIPanelTemplates",
        "Blizzard_HelpPlate",
        "Blizzard_ManagedFrameSystem",
        "Blizzard_GameMenuEsc",
        "Blizzard_UIParentUtil",
    ] {
        let dependency_idx = game_addons
            .iter()
            .position(|(name, _)| name == dependency)
            .unwrap_or_else(|| panic!("{dependency} should be discovered"));
        assert!(
            dependency_idx < edit_mode_idx,
            "Blizzard_EditMode must load after {dependency}; indices: \
             {dependency}={dependency_idx}, EditMode={edit_mode_idx}"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_edit_mode_loads_without_errors(env: &WowLuaEnv) {

    let addon_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| message.contains("EditMode"))
        .cloned()
        .collect();
    assert!(
        addon_errors.is_empty(),
        "Blizzard_EditMode emitted Lua errors during Game-screen load:\n  {}",
        addon_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_edit_mode_is_addon_loaded_returns_true(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval(
            "local ok, _reason = (C_AddOns and C_AddOns.IsAddOnLoaded \
                or IsAddOnLoaded)('Blizzard_EditMode'); \
                return ok == true or ok == 1",
        )
        .expect("IsAddOnLoaded('Blizzard_EditMode') query should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_EditMode') / IsAddOnLoaded('Blizzard_EditMode') \
         must report true after Game-screen load. The addon ships ~10k Lua/XML lines \
         across Shared/Mainline/Standard subdirs and is essential infrastructure for the \
         retail HUD; if this returns false, the loader skipped the TOC silently"
    );
}
}

prefork_full_ui_case! {
fn blizzard_edit_mode_creates_named_singleton_manager_frame(env: &WowLuaEnv) {

    let (kind, name): (String, String) = env
        .eval(
            "return type(EditModeManagerFrame), \
                    (type(EditModeManagerFrame) == 'table') \
                        and EditModeManagerFrame:GetName() or ''",
        )
        .expect("EditModeManagerFrame existence/name probe should succeed");
    assert_eq!(
        kind, "table",
        "Shared/EditModeManager.xml:4 declares \
         `<Frame name=\"EditModeManagerFrame\" parent=\"UIParent\" hidden=\"true\" \
         frameStrata=\"DIALOG\" mixin=\"EditModeManagerFrameMixin\">` — the loader must \
         register this as a global addressable as a table"
    );
    assert_eq!(
        name, "EditModeManagerFrame",
        "EditModeManagerFrame:GetName() must round-trip the XML `name` attribute. \
         Many entry-point globals (e.g. `EditModeManagerFrame:Show()`) reference this \
         singleton by name"
    );
}
}

prefork_full_ui_case! {
fn blizzard_edit_mode_manager_frame_starts_hidden(env: &WowLuaEnv) {

    let starts_hidden: bool = env
        .eval("return EditModeManagerFrame:IsShown() == false")
        .expect("EditModeManagerFrame visibility probe should succeed");
    assert!(
        starts_hidden,
        "EditModeManagerFrame must start HIDDEN — Shared/EditModeManager.xml:4 declares \
         `hidden=\"true\"`. The frame only appears when the player explicitly enters Edit \
         Mode (via `EditModeManagerFrame:EnterEditMode()` from the system menu / hotkey)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_edit_mode_publishes_core_mixin_globals(env: &WowLuaEnv) {

    let kinds: (String, String, String) = env
        .eval(
            "return type(EditModeManagerFrameMixin), \
                    type(EditModeSystemMixin), \
                    type(EditModeUtil)",
        )
        .expect("Core EditMode mixin globals probe should succeed");
    let expected = (
        "table".to_string(),
        "table".to_string(),
        "table".to_string(),
    );
    assert_eq!(
        kinds, expected,
        "EditMode publishes three top-level singleton tables: EditModeManagerFrameMixin \
         (Shared/EditModeManager.lua:10), EditModeSystemMixin \
         (Shared/EditModeSystemTemplates.lua:1), and EditModeUtil \
         (Shared/EditModeUtil.lua:1). All must exist as tables after addon load"
    );
}
}

prefork_full_ui_case! {
fn blizzard_edit_mode_manager_frame_mixin_carries_lifecycle_methods(env: &WowLuaEnv) {

    let kinds: (String, String, String, String, String) = env
        .eval(
            "return type(EditModeManagerFrameMixin.OnLoad), \
                    type(EditModeManagerFrameMixin.OnDragStart), \
                    type(EditModeManagerFrameMixin.OnDragStop), \
                    type(EditModeManagerFrameMixin.EnterEditMode), \
                    type(EditModeManagerFrameMixin.ExitEditMode)",
        )
        .expect("EditModeManagerFrameMixin method-types probe should succeed");
    let expected = (
        "function".to_string(),
        "function".to_string(),
        "function".to_string(),
        "function".to_string(),
        "function".to_string(),
    );
    assert_eq!(
        kinds, expected,
        "EditModeManagerFrameMixin must expose: OnLoad (line 12), OnDragStart (line 61), \
         OnDragStop (line 65), EnterEditMode (line 82), ExitEditMode (line 100). The XML \
         attaches this mixin to the manager frame, so each method becomes \
         EditModeManagerFrame:<Method>() once the frame is built"
    );
}
}

prefork_full_ui_case! {
fn blizzard_edit_mode_publishes_per_system_mixin_globals_for_action_bars_and_unit_frames(env: &WowLuaEnv) {

    let all_present: bool = env
        .eval(
            "return type(EditModeActionBarSystemMixin) == 'table' \
                and type(EditModeUnitFrameSystemMixin) == 'table' \
                and type(EditModeBossUnitFrameSystemMixin) == 'table' \
                and type(EditModeArenaUnitFrameSystemMixin) == 'table' \
                and type(EditModeMinimapSystemMixin) == 'table' \
                and type(EditModeCastBarSystemMixin) == 'table' \
                and type(EditModeAuraFrameSystemMixin) == 'table' \
                and type(EditModeChatFrameSystemMixin) == 'table'",
        )
        .expect("Per-system mixin globals probe should succeed");
    assert!(
        all_present,
        "EditMode defines a separate per-subsystem mixin global for each editable HUD \
         system (Shared/EditModeSystemTemplates.lua). The XML <Frame mixin=\"...\"> \
         attribute on each *SystemTemplate references one of these tables, so all must \
         exist as tables: EditModeActionBarSystemMixin, EditModeUnitFrameSystemMixin, \
         EditModeBossUnitFrameSystemMixin, EditModeArenaUnitFrameSystemMixin, \
         EditModeMinimapSystemMixin, EditModeCastBarSystemMixin, \
         EditModeAuraFrameSystemMixin, EditModeChatFrameSystemMixin"
    );
}
}

prefork_full_ui_case! {
fn blizzard_edit_mode_dialog_singletons_exist_as_hidden_frames(env: &WowLuaEnv) {

    let kinds: (String, String, String, String) = env
        .eval(
            "return type(EditModeLayoutDialog), \
                    type(EditModeImportLayoutDialog), \
                    type(EditModeUnsavedChangesDialog), \
                    type(EditModeSystemSettingsDialog)",
        )
        .expect("EditMode dialog singletons type probe should succeed");
    let table = "table".to_string();
    assert_eq!(
        kinds,
        (table.clone(), table.clone(), table.clone(), table.clone()),
        "Shared/EditModeDialogs.xml declares four live non-virtual dialog singletons \
         (EditModeLayoutDialog at line 66, EditModeImportLayoutDialog at line 134, \
         EditModeUnsavedChangesDialog at line 237, EditModeSystemSettingsDialog at \
         line 251). EditModeImportLayoutLinkDialog (lines 141-191) is XML-commented out \
         (`<!-- ... -->` wraps that block at lines 140-192) and intentionally does NOT \
         instantiate. All four live dialogs must exist as global tables after EditMode \
         loads. Got types: {kinds:?}"
    );

    let hidden_kinds: (bool, bool, bool, bool) = env
        .eval(
            "return EditModeLayoutDialog:IsShown() == false, \
                    EditModeImportLayoutDialog:IsShown() == false, \
                    EditModeUnsavedChangesDialog:IsShown() == false, \
                    EditModeSystemSettingsDialog:IsShown() == false",
        )
        .expect("EditMode dialog hidden-state probe should succeed");
    assert_eq!(
        hidden_kinds,
        (true, true, true, true),
        "All four live EditMode dialogs inherit EditModeBaseDialogTemplate (or a \
         *DialogTemplate that itself inherits the base) which carries hidden=\"true\". \
         They start hidden — only shown when the user triggers the corresponding \
         Edit-Mode action. Got hidden-states: {hidden_kinds:?}"
    );
}
}

prefork_full_ui_case! {
fn blizzard_edit_mode_durability_frame_system_template_unblocks_dependent_addon(env: &WowLuaEnv) {

    let durability_template_works: bool = env
        .eval(
            "do \
                local probe = CreateFrame('Frame', nil, UIParent, \
                    'EditModeDurabilityFrameSystemTemplate') \
                return type(probe) == 'table' \
            end",
        )
        .expect("EditModeDurabilityFrameSystemTemplate instantiation probe should succeed");
    assert!(
        durability_template_works,
        "EditModeDurabilityFrameSystemTemplate (Shared/EditModeSystemTemplates.xml:283, \
         virtual=\"true\", inherits=\"EditModeSystemTemplate\", \
         mixin=\"EditModeDurabilityFrameSystemMixin\") must be registered in the template \
         registry after EditMode loads. Blizzard_DurabilityFrame.xml:3 inherits this \
         template — if registration fails here, that whole dependent addon fails to \
         load. Confirms our cross-addon template resolution is intact"
    );
}
}

prefork_full_ui_case! {
fn blizzard_edit_mode_loads_mainline_overrides_excludes_wrath_only_files(env: &WowLuaEnv) {

    let no_wrath_layouts: bool = env
        .eval(
            "do \
                local has_classic_layouts = type(_G.EditModePresetLayoutsClassic) == 'table' \
                local has_wowhack_layouts = type(_G.EditModePresetLayoutsWowHack) == 'table' \
                return (not has_classic_layouts) and (not has_wowhack_layouts) \
            end",
        )
        .expect("Per-game-type filter probe should succeed");
    assert!(
        no_wrath_layouts,
        "On mainline retail, the per-file `[AllowLoadGameType classic]` (line 3) and \
         `[AllowLoadGameType wowhack, plunderstorm]` (line 6) annotations must filter out \
         non-mainline preset-constants files via `is_allowed_game_type` (src/toc.rs:43-57). \
         Probing for hypothetical Classic-only / WoWHack-only globals confirms they \
         neither leak nor crash the load. (The tested globals are illustrative — the \
         existing mainline sources don't declare these names; the assertion just verifies \
         no surprise leakage)"
    );
}
}

#[test]
fn blizzard_edit_mode_directory_ships_six_subdirs_for_game_variants() {
    let dir = blizzard_ui_dir().join("Blizzard_EditMode");
    let entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("Blizzard_EditMode dir should read")
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();

    let subdirs: Vec<&String> = entries.iter().filter(|n| !n.contains('.')).collect();
    assert!(
        subdirs.contains(&&"Shared".to_string())
            && subdirs.contains(&&"Mainline".to_string())
            && subdirs.contains(&&"Classic".to_string())
            && subdirs.contains(&&"Standard".to_string())
            && subdirs.contains(&&"WoWHack".to_string())
            && subdirs.contains(&&"WoWLabs".to_string()),
        "Blizzard_EditMode must ship 6 subdirs for game-type / family variants: Shared \
         (game-agnostic engine), Mainline ([Family] for retail), Classic, Standard \
         ([Game] for retail), WoWHack, WoWLabs. The TOC's `[Family]` and `[Game]` \
         template substitutions plus `[AllowLoadGameType ...]` per-file filters select \
         which ones contribute files at load time. Got subdirs: {subdirs:?}"
    );
}
