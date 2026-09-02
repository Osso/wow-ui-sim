#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn inspect_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_HousingInspectModeUI")
}

fn inspect_toc() -> PathBuf {
    inspect_dir().join("Blizzard_HousingInspectModeUI.toc")
}

fn house_editor_toc() -> PathBuf {
    blizzard_ui_dir()
        .join("Blizzard_HouseEditor")
        .join("Blizzard_HouseEditor.toc")
}

fn load_housing_inspect_mode_ui_with_dependency(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &house_editor_toc())
        .expect("Blizzard_HouseEditor (the only declared dep) should load via explicit LoD call");
    load_addon(&env.loader_env(), &inspect_toc())
        .expect("Blizzard_HousingInspectModeUI should load via explicit Rust loader call");
}

fn assert_mixin_methods(env: &WowLuaEnv, mixin: &str, methods: &[&str], rationale: &str) {
    for method in methods {
        let exists: bool = env
            .eval(&format!("return type({mixin}['{method}']) == 'function'"))
            .unwrap_or_else(|err| panic!("{mixin}.{method} existence query failed: {err}"));
        assert!(exists, "{mixin} must expose `:{method}()` — {rationale}");
    }
}

#[test]
fn blizzard_housing_inspect_mode_ui_find_toc_resolves_bare_variant() {
    let resolved =
        find_toc_file(&inspect_dir()).expect("Blizzard_HousingInspectModeUI TOC should resolve");
    assert_eq!(
        resolved,
        inspect_toc(),
        "Blizzard_HousingInspectModeUI ships exactly one bare TOC — retail-only addon resolves \
         via `find_toc_file` fallthrough"
    );
}

#[test]
fn blizzard_housing_inspect_mode_ui_toc_declares_lod_with_two_dependencies() {
    let toc = TocFile::from_file(&inspect_toc()).expect("InspectModeUI TOC should parse");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_HousingInspectModeUI declares `## LoadOnDemand: 1` — pulled via explicit \
         LoadAddOn from Blizzard_HousingControls/Blizzard_HousingControlButton.lua:163 \
         (HouseInspectorButtonMixin:EnterMode loads this addon then calls \
         C_HousingInspectMode.EnterInspectMode to flip the inspect-mode active flag)"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert_eq!(
        toc.dependencies(),
        vec![
            "Blizzard_HouseEditor".to_string(),
            "Blizzard_GameMenuEsc".to_string(),
        ],
        "Blizzard_HousingInspectModeUI declares Blizzard_HouseEditor and Blizzard_GameMenuEsc \
         in published dependency order."
    );
}

#[test]
fn blizzard_housing_inspect_mode_ui_toc_is_retail_only_and_omits_allow_load() {
    let toc = TocFile::from_file(&inspect_toc()).expect("InspectModeUI TOC should parse");
    let toc_text = std::fs::read_to_string(inspect_toc()).expect("TOC should read");
    assert!(
        toc_text.contains("## AllowLoadGameType: standard"),
        "Declares `## AllowLoadGameType: standard` — retail-only Midnight feature"
    );
    assert!(!toc.is_game_type_restricted());
    assert!(
        !toc_text.contains("## AllowLoad:"),
        "Omits `## AllowLoad:` — LoadOnDemand precludes auto-discovery gating"
    );
    assert!(!toc_text.contains("## DefaultState:"));
    assert!(
        toc.saved_variables().is_empty(),
        "No `## SavedVariables*` — inspect mode is server-driven via \
         HOUSING_INSPECT_MODE_DECOR_HOVERED_CHANGED + HOUSING_INSPECT_MODE_STATE_UPDATED events \
         plus the C_HousingInspectMode API surface; no client-side persistence"
    );
}

#[test]
fn blizzard_housing_inspect_mode_ui_toc_lists_two_files_in_order() {
    let toc = TocFile::from_file(&inspect_toc()).expect("InspectModeUI TOC should parse");
    let files: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(
        files,
        vec![
            "Blizzard_HousingInspectModeUI.lua".to_string(),
            "Blizzard_HousingInspectModeUI.xml".to_string(),
        ],
        "TOC body lists exactly 2 source files in this exact order — the .lua defines \
         HousingInspectModeManagerMixin first, then the .xml's `mixin=HousingInspectModeManagerMixin` \
         attribute on the named non-virtual Button can resolve the mixin reference"
    );
}

#[test]
fn blizzard_housing_inspect_mode_ui_directory_holds_three_entries() {
    let dir = inspect_dir();
    let entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("Blizzard_HousingInspectModeUI directory should exist")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        entries.len(),
        3,
        "Directory ships exactly 3 entries (1 lua + 1 xml + 1 TOC, no flavor subdirectory). Got: \
         {entries:?}"
    );
    assert!(entries.contains(&"Blizzard_HousingInspectModeUI.toc".to_string()));
    assert!(entries.contains(&"Blizzard_HousingInspectModeUI.lua".to_string()));
    assert!(entries.contains(&"Blizzard_HousingInspectModeUI.xml".to_string()));
}

#[test]
fn blizzard_housing_inspect_mode_ui_excluded_from_all_screen_auto_discovery() {
    let ui = blizzard_ui_dir();
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
        ScreenKind::Game,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let discovered = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_HousingInspectModeUI");
        assert!(
            !discovered,
            "Must NOT appear in {screen:?} auto-discovery — `## LoadOnDemand: 1` excludes the \
             addon from every ScreenKind pass; it is only ever pulled via explicit LoadAddOn \
             from the in-house Controls panel's House Inspector button"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_housing_inspect_mode_ui_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_housing_inspect_mode_ui_with_dependency(env);

    let lua_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let related: Vec<&String> = lua_errors
        .iter()
        .filter(|e| {
            e.contains("Blizzard_HousingInspectModeUI/")
                || e.contains("Blizzard_HousingInspectModeUI\\")
                || e.contains("HousingInspectModeManagerMixin")
        })
        .collect();
    assert!(
        related.is_empty(),
        "Loading Blizzard_HousingInspectModeUI must not emit any addon-specific Lua errors. Got \
         {} errors: {:?}",
        related.len(),
        related
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_inspect_mode_ui_is_addon_loaded_via_explicit_lod_call(env: &WowLuaEnv) {
    load_housing_inspect_mode_ui_with_dependency(env);
    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_HousingInspectModeUI')")
        .expect("IsAddOnLoaded query should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_HousingInspectModeUI') must return true after the \
         explicit LoD load — proves the loader registered the addon name"
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_inspect_mode_ui_house_editor_dependency_loads_via_explicit_lod_call(env: &WowLuaEnv) {
    load_housing_inspect_mode_ui_with_dependency(env);
    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_HouseEditor')")
        .expect("HouseEditor IsAddOnLoaded query should succeed");
    assert!(
        loaded,
        "Blizzard_HouseEditor (the only declared dep, also LoadOnDemand) must be loaded before \
         InspectModeUI — both addons are LoD so neither is in the auto-discovery sweep; the \
         test driver explicitly LoDs HouseEditor first to satisfy the dependency"
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_inspect_mode_ui_publishes_manager_frame_global(env: &WowLuaEnv) {
    load_housing_inspect_mode_ui_with_dependency(env);
    let frame_type: String = env
        .eval("return type(HousingInspectModeManagerFrame)")
        .expect("HousingInspectModeManagerFrame type query should succeed");
    assert_eq!(
        frame_type, "table",
        "HousingInspectModeManagerFrame must publish as a `_G` Button — the XML declares the \
         named non-virtual Button at line 3 (mixin HousingInspectModeManagerMixin, parent=UIParent, \
         frameStrata=LOW, frameLevel=1, propagateKeyboardInput=true, propagateMouseInput=Both) \
         which is the addon's sole top-level frame"
    );
    let frame_name: String = env
        .eval("return HousingInspectModeManagerFrame:GetName()")
        .expect("HousingInspectModeManagerFrame:GetName query should succeed");
    assert_eq!(frame_name, "HousingInspectModeManagerFrame");
}
}

prefork_full_ui_case! {
fn blizzard_housing_inspect_mode_ui_manager_mixin_publishes_eighteen_methods(env: &WowLuaEnv) {
    load_housing_inspect_mode_ui_with_dependency(env);
    assert_mixin_methods(
        &env,
        "HousingInspectModeManagerMixin",
        &[
            "OnLoad",
            "OnShow",
            "OnHide",
            "OnEvent",
            "IsInspectModeActive",
            "OnUpdateInspectModeActive",
            "OnInspectModeActivated",
            "OnInspectModeDeactivated",
            "OnClick",
            "OnDecorHoveredChanged",
            "OnNonDecorHovered",
            "OnEnter",
            "OnLeave",
            "OnDecorHoverStarted",
            "OnDecorHoverEnded",
            "OnHouseEditorStateUpdated",
            "OnShopVisibilityUpdated",
            "ExitInspectMode",
        ],
        "HousingInspectModeManagerMixin owns 18 methods on the umbrella inspect-mode manager: \
         OnLoad (FrameUtil.RegisterFrameForEvents for HOUSING_INSPECT_MODE_DECOR_HOVERED_CHANGED \
         + HOUSING_INSPECT_MODE_STATE_UPDATED), OnShow / OnHide (subscribe and unsubscribe \
         EventRegistry callbacks for HouseEditor.StateUpdated / GameMenuFrame.Shown / \
         CatalogShopFrame.VisibilityUpdated), OnEvent (dispatches the 2 frame events), \
         IsInspectModeActive / OnUpdateInspectModeActive (sync the cached \
         self.isInspectModeActive flag against C_HousingInspectMode.IsInInspectMode and trigger \
         the activated/deactivated transitions), OnInspectModeActivated / OnInspectModeDeactivated \
         (TriggerEvent HousingInspectMode.Activated/Deactivated, set fullscreen anchors via \
         SetPoint TOPLEFT + BOTTOMRIGHT or ClearAllPoints, Show/Hide), OnClick (clicks routed to \
         ExitInspectMode if no decor hovered, or LoadAddOn('Blizzard_HousingDashboard') + \
         TriggerEvent HousingCatalogFrame.OpenToDecorID with the hovered decorID), \
         OnDecorHoveredChanged / OnDecorHoverStarted / OnDecorHoverEnded / OnNonDecorHovered \
         (drive the GameTooltip showing decor name + dye slot info via C_DyeColor.GetDyeColorInfo \
         or the no-decor instruction text), OnEnter / OnLeave (delegate hover state to \
         OnNonDecorHovered or hide the tooltip), OnHouseEditorStateUpdated (auto-exit inspect \
         mode whenever HouseEditor activates — they are mutually exclusive), \
         OnShopVisibilityUpdated (auto-exit inspect mode when the catalog shop opens), \
         ExitInspectMode (call C_HousingInspectMode.ExitInspectMode RPC)",
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_inspect_mode_ui_manager_frame_uses_button_widget_type(env: &WowLuaEnv) {
    load_housing_inspect_mode_ui_with_dependency(env);
    let object_type: String = env
        .eval("return HousingInspectModeManagerFrame:GetObjectType()")
        .expect("HousingInspectModeManagerFrame:GetObjectType query should succeed");
    assert_eq!(
        object_type, "Button",
        "HousingInspectModeManagerFrame is declared as `<Button>` (NOT `<Frame>`) — the XML root \
         element is `<Button name=\"HousingInspectModeManagerFrame\" ...>` so it must resolve to \
         a Button widget; this matters because the OnClick handler relies on Button click \
         dispatch (a Frame would not fire OnClick for unconsumed mouse-up events)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_inspect_mode_ui_manager_frame_uses_low_frame_strata(env: &WowLuaEnv) {
    load_housing_inspect_mode_ui_with_dependency(env);
    let strata: String = env
        .eval("return HousingInspectModeManagerFrame:GetFrameStrata()")
        .expect("HousingInspectModeManagerFrame:GetFrameStrata query should succeed");
    assert_eq!(
        strata, "LOW",
        "HousingInspectModeManagerFrame must sit at frameStrata=LOW — the XML pins it there so \
         the fullscreen click-catcher Button (anchored TOPLEFT/BOTTOMRIGHT during inspect mode) \
         sits below every interactive UI panel and only intercepts clicks that pass through \
         everything else (propagateMouseInput=Both lets clicks hit world geometry too)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_inspect_mode_ui_manager_registers_two_frame_events(env: &WowLuaEnv) {
    load_housing_inspect_mode_ui_with_dependency(env);
    for event in [
        "HOUSING_INSPECT_MODE_DECOR_HOVERED_CHANGED",
        "HOUSING_INSPECT_MODE_STATE_UPDATED",
    ] {
        let registered: bool = env
            .eval(&format!(
                "return HousingInspectModeManagerFrame:IsEventRegistered('{event}')"
            ))
            .expect("IsEventRegistered query should succeed");
        assert!(
            registered,
            "HousingInspectModeManagerFrame must register {event} — OnLoad calls \
             FrameUtil.RegisterFrameForEvents on the local HousingInspectModeManagerEvents \
             array (lines 3-7 of the .lua) which contains exactly these 2 events; OnEvent then \
             dispatches them to OnDecorHoveredChanged or OnUpdateInspectModeActive"
        );
    }
}
}
