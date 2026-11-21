use std::path::PathBuf;

use crate::common::blizzard_addon_harness::{
    build_blizzard_addon_closure_env, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::loader::{
    BlizzardAddonOverride, discover_blizzard_addons_for_screen, find_toc_file,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

#[path = "common/blizzard_addon_manifest.rs"]
mod blizzard_addon_manifest;
#[path = "common/toc_layout.rs"]
mod toc_layout;

const ROOT_ADDON: &str = "Blizzard_SharedMapDataProviders";
const IMPLICIT_DEPENDENCIES: &[&str] = &[
    "Blizzard_SharedXML",
    "Blizzard_MapCanvas",
    "Blizzard_ChatFrameBase",
];
const ADDON_OVERRIDES: &[BlizzardAddonOverride<'static>] = &[BlizzardAddonOverride {
    addon: ROOT_ADDON,
    extra_roots: IMPLICIT_DEPENDENCIES,
}];

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn map_data_providers_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_SharedMapDataProviders")
}

fn map_data_providers_toc() -> PathBuf {
    find_toc_file(&map_data_providers_dir()).expect("Blizzard_SharedMapDataProviders TOC resolves")
}

const PROVIDER_MIXINS: &[&str] = &[
    "AreaLabelDataProviderMixin",
    "AreaPOIDataProviderMixin",
    "AreaPOIEventDataProviderMixin",
    "BattlefieldFlagDataProviderMixin",
    "BonusObjectiveDataProviderMixin",
    "ClickToZoomDataProviderMixin",
    "ContentTrackingDataProviderMixin",
    "ContributionCollectorDataProviderMixin",
    "DeathMapDataProviderMixin",
    "DelveEntranceDataProviderMixin",
    "DigSiteDataProviderMixin",
    "DragonridingRaceDataProviderMixin",
    "DungeonEntranceDataProviderMixin",
    "EncounterJournalDataProviderMixin",
    "FlightPointDataProviderMixin",
    "FogOfWarDataProviderMixin",
    "GarrisonPlotDataProviderMixin",
    "GossipDataProviderMixin",
    "GroupMembersDataProviderMixin",
    "InvasionDataProviderMixin",
    "MapExplorationDataProviderMixin",
    "MapHighlightDataProviderMixin",
    "MapLinkDataProviderMixin",
    "NeighborhoodMapDataProviderMixin",
    "PetTamerDataProviderMixin",
    "PlunderstormCircleDataProviderMixin",
    "QuestBlobDataProviderMixin",
    "QuestDataProviderMixin",
    "QuestOfferDataProviderMixin",
    "QuestSessionDataProviderMixin",
    "ScenarioDataProviderMixin",
    "SelectableGraveyardDataProviderMixin",
    "SuperTrackWaypointDataProviderMixin",
    "VehicleDataProviderMixin",
    "VignetteDataProviderMixin",
    "WaypointLocationDataProviderMixin",
    "WorldQuestDataProviderMixin",
    "ZoneLabelDataProviderMixin",
];

const SHARED_PIN_MIXINS: &[&str] = &[
    "BaseMapPoiPinMixin",
    "AreaPOIPinMixin",
    "AreaPOIEventPinMixin",
    "BattlefieldFlagMixin",
    "BonusObjectivePinMixin",
    "ContentTrackingPinMixin",
    "ContributionCollectorPinMixin",
    "DelveEntrancePinMixin",
    "DigSitePinMixin",
    "DragonridingRacePinMixin",
    "DungeonEntrancePinMixin",
    "EncounterJournalPinMixin",
    "EncounterMapTrackingPinMixin",
    "FlightPointPinMixin",
    "FogOfWarPinMixin",
    "GarrisonPlotPinMixin",
    "GossipPinMixin",
    "IconWithHeightIndicatorMapPinMixin",
    "MapExplorationPinMixin",
    "MapHighlightPinMixin",
    "MapLinkPinMixin",
    "PetTamerPinMixin",
    "PlunderstormInnerCirclePinMixin",
    "PlunderstormOuterCirclePinMixin",
    "QuestBlobPinMixin",
    "QuestPinMixin",
    "ScenarioBlobPinMixin",
    "ScenarioPinMixin",
    "SelectableGraveyardPinMixin",
    "SuperTrackWaypointPinMixin",
    "SuperTrackablePinMixin",
    "ThreatObjectivePinMixin",
    "VehiclePinMixin",
    "VignettePinMixin",
    "WaypointLocationPinMixin",
    "WorldQuestPinMixin",
];

fn load_map_data_providers_closure() -> WowLuaEnv {
    let (env, _) =
        build_blizzard_addon_closure_env(&blizzard_ui_dir(), &[ROOT_ADDON], ADDON_OVERRIDES);
    env.apply_post_load_workarounds();
    env
}

// Literal cached inputs absent from Wrath manifest1-382/cache; behavior cases stay enabled.
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
const TAXI_MAP_ADDON_OVERRIDES: &[BlizzardAddonOverride<'static>] = &[
    BlizzardAddonOverride {
        addon: "Blizzard_FlightMap",
        extra_roots: &[
            "Blizzard_FrameXML",
            "Blizzard_UIParentPanelManager",
            "Blizzard_SharedXML",
        ],
    },
    BlizzardAddonOverride {
        addon: "Blizzard_WorldMap",
        extra_roots: &[
            "Blizzard_FrameXML",
            "Blizzard_UIParentPanelManager",
            "Blizzard_SharedXML",
        ],
    },
];

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
const TAXI_MAP_NODE_INPUT_LUA: &str = r#"
A_Admin.SetTaxiMapNodes({
    { mapID=2248, nodeID=101, name="Node Alpha", atlasName="TaxiNode", textureKit="", faction=0,
      x=0.25, y=0.75, state=2, slotIndex=1, isUndiscovered=false,
      useSpecialIcon=false, isMapLayerTransition=false },
    { mapID=2248, nodeID=102, name="Node Beta", atlasName="TaxiNode", textureKit="", faction=0,
      x=0.75, y=0.25, state=2, slotIndex=2, isUndiscovered=false,
      useSpecialIcon=false, isMapLayerTransition=false },
    { mapID=9999, nodeID=103, name="Other Map", atlasName="TaxiNode", textureKit="", faction=0,
      x=0.5, y=0.5, state=2, slotIndex=3, isUndiscovered=false,
      useSpecialIcon=false, isMapLayerTransition=false },
})
"#;

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
const TAXI_MAP_REFRESH_LUA: &str = r#"
FlightMapFrame:SetMapID(2248)
WorldMapFrame:SetMapID(2248)

local flightProvider = CreateFromMixins(FlightMap_FlightPathDataProviderMixin)
FlightMapFrame:AddDataProvider(flightProvider)
flightProvider:RefreshAllData()

local sharedProvider = CreateFromMixins(FlightPointDataProviderMixin)
WorldMapFrame:AddDataProvider(sharedProvider)
sharedProvider:RefreshAllData()

local flightPins, sharedPins = {}, {}
for pin in FlightMapFrame:EnumeratePinsByTemplate("FlightMap_FlightPointPinTemplate") do
    flightPins[pin.taxiNodeData.name] = true
end
for pin in WorldMapFrame:EnumeratePinsByTemplate("FlightPointPinTemplate") do
    sharedPins[pin.poiInfo.name] = true
end

return FlightMapFrame:GetNumActivePinsByTemplate("FlightMap_FlightPointPinTemplate"),
       flightPins["Node Alpha"] == true, flightPins["Node Beta"] == true,
       WorldMapFrame:GetNumActivePinsByTemplate("FlightPointPinTemplate"),
       sharedPins["Node Alpha"] == true, sharedPins["Node Beta"] == true,
       C_TaxiMap.ShouldMapShowTaxiNodes(1), C_TaxiMap.ShouldMapShowTaxiNodes(9999),
       C_TaxiMap.ShouldMapShowTaxiNodes(7777)
"#;

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn load_taxi_map_addon_closure() -> (WowLuaEnv, Vec<String>) {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    env.exec(TAXI_MAP_NODE_INPUT_LUA)
        .expect("supply taxi nodes before loading the UI consumers");
    let loaded = load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &["Blizzard_FlightMap", "Blizzard_WorldMap"],
        TAXI_MAP_ADDON_OVERRIDES,
    );
    env.apply_post_load_workarounds();
    (env, loaded)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
const GOSSIP_MAP_OVERRIDES: &[BlizzardAddonOverride<'static>] = &[BlizzardAddonOverride {
    addon: "Blizzard_WorldMap",
    extra_roots: &["Blizzard_FrameXML", "Blizzard_UIParentPanelManager"],
}];

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
const GOSSIP_MAP_REFRESH_LUA: &str = r#"
local provider = CreateFromMixins(GossipDataProviderMixin)
WorldMapFrame:AddDataProvider(provider)
local function readGossipPin(mapID)
    WorldMapFrame:SetMapID(mapID)
    provider:RefreshAllData()
    local pin = WorldMapFrame:EnumeratePinsByTemplate("GossipPinTemplate")()
    if not pin then return 0, "", -1, -1 end
    local x, y = pin:GetPosition()
    return WorldMapFrame:GetNumActivePinsByTemplate("GossipPinTemplate"), pin.poiInfo.name, x, y
end
local countA, nameA, xA, yA = readGossipPin(2248)
local countB, nameB, xB, yB = readGossipPin(84)
return countA, nameA, xA, yA, countB, nameB, xB, yB
"#;

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn load_gossip_map_addon_closure() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    env.exec(
        "A_Admin.SetGossipPoi(2248, 401, 'North Gossip', 0.2, 0.8, nil, 3); A_Admin.SetGossipPoi(84, 402, 'South Gossip', 0.7, 0.3, nil, 7)",
    )
    .expect("supply map-specific gossip POIs before loading Blizzard consumers");
    let loaded = load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &["Blizzard_SharedMapDataProviders", "Blizzard_WorldMap"],
        GOSSIP_MAP_OVERRIDES,
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_FrameXML"));
    assert!(loaded.iter().any(|name| name == ROOT_ADDON));
    assert!(loaded.iter().any(|name| name == "Blizzard_WorldMap"));
    env.apply_post_load_workarounds();
    env
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn gossip_map_provider_refreshes_the_supplied_poi_for_each_map() {
    let env = load_gossip_map_addon_closure();
    let observed: (i32, String, f64, f64, i32, String, f64, f64) = env
        .eval(GOSSIP_MAP_REFRESH_LUA)
        .expect("original gossip provider should refresh pins on the real WorldMapFrame");
    assert_eq!(
        observed,
        (
            1,
            "North Gossip".into(),
            0.2,
            0.8,
            1,
            "South Gossip".into(),
            0.7,
            0.3
        )
    );
}

// INFERRED R88cr: Forever uses the mainline SoftCursor addon through FrameXML.
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn taxi_map_providers_refresh_pins_from_supplied_nodes() {
    let (env, loaded) = load_taxi_map_addon_closure();
    for addon in [
        "Blizzard_UIParentPanelManager",
        "Blizzard_FrameXML",
        "Blizzard_SharedXML",
        "Blizzard_FlightMap",
        "Blizzard_SharedMapDataProviders",
        "Blizzard_WorldMap",
    ] {
        assert!(
            loaded.iter().any(|name| name == addon),
            "{addon} was not loaded"
        );
    }
    if cfg!(feature = "client-wowforever") {
        assert!(
            loaded
                .iter()
                .any(|name| name == "Blizzard_GamepadSoftCursor"),
            "Forever FrameXML must load its mainline SoftCursor dependency"
        );
    }
    let observed: (i32, bool, bool, i32, bool, bool, bool, bool, bool) = env
        .eval(TAXI_MAP_REFRESH_LUA)
        .expect("original taxi map data providers should refresh on real map canvases");
    assert_eq!(observed, (2, true, true, 2, true, true, true, false, false));
}

post_wrath_ui_fixture! {
#[test]
fn find_toc_file_resolves_mainline_variant() {
    let resolved =
        find_toc_file(&map_data_providers_dir()).expect("SharedMapDataProviders TOC resolves");
    assert_eq!(
        resolved.file_name().and_then(|name| name.to_str()),
        Some(toc_layout::expected_toc_name("Blizzard_SharedMapDataProviders").as_str()),
        "find_toc_file resolves the active profile's SharedMapDataProviders TOC: Retail ships \
         `_Mainline.toc`, Mists `_Mists.toc`, and Era/Anniversary the `_Classic` companion"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_declares_load_on_demand_with_no_dependencies() {
    let toc = TocFile::from_file(&map_data_providers_toc()).expect("TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "TOC must declare `## LoadOnDemand: 1` — the addon is loaded by the \
         WorldMapFrame and FlightMapFrame addons via explicit \
         C_AddOns.LoadAddOn(\"Blizzard_SharedMapDataProviders\") calls right \
         before they call `:AddDataProvider(CreateFromMixins(<ProviderMixin>))`. \
         Eager loading would inflate the working-set with ~90 mixin tables and \
         pin pool memory for data providers no map currently uses"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(!toc.is_glue_only());

    assert!(
        toc.dependencies().is_empty(),
        "TOC must declare zero hard Dependencies — the addon's mixins reference \
         globals (MapCanvasDataProviderMixin / MapCanvasPinMixin / \
         CVarMapCanvasDataProviderMixin / CreateFromMixins / \
         CreateAndInitFromMixin) that are guaranteed to be present at \
         load-on-demand time because the calling addon (Blizzard_WorldMap) is \
         already loaded with its own dependency graph satisfied. The lazy-load \
         pattern relies on the consumer's deps being live, NOT on this addon \
         declaring them"
    );
    assert!(toc.optional_deps().is_empty());
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_declares_account_saved_variable_for_glow_hub_quest_acknowledgements() {
    let toc = TocFile::from_file(&map_data_providers_toc()).expect("TOC parses");

    let saved = toc.saved_variables();
    if toc_layout::classic_cache() {
        assert!(
            saved.is_empty(),
            "the classic data-provider TOC declares no account SavedVariables. Got: {saved:?}"
        );
        assert!(toc.saved_variables_per_character().is_empty());
        return;
    }
    assert_eq!(
        saved.len(),
        1,
        "TOC must declare exactly 1 account-level SavedVariable — the GLOW_HUB \
         quest acknowledgement set is a per-account flag map keyed by quest ID, \
         remembering which level-up zone hub quests the player has dismissed the \
         glowing-pin tutorial highlight on. Got: {saved:?}"
    );
    assert_eq!(
        saved[0], "GLOW_HUB_QUESTS_ACKNOWLEDGED",
        "SavedVariable name must be exactly GLOW_HUB_QUESTS_ACKNOWLEDGED — \
         consumed by QuestDataProvider.lua's glow-hub pin highlight logic to \
         persist dismissal across sessions"
    );
    assert!(toc.saved_variables_per_character().is_empty());
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_lacks_screen_loads_only_in_game_screen() {
    let toc = TocFile::from_file(&map_data_providers_toc()).expect("TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "TOC must allow Game screen — `## AllowLoad: game` explicitly grants \
         in-game loads. Map data providers are in-game only because the \
         WorldMapFrame addon that consumes them is itself an in-game addon"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Glue screen {screen:?} must NOT be allowed — `## AllowLoad: game` \
             excludes glue screens, and the data-provider mixins reference \
             in-game-only globals (MapCanvasDataProviderMixin lives in \
             Blizzard_MapCanvas, an in-game addon)"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_raw_bytes_pin_lod_with_account_sv_and_mainline_game_type() {
    let raw = std::fs::read_to_string(map_data_providers_toc())
        .expect("SharedMapDataProviders TOC reads utf-8");

    if toc_layout::mists() {
        assert!(raw.contains("## Title: Blizzard_SharedMapDataProviders"));
        assert!(raw.contains("## Author: Blizzard Entertainment"));
        assert!(raw.contains("## LoadOnDemand: 1"));
        assert!(raw.contains("## AllowLoad: game"));
        assert!(raw.contains("## AllowLoadGameType: mists"));
        assert!(!raw.contains("## Version"));
    } else if toc_layout::classic_cache() {
        assert!(raw.contains("## Title: Blizzard Shared Map Data Providers"));
        assert!(raw.contains("## Author: Blizzard Entertainment"));
        assert!(raw.contains("## Version: 1.0"));
        assert!(raw.contains("## LoadOnDemand: 1"));
        assert!(raw.contains("## AllowLoad: game"));
        assert!(raw.contains("## AllowLoadGameType: vanilla, tbc"));
    } else {
        assert!(raw.contains("## Title: Blizzard Shared Map Data Providers"));
        assert!(raw.contains("## Author: Blizzard Entertainment"));
        assert!(raw.contains("## Version: 1.0"));
        assert!(raw.contains("## LoadOnDemand: 1"));
        assert!(raw.contains("## AllowLoad: game"));
        assert!(raw.contains("## AllowLoadGameType: mainline"));
        assert!(raw.contains("## SavedVariables: GLOW_HUB_QUESTS_ACKNOWLEDGED"));
    }
    assert!(!raw.contains("## SavedVariablesPerCharacter"));
    assert!(!raw.contains("## Dependencies"));
    assert!(!raw.contains("## RequiredDep"));
    assert!(!raw.contains("## OptionalDep"));
    assert!(!raw.contains("## DefaultState"));
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_body_lists_initialization_files_before_data_provider_xml_pairs() {
    let raw = std::fs::read_to_string(map_data_providers_toc()).expect("TOC reads utf-8");

    let body_lines: Vec<&str> = raw
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        })
        .collect();

    let first_three = &body_lines[..3];
    if toc_layout::classic_cache() {
        assert_eq!(
            first_three,
            [
                "SharedMapPoiTemplates.xml",
                "QuestDataProvider.xml",
                "GroupMembersDataProvider.xml",
            ],
            "the classic data-provider TOC starts with the shared POI templates and the quest \
             providers, without the mainline MapPinTags/FrameCloneManager prelude"
        );
        return;
    }
    assert_eq!(
        first_three,
        [
            "MapPinTags.lua",
            "FrameCloneManager.lua",
            "SharedMapPoiTemplates.xml",
        ],
        "TOC body MUST start with these 3 entries in order: (1) MapPinTags.lua \
         declares `_G.MapPinTags = EnumUtil.MakeEnum(\"QuestOffer\", \"Event\", \
         \"FlightPoint\", \"WorldQuest\", \"BonusObjective\", \
         \"IsSuppressible\")` — the enum every PinMixin's GetPinTags() returns \
         entries from. (2) FrameCloneManager.lua declares `_G.FrameCloneManager` \
         table with framePool/regionPool — used by WorldMapFrame's data provider \
         loop to clone pin-template frames cheaply. (3) SharedMapPoiTemplates.xml \
         registers the BaseMapPoi virtual templates that PinMixin XMLs inherit \
         from. Reordering would break: data provider XMLs reference \
         `inherits=\"BaseMapPoiTemplate\"` from SharedMapPoiTemplates.xml; \
         PinMixin OnLoad reads MapPinTags constants. Got: {first_three:?}"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_body_count_breakdown_matches_filesystem_layout() {
    let toc = TocFile::from_file(&map_data_providers_toc()).expect("TOC parses");

    let lua_count = toc
        .files
        .iter()
        .filter(|f| f.extension().is_some_and(|ext| ext == "lua"))
        .count();
    let xml_count = toc
        .files
        .iter()
        .filter(|f| f.extension().is_some_and(|ext| ext == "xml"))
        .count();

    if toc_layout::classic_cache() {
        if toc_layout::mists() {
            assert_eq!((lua_count, xml_count), (2, 32), "pinned Mists data-provider counts");
        } else {
            assert_eq!((lua_count, xml_count), (1, 30), "pinned Era/Anniversary data-provider counts");
        }
        return;
    }
    assert!(
        lua_count >= 5,
        "TOC must list at least 5 .lua entries: MapPinTags, FrameCloneManager, \
         AreaPOIEventDataProvider, QuestOfferDataProvider, QuestSessionDataProvider, \
         SuperTrackWaypointDataProvider, NeighborhoodMapDataProvider. Most \
         data-provider .lua files are loaded indirectly via their XML's `<Script \
         file=\"X.lua\"/>` directives, so the TOC body lists XMLs and the .lua \
         load via XML chaining. Got lua_count={lua_count}"
    );
    assert!(
        xml_count >= 30,
        "TOC must list at least 30 .xml entries — one per data provider widget \
         family (QuestDataProvider, ContentTrackingDataProvider, \
         GroupMembersDataProvider, etc.). Got xml_count={xml_count}"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn lod_addon_dragged_into_game_eager_discovery_via_worldmap_required_dep() {
    let ui = blizzard_ui_dir();

    let game_addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let game_found = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_SharedMapDataProviders");
    assert!(
        game_found,
        "Blizzard_SharedMapDataProviders MUST appear in Game-screen eager \
         discovery despite its `## LoadOnDemand: 1` flag — \
         Blizzard_WorldMap_Mainline.toc declares `## RequiredDep: \
         Blizzard_MapCanvas, Blizzard_SharedMapDataProviders, \
         Blizzard_HelpPlate` and Blizzard_WorldMap is itself eager (no LoD), \
         which drags this LoD addon in as a hard dependency of an eager addon. \
         The discovery pass at src/loader/mod.rs follows RequiredDep edges from \
         eager addons even into LoD targets to prevent the eager addon from \
         crashing on missing globals at parse time"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_SharedMapDataProviders");
        assert!(
            !found,
            "Blizzard_SharedMapDataProviders must be excluded from eager \
             discovery on {screen:?} — `## AllowLoad: game` blocks glue \
             screens, and Blizzard_WorldMap (the eager dependent) is itself \
             Game-only. Without WorldMap to drag it in, the LoD addon stays \
             out of the eager pool"
        );
    }
}
}

#[test]
fn explicit_load_emits_no_addon_specific_lua_errors() {
    let env = load_map_data_providers_closure();

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let relevant: Vec<&String> = errors
        .iter()
        .filter(|e| {
            e.contains("Blizzard_SharedMapDataProviders")
                || e.contains("DataProvider.lua")
                || e.contains("DataProvider.xml")
                || e.contains("MapPinTags")
                || e.contains("FrameCloneManager")
                || e.contains("BaseMapPoiPin")
        })
        .collect();
    assert!(
        relevant.is_empty(),
        "Explicit LoadOnDemand load must emit zero addon-specific Lua errors. \
         Got:\n  {}",
        relevant
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

post_wrath_ui_fixture! {
#[test]
fn is_addon_loaded_transitions_false_to_true_after_explicit_load() {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);

    let before: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_SharedMapDataProviders')")
        .expect("pre-load IsAddOnLoaded probe succeeds");
    assert!(
        !before,
        "C_AddOns.IsAddOnLoaded must be false before explicit load — \
         LoadOnDemand=1 keeps the addon out of the eager pool"
    );

    load_blizzard_addon_closure_into_env(&env, &ui, &[ROOT_ADDON], ADDON_OVERRIDES);

    let after: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_SharedMapDataProviders')")
        .expect("post-load IsAddOnLoaded probe succeeds");
    assert!(
        after,
        "C_AddOns.IsAddOnLoaded must be true after explicit load — the AddOn \
         state machine flips loaded=true after executing the body"
    );
}
}

#[test]
fn map_pin_tags_enum_published_with_six_canonical_keys() {
    let env = load_map_data_providers_closure();

    let kind: String = env
        .eval("return type(MapPinTags)")
        .expect("MapPinTags type probe");
    assert_eq!(
        kind, "table",
        "_G.MapPinTags must be an enum table — MapPinTags.lua line 1 declares \
         `MapPinTags = EnumUtil.MakeEnum(\"QuestOffer\", \"Event\", \
         \"FlightPoint\", \"WorldQuest\", \"BonusObjective\", \
         \"IsSuppressible\")`. Every PinMixin's GetPinTags() returns one of \
         these tags so the WorldMap suppression logic can hide pins of a given \
         tag at a given zoom level"
    );

    for tag in [
        "QuestOffer",
        "Event",
        "FlightPoint",
        "WorldQuest",
        "BonusObjective",
        "IsSuppressible",
    ] {
        let id_kind: String = env
            .eval(&format!("return type(MapPinTags.{tag})"))
            .unwrap_or_else(|_| panic!("MapPinTags.{tag} probe failed"));
        assert_eq!(
            id_kind, "number",
            "MapPinTags.{tag} must be a numeric enum value — EnumUtil.MakeEnum \
             assigns sequential ids starting at 1"
        );
    }
}

#[test]
fn frame_clone_manager_published_as_table_with_init_method() {
    let env = load_map_data_providers_closure();

    let kind: String = env
        .eval("return type(FrameCloneManager)")
        .expect("FrameCloneManager probe");
    assert_eq!(
        kind, "table",
        "_G.FrameCloneManager must be a table — FrameCloneManager.lua line 1 \
         declares `FrameCloneManager = {{}}` and adds Init/Acquire/Release methods. \
         WorldMapFrame's pin-spawn loop calls FrameCloneManager:Acquire(\
         template) to clone pin-template frames out of a shared pool, avoiding \
         per-pin CreateFrame allocations"
    );

    let init_kind: String = env
        .eval("return type(FrameCloneManager.Init)")
        .expect("FrameCloneManager.Init probe");
    assert_eq!(
        init_kind, "function",
        "FrameCloneManager.Init must be a function — initializes the framePool \
         via CreateFramePool. Without Init, the pool wouldn't exist on first \
         Acquire call"
    );
}

#[test]
fn publishes_profile_selected_data_provider_mixin_tables_for_widget_families() {
    let env = load_map_data_providers_closure();
    let provider_mixins = PROVIDER_MIXINS
        .iter()
        .copied()
        .filter(|mixin| !toc_layout::forever() || *mixin != "EncounterJournalDataProviderMixin");

    // The Forever TOC excludes EncounterJournalDataProvider.xml for camelot.
    for mixin in provider_mixins {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|_| panic!("{mixin} probe failed"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must be a table — every WorldMapFrame data-provider \
             registration calls AddDataProvider(CreateFromMixins(<Mixin>)). \
             Missing any one means that widget family fails to register"
        );
    }
}

#[test]
fn publishes_pin_mixin_tables_for_per_widget_pins() {
    let env = load_map_data_providers_closure();
    let pin_mixins = SHARED_PIN_MIXINS
        .iter()
        .copied()
        .filter(|mixin| !toc_layout::forever() || *mixin != "EncounterJournalPinMixin");

    for mixin in pin_mixins {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|_| panic!("{mixin} probe failed"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must be a table — pin templates set `mixin=\"{mixin}\"` \
             in their XML, which the loader resolves at template-registration \
             time. Missing any one breaks the corresponding XML template parse"
        );
    }
}

#[test]
fn base_map_poi_pin_mixin_create_subpin_helper_returns_table() {
    let env = load_map_data_providers_closure();

    let kind: String = env
        .eval("return type(BaseMapPoiPinMixin.CreateSubPin)")
        .expect("CreateSubPin probe");
    assert_eq!(
        kind, "function",
        "BaseMapPoiPinMixin:CreateSubPin must be a function — declared in \
         SharedMapPoiTemplates.lua line 3, used by sibling pin mixins to create \
         frame-level-tagged sub-pins (e.g., \
         FlightPointPinMixin = BaseMapPoiPinMixin:CreateSubPin(\
         \"PIN_FRAME_LEVEL_FLIGHT_POINT\")). Returns CreateFromMixins(self, \
         {{pinFrameLevel = pinFrameLevel}}) — a copy of self with pinFrameLevel \
         pinned"
    );

    let result_kind: String = env
        .eval("return type(BaseMapPoiPinMixin:CreateSubPin('PIN_FRAME_LEVEL_TEST'))")
        .expect("CreateSubPin invocation probe");
    assert_eq!(
        result_kind, "table",
        "BaseMapPoiPinMixin:CreateSubPin('PIN_FRAME_LEVEL_TEST') must return a \
         table — the sub-pin is itself a CreateFromMixins descendant of \
         BaseMapPoiPinMixin and inherits all base methods"
    );
}

#[test]
fn quest_data_provider_mixin_inherits_map_canvas_data_provider_mixin() {
    let env = load_map_data_providers_closure();

    let inherits: bool = env
        .eval(
            "return type(QuestDataProviderMixin) == 'table' and \
             type(QuestDataProviderMixin.OnAdded) == 'function'",
        )
        .expect("QuestDataProviderMixin inheritance probe");
    assert!(
        inherits,
        "QuestDataProviderMixin must inherit OnAdded from \
         MapCanvasDataProviderMixin via CreateFromMixins. OnAdded is the \
         lifecycle hook called by the WorldMap framework when the provider \
         attaches to a map; without inheritance the data provider couldn't \
         register itself with the WorldMap event router"
    );
}

#[test]
fn glow_hub_quests_acknowledged_saved_var_resolves_after_load() {
    let env = load_map_data_providers_closure();

    let kind: String = env
        .eval("return type(GLOW_HUB_QUESTS_ACKNOWLEDGED)")
        .expect("GLOW_HUB_QUESTS_ACKNOWLEDGED probe");
    assert!(
        kind == "table" || kind == "nil",
        "_G.GLOW_HUB_QUESTS_ACKNOWLEDGED must be either a table (SV restored or \
         seeded by addon code) or nil (SV restoration deferred / disabled). The \
         addon body itself does not seed the table — it relies on the SV runtime \
         materializing the table from the saved blob OR the consumer code \
         creating it on first write. Either state is valid at post-load probe \
         time. Got: {kind}"
    );
}

#[test]
fn shared_map_poi_templates_register_in_template_registry_not_global_frames() {
    let env = load_map_data_providers_closure();

    let kind: String = env
        .eval("return type(BaseMapPoiTemplate)")
        .expect("BaseMapPoiTemplate global probe");
    assert_eq!(
        kind, "nil",
        "_G.BaseMapPoiTemplate must be nil — virtual XML templates live in the \
         template registry only, never in `_G`. PinMixin XMLs reference them \
         via `inherits=\"BaseMapPoiTemplate\"` which the XML loader resolves at \
         parse time against the registry. A non-nil result here would mean the \
         loader is leaking virtual templates into the global namespace"
    );
}

#[cfg(all(feature = "optional-tests", feature = "client-retail"))]
#[test]
fn empty_contribution_id_list_adds_no_map_tooltip_rows() {
    let env = load_map_data_providers_closure();

    let line_count: i32 = env
        .eval(
            r#"
            assert(type(ContributionCollectorPinMixin.AddContributionsToTooltip) == "function")
            C_ContributionCollector.GetContributionAppearance = function()
                return { stateColor = HIGHLIGHT_FONT_COLOR }
            end

            local tooltip = { lineCount = 0 }
            function tooltip:AddLine(...)
                self.lineCount = self.lineCount + 1
            end

            ContributionCollectorPinMixin:AddContributionsToTooltip(
                tooltip,
                C_ContributionCollector.GetManagedContributionsForCreatureID(123456))
            return tooltip.lineCount
            "#,
        )
        .expect("original contribution-map tooltip consumer should run");

    assert_eq!(
        line_count, 0,
        "an empty managed-contribution result must not create a blank tooltip row"
    );
}

#[cfg(all(feature = "optional-tests", feature = "client-retail"))]
#[test]
fn world_map_provider_pins_keep_original_backref_ownership() {
    let ui = blizzard_ui_dir();
    let (env, loaded) = build_blizzard_addon_closure_env(
        &ui,
        blizzard_addon_manifest::WORLD_MAP_SMOKE_ROOTS,
        &[
            BlizzardAddonOverride {
                addon: "Blizzard_SharedMapDataProviders",
                // These two owners are needed while provider Lua chunks load.
                extra_roots: &["Blizzard_SharedXML", "Blizzard_ChatFrameBase"],
            },
            BlizzardAddonOverride {
                addon: "Blizzard_WorldMap",
                // FogOfWarFrameTemplate must exist before WorldMap acquires its pin.
                extra_roots: &["Blizzard_UIPanels_Game"],
            },
        ],
    );

    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "Retail WorldMap fixture emitted Lua errors: {:?}",
        env.state().borrow().lua_errors,
    );

    for addon in [
        "Blizzard_MapCanvas",
        "Blizzard_SharedMapDataProviders",
        "Blizzard_WorldMap",
    ] {
        assert!(
            loaded.iter().any(|loaded_name| loaded_name == addon),
            "WorldMap's Retail dependency closure must load {addon}"
        );
    }

    let (highlight_stays_unassociated, exploration_has_original_owner): (bool, bool) = env
        .eval(
            r#"
            local function find_provider(mixin)
                for provider in pairs(WorldMapFrame.dataProviders) do
                    if provider.OnAdded == mixin.OnAdded then
                        return provider
                    end
                end
            end

            local highlightProvider = assert(find_provider(MapHighlightDataProviderMixin))
            local explorationProvider = assert(find_provider(MapExplorationDataProviderMixin))

            local highlightPin = assert(highlightProvider.pin)
            local highlightWasUnassociated = highlightPin:GetDataProvider() == nil
            WorldMapFrame:RemoveDataProvider(highlightProvider)
            WorldMapFrame:AddDataProvider(highlightProvider)
            local highlightStayedUnassociated = highlightProvider.pin:GetDataProvider() == nil

            local explorationPin = assert(explorationProvider.pin)
            local explorationWasOwned = explorationPin:GetDataProvider() == explorationProvider
            WorldMapFrame:RemoveDataProvider(explorationProvider)
            WorldMapFrame:AddDataProvider(explorationProvider)
            local explorationStayedOwned = explorationProvider.pin:GetDataProvider() == explorationProvider

            return highlightWasUnassociated and highlightStayedUnassociated,
                explorationWasOwned and explorationStayedOwned
            "#,
        )
        .expect("original WorldMap provider attachment and pin acquisition should run");

    assert!(highlight_stays_unassociated);
    assert!(exploration_has_original_owner);

    env.apply_post_load_workarounds();

    let new_highlight_stays_unassociated: bool = env
        .eval(
            r#"
            local provider = CreateFromMixins(MapHighlightDataProviderMixin)
            WorldMapFrame:AddDataProvider(provider)
            local staysUnassociated = provider.pin:GetDataProvider() == nil
            WorldMapFrame:RemoveDataProvider(provider)
            return staysUnassociated
            "#,
        )
        .expect("original AddDataProvider callback and pin acquisition should run");

    assert!(
        new_highlight_stays_unassociated,
        "MapCanvasMixin:AddDataProvider must not assign a back-reference that the original provider does not set"
    );
}
