use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::{
    fire_gui_startup_on_update_tick, fire_startup_events_for_screen, process_pending_timers,
};
use wow_ui_sim::toc::TocFile;

#[path = "blizzard_map_canvas_loads/support.rs"]
mod support;

use support::*;

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
        wow_ui_sim::loader::load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);

    env
}

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_world_map_show_and_sustained_updates_are_error_free() {
    let env = load_full_game_ui();
    env.state().borrow_mut().lua_errors.clear();

    env.exec("WorldMapFrame:Show()")
        .expect("showing the Forever world map should dispatch its handlers");
    for _ in 0..60 {
        env.state().borrow_mut().ensure_layout_rects();
        fire_gui_startup_on_update_tick(&env);
        process_pending_timers(&env);
    }

    let (target_scale, quest_limit): (f64, i64) = env
        .eval(
            "return WorldMapFrame.ScrollContainer.targetScale, \
             Constants.QuestLogConsts.MAXIMUM_NUM_QUESTS_LOG_CAN_ACCEPT",
        )
        .expect("Forever world-map runtime state should be initialized");
    assert!(target_scale > 0.0);
    assert_eq!(quest_limit, 40);

    let errors = env.state().borrow().lua_errors.clone();
    assert!(
        errors.is_empty(),
        "showing and ticking the Forever world map emitted Lua errors:\n{}",
        errors.join("\n")
    );
}

#[test]
fn blizzard_map_canvas_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&map_canvas_dir()).expect("Blizzard_MapCanvas TOC should resolve");
    assert_eq!(
        resolved,
        map_canvas_toc(),
        "Blizzard_MapCanvas ships exactly one bare TOC. Map canvas is the cross-flavor pan / \
         zoom / pin / data-provider scaffolding consumed by every map UI in the game \
         (WorldMap, FlightMap, BattlefieldMap, AdventureMap, HybridMinimap, HousingHouseFinder), \
         so the retail tree carries one Blizzard_MapCanvas.toc with no flavor-suffixed \
         variants — `find_toc_file` resolves to the bare file after the `_Mainline.toc` \
         lookup misses"
    );
}

#[test]
fn blizzard_map_canvas_toc_declares_load_on_demand_with_required_deps() {
    let toc = TocFile::from_file(&map_canvas_toc()).expect("Blizzard_MapCanvas TOC parses");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_MapCanvas declares `## LoadOnDemand: 1` — deferred-load only. The map \
         canvas scaffolding has no in-world side effects of its own (no events to register, \
         no concrete frames to spawn); it only matters when one of its consumers (WorldMap, \
         FlightMap, etc) gets pulled in. The eager-load slot is therefore wasted, so the \
         addon stays in the lod_pool until a `## RequiredDep:` chain pulls it forward via \
         pull_required_lod_addons (src/loader/mod.rs:553)"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert_eq!(
        toc.dependencies(),
        MAP_CANVAS_DEPENDENCIES,
        "TOC declares `## RequiredDep: Blizzard_SharedXMLBase, Blizzard_MapCanvasSecureUtil` \
         — `dependencies()` reads from RequiredDep at src/toc.rs:212 (one of the three \
         aliased keys: RequiredDep / Dependencies / RequiredDeps). Blizzard_SharedXMLBase \
         supplies the CallbackRegistryMixin / TextureLoadingGroupMixin / TaggableObjectMixin \
         / CreateTexturePool / Clamp helpers, Blizzard_MapCanvasSecureUtil supplies the \
         protected pin-attribute / area-trigger contracts that bridge insecure addons into \
         the secure-only map state without taint-leaking"
    );
    assert!(toc.optional_deps().is_empty());
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — map canvas is pure runtime scaffolding (mixin tables + \
         virtual templates). Persistent map-related state lives in the consumer addons \
         (WorldMap saves last-shown mapID, FlightMap saves last-known flight node) or in \
         CVars (mapAnimDuration, etc), never in this addon"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "TOC omits `## AllowLoadGameType:` — `is_game_type_restricted` returns false. The \
         scaffolding is a universal contract that every shipping flavor (Mainline, Classic \
         Era, Cataclysm, MoP Classic, etc) consumes via WorldMap / FlightMap / BattlefieldMap"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Blizzard_MapCanvas omits `## AllowLoad:` — the default branch at src/toc.rs:311 \
         routes to Game-screen-only. Maps only render in-world; the glue screens \
         (login / character-select) have no map UI"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Blizzard_MapCanvas must NOT auto-discover on glue screen {screen:?} — missing \
             `## AllowLoad:` falls through to the default-Game branch at src/toc.rs:311. \
             Glue screens have no consumer addon (Blizzard_WorldMap is itself Game-only) \
             that could pull MapCanvas via dep edge"
        );
    }
}

#[test]
fn blizzard_map_canvas_toc_lists_three_files_excluding_cross_xml_lua() {
    let toc = TocFile::from_file(&map_canvas_toc()).expect("Blizzard_MapCanvas TOC parses");
    assert_eq!(
        toc.files
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        MAP_CANVAS_TOC_FILES,
        "TOC body lists exactly 3 files in load order — MapCanvas_DataProviderBase.lua \
         publishes the data-provider base + CVarDataProvider + Pin mixin, \
         MapCanvas_PinFrameLevelsManager.lua publishes the pin frame-level allocator, \
         Blizzard_MapCanvas.xml declares the 5 virtual templates and uses \
         `<Script file=\"...\"/>` to cross-load 3 additional .lua files \
         (Blizzard_MapCanvasDetailLayer.lua, MapCanvas_ScrollContainerMixin.lua, \
         Blizzard_MapCanvas.lua) NOT listed in the TOC body"
    );
}

#[test]
fn blizzard_map_canvas_directory_holds_seven_entries() {
    let entries = std::fs::read_dir(map_canvas_dir())
        .expect("Blizzard_MapCanvas directory reads")
        .count();
    assert_eq!(
        entries, 7,
        "Directory holds exactly 7 entries — Blizzard_MapCanvas.toc + 6 source files \
         (3 TOC-listed + 3 cross-XML-loaded via `<Script file=\"...\"/>` at the top of \
         Blizzard_MapCanvas.xml: Blizzard_MapCanvasDetailLayer.lua / \
         MapCanvas_ScrollContainerMixin.lua / Blizzard_MapCanvas.lua)"
    );
}

#[test]
fn blizzard_map_canvas_pulled_into_game_screen_via_world_map_dep_edge() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let game_found = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_MapCanvas");
    assert!(
        game_found,
        "Blizzard_MapCanvas must surface on the Game screen even though it declares \
         `## LoadOnDemand: 1` — `pull_required_lod_addons` (src/loader/mod.rs:553) walks \
         the eager addons' RequiredDep lists and pulls each named LOD addon into the loaded \
         set. Blizzard_WorldMap (eager-loaded on Game) declares \
         `## RequiredDep: Blizzard_MapCanvas, ...` so MapCanvas gets promoted out of the \
         lod_pool into the loaded `addons` HashMap during the closure walk"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons.iter().any(|(name, _)| name == "Blizzard_MapCanvas");
        assert!(
            !found,
            "Blizzard_MapCanvas must NOT surface on glue screen {screen:?} — the \
             allows_screen filter at src/loader/mod.rs:527 strips it from BOTH addons + \
             lod_pool BEFORE pull_required_lod_addons runs (the missing `## AllowLoad:` \
             routes to Game-only at src/toc.rs:311). No glue-screen eager addon RequiredDeps \
             MapCanvas, so even ignoring the allows_screen filter, the closure walk would \
             find no entry-point"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_map_canvas_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_MapCanvas")
                || message.contains("MapCanvas_DataProviderBase")
                || message.contains("MapCanvas_PinFrameLevelsManager")
                || message.contains("MapCanvas_ScrollContainerMixin")
                || message.contains("Blizzard_MapCanvasDetailLayer")
                || message.contains("MapCanvasMixin")
                || message.contains("MapCanvasScrollControllerMixin")
                || message.contains("MapCanvasDataProviderMixin")
                || message.contains("MapCanvasPinMixin")
                || message.contains("MapCanvasDetailLayerMixin")
                || message.contains("MapCanvasPinFrameLevelsManagerMixin")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_MapCanvas emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_map_canvas_is_addon_loaded_after_dep_edge_pull(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_MapCanvas')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_MapCanvas') must return true after the eager \
         Game-screen boot pipeline — proves the pull_required_lod_addons walk promoted the \
         LOD addon from the lod_pool to the loaded set via the Blizzard_WorldMap \
         RequiredDep edge, AND that the standard load_addon path then ran its source files \
         + registered the addon name in the loaded-set"
    );
}
}

prefork_full_ui_case! {
fn blizzard_map_canvas_publishes_canvas_mixin_with_callback_registry_inheritance(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(MapCanvasMixin)")
        .expect("MapCanvasMixin type probe succeeds");
    assert_eq!(
        kind, "table",
        "MapCanvasMixin must publish at `_G` as a table — declared at \
         Blizzard_MapCanvas.lua:1 as `MapCanvasMixin = CreateFromMixins(CallbackRegistryMixin)`. \
         The CreateFromMixins shallow-copies CallbackRegistryMixin's methods (OnLoad, \
         GenerateCallbackEvents, RegisterCallback, TriggerEvent, etc) so the canvas inherits \
         the cross-frame event bus pattern. Consumed by `mixin=\"MapCanvasMixin\"` on \
         MapCanvasFrameTemplate (Blizzard_MapCanvas.xml:52)"
    );

    let onload_kind: String = env
        .eval("return type(MapCanvasMixin.OnLoad)")
        .expect("MapCanvasMixin.OnLoad type probe succeeds");
    assert_eq!(
        onload_kind, "function",
        "MapCanvasMixin.OnLoad must be a function — the canvas's lifecycle entry point \
         (chains CallbackRegistryMixin OnLoad, initializes pin pools / data providers / \
         frame-levels manager / area-trigger registry / pin nudging dirty flags / \
         debug-area-trigger pool / global click handlers / cursor handlers / mask textures)"
    );

    let trigger_event_kind: String = env
        .eval("return type(MapCanvasMixin.TriggerEvent)")
        .expect("MapCanvasMixin.TriggerEvent type probe succeeds");
    assert_eq!(
        trigger_event_kind, "function",
        "MapCanvasMixin.TriggerEvent must be a function — inherited from \
         CallbackRegistryMixin via the CreateFromMixins shallow copy. Proves the inheritance \
         chain landed: data providers / pins fire events on the canvas via this method, and \
         consumers (like Blizzard_WorldMap) RegisterCallback to hook them"
    );
}
}

prefork_full_ui_case! {
fn blizzard_map_canvas_mixin_carries_full_method_surface(env: &WowLuaEnv) {

    let method_count = count_mixin_methods(&env, "MapCanvasMixin");
    assert!(
        method_count >= 123,
        "MapCanvasMixin must carry at least 123 own methods (the count of \
         `function MapCanvasMixin:...` declarations in Blizzard_MapCanvas.lua). Got \
         {method_count}. The actual Lua-runtime count may exceed 123 because \
         CreateFromMixins(CallbackRegistryMixin) pre-seeds the table with 5+ inherited \
         methods (OnLoad / RegisterCallback / TriggerEvent / etc)"
    );

    assert_mixin_methods_present(
        &env,
        "MapCanvasMixin",
        &[
            "OnUpdate",
            "SetMapID",
            "GetMapID",
            "AddDataProvider",
            "RemoveDataProvider",
            "ZoomIn",
            "ZoomOut",
            "ResetZoom",
            "PanTo",
            "PanAndZoomTo",
            "GetCanvasScale",
            "GetCanvasZoomPercent",
            "GetGlobalPosition",
            "GetNormalizedCursorPosition",
            "AddCanvasClickHandler",
            "RemoveCanvasClickHandler",
            "GetPinFrameLevelsManager",
            "AcquireAreaTrigger",
            "AddMaskableTexture",
            "RefreshAll",
        ],
        "covers a canonical slice of the canvas API surface (lifecycle, map ID, data \
         providers, zoom, pan, coordinate space, click handlers, frame-level allocator, \
         area triggers, mask textures, refresh)",
    );
}
}

prefork_full_ui_case! {
fn blizzard_map_canvas_publishes_scroll_controller_mixin(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(MapCanvasScrollControllerMixin)")
        .expect("MapCanvasScrollControllerMixin type probe succeeds");
    assert_eq!(
        kind, "table",
        "MapCanvasScrollControllerMixin must publish at `_G` as a table — declared at \
         MapCanvas_ScrollContainerMixin.lua:1. Consumed by \
         `mixin=\"MapCanvasScrollControllerMixin\"` on MapCanvasFrameScrollContainerTemplate \
         (Blizzard_MapCanvas.xml:21), the ScrollFrame that wraps the inner Canvas frame and \
         drives mouse-wheel zoom + click-and-drag pan + momentum + zoom-target lerp"
    );

    let method_count = count_mixin_methods(&env, "MapCanvasScrollControllerMixin");
    assert!(
        method_count >= 76,
        "MapCanvasScrollControllerMixin must carry at least 76 methods (the count of \
         `function MapCanvasScrollControllerMixin:...` declarations in \
         MapCanvas_ScrollContainerMixin.lua). Got {method_count}. Drives the full pan / \
         zoom / mouse / scroll / cursor / clicks state machine"
    );

    assert_mixin_methods_present(
        &env,
        "MapCanvasScrollControllerMixin",
        &[
            "OnLoad",
            "OnHide",
            "OnMouseUp",
            "OnMouseDown",
            "OnMouseWheel",
            "OnUpdate",
            "GetCursorPosition",
            "ZoomIn",
            "ZoomOut",
            "SetPanTarget",
        ],
        "wired to the ScrollFrame's XML script handlers (OnLoad / OnHide / OnMouseUp / \
         OnMouseDown / OnMouseWheel / OnUpdate at Blizzard_MapCanvas.xml:43-49) or called \
         by the outer MapCanvasMixin to delegate pan / zoom",
    );
}
}

prefork_full_ui_case! {
fn blizzard_map_canvas_publishes_pin_frame_levels_manager_mixin(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(MapCanvasPinFrameLevelsManagerMixin)")
        .expect("MapCanvasPinFrameLevelsManagerMixin type probe succeeds");
    assert_eq!(
        kind, "table",
        "MapCanvasPinFrameLevelsManagerMixin must publish at `_G` as a table — declared at \
         MapCanvas_PinFrameLevelsManager.lua:1. Tracks the contiguous frame-level allocator \
         (default range 2000..2000, expandable in either direction up to MAX_FRAME_LEVEL=9000 \
         or down to MIN_FRAME_LEVEL=0). Each map-pin template gets a named frame-level type \
         (PIN_FRAME_LEVEL_QUEST, PIN_FRAME_LEVEL_VIGNETTE, etc) registered via AddFrameLevel \
         / InsertFrameLevelAbove / InsertFrameLevelBelow"
    );

    for method in MAP_CANVAS_PIN_FRAME_LEVELS_MANAGER_METHODS {
        let method_kind: String = env
            .eval(&format!(
                "return type(MapCanvasPinFrameLevelsManagerMixin.{method})"
            ))
            .unwrap_or_else(|err| {
                panic!("MapCanvasPinFrameLevelsManagerMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            method_kind, "function",
            "MapCanvasPinFrameLevelsManagerMixin.{method} must be a function — covers the \
             11-method allocator surface (Initialize seeds the default range, \
             ValidateContiguous walks the range looking for gaps, AddDefinition slots a new \
             range above or below the default and shifts neighbors, AddFrameLevel / \
             InsertFrameLevelAbove / InsertFrameLevelBelow are the public entry-points, \
             SetOverride / ClearOverride redirect a frame-level type to another's range, \
             GetFrameLevelStart / GetFrameLevelRange / GetValidFrameLevel are the lookup \
             paths consumed by pin Init/OnAcquired)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_map_canvas_publishes_data_provider_mixin(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(MapCanvasDataProviderMixin)")
        .expect("MapCanvasDataProviderMixin type probe succeeds");
    assert_eq!(
        kind, "table",
        "MapCanvasDataProviderMixin must publish at `_G` as a table — declared at \
         MapCanvas_DataProviderBase.lua:2. The base mixin every map data provider \
         (Blizzard_SharedMapDataProviders + per-map providers in Blizzard_WorldMap / \
         Blizzard_FlightMap / Blizzard_BattlefieldMap) extends via CreateFromMixins. \
         Provides the canvas-binding API (OnAdded / OnRemoved / RefreshAllData / \
         RegisterEvent / UnregisterEvent / GetMap) so consumer providers can plug into the \
         MapCanvas event flow"
    );

    let method_count = count_mixin_methods(&env, "MapCanvasDataProviderMixin");
    assert!(
        method_count >= 23,
        "MapCanvasDataProviderMixin must carry at least 23 methods (the count of \
         `function MapCanvasDataProviderMixin:...` declarations in \
         MapCanvas_DataProviderBase.lua). Got {method_count}"
    );

    assert_mixin_methods_present(
        &env,
        "MapCanvasDataProviderMixin",
        &[
            "OnAdded",
            "OnRemoved",
            "OnShow",
            "OnHide",
            "RefreshAllData",
            "GetMap",
            "RegisterEvent",
            "UnregisterEvent",
            "OnEvent",
        ],
        "part of the canonical data-provider lifecycle / event-binding contract",
    );
}
}

prefork_full_ui_case! {
fn blizzard_map_canvas_publishes_cvar_data_provider_mixin_inheriting_base(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(CVarMapCanvasDataProviderMixin)")
        .expect("CVarMapCanvasDataProviderMixin type probe succeeds");
    assert_eq!(
        kind, "table",
        "CVarMapCanvasDataProviderMixin must publish at `_G` as a table — declared at \
         MapCanvas_DataProviderBase.lua:165 as \
         `CreateFromMixins(MapCanvasDataProviderMixin)`. The CVar-driven variant shows / \
         hides the data provider based on a CVar (e.g. showQuestObjectives, \
         showMapPlayerLocation). The CreateFromMixins shallow-copy carries the base \
         data-provider surface forward, then OnShow/OnHide/OnEvent override the lifecycle \
         to gate on the CVar value"
    );

    for method in CVAR_DATA_PROVIDER_METHODS {
        let method_kind: String = env
            .eval(&format!(
                "return type(CVarMapCanvasDataProviderMixin.{method})"
            ))
            .unwrap_or_else(|err| {
                panic!("CVarMapCanvasDataProviderMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            method_kind, "function",
            "CVarMapCanvasDataProviderMixin.{method} must be a function — Init carries the \
             CVar-name registration, IsCVarSet reads GetCVarBool, OnShow/OnHide/OnEvent \
             dispatch RefreshAllData when the CVar flips. Hooks the CVAR_UPDATE event so \
             external CVar mutations propagate"
        );
    }

    let inherits_base: bool = env
        .eval("return type(CVarMapCanvasDataProviderMixin.RefreshAllData) == 'function'")
        .expect("CVarMapCanvasDataProviderMixin.RefreshAllData probe succeeds");
    assert!(
        inherits_base,
        "CVarMapCanvasDataProviderMixin.RefreshAllData must inherit from \
         MapCanvasDataProviderMixin via CreateFromMixins shallow-copy. Proves the inheritance \
         chain landed: the CVar variant gets the base provider's RefreshAllData / GetMap / \
         RegisterEvent surface for free"
    );
}
}

prefork_full_ui_case! {
fn blizzard_map_canvas_publishes_pin_mixin_inheriting_taggable_object(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(MapCanvasPinMixin)")
        .expect("MapCanvasPinMixin type probe succeeds");
    assert_eq!(
        kind, "table",
        "MapCanvasPinMixin must publish at `_G` as a table — declared at \
         MapCanvas_DataProviderBase.lua:193 as `CreateFromMixins(TaggableObjectMixin)`. \
         The base mixin every map pin extends; provides position / display ID / tooltip \
         / template-type / pin-collision / nudging / suppression API consumed by every \
         per-pin template across the WorldMap / FlightMap / BattlefieldMap data providers"
    );

    let method_count = count_mixin_methods(&env, "MapCanvasPinMixin");
    assert!(
        method_count >= 72,
        "MapCanvasPinMixin must carry at least 72 own methods (the count of \
         `function MapCanvasPinMixin:...` declarations in MapCanvas_DataProviderBase.lua). \
         Got {method_count}. The inherited TaggableObjectMixin methods (AddTag / HasTag / \
         RemoveTag / EnumerateTags) lift the count further at runtime"
    );

    assert_mixin_methods_present(
        &env,
        "MapCanvasPinMixin",
        &[
            "OnLoad",
            "OnAcquired",
            "OnReleased",
            "SetPosition",
            "GetPosition",
            "GetMap",
            "PanTo",
            "PanAndZoomTo",
            "ApplyFrameLevel",
            "GetGlobalPosition",
        ],
        "covers the canonical pin lifecycle (OnLoad/OnAcquired/OnReleased), positioning \
         (SetPosition/GetPosition/GetGlobalPosition), camera-routing (PanTo/PanAndZoomTo \
         delegate to the ScrollContainer), parent map binding (GetMap), frame-level \
         allocation (ApplyFrameLevel calls into MapCanvasPinFrameLevelsManagerMixin)",
    );
}
}

prefork_full_ui_case! {
fn blizzard_map_canvas_publishes_detail_layer_mixin(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(MapCanvasDetailLayerMixin)")
        .expect("MapCanvasDetailLayerMixin type probe succeeds");
    assert_eq!(
        kind, "table",
        "MapCanvasDetailLayerMixin must publish at `_G` as a table — declared at \
         Blizzard_MapCanvasDetailLayer.lua:2. Consumed by `mixin=\"MapCanvasDetailLayerMixin\"` \
         on MapCanvasDetailLayerTemplate (Blizzard_MapCanvas.xml:7). Drives the per-layer \
         tile-pool acquisition + texture-load-group tracking + map-art-ID phasing for the \
         sliced detail-tile texture grid"
    );

    assert_mixin_methods_present(
        &env,
        "MapCanvasDetailLayerMixin",
        MAP_CANVAS_DETAIL_LAYER_METHODS,
        "covers the 11-method surface (OnLoad / OnUpdate XML script hooks at xml:9-10, \
         SetMapAndLayer dispatches RefreshDetailTiles, IsFullyLoaded checks textureLoadGroup, \
         SetLayerAlpha / GetLayerAlpha / SetGlobalAlpha / GetGlobalAlpha drive the dual alpha \
         multiply, RefreshDetailTiles re-acquires the tile pool, RefreshAlpha gates on \
         isWaitingForLoad)",
    );
}
}

prefork_full_ui_case! {
fn blizzard_map_canvas_publishes_squared_distance_free_function(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(SquaredDistanceBetweenPoints)")
        .expect("SquaredDistanceBetweenPoints type probe succeeds");
    assert_eq!(
        kind, "function",
        "SquaredDistanceBetweenPoints must publish at `_G` as a function — declared at \
         Blizzard_MapCanvas.lua:493 as a top-level (non-mixin-method) helper. Used by the \
         pin-nudging algorithm (CalculatePinNudging at line 500) to compute pairwise \
         pin-to-pin distances without paying the sqrt cost — comparing squared distances \
         is sufficient for the closeness test"
    );

    let result: f64 = env
        .eval("return SquaredDistanceBetweenPoints(0, 0, 3, 4)")
        .expect("SquaredDistanceBetweenPoints sample probe succeeds");
    assert_eq!(
        result, 25.0,
        "SquaredDistanceBetweenPoints(0, 0, 3, 4) must equal 25 — the canonical 3-4-5 \
         right triangle has squared hypotenuse 9 + 16 = 25. Verifies the function performs \
         the actual squared-distance math (dx*dx + dy*dy) rather than e.g. accidentally \
         returning the unsquared sum"
    );
}
}

prefork_full_ui_case! {
fn blizzard_map_canvas_constants_local_to_pin_frame_levels_module(env: &WowLuaEnv) {

    for name in [
        "MAP_CANVAS_PIN_FRAME_LEVEL_DEFAULT",
        "MAX_FRAME_LEVEL",
        "MIN_FRAME_LEVEL",
    ] {
        let kind: String = env
            .eval(&format!("return type({name})"))
            .unwrap_or_else(|err| panic!("{name} type probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "{name} must remain unexposed — declared at \
             MapCanvas_PinFrameLevelsManager.lua:3-5 as `local` constants (file-scoped). \
             The frame-level manager owns the 0..9000 range allocator privately; the bounds \
             only matter inside the manager's add / insert / validate methods. Other addons \
             that probe `_G` would see nil"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_map_canvas_world_map_frame_inherits_canvas_template(env: &WowLuaEnv) {

    let world_map_kind: String = env
        .eval("return type(WorldMapFrame)")
        .expect("WorldMapFrame type probe succeeds");
    assert_eq!(
        world_map_kind, "table",
        "WorldMapFrame must publish at `_G` as a frame — created by Blizzard_WorldMap.xml's \
         `<Frame name=\"WorldMapFrame\" inherits=\"MapCanvasFrameTemplate\">`. The fact that \
         WorldMapFrame instantiates without error proves Blizzard_MapCanvas.xml's \
         `MapCanvasFrameTemplate` (and its embedded MapCanvasFrameScrollContainerTemplate, \
         registered via the same XML parse) populated the simulator's template registry. \
         If any of those virtual templates failed to register, Blizzard_WorldMap.xml would \
         fail to instantiate and either omit WorldMapFrame from `_G` or emit a Lua error \
         during load"
    );

    let scroll_container_kind: String = env
        .eval("return type(WorldMapFrame.ScrollContainer)")
        .expect("WorldMapFrame.ScrollContainer probe succeeds");
    assert_eq!(
        scroll_container_kind, "table",
        "WorldMapFrame.ScrollContainer must resolve to the embedded ScrollFrame child — \
         MapCanvasFrameTemplate (Blizzard_MapCanvas.xml:52) declares a child Frame block \
         that inherits MapCanvasFrameScrollContainerTemplate (xml:21) with \
         parentKey=\"ScrollContainer\". A live ScrollContainer reference proves the nested \
         virtual-template inheritance chain (MapCanvasFrameTemplate → \
         MapCanvasFrameScrollContainerTemplate) wired up correctly during XML parse"
    );
}
}

prefork_full_ui_case! {
fn blizzard_map_canvas_pin_frame_levels_manager_default_range_seeds_at_2000(env: &WowLuaEnv) {

    let start: i64 = env
        .eval(
            "local mgr = CreateFromMixins(MapCanvasPinFrameLevelsManagerMixin) \
             mgr:Initialize() \
             return mgr:GetFrameLevelStart('PIN_FRAME_LEVEL_DEFAULT')",
        )
        .expect("PIN_FRAME_LEVEL_DEFAULT start probe succeeds");
    assert_eq!(
        start, 2000,
        "Initialize must seed PIN_FRAME_LEVEL_DEFAULT.startLevel = 2000 — the \
         `MAP_CANVAS_PIN_FRAME_LEVEL_DEFAULT` local constant at \
         MapCanvas_PinFrameLevelsManager.lua:3. The default range gives the allocator room \
         to grow downward (towards 0) for below-default frame-level types AND upward \
         (towards 9000) for above-default types"
    );

    let range: i64 = env
        .eval(
            "local mgr = CreateFromMixins(MapCanvasPinFrameLevelsManagerMixin) \
             mgr:Initialize() \
             return mgr:GetFrameLevelRange('PIN_FRAME_LEVEL_DEFAULT')",
        )
        .expect("PIN_FRAME_LEVEL_DEFAULT range probe succeeds");
    assert_eq!(
        range, 1,
        "Initialize must seed PIN_FRAME_LEVEL_DEFAULT.range = 1 — the default frame-level \
         type holds a single slot (start = 2000, end = 2000). Sub-ranges expand the \
         allocator on either side, but the default itself stays minimal"
    );
}
}
