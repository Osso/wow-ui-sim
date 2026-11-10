use std::path::PathBuf;

use wow_ui_sim::loader::{discover_all_blizzard_addons, discover_blizzard_addons_for_screen};
use wow_ui_sim::loader::{find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn poi_button_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_POIButton")
}

fn poi_button_toc() -> PathBuf {
    poi_button_dir().join("Blizzard_POIButton.toc")
}

const POI_BUTTON_TOC_FILES: &[&str] = &[
    "POIButtonHighlightManager.lua",
    "POIButtonUtil.lua",
    "POIButton.lua",
    "POIButton.xml",
    "POIButtonOwner.lua",
    "POIButtonOwner.xml",
];

const PUBLIC_LIBRARIES: &[&str] = &["POIButtonHighlightManager", "POIButtonUtil"];

const PUBLIC_MIXINS: &[&str] = &[
    "POIButtonDisplayLayerMixin",
    "POIButtonMixin",
    "POIButtonOwnerMixin",
];

const VIRTUAL_TEMPLATES_NOT_IN_GLOBALS: &[&str] = &[
    "POIButtonDisplayLayerTemplate",
    "POIButtonTemplate",
    "POIButtonOwnerTemplate",
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

// Classic caches have zero Blizzard_POIButton files; TOC and discovery checks use only shipped profiles.
retail_ptr_forever_toc_fixture! {
#[test]
fn blizzard_poi_button_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&poi_button_dir()).expect("Blizzard_POIButton TOC resolves");
    assert_eq!(
        resolved,
        poi_button_toc(),
        "Blizzard_POIButton ships exactly one bare TOC — no `_Mainline.toc` variant. The \
         POI Button library is a foundational quest/content/area-POI/vignette button factory \
         consumed by the world map, objective tracker, super-track UI and minimap content \
         tracking; it has no flavor-specific behavior so a single bare TOC drives every \
         flavor that ships the addon"
    );

    let mainline = poi_button_dir().join("Blizzard_POIButton_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {} — the bare TOC is the canonical entry point",
        mainline.display()
    );
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn blizzard_poi_button_toc_declares_eager_in_game() {
    let toc = TocFile::from_file(&poi_button_toc()).expect("Blizzard_POIButton TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "TOC must NOT declare `## LoadOnDemand:` — eager-load on Game-screen so the \
         POIButtonTemplate / POIButtonOwnerTemplate pools are wired up before any quest \
         markers / world map pins / objective tracker rows materialize"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());

    assert!(
        !toc.is_game_type_restricted(),
        "TOC must NOT declare `## AllowLoadGameType:` — POIButton is a foundational \
         super-track UI library used by every flavor of the live retail client (mainline). \
         `is_game_type_restricted` at src/toc.rs:294-302 returns false when no \
         AllowLoadGameType token is set"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: Game` must enable Game screen — the POI Button library hooks the \
         super-track event registry, queries C_QuestLog / C_SuperTrack / C_ContentTracking \
         / C_QuestLog.GetQuestTagInfo all of which are in-world only"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "`## AllowLoad: Game` must NOT enable {screen:?} — quest super-tracking has no \
             meaning on glue screens (no QuestCache, no C_SuperTrack state)"
        );
    }

    let dependencies = toc.dependencies();
    let deps: Vec<&str> = dependencies.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        deps,
        vec!["Blizzard_FrameXMLBase"],
        "TOC must declare exactly `## Dependencies: Blizzard_FrameXMLBase` — the FrameXMLBase \
         core publishes Pool_HideAndClearAnchors / CreateFramePool (consumed by \
         POIButtonOwnerMixin:Init via `CreateFramePool('Button', self, 'POIButtonTemplate', \
         HideAndClearAnchorsWithReset)`), PixelUtil.SetPoint (consumed by \
         POIButtonDisplayLayerMixin:UpdatePoint for sub-pixel-accurate centering), \
         GameTooltip_SetTitle / GameTooltip_AddNormalLine, GetAppropriateTooltip (used by \
         POIButtonMixin:OnEnter to pick the right tooltip frame for the QUEST_SESSION \
         on-hold tooltip), TextureKitConstants.UseAtlasSize / IgnoreAtlasSize (consumed by \
         the underlay-banner / sub-type-icon / lock-icon factories in POIButton.lua's \
         do-block at lines 803-906), and the QuestSuperTracking_GetSuperTrackedQuestID / \
         GetSuperTrackedContent / GetSuperTrackedMapPin / GetSuperTrackedVignette helpers"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` — no soft sibling addons; the cross-addon refs (QuestCache, \
         QuestUtil, QuestUtils_IsQuestWatched, QuestSuperTracking_*, ChatFrameUtil, \
         ContentTrackingUtil, EventRegistry) all come from the FrameXML core which \
         FrameXMLBase indirectly drags in"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — pure stateless library; super-track state is server-driven \
         via Supertracking.OnChanged events; quest watch state is C_QuestLog-managed; \
         highlight state is in-memory transient (POIButtonHighlightManager.questID)"
    );
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn blizzard_poi_button_toc_declares_metadata_in_raw_bytes() {
    let raw =
        std::fs::read_to_string(poi_button_toc()).expect("Blizzard_POIButton TOC reads utf-8");
    assert!(
        raw.contains("## Title: Blizzard_POIButton"),
        "TOC must declare `## Title: Blizzard_POIButton` exactly — underscored-camelcase \
         form mirroring the addon directory name (NOT space-and-prose like \
         `Blizzard Plunderstorm Basics`); this is a library-flavored title for a \
         developer-facing component"
    );
    assert!(
        raw.contains("## Author: Blizzard Entertainment"),
        "TOC must declare `## Author: Blizzard Entertainment` exactly"
    );
    assert!(
        raw.contains("## DefaultState: enabled"),
        "TOC must declare `## DefaultState: enabled` exactly — the POI Button library is \
         non-optional; disabling it would break the world map / objective tracker / super- \
         track UI consumers"
    );
    assert!(
        raw.contains("## Dependencies: Blizzard_FrameXMLBase"),
        "TOC must declare `## Dependencies: Blizzard_FrameXMLBase` exactly — single hard \
         dep; Blizzard_FrameXMLBase publishes the FramePool / PixelUtil / TextureKitConstants \
         / GameTooltip helpers consumed by POIButton.lua / POIButtonOwner.lua"
    );
    assert!(
        raw.contains("## AllowLoad: Game"),
        "TOC must declare `## AllowLoad: Game` exactly — in-world UI only, not glue"
    );
    assert!(
        !raw.contains("## LoadOnDemand"),
        "TOC must NOT declare `## LoadOnDemand:` — eager-load on Game-screen"
    );
    assert!(
        !raw.contains("## AllowLoadGameType"),
        "TOC must NOT declare `## AllowLoadGameType:` — foundational POI library, not \
         flavor-restricted"
    );
    assert!(
        !raw.contains("## RequiredDep"),
        "TOC must NOT declare `## RequiredDep:` or `## RequiredDeps:` — `Dependencies:` is \
         the canonical key"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT declare any `## SavedVariables*` keys — pure stateless library"
    );
    assert!(
        !raw.contains("## OptionalDeps"),
        "TOC must NOT declare any `## OptionalDeps:` — zero soft siblings"
    );
    assert!(
        !raw.contains("## UseSecureEnvironment"),
        "TOC must NOT declare `## UseSecureEnvironment:` — display-only / non-secure"
    );
    assert!(
        !raw.contains("## Version"),
        "TOC must NOT declare `## Version:` — unversioned"
    );
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn blizzard_poi_button_toc_lists_six_files_in_canonical_order() {
    let toc = TocFile::from_file(&poi_button_toc()).expect("Blizzard_POIButton TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, POI_BUTTON_TOC_FILES,
        "TOC body must list exactly 6 files in canonical dependency order: \
         POIButtonHighlightManager.lua FIRST (declares `POIButtonHighlightManager = {{}}` \
         which POIButtonMixin:OnEnter / OnLeave reference for managed-highlight state); \
         POIButtonUtil.lua SECOND (declares `POIButtonUtil = {{}}` with .Type / .Style \
         enums + .GetStyle / .GetTypeFromStyle / .ShowLegendGlow / .HideLegendGlow that \
         POIButton.lua and external consumers depend on); POIButton.lua THIRD (declares \
         POIButtonDisplayLayerMixin + POIButtonMixin); POIButton.xml FOURTH (defines \
         POIButtonDisplayLayerTemplate + POIButtonTemplate which both reference the \
         already-declared mixins); POIButtonOwner.lua FIFTH (declares POIButtonOwnerMixin \
         which CreateFramePool's the POIButtonTemplate from POIButton.xml); \
         POIButtonOwner.xml SIXTH (defines POIButtonOwnerTemplate as a `virtual=\"true\"` \
         Frame with mixin=POIButtonOwnerMixin)"
    );
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn blizzard_poi_button_appears_in_game_screen_eager_discovery() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let found = addons.iter().any(|(name, _)| name == "Blizzard_POIButton");
    assert!(
        found,
        "Blizzard_POIButton must appear in eager discovery for Game-screen — the addon is \
         not LoadOnDemand and not game-type restricted; it should auto-load before any \
         consumer addon (Blizzard_WorldMap, Blizzard_ObjectiveTracker, \
         Blizzard_QuestSuperTracking) needs the POIButtonTemplate / POIButtonOwnerTemplate"
    );
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn blizzard_poi_button_does_not_appear_in_glue_screen_eager_discovery() {
    let ui = blizzard_ui_dir();

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let found = addons.iter().any(|(name, _)| name == "Blizzard_POIButton");
        assert!(
            !found,
            "Blizzard_POIButton must NOT appear in eager discovery for {screen:?} — \
             `## AllowLoad: Game` blocks the addon from glue screens because the super- \
             tracking + quest-cache state machine has no meaning outside the in-world \
             session"
        );
    }
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn blizzard_poi_button_appears_in_full_addon_inventory() {
    let inventory = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = inventory
        .iter()
        .any(|(name, _)| name == "Blizzard_POIButton");
    assert!(
        found,
        "Blizzard_POIButton must appear in `discover_all_blizzard_addons` — the unfiltered \
         inventory at src/loader/mod.rs:309-343 lists every parseable Blizzard_* TOC \
         regardless of screen / game-type / LoadOnDemand filters"
    );
}
}

prefork_full_ui_case! {
fn blizzard_poi_button_is_addon_loaded_after_game_screen_boot(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_POIButton')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_POIButton') must return true after the eager \
         Game-screen sweep — the addon is auto-discovered and loaded as part of the \
         standard Game-screen pool (no explicit load_addon call needed)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_poi_button_publishes_two_static_libraries(env: &WowLuaEnv) {

    for library in PUBLIC_LIBRARIES {
        let kind: String = env
            .eval(&format!("return type(_G.{library})"))
            .unwrap_or_else(|err| panic!("type(_G.{library}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{library} must publish as a table — POIButtonHighlightManager owns the \
             single highlight-quest-id slot (SetHighlight / ClearHighlight / HasHighlight \
             / GetQuestID, fires EventRegistry SetHighlightedQuestPOI / \
             ClearHighlightedQuestPOI events when the slot transitions); POIButtonUtil owns \
             the Type {{Custom=1, Quest=2, Content=3, AreaPOI=4, Vignette=5}} and Style \
             {{Waypoint=1, QuestInProgress=2, QuestComplete=3, QuestDisabled=4, \
             QuestThreat=5, ContentTracking=6, WorldQuest=7, BonusObjective=9, \
             AreaPOI=10, Vignette=11}} enum tables (note: BonusObjective is value 9 — \
             value 8 is intentionally skipped to leave room for a future enum) plus the \
             styleXType lookup that GetTypeFromStyle queries, and the LegendGlow factory \
             helpers ShowLegendGlow / HideLegendGlow"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_poi_button_publishes_three_mixins(env: &WowLuaEnv) {

    for mixin in PUBLIC_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{mixin})"))
            .unwrap_or_else(|err| panic!("type(_G.{mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table — the 3 mixins back the POI button \
             surface: POIButtonDisplayLayerMixin (the inner display-layer Frame inside \
             every POIButton, owns SetOffset / UpdatePoint for sub-pixel mouse-down \
             nudging via PixelUtil.SetPoint, UpdateInProgress / SetTextureSize / SetAtlas \
             / SetIconShown / IsIconShow / GetPinScale; rendered as a parentKey=\"Display\" \
             child of POIButtonTemplate); POIButtonMixin (the Button itself with 70+ \
             methods covering OnShow/OnHide registering Supertracking.OnChanged via \
             EventRegistry, OnMouseDown/OnMouseUp pixel-nudging Pushed/Highlight/Glow \
             textures by 1px, OnClick branching across 4 trackable kinds (questID via \
             C_QuestLog.AddQuestWatch + C_SuperTrack.SetSuperTrackedQuestID, trackable \
             content via C_ContentTracking.IsTracking + C_SuperTrack.SetSuperTrackedContent, \
             areaPOI via C_SuperTrack.SetSuperTrackedMapPin, vignette via \
             C_SuperTrack.SetSuperTrackedVignette), 11 atlas-resolver helper functions \
             selecting the right NumberIcons / Campaign / Legendary / Important / Meta / \
             Recurring / BonusObjective / AreaPOI / ContentTracking / QuestComplete \
             quest-classification atlas family, UpdateButtonStyle / UpdateUnderlay / \
             UpdateSubTypeIcon / UpdateLockIcon lazy-creating extra textures via the \
             CheckCreateExtraTexture closure factory, OnSuperTrackingChanged dispatching \
             through a 4-handler superTrackerChangeHandlers table keyed by Type.Quest / \
             Type.Content / Type.AreaPOI / Type.Vignette, Reset clearing all per-instance \
             state for FramePool reuse); POIButtonOwnerMixin (the FramePool wrapper for \
             POIButtonTemplate — Init creates `CreateFramePool('Button', self, \
             'POIButtonTemplate', HideAndClearAnchorsWithReset)`, FindButtonByQuestID / \
             FindButtonByTrackable iterates EnumerateActive, SelectButton / \
             SelectSuperTrackedButton / ClearSelection updates the active super-tracked \
             button, GetButtonForQuest / GetButtonForTrackable / GetButtonForAreaPOI \
             acquires + initializes from the pool with style-specific setup)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_poi_button_util_exposes_style_and_type_enums(env: &WowLuaEnv) {

    let custom: i64 = env
        .eval("return POIButtonUtil.Type.Custom")
        .expect("POIButtonUtil.Type.Custom resolves");
    assert_eq!(
        custom, 1,
        "POIButtonUtil.Type.Custom must equal 1 — the Type enum is {{Custom=1, Quest=2, \
         Content=3, AreaPOI=4, Vignette=5}}; consumers in C_SuperTrack dispatch and \
         POIButtonOwnerMixin.GetButtonFor* helpers branch on the type integer"
    );

    let world_quest: i64 = env
        .eval("return POIButtonUtil.Style.WorldQuest")
        .expect("POIButtonUtil.Style.WorldQuest resolves");
    assert_eq!(
        world_quest, 7,
        "POIButtonUtil.Style.WorldQuest must equal 7 — the Style enum has values \
         {{Waypoint=1, QuestInProgress=2, QuestComplete=3, QuestDisabled=4, QuestThreat=5, \
         ContentTracking=6, WorldQuest=7, BonusObjective=9, AreaPOI=10, Vignette=11}}; \
         note the gap at 8 (no enum value)"
    );

    let bonus_objective: i64 = env
        .eval("return POIButtonUtil.Style.BonusObjective")
        .expect("POIButtonUtil.Style.BonusObjective resolves");
    assert_eq!(
        bonus_objective, 9,
        "POIButtonUtil.Style.BonusObjective must equal 9 — explicit gap-after-7 (the \
         missing value 8 is intentional, leaving room for a future style without \
         renumbering existing entries)"
    );

    let mapped: i64 = env
        .eval("return POIButtonUtil.GetTypeFromStyle(POIButtonUtil.Style.QuestComplete)")
        .expect("GetTypeFromStyle for QuestComplete resolves");
    assert_eq!(
        mapped, 2,
        "POIButtonUtil.GetTypeFromStyle(Style.QuestComplete) must return Type.Quest (=2) \
         — the styleXType lookup at POIButtonUtil.lua:25-36 maps every quest-bucket Style \
         (QuestInProgress / QuestComplete / QuestDisabled / QuestThreat / WorldQuest / \
         BonusObjective) to Type.Quest"
    );
}
}

prefork_full_ui_case! {
fn blizzard_poi_button_highlight_manager_starts_empty(env: &WowLuaEnv) {

    let has_highlight: bool = env
        .eval("return POIButtonHighlightManager:HasHighlight()")
        .expect("HasHighlight probe succeeds");
    assert!(
        !has_highlight,
        "POIButtonHighlightManager:HasHighlight() must return false on a fresh load — the \
         manager starts with no highlighted quest; SetHighlight populates self.questID, \
         ClearHighlight wipes it, HasHighlight returns `self.questID ~= nil`"
    );

    let qid: Option<i64> = env
        .eval("return POIButtonHighlightManager:GetQuestID()")
        .expect("GetQuestID probe succeeds");
    assert!(
        qid.is_none(),
        "POIButtonHighlightManager:GetQuestID() must return nil on a fresh load"
    );
}
}

prefork_full_ui_case! {
fn blizzard_poi_button_does_not_leak_virtual_templates_to_globals(env: &WowLuaEnv) {

    for template in VIRTUAL_TEMPLATES_NOT_IN_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G.{template})"))
            .unwrap_or_else(|err| panic!("type(_G.{template}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{template} must be nil — virtual templates (`virtual=\"true\"` on the XML \
             element) live in the template registry only, not in globals. \
             POIButtonDisplayLayerTemplate (the Display child Frame with mixin=\
             POIButtonDisplayLayerMixin and a parentKey=Icon Texture in ARTWORK / \
             textureSubLevel=1); POIButtonTemplate (the main Button with mixin=\
             POIButtonMixin, KeyValue shouldShowGlow=true, OVERLAY HighlightTexture + \
             BACKGROUND Glow + NormalTexture + PushedTexture all from \
             Interface\\WorldMap\\UI-QuestPoi-NumberIcons, parentKey=Display child); \
             POIButtonOwnerTemplate (the FramePool-owning Frame with mixin=\
             POIButtonOwnerMixin — empty body, all behavior comes from the mixin)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_poi_button_does_not_publish_named_non_virtual_frames(env: &WowLuaEnv) {

    for name in [
        "POIButton",
        "POIButtonDisplayLayer",
        "POIButtonOwner",
        "POIButtonHighlight",
    ] {
        let kind: String = env
            .eval(&format!("return type(_G.{name})"))
            .unwrap_or_else(|err| panic!("type(_G.{name}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{name} must be nil — Blizzard_POIButton is a pure library addon: it \
             publishes 2 static libraries (POIButtonHighlightManager / POIButtonUtil), 3 \
             mixins (POIButtonDisplayLayerMixin / POIButtonMixin / POIButtonOwnerMixin), \
             and 3 virtual templates (POIButtonDisplayLayerTemplate / POIButtonTemplate / \
             POIButtonOwnerTemplate) but ZERO named non-virtual frames; consumers like \
             Blizzard_WorldMap or Blizzard_ObjectiveTracker instantiate the templates \
             themselves under their own pool"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_poi_button_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_POIButton")
                || message.contains("POIButton")
                || message.contains("POIButtonOwner")
                || message.contains("POIButtonUtil")
                || message.contains("POIButtonHighlightManager")
                || message.contains("POIButtonDisplayLayer")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_POIButton emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

// The Mists/Era/Anniversary caches ship 218 addons each and Wrath ships 23;
// none contains Blizzard_POIButton, so the pinned TOC facts above describe no file the
// classic loaders can read.
classic_cache_absent_toc_fixture! {
#[test]
fn blizzard_poibutton_toc_is_absent_from_classic_caches() {
    let dir = poi_button_dir();
    assert!(
        !dir.is_dir(),
        "classic profile caches must not contain Blizzard_POIButton"
    );
    assert!(wow_ui_sim::loader::find_toc_file(&dir).is_none());
}
}
