use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn world_map_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_WorldMap")
}

fn world_map_toc() -> PathBuf {
    world_map_dir().join("Blizzard_WorldMap_Mainline.toc")
}

const REQUIRED_DEPS: &[&str] = &[
    "Blizzard_MapCanvas",
    "Blizzard_SharedMapDataProviders",
    "Blizzard_HelpPlate",
];

const MAINLINE_BODY_FILES: &[&str] = &[
    "QuestLogOwnerMixin.lua",
    "MapTexturePreloader.lua",
    "WM_WorldQuestDataProvider.xml",
    "WM_EventOverlayDataProvider.xml",
    "WM_DebugDataProvider.xml",
    "Blizzard_WorldMapTemplates.xml",
    "Blizzard_WorldMap.xml",
];

const PRIMARY_MIXINS: &[&str] = &[
    "WorldMapMixin",
    "WorldMapTutorialMixin",
    "QuestLogOwnerMixin",
    "WorldMap_WorldQuestDataProviderMixin",
    "WorldMap_WorldQuestPinMixin",
    "WorldMap_EventOverlayDataProviderMixin",
    "WorldMap_EventOverlayPinMixin",
];

const GM_CLIENT_GATED_MIXINS: &[&str] = &[
    "WorldMap_DebugDataProviderMixin",
    "WorldMap_DebugObjectPinMixin",
    "WorldMap_DebugPortLocPinMixin",
];

const TEMPLATE_MIXINS: &[&str] = &[
    "WorldMapFloorNavigationFrameMixin",
    "WorldMapTrackingOptionsFilterCounterMixin",
    "WorldMapTrackingOptionsButtonMixin",
    "WorldMapTrackingPinButtonMixin",
    "WorldMapNavBarMixin",
    "WorldMapNavBarButtonMixin",
    "WorldMapSidePanelToggleMixin",
    "WorldMapZoneTimerMixin",
    "WorldMapThreatFrameMixin",
    "WorldMapThreatEyeMixin",
];

const VIRTUAL_TEMPLATES: &[&str] = &[
    "WorldMapFrameTemplate",
    "WorldMap_WorldQuestPinTemplate",
    "WorldMapInvasionOverlayPinTemplate",
    "WorldMapThreatOverlayPinTemplate",
    "WorldMap_DebugObjectPinTemplate",
    "WorldMap_DebugPortLocPinTemplate",
    "WorldMapFloorNavigationFrameTemplate",
    "WorldMapTrackingOptionsButtonTemplate",
    "WorldMapTrackingPinButtonTemplate",
    "WorldMapNavBarTemplate",
    "WorldMapSidePanelToggleTemplate",
    "WorldMapZoneTimerTemplate",
    "WorldMapThreatFrameTemplate",
];

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

// Literal cached inputs absent from Wrath manifest1-382/cache; behavior cases stay enabled.
post_wrath_ui_fixture! {
#[test]
fn find_toc_file_resolves_mainline_variant() {
    let resolved = find_toc_file(&world_map_dir()).expect("Blizzard_WorldMap TOC should resolve");
    assert_eq!(
        resolved,
        world_map_toc(),
        "Blizzard_WorldMap ships only `Blizzard_WorldMap_Mainline.toc` — find_toc_file probes \
         the `_Mainline.toc` variant first and hits. The classic-flavor map UI ships its own \
         standalone Blizzard_WorldMap_ClassicEra/_TBC/_Wrath/etc. addons rather than sharing this \
         one"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_declares_three_required_deps_via_singular_directive() {
    let toc = TocFile::from_file(&world_map_toc()).expect("Blizzard_WorldMap TOC should parse");

    let deps: Vec<String> = toc.dependencies().to_vec();
    assert_eq!(
        deps,
        REQUIRED_DEPS
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
        "Blizzard_WorldMap declares `## RequiredDep: Blizzard_MapCanvas, Blizzard_SharedMapDataProviders, \
         Blizzard_HelpPlate` (singular `RequiredDep` with a comma-list of 3 deps). FIRST campaign \
         addon analyzed exercising the multi-dep singular RequiredDep variant — toc.rs:210-217 \
         resolves comma-split values from any of `RequiredDep` / `Dependencies` / `RequiredDeps` \
         keys through the same accessor, so this 3-dep list collapses to the standard \
         dependencies() vector"
    );

    assert!(!toc.is_load_on_demand());
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_omits_allow_load_directives() {
    let toc = TocFile::from_file(&world_map_toc()).expect("Blizzard_WorldMap TOC should parse");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Without `## AllowLoad`, allows_screen falls through to the default Game-only branch"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Without `## AllowLoad`, allows_screen rejects glue screen {screen:?}"
        );
    }

    assert!(
        !toc.is_game_type_restricted(),
        "Without a top-level `## AllowLoadGameType`, is_game_type_restricted returns false. The \
         per-file `[AllowLoadGameType plunderstorm]` annotations on lines 10-11 of the TOC body \
         only gate THOSE TWO files (the WoWLabs/WM_WoWLabsAreaDataProvider.lua + .xml plunderstorm \
         overlay) — they do NOT restrict the addon itself; the rest of the addon ships on every \
         retail flavor"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_body_excludes_plunderstorm_files_on_mainline_build() {
    let toc = TocFile::from_file(&world_map_toc()).expect("Blizzard_WorldMap TOC should parse");

    let body_files: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();

    assert_eq!(
        body_files,
        MAINLINE_BODY_FILES
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
        "TOC body must collapse to 7 files on mainline — the 2 lines tagged \
         `[AllowLoadGameType plunderstorm]` (WoWLabs/WM_WoWLabsAreaDataProvider.lua + .xml) are \
         dropped at toc.rs:141 by push_file_entry's is_allowed_game_type guard since plunderstorm \
         is not in {{mainline, standard}}. FIRST campaign addon analyzed exercising the per-file \
         game-type gate (distinct from Blizzard_WorldLootObjectList's TOP-LEVEL \
         `## AllowLoadGameType: plunderstorm`, which restricts the entire addon)"
    );

    for excluded_file in [
        "WoWLabs/WM_WoWLabsAreaDataProvider.lua",
        "WoWLabs/WM_WoWLabsAreaDataProvider.xml",
        "WoWLabs\\WM_WoWLabsAreaDataProvider.lua",
        "WoWLabs\\WM_WoWLabsAreaDataProvider.xml",
    ] {
        assert!(
            !body_files.iter().any(|f| f == excluded_file),
            "TOC body must NOT include `{excluded_file}` on a mainline build"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_raw_bytes_pin_directives_and_per_file_gates() {
    let raw = std::fs::read_to_string(world_map_toc()).expect("Blizzard_WorldMap TOC should read");

    for directive in [
        "## Title: Blizzard_WorldMap",
        "## Author: Blizzard Entertainment",
        "## RequiredDep: Blizzard_MapCanvas, Blizzard_SharedMapDataProviders, Blizzard_HelpPlate",
    ] {
        assert!(
            raw.contains(directive),
            "TOC must contain directive line `{directive}`"
        );
    }

    assert!(
        raw.contains("WoWLabs/WM_WoWLabsAreaDataProvider.lua [AllowLoadGameType plunderstorm]"),
        "TOC raw bytes must pin the per-file `[AllowLoadGameType plunderstorm]` annotation on \
         the WoWLabs lua file at line 10 — push_file_entry strips this annotation before \
         pushing to the files vec, so the parsed body won't show it; the raw assertion preserves \
         evidence that the inline gate exists"
    );
    assert!(
        raw.contains("WoWLabs/WM_WoWLabsAreaDataProvider.xml [AllowLoadGameType plunderstorm]"),
        "TOC raw bytes must pin the per-file gate on the matching WoWLabs xml at line 11"
    );

    for absent_directive in [
        "## Version:",
        "## Notes:",
        "## DefaultState:",
        "## Dependencies:",
        "## RequiredDeps:",
        "## OptionalDeps:",
        "## SavedVariables:",
        "## LoadOnDemand:",
        "## LoadFirst:",
        "## AllowLoad:",
        "## AllowLoadGameType:",
    ] {
        assert!(
            !raw.contains(absent_directive),
            "TOC must NOT contain `{absent_directive}` — Blizzard_WorldMap declares only \
             Title + Author + RequiredDep at the top level"
        );
    }
}
}

#[test]
fn dep_directories_exist_on_disk() {
    if cfg!(feature = "client-wrath") {
        let toc = crate::common::native_framexml_toc();
        for file in ["UIPanelTemplates.xml", "WorldMapFrame.xml"] {
            assert!(
                toc.files
                    .iter()
                    .any(|path| path == std::path::Path::new(file)),
                "native dependency absent from committed FrameXML order: {file}"
            );
            assert!(
                toc.addon_dir.join(file).is_file(),
                "native dependency missing on disk: {file}"
            );
        }
        return;
    }
    for dep in REQUIRED_DEPS {
        let dir = blizzard_ui_dir().join(dep);
        assert!(
            dir.is_dir(),
            "Required dep directory `{dep}` must exist on disk at `{}`",
            dir.display()
        );
        let toc = find_toc_file(&dir);
        assert!(
            toc.is_some(),
            "Required dep `{dep}` must have a discoverable TOC"
        );
    }
}

post_wrath_ui_fixture! {
#[test]
fn appears_in_game_eager_discovery() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let found = addons.iter().any(|(name, _)| name == "Blizzard_WorldMap");
    assert!(
        found,
        "Blizzard_WorldMap must appear in Game eager discovery — eager (no LoadOnDemand), \
         not game-type-restricted at the top level, AllowLoad defaults to Game"
    );
}
}

#[test]
fn absent_from_glue_screen_auto_discovery() {
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons.iter().any(|(name, _)| name == "Blizzard_WorldMap");
        assert!(
            !found,
            "Blizzard_WorldMap must NOT appear in {screen:?} auto-discovery (default Game-only)"
        );
    }
}

prefork_full_ui_case! {
fn loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_WorldMap")
                || message.contains("WorldMapFrame")
                || message.contains("WorldMapMixin")
                || message.contains("QuestLogOwnerMixin")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_WorldMap emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_after_eager_pass(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_WorldMap')")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_WorldMap') must return true after the eager Game pass"
    );

    for dep in REQUIRED_DEPS {
        let dep_loaded: bool = env
            .eval(&format!("return C_AddOns.IsAddOnLoaded('{dep}')"))
            .unwrap_or_else(|err| panic!("dep {dep} probe failed: {err}"));
        assert!(
            dep_loaded,
            "Required dep `{dep}` must also be loaded — eager discovery resolves the dep chain"
        );
    }
}
}

prefork_full_ui_case! {
fn primary_mixins_publish_at_globals(env: &WowLuaEnv) {

    for mixin_name in PRIMARY_MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin_name})"))
            .unwrap_or_else(|err| panic!("{mixin_name} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin_name} must publish at `_G` as a table — file-scope mixin declaration in the \
             corresponding lua file (Blizzard_WorldMap.lua / QuestLogOwnerMixin.lua / \
             WM_WorldQuestDataProvider.lua / WM_EventOverlayDataProvider.lua)"
        );
    }
}
}

prefork_full_ui_case! {
fn debug_data_provider_mixins_remain_gated_off_for_non_gm_clients(env: &WowLuaEnv) {

    for mixin_name in GM_CLIENT_GATED_MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin_name})"))
            .unwrap_or_else(|err| panic!("{mixin_name} probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "{mixin_name} must remain unpublished — WM_DebugDataProvider.lua early-returns when \
             `IsGMClient()` is false (line 1-3), so the file-scope mixin assignments at lines 5, \
             131, and 154 are skipped. The simulator's `IsGMClient` always returns false."
        );
    }
}
}

prefork_full_ui_case! {
fn template_mixins_publish_from_world_map_templates_lua(env: &WowLuaEnv) {

    for mixin_name in TEMPLATE_MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin_name})"))
            .unwrap_or_else(|err| panic!("{mixin_name} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin_name} must publish at `_G` as a table — declared in \
             Blizzard_WorldMapTemplates.lua and consumed by the corresponding XML \
             `mixin=\"{mixin_name}\"` attribute. The lua is loaded via \
             `<Script file=\"Blizzard_WorldMapTemplates.lua\"/>` at line 3 of \
             Blizzard_WorldMapTemplates.xml — the lua is NOT in the TOC body itself"
        );
    }
}
}

prefork_full_ui_case! {
fn xml_templates_register_in_template_registry(env: &WowLuaEnv) {
    let _ = env;

    for template_name in VIRTUAL_TEMPLATES {
        assert!(
            wow_ui_sim::xml::get_template(template_name).is_some(),
            "{template_name} (`<Frame virtual=\"true\">` from one of the WorldMap XMLs) must be \
             registered in the template registry after the addon loads"
        );
    }
}
}

prefork_full_ui_case! {
fn world_map_frame_publishes_with_dual_mixin_and_parent_uiparent(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(WorldMapFrame)")
        .expect("WorldMapFrame probe should succeed");
    assert_eq!(
        kind, "table",
        "WorldMapFrame must publish at `_G` as a table — declared at \
         Blizzard_WorldMap.xml:28 with `parent=\"UIParent\"` `inherits=\"WorldMapFrameTemplate\"` \
         and `frameBuffer=\"true\"`. The template at xml:5 carries `mixin=\"QuestLogOwnerMixin, \
         WorldMapMixin\"` (comma-list of TWO mixins — both copy onto the frame). \
         frameBuffer=\"true\" puts the frame on its own render target so the map's heavy \
         per-frame redraw doesn't dirty the rest of the UI"
    );

    let name: String = env
        .eval("return WorldMapFrame:GetName()")
        .expect("WorldMapFrame:GetName() probe should succeed");
    assert_eq!(name, "WorldMapFrame");
}
}

prefork_full_ui_case! {
fn world_map_frame_carries_blackout_and_border_children(env: &WowLuaEnv) {

    let blackout_kind: String = env
        .eval("return type(WorldMapFrame.BlackoutFrame)")
        .expect("WorldMapFrame.BlackoutFrame probe should succeed");
    assert_eq!(
        blackout_kind, "table",
        "WorldMapFrame.BlackoutFrame parentKey child must publish — declared at \
         Blizzard_WorldMap.xml:30 with frameStrata=\"LOW\" and a BACKGROUND-layer Blackout \
         texture (rgba=0,0,0,1) anchored to UIParent. This is the fullscreen black overlay \
         shown when the world map is maximized"
    );

    let border_kind: String = env
        .eval("return type(WorldMapFrame.BorderFrame)")
        .expect("WorldMapFrame.BorderFrame probe should succeed");
    assert_eq!(
        border_kind, "table",
        "WorldMapFrame.BorderFrame parentKey child must publish — declared at xml:43 inheriting \
         `PortraitFrameTemplateMinimizable` with `frameStrata=\"HIGH\"` and `setAllPoints=\"true\"`. \
         Owns the Tutorial button (MainHelpPlateButton mixin chain), the MaximizeMinimizeFrame \
         (anchored RIGHT to its CloseButton's LEFT), and the InsetBorderTop texture inheriting \
         `_UI-Frame-InnerTopTile`"
    );
}
}

prefork_full_ui_case! {
fn world_map_frame_template_carries_scroll_container(env: &WowLuaEnv) {

    let scroll_kind: String = env
        .eval("return type(WorldMapFrame.ScrollContainer)")
        .expect("WorldMapFrame.ScrollContainer probe should succeed");
    assert_eq!(
        scroll_kind, "table",
        "WorldMapFrame.ScrollContainer parentKey child must publish — declared on the template \
         at xml:12 inheriting `MapCanvasFrameScrollContainerTemplate` (provided by the \
         Blizzard_MapCanvas dep at the top of the dep chain). This is the frame that hosts the \
         actual map canvas: pan/zoom/pin-rendering all happen inside the ScrollContainer's \
         content frame"
    );
}
}

#[test]
fn world_map_tracking_filters_default_to_visible() {
    let env = load_full_game_ui();
    env.exec(r#"
        local button = CreateFromMixins(WorldMapTrackingOptionsButtonMixin)
        button:BuildFilterTable()
        for _, name in ipairs({'trivialQuests', 'showAccountCompletedQuests'}) do
            assert(button:GetWorldMapFilter(name):Get() == true)
        end
        assert(not pcall(C_Minimap.IsFilteredOut, {}))
    "#).expect("WorldMapFilterMixin should read default tracking visibility");
}
