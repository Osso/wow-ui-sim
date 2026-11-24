#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

const FLIGHT_MAP_LABEL_ANCHORS: &str = r#"
    local provider
    for candidate in pairs(FlightMapFrame.dataProviders) do
        if candidate.MapLabel then
            provider = candidate
            break
        end
    end
    assert(provider, "FlightMap must register ClickToZoomDataProviderMixin")
    local openedMapID = FlightMapFrame:GetMapID()
    FlightMapFrame:SetMapID(14)
    local bottomLeft = select(1, provider.MapLabel:GetPoint())
    FlightMapFrame:SetMapID(994)
    local topLeft = select(1, provider.MapLabel:GetPoint())
    return C_AddOns.IsAddOnLoaded("Blizzard_FlightMap"),
           FlightMapFrame:IsShown(), GetTaxiMapID(), openedMapID,
           bottomLeft, topLeft
"#;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn flight_map_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_FlightMap/Blizzard_FlightMap.toc")
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
fn blizzard_flight_map_toc_is_load_on_demand_with_two_required_deps() {
    let toc = TocFile::from_file(&flight_map_toc()).expect("Blizzard_FlightMap TOC should parse");

    assert!(
        toc.is_load_on_demand(),
        "Blizzard_FlightMap declares `## LoadOnDemand: 1` — the flight map UI is brought \
         in on-demand when the player interacts with a flight master (TAXIMAP_OPENED \
         opens the panel via UIPanelWindows registration), so it must NOT auto-load on \
         standard Game-screen bring-up"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_FlightMap does not declare `## UseSecureEnvironment` — the flight map \
         is a standard map UI that runs in the standard taint environment"
    );
    assert_eq!(
        toc.dependencies(),
        vec![
            "Blizzard_MapCanvas".to_string(),
            "Blizzard_SharedMapDataProviders".to_string(),
        ],
        "Blizzard_FlightMap declares `## RequiredDep: Blizzard_MapCanvas, \
         Blizzard_SharedMapDataProviders` — `RequiredDep` is the same as `Dependencies` \
         per `TocFile::dependencies()` (src/toc.rs:209-214). Order matters: MapCanvas \
         must load first to publish `MapCanvasMixin` / `MapCanvasFrameTemplate` / \
         `MapCanvasDataProviderMixin` / `MapCanvasPinMixin`, then SharedMapDataProviders \
         publishes the standard data-provider mixins (AreaPOIDataProviderMixin / \
         VignettePinMixin / GroupMembersDataProviderMixin / etc.) that FlightMapMixin \
         layers its FlightMap_* providers on top of"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_FlightMap declares no `## AllowLoadGameType:` line, so \
         `is_game_type_restricted()` returns false and the addon is reachable from \
         standard-retail discovery (just gated behind `LoadOnDemand`)"
    );

    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_FlightMap declares no `## SavedVariables` — flight map UI state is \
         transient (the panel is registered with UIPanelWindows but its open/closed state \
         is driven by TAXIMAP_OPENED/TAXIMAP_CLOSED events, not persisted)"
    );
}

#[test]
fn blizzard_flight_map_allows_only_game_screen_by_default() {
    let toc = TocFile::from_file(&flight_map_toc()).expect("Blizzard_FlightMap TOC should parse");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Missing `## AllowLoad` defaults to Game-only (src/toc.rs:311) — the flight map \
         needs the in-game environment (GetTaxiMapID, C_Map.GetPlayerMapPosition, taxi \
         events)"
    );
    assert!(
        !toc.allows_screen(ScreenKind::Login),
        "Missing `## AllowLoad` excludes Login (src/toc.rs:311) — there is no taxi map \
         on the login screen"
    );
}

#[test]
fn blizzard_flight_map_is_absent_from_auto_discovery() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_FlightMap");
    assert!(
        !in_game,
        "Blizzard_FlightMap is `## LoadOnDemand: 1`, so it must NOT appear in Game-screen \
         auto-discovery — it is loaded explicitly when the player opens a flight master \
         (TAXIMAP_OPENED triggers `LoadAddOn('Blizzard_FlightMap')` from the TaxiFrame \
         dispatcher in the FrameXML core)"
    );
}

prefork_full_ui_case! {
fn blizzard_flight_map_loads_via_load_addon_without_errors(env: &WowLuaEnv) {

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_addon(&env.loader_env(), &flight_map_toc())
        .expect("Blizzard_FlightMap should load via Rust loader");

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("FlightMap")
                || message.contains("Blizzard_FlightMap")
                || message.contains("FM_")
        })
        .cloned()
        .collect();

    assert!(
        load_errors.is_empty(),
        "Blizzard_FlightMap emitted Lua errors during explicit load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_flight_map_is_addon_loaded_returns_true_after_explicit_load(env: &WowLuaEnv) {

    let pre_load: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_FlightMap') and true or false")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        !pre_load,
        "Before the explicit load, IsAddOnLoaded('Blizzard_FlightMap') must return false \
         — confirms Game-screen auto-discovery did not load this LoadOnDemand addon"
    );

    load_addon(&env.loader_env(), &flight_map_toc())
        .expect("Blizzard_FlightMap should load via Rust loader");

    let post_load: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_FlightMap') and true or false")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        post_load,
        "After explicit `load_addon`, IsAddOnLoaded('Blizzard_FlightMap') must return \
         true — `mark_addon_loaded` (src/loader/addon.rs:131) registers the folder name \
         in the loaded-set"
    );
}
}

prefork_full_ui_case! {
fn blizzard_flight_map_singleton_publishes_with_correct_parent(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &flight_map_toc())
        .expect("Blizzard_FlightMap should load via Rust loader");

    let probe: (String, String) = env
        .eval("return FlightMapFrame:GetName(), FlightMapFrame:GetParent():GetName()")
        .expect("FlightMapFrame name+parent probe should succeed");
    assert_eq!(
        probe,
        ("FlightMapFrame".to_string(), "UIParent".to_string()),
        "The non-virtual `<Frame name=\"FlightMapFrame\" parent=\"UIParent\" \
         inherits=\"MapCanvasFrameTemplate\" mixin=\"FlightMapMixin\">` (xml:5) must \
         publish as a global table whose `:GetName()` is 'FlightMapFrame' and \
         `:GetParent()` is UIParent — confirms the MapCanvasFrameTemplate intrinsic \
         resolves and the singleton parents under UIParent"
    );
}
}

prefork_full_ui_case! {
fn blizzard_flight_map_publishes_eight_data_provider_mixin_globals(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &flight_map_toc())
        .expect("Blizzard_FlightMap should load via Rust loader");

    let provider_mixins_present: (bool, bool, bool, bool, bool, bool, bool, bool) = env
        .eval(
            "return type(FlightMapMixin) == 'table', \
                    type(FlightMap_AreaPOIProviderMixin) == 'table', \
                    type(FlightMap_QuestDataProviderMixin) == 'table', \
                    type(FlightMap_VignetteDataProviderMixin) == 'table', \
                    type(FlightMap_WorldQuestDataProviderMixin) == 'table', \
                    type(FlightMap_ZoneSummaryDataProvider) == 'table', \
                    type(FlightMap_FlightPathDataProviderMixin) == 'table', \
                    type(FlightMap_FlightPointPinMixin) == 'table'",
        )
        .expect("data-provider mixin probe should succeed");
    assert_eq!(
        provider_mixins_present,
        (true, true, true, true, true, true, true, true),
        "Blizzard_FlightMap publishes eight data-provider mixin globals across its seven \
         Lua files: `FlightMapMixin` (Blizzard_FlightMap.lua:3 — the panel driver), \
         `FlightMap_AreaPOIProviderMixin` (FM_AreaPOIDataProvider.lua:1, extends \
         AreaPOIDataProviderMixin from SharedMapDataProviders), \
         `FlightMap_QuestDataProviderMixin` (FM_QuestDataProvider.lua:1, extends \
         QuestDataProviderMixin), `FlightMap_VignetteDataProviderMixin` \
         (FM_VignetteDataProvider.lua:1, extends VignetteDataProviderMixin), \
         `FlightMap_WorldQuestDataProviderMixin` (FM_WorldQuestDataProvider.lua:1, \
         extends WorldQuestDataProviderMixin), `FlightMap_ZoneSummaryDataProvider` \
         (FM_ZoneSummaryDataProvider.lua:1, extends MapCanvasDataProviderMixin — note \
         this one drops the `Mixin` suffix), `FlightMap_FlightPathDataProviderMixin` \
         (FM_FlightPathDataProvider.lua:1, extends MapCanvasDataProviderMixin — the \
         flight-path edge renderer), and `FlightMap_FlightPointPinMixin` \
         (FM_FlightPathDataProvider.lua:2, extends MapCanvasPinMixin — the per-airport \
         icon pin)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_flight_map_publishes_four_pin_mixin_globals(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &flight_map_toc())
        .expect("Blizzard_FlightMap should load via Rust loader");

    let pin_mixins_present: (bool, bool, bool, bool) = env
        .eval(
            "return type(FlightMap_AreaPOIPinMixin) == 'table', \
                    type(FlightMap_QuestPinMixin) == 'table', \
                    type(FlightMap_VignettePinMixin) == 'table', \
                    type(FlightMap_WorldQuestPinMixin) == 'table'",
        )
        .expect("pin-mixin probe should succeed");
    assert_eq!(
        pin_mixins_present,
        (true, true, true, true),
        "Each FlightMap-namespaced data provider also publishes a per-pin renderer \
         mixin: `FlightMap_AreaPOIPinMixin` (FM_AreaPOIDataProvider.lua:2, extends \
         AreaPOIPinMixin), `FlightMap_QuestPinMixin` (FM_QuestDataProvider.lua:2, plain \
         empty mixin — provider creates pins via standard QuestDataProviderMixin code \
         path), `FlightMap_VignettePinMixin` (FM_VignetteDataProvider.lua:2, extends \
         VignettePinMixin), and `FlightMap_WorldQuestPinMixin` \
         (FM_WorldQuestDataProvider.lua:2, extends WorldQuestPinMixin)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_flight_map_mixin_inherits_map_canvas_methods(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &flight_map_toc())
        .expect("Blizzard_FlightMap should load via Rust loader");

    let methods_present: (bool, bool, bool, bool) = env
        .eval(
            "return type(FlightMapMixin.OnLoad) == 'function', \
                    type(FlightMapMixin.OnShow) == 'function', \
                    type(FlightMapMixin.OnHide) == 'function', \
                    type(FlightMapMixin.AddStandardDataProviders) == 'function'",
        )
        .expect("FlightMapMixin method probe should succeed");
    assert_eq!(
        methods_present,
        (true, true, true, true),
        "FlightMapMixin (lua:3) defines four script-callable methods (OnLoad/OnShow/\
         OnHide wired by Blizzard_FlightMap.xml:39-43, plus AddStandardDataProviders \
         called from OnLoad to register the 11 standard data providers \
         FlightMap_ZoneSummaryDataProvider / FlightPath / Quest / ClickToZoom / \
         ZoneLabel / AreaPOI / Vignette / QuestSession / MapHighlight / DelveEntrance / \
         GroupMembers + WorldQuest)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_flight_map_registers_with_ui_panel_windows(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &flight_map_toc())
        .expect("Blizzard_FlightMap should load via Rust loader");

    let panel_registration: (String, f64, f64) = env
        .eval(
            "local entry = UIPanelWindows['FlightMapFrame'] \
             return entry.area, entry.pushable, entry.allowOtherPanels",
        )
        .expect("UIPanelWindows probe should succeed");
    assert_eq!(
        panel_registration,
        ("center".to_string(), 1.0, 1.0),
        "Blizzard_FlightMap.lua:1 registers `UIPanelWindows[\"FlightMapFrame\"] = \
         {{area=\"center\", pushable=1, showFailedFunc=CloseTaxiMap, \
         allowOtherPanels=1}}` — the entry tells the central panel manager to put the \
         flight map in the center slot, that other panels can push it aside (pushable=1), \
         and that other panels are allowed to be shown alongside it. The \
         `showFailedFunc=CloseTaxiMap` callback runs if ShowUIPanel refuses to display \
         the panel (e.g. during cinematics) so the server-side taxi state is released"
    );
}
}

prefork_full_ui_case! {
fn blizzard_flight_map_singleton_starts_hidden_after_load(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &flight_map_toc())
        .expect("Blizzard_FlightMap should load via Rust loader");

    let is_shown: bool = env
        .eval("return FlightMapFrame:IsShown()")
        .expect("FlightMapFrame:IsShown() probe should succeed");
    assert!(
        !is_shown,
        "FlightMapFrame must start hidden after load — the parent template \
         MapCanvasFrameTemplate (Blizzard_MapCanvas.xml:52) declares `hidden=\"true\"`. \
         The panel is summoned by ShowUIPanel(FlightMapFrame) when TAXIMAP_OPENED fires \
         from the FrameXML core taxi handler"
    );
}
}

prefork_full_ui_case! {
fn flight_map_open_path_places_click_to_zoom_labels_from_uimap_positions(env: &WowLuaEnv) {
    assert_flight_map_is_unloaded_and_taxi_map_id_is_zero_values(env);
    open_flight_map_for_taxi_map(env, 14);
    let observed = observe_flight_map_label_anchors(env);
    assert_eq!(
        observed,
        (true, true, 14, 14, "BOTTOMLEFT".into(), "TOPLEFT".into())
    );
}

fn assert_flight_map_is_unloaded_and_taxi_map_id_is_zero_values(env: &WowLuaEnv) {
    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_FlightMap')")
        .expect("FlightMap load-state query should succeed");
    let map_id_values: i32 = env
        .eval("return select('#', GetTaxiMapID())")
        .expect("zero taxi-map ID should produce no Lua values");
    assert!(!loaded, "FlightMap must be LoadOnDemand before TAXIMAP_OPENED");
    assert_eq!(map_id_values, 0);
}

fn open_flight_map_for_taxi_map(env: &WowLuaEnv, map_id: i32) {
    let set_map_id = format!("A_Admin.SetTaxiMapID({map_id})");
    env.exec(&set_map_id)
        .expect("supply the map selected by the flight-master interaction");
    let world_system: f64 = env
        .eval("return Enum.UIMapSystem.World")
        .expect("World UIMapSystem enum should be available");
    env.fire_event_with_args("TAXIMAP_OPENED", &[rilua::Val::Num(world_system)])
        .expect("World-system TAXIMAP_OPENED should load and open FlightMapFrame");
}

fn observe_flight_map_label_anchors(
    env: &WowLuaEnv,
) -> (bool, bool, i32, i32, String, String) {
    env.eval(FLIGHT_MAP_LABEL_ANCHORS)
        .expect("original FlightMap consumer should place both click-to-zoom labels")
}
}

prefork_full_ui_case! {
fn blizzard_flight_map_border_frame_and_scroll_container_publish(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &flight_map_toc())
        .expect("Blizzard_FlightMap should load via Rust loader");

    let children_present: (bool, bool) = env
        .eval(
            "return type(FlightMapFrame.BorderFrame) == 'table', \
                    type(FlightMapFrame.ScrollContainer) == 'table'",
        )
        .expect("FlightMapFrame children probe should succeed");
    assert_eq!(
        children_present,
        (true, true),
        "Blizzard_FlightMap.xml declares two parent-key children of FlightMapFrame: \
         `BorderFrame` (xml:11, inherits PortraitFrameTemplate, frameStrata=HIGH, \
         setAllPoints=true — provides the standard portrait-framed window chrome with \
         the Underlay/TopBorder atlas regions and a HideParentPanel close callback) and \
         `ScrollContainer` (xml:36, ScrollFrame inherits \
         MapCanvasFrameScrollContainerTemplate — the pannable/zoomable canvas surface \
         that hosts the actual map textures and pins)"
    );
}
}
