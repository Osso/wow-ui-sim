use std::path::PathBuf;

use crate::common::blizzard_addon_harness::{
    build_blizzard_addon_closure_env, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::loader::{
    discover_blizzard_addons_for_screen, find_toc_file, BlizzardAddonOverride,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

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
    map_data_providers_dir().join("Blizzard_SharedMapDataProviders_Mainline.toc")
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

#[test]
fn find_toc_file_resolves_mainline_variant() {
    let resolved =
        find_toc_file(&map_data_providers_dir()).expect("SharedMapDataProviders TOC resolves");
    assert_eq!(
        resolved,
        map_data_providers_toc(),
        "Blizzard_SharedMapDataProviders ships only the `_Mainline.toc` flavor — \
         no bare `.toc`. The data-provider family is Mainline-only because it \
         depends on retail-flavor map APIs (C_Map.GetMapPosFromWorldPos, \
         C_AreaPoiInfo, C_QuestOffer, C_DragonRiding, C_Delves, C_DigSites, etc.) \
         that don't exist in Classic flavors. Classic clients ship a different \
         data-provider family with reduced surface area"
    );
}

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

#[test]
fn toc_declares_account_saved_variable_for_glow_hub_quest_acknowledgements() {
    let toc = TocFile::from_file(&map_data_providers_toc()).expect("TOC parses");

    let saved = toc.saved_variables();
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

#[test]
fn toc_raw_bytes_pin_lod_with_account_sv_and_mainline_game_type() {
    let raw = std::fs::read_to_string(map_data_providers_toc())
        .expect("SharedMapDataProviders TOC reads utf-8");

    assert!(raw.contains("## Title: Blizzard Shared Map Data Providers"));
    assert!(raw.contains("## Author: Blizzard Entertainment"));
    assert!(raw.contains("## Version: 1.0"));
    assert!(raw.contains("## LoadOnDemand: 1"));
    assert!(raw.contains("## AllowLoad: game"));
    assert!(raw.contains("## AllowLoadGameType: mainline"));
    assert!(raw.contains("## SavedVariables: GLOW_HUB_QUESTS_ACKNOWLEDGED"));
    assert!(
        !raw.contains("## SavedVariablesPerCharacter"),
        "TOC must NOT declare per-character SavedVariables — quest \
         acknowledgements are account-shared"
    );
    assert!(!raw.contains("## Dependencies"));
    assert!(!raw.contains("## RequiredDep"));
    assert!(!raw.contains("## OptionalDep"));
    assert!(!raw.contains("## DefaultState"));
}

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
fn publishes_38_data_provider_mixin_tables_for_widget_families() {
    let env = load_map_data_providers_closure();

    for mixin in PROVIDER_MIXINS {
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

    for mixin in SHARED_PIN_MIXINS {
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
