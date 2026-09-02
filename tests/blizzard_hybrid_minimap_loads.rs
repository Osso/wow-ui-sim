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

fn hybrid_minimap_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_HybridMinimap")
}

fn hybrid_minimap_toc() -> PathBuf {
    hybrid_minimap_dir().join("Blizzard_HybridMinimap.toc")
}

fn map_canvas_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_MapCanvas/Blizzard_MapCanvas.toc")
}

const MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "Enable",
    "Disable",
    "OnShow",
    "OnHide",
    "OnEvent",
    "OnUpdate",
    "CheckMap",
    "SetMapID",
    "GetMapID",
    "UpdateZoom",
    "SetZoom",
    "UpdatePosition",
];

fn load_hybrid_minimap_with_dependency(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &map_canvas_toc())
        .expect("Blizzard_MapCanvas should load via explicit Rust loader call");
    load_addon(&env.loader_env(), &hybrid_minimap_toc())
        .expect("Blizzard_HybridMinimap should load via explicit Rust loader call");
}

#[test]
fn blizzard_hybrid_minimap_find_toc_resolves_bare_variant() {
    let resolved =
        find_toc_file(&hybrid_minimap_dir()).expect("Blizzard_HybridMinimap TOC should resolve");
    assert_eq!(
        resolved,
        hybrid_minimap_toc(),
        "Blizzard_HybridMinimap ships exactly one bare TOC — LoadOnDemand minimap variant \
         resolves via `find_toc_file` fallthrough"
    );
}

#[test]
fn blizzard_hybrid_minimap_toc_declares_lod_with_one_required_dep() {
    let toc =
        TocFile::from_file(&hybrid_minimap_toc()).expect("Blizzard_HybridMinimap TOC should parse");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_HybridMinimap declares `## LoadOnDemand: 1` — the WorldMap / Plunderstorm code \
         calls LoadAddOn('Blizzard_HybridMinimap') on demand to swap the round circular minimap \
         for the rectangular MapCanvas-backed hybrid view; not loaded eagerly"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert_eq!(
        toc.dependencies(),
        vec!["Blizzard_MapCanvas".to_string()],
        "Blizzard_HybridMinimap declares `## RequiredDep: Blizzard_MapCanvas` — `RequiredDep` \
         resolves through the same `dependencies()` accessor as `Dependencies` (src/toc.rs:209-214). \
         MapCanvas must load first to publish MapCanvasFrameTemplate / MapCanvasMixin / \
         MapCanvasFrameScrollContainerTemplate / MapExplorationDataProviderMixin / \
         MapCanvasDataProviderMixin / GetPinFrameLevelsManager — every one of which the hybrid \
         minimap consumes via `inherits=` or via `CreateFromMixins(MapExplorationDataProviderMixin)`"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_HybridMinimap declares zero saved variables — the active mapID is recomputed \
         from C_Minimap.GetUiMapID() every CheckMap call, the zoom from C_Minimap.GetViewRadius(), \
         and the player position from C_Map.GetPlayerMapPosition; nothing persists across logins"
    );
}

#[test]
fn blizzard_hybrid_minimap_toc_omits_game_type_and_screen_restrictions() {
    let toc =
        TocFile::from_file(&hybrid_minimap_toc()).expect("Blizzard_HybridMinimap TOC should parse");

    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_HybridMinimap omits `## AllowLoadGameType` — `is_game_type_restricted()` \
         returns false because no restriction key is present (src/toc.rs:301 default branch). \
         Available across every mainline game type that loads this addon"
    );

    let raw =
        std::fs::read_to_string(hybrid_minimap_toc()).expect("Blizzard_HybridMinimap should read");
    assert!(
        !raw.contains("## AllowLoad:"),
        "Blizzard_HybridMinimap omits `## AllowLoad:` — falls through to the default Game-screen \
         policy at src/toc.rs:311 (`None => screen == ScreenKind::Game`)"
    );
    assert!(
        !raw.contains("## DefaultState:"),
        "Blizzard_HybridMinimap omits `## DefaultState:` — LoD addons need no DefaultState because \
         they are pulled in explicitly, not enabled by AddOnList"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "Blizzard_HybridMinimap omits every `## SavedVariables*` key — confirms the addon stores \
         no per-character or per-account state"
    );
}

#[test]
fn blizzard_hybrid_minimap_toc_lists_bootstrap_then_xml() {
    let toc =
        TocFile::from_file(&hybrid_minimap_toc()).expect("Blizzard_HybridMinimap TOC should parse");
    assert_eq!(
        toc.files
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        vec![
            "Blizzard_HybridMinimap_Bootstrap.lua",
            "Blizzard_HybridMinimap.xml",
        ],
        "TOC body must list the bootstrap before the XML. Blizzard_HybridMinimap.lua is NOT in \
         the TOC because the XML pulls it in via `<Script file=\"Blizzard_HybridMinimap.lua\"/>`; \
         the bootstrap is explicitly marked `[Bootstrap]` in the current retail TOC"
    );
}

#[test]
fn blizzard_hybrid_minimap_directory_holds_four_entries() {
    let entries = std::fs::read_dir(hybrid_minimap_dir())
        .expect("Blizzard_HybridMinimap directory should read")
        .count();
    assert_eq!(
        entries, 4,
        "Directory must hold exactly 4 entries: the TOC, bootstrap, lua, and XML files"
    );
}

#[test]
fn blizzard_hybrid_minimap_excluded_from_game_screen_auto_discovery() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_HybridMinimap");
    assert!(
        !in_game,
        "Blizzard_HybridMinimap must NOT appear in Game-screen auto-discovery — `## LoadOnDemand: 1` \
         excludes it from the eager pass at src/loader/mod.rs:527 unless another loaded addon \
         declares it as a dependency. No Blizzard addon declares HybridMinimap as a dep, so it \
         only loads via explicit LoadAddOn('Blizzard_HybridMinimap')"
    );
}

prefork_full_ui_case! {
fn blizzard_hybrid_minimap_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_hybrid_minimap_with_dependency(env);

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_HybridMinimap")
                || message.contains("HybridMinimapMixin")
                || message.contains("HybridMinimap.")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_HybridMinimap emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_hybrid_minimap_is_addon_loaded_via_explicit_load(env: &WowLuaEnv) {
    load_hybrid_minimap_with_dependency(env);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_HybridMinimap')")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_HybridMinimap') must return true after the explicit \
         load_addon call — confirms the loader registers the LoD addon with the loaded-set"
    );

    let canvas_loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_MapCanvas')")
        .expect("Blizzard_MapCanvas IsAddOnLoaded probe should succeed");
    assert!(
        canvas_loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_MapCanvas') must return true — the only declared dep \
         was loaded explicitly before HybridMinimap to publish the MapCanvas templates HybridMinimap \
         consumes"
    );
}
}

prefork_full_ui_case! {
fn blizzard_hybrid_minimap_publishes_mixin_with_thirteen_methods(env: &WowLuaEnv) {
    load_hybrid_minimap_with_dependency(env);

    let mixin_kind: String = env
        .eval("return type(HybridMinimapMixin)")
        .expect("HybridMinimapMixin probe should succeed");
    assert_eq!(
        mixin_kind, "table",
        "HybridMinimapMixin must publish at `_G` as a table — the addon's only public mixin, \
         attached to the named non-virtual `HybridMinimap` frame via `mixin=\"HybridMinimapMixin\"`"
    );

    for method in MIXIN_METHODS {
        let exists: bool = env
            .eval(&format!(
                "return type(HybridMinimapMixin['{method}']) == 'function'"
            ))
            .unwrap_or_else(|err| panic!("HybridMinimapMixin.{method} probe failed: {err}"));
        assert!(
            exists,
            "HybridMinimapMixin must expose `:{method}()` — one of the 13 methods covering 5 \
             XML-wired script handlers (OnLoad / OnShow / OnHide / OnEvent / OnUpdate) plus 8 \
             helpers (Enable / Disable lifecycle event registration, CheckMap polling \
             C_Minimap.GetUiMapID and gating Show/Hide, SetMapID caching content dimensions from \
             C_Map.GetMapArtLayers, GetMapID accessor, UpdateZoom computing pixels-per-yard from \
             C_Minimap.GetViewRadius and Minimap:GetWidth, SetZoom resizing MapCanvas + calling \
             ScrollContainer:InstantPanAndZoom, UpdatePosition centering the canvas via \
             C_Map.GetPlayerMapPosition)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_hybrid_minimap_named_frame_publishes_with_minimap_parent(env: &WowLuaEnv) {
    load_hybrid_minimap_with_dependency(env);

    let kind: String = env
        .eval("return type(HybridMinimap)")
        .expect("HybridMinimap probe should succeed");
    assert_eq!(
        kind, "table",
        "HybridMinimap must publish at `_G` as a table — the only named non-virtual frame this \
         addon ships, declared at Blizzard_HybridMinimap.xml:5 with `name=\"HybridMinimap\"` \
         `parent=\"Minimap\"` `mixin=\"HybridMinimapMixin\"`"
    );

    let name: String = env
        .eval("return HybridMinimap:GetName()")
        .expect("HybridMinimap:GetName() probe should succeed");
    assert_eq!(
        name, "HybridMinimap",
        "HybridMinimap:GetName() must echo the XML `name` attribute"
    );

    let hidden: bool = env
        .eval("return HybridMinimap:IsShown() == false")
        .expect("HybridMinimap:IsShown() probe should succeed");
    assert!(
        hidden,
        "HybridMinimap declares `hidden=\"true\"` (Blizzard_HybridMinimap.xml:5) — must start \
         hidden until :Enable() is called and :CheckMap() finds a valid C_Minimap.GetUiMapID()"
    );
}
}

prefork_full_ui_case! {
fn blizzard_hybrid_minimap_frame_carries_mapcanvas_child(env: &WowLuaEnv) {
    load_hybrid_minimap_with_dependency(env);

    let kind: String = env
        .eval("return type(HybridMinimap.MapCanvas)")
        .expect("HybridMinimap.MapCanvas probe should succeed");
    assert_eq!(
        kind, "table",
        "HybridMinimap.MapCanvas must publish as a parentKey child frame inheriting \
         `MapCanvasFrameTemplate` from Blizzard_MapCanvas — the actual map renderer the hybrid \
         minimap reuses to display rectangular map art clipped to the circular minimap mask"
    );
}
}

prefork_full_ui_case! {
fn blizzard_hybrid_minimap_frame_carries_circle_mask_overlay_texture(env: &WowLuaEnv) {
    load_hybrid_minimap_with_dependency(env);

    let kind: String = env
        .eval("return type(HybridMinimap.CircleMask)")
        .expect("HybridMinimap.CircleMask probe should succeed");
    assert_eq!(
        kind, "table",
        "HybridMinimap.CircleMask must publish as a parentKey MaskTexture (OVERLAY layer, \
         Blizzard_HybridMinimap.xml:13) using `Interface\\CharacterFrame\\TempPortraitAlphaMask` \
         with CLAMPTOBLACKADDITIVE wrap modes — masks the Background Texture so the rectangular \
         map content renders as a circle. OnLoad calls mapCanvas:SetMaskTexture(self.CircleMask)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_hybrid_minimap_frame_starts_at_background_strata(env: &WowLuaEnv) {
    load_hybrid_minimap_with_dependency(env);

    let strata: String = env
        .eval("return HybridMinimap:GetFrameStrata()")
        .expect("HybridMinimap:GetFrameStrata() probe should succeed");
    assert_eq!(
        strata, "BACKGROUND",
        "HybridMinimap declares `frameStrata=\"BACKGROUND\"` + `fixedFrameStrata=\"true\"` \
         (Blizzard_HybridMinimap.xml:5) — pinned to BACKGROUND so the rectangular map content \
         renders behind every other minimap overlay (POI pins, the rotation arrow, the outer \
         ring). `fixedFrameStrata` blocks any later SetFrameStrata override"
    );
}
}

#[test]
fn blizzard_hybrid_minimap_uses_xml_driven_script_loading() {
    let raw = std::fs::read_to_string(hybrid_minimap_dir().join("Blizzard_HybridMinimap.xml"))
        .expect("Blizzard_HybridMinimap.xml should read");
    assert!(
        raw.contains("<Script file=\"Blizzard_HybridMinimap.lua\"/>"),
        "Blizzard_HybridMinimap.xml must contain the `<Script file=\"Blizzard_HybridMinimap.lua\"/>` \
         declaration — the lua file is NOT listed in the TOC body, so XML-driven script loading is \
         the only path that registers HybridMinimapMixin. Without this directive HybridMinimap \
         (the named frame) would resolve to a mixin-less frame with no methods"
    );
}
