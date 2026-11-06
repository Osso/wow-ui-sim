use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::loader::{discover_all_blizzard_addons, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn quest_navigation_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_QuestNavigation")
}

fn quest_navigation_toc() -> PathBuf {
    quest_navigation_dir().join("Blizzard_QuestNavigation.toc")
}

const TOC_FILES: &[&str] = &["SuperTrackedFrame.xml"];

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

// Literal cached inputs absent from Wrath manifest1-382/cache; behavior cases stay enabled.
retail_ptr_forever_toc_fixture! {
#[test]
fn blizzard_quest_navigation_find_toc_resolves_bare_variant() {
    let resolved =
        find_toc_file(&quest_navigation_dir()).expect("Blizzard_QuestNavigation TOC resolves");
    assert_eq!(
        resolved,
        quest_navigation_toc(),
        "Blizzard_QuestNavigation ships a SINGLE bare `Blizzard_QuestNavigation.toc` \
         (NO `_Mainline.toc` variant). `find_toc_file` at src/loader/mod.rs:65-95 \
         walks the suffix-priority list `[_Mainline.toc, .toc]`, falling through to \
         the bare form because no Mainline-suffixed variant exists. This is the \
         baseline TOC layout most Blizzard_* addons use"
    );

    for variant_suffix in ["_Mainline.toc", "_Mists.toc", "_Cata.toc", "_Wrath.toc"] {
        let variant =
            quest_navigation_dir().join(format!("Blizzard_QuestNavigation{variant_suffix}"));
        assert!(
            !variant.exists(),
            "Blizzard_QuestNavigation must NOT ship a {variant_suffix} variant — \
             single bare TOC only"
        );
    }
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn blizzard_quest_navigation_toc_pins_eager_no_lod_no_deps() {
    let toc =
        TocFile::from_file(&quest_navigation_toc()).expect("Blizzard_QuestNavigation TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "TOC must NOT declare `## LoadOnDemand` — eager load: the SuperTrackedFrame \
         is the persistent on-screen quest-tracker arrow / icon that needs to be \
         wired up the moment the player enters the world"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(!toc.is_ptr_only());

    assert!(
        !toc.is_game_type_restricted(),
        "TOC must NOT declare `## AllowLoadGameType` — `is_game_type_restricted()` \
         returns FALSE because the metadata key is missing entirely. Cross-flavor: \
         the SuperTrack system exists on retail and most classic flavors"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "TOC has no `## AllowLoad` directive — defaults to Game-only via the \
         parser at src/toc.rs:306-313 (absent AllowLoad treated as Game-only)"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Game-only default must EXCLUDE {screen:?} — quest navigation runs \
             only in the world"
        );
    }

    assert!(
        toc.dependencies().is_empty(),
        "TOC must declare ZERO `## Dependencies:` — the SuperTrackedFrame stands \
         alone, hooking only the C_Navigation / C_SuperTrack core C API namespaces"
    );
    assert!(toc.optional_deps().is_empty());
    assert!(toc.load_with().is_empty());
    assert!(
        toc.saved_variables().is_empty(),
        "TOC must declare ZERO `## SavedVariables:` — pure stateless display widget"
    );
    assert!(toc.saved_variables_per_character().is_empty());
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn blizzard_quest_navigation_toc_declares_metadata_in_raw_bytes() {
    let raw = std::fs::read_to_string(quest_navigation_toc()).expect("TOC reads utf-8");

    assert!(raw.contains("## Title: Blizzard Quest Navigation"));
    assert!(raw.contains("## Author: Blizzard Entertainment"));
    assert!(raw.contains("## Version: 1.0"));
    assert!(
        !raw.contains("## LoadOnDemand"),
        "TOC must NOT carry any LoadOnDemand directive (eager-loaded)"
    );
    assert!(
        !raw.contains("## Dependencies"),
        "TOC must NOT carry any Dependencies directive"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT carry any SavedVariables directive"
    );
    assert!(
        !raw.contains("## AllowLoad"),
        "TOC must NOT carry any AllowLoad / AllowLoadGameType directive"
    );
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn blizzard_quest_navigation_toc_lists_one_xml_file() {
    let toc = TocFile::from_file(&quest_navigation_toc()).expect("TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, TOC_FILES,
        "TOC body lists EXACTLY ONE file — SuperTrackedFrame.xml. The matching \
         SuperTrackedFrame.lua is NOT in the TOC body; it is pulled in by the XML \
         file via `<Script file=\"SuperTrackedFrame.lua\"/>` at the top of the XML \
         body. This is the canonical XML-driven companion-Lua loading pattern \
         (same shape as Blizzard_PVPMatch's 3 XML+Lua pairs, just with a single \
         pair here)"
    );

    let lua_in_dir = quest_navigation_dir().join("SuperTrackedFrame.lua");
    assert!(
        lua_in_dir.exists(),
        "SuperTrackedFrame.lua must exist in the addon directory even though \
         the TOC body does not list it — the XML pulls it in"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_quest_navigation_appears_in_eager_game_discovery() {
    let ui = blizzard_ui_dir();

    let game_addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_QuestNavigation");
    assert!(
        in_game,
        "Blizzard_QuestNavigation MUST appear in Game-screen eager discovery — \
         eager (no LoadOnDemand), Game-only (default), unrestricted flavor"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let glue_addons = discover_blizzard_addons_for_screen(&ui, screen);
        let in_glue = glue_addons
            .iter()
            .any(|(name, _)| name == "Blizzard_QuestNavigation");
        assert!(
            !in_glue,
            "Blizzard_QuestNavigation MUST NOT appear in {screen:?} eager \
             discovery — Game-only default screen gate"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_quest_navigation_appears_in_full_addon_inventory() {
    let inventory = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = inventory
        .iter()
        .any(|(name, _)| name == "Blizzard_QuestNavigation");
    assert!(
        found,
        "Blizzard_QuestNavigation MUST appear in `discover_all_blizzard_addons`"
    );
}
}

prefork_full_ui_case! {
fn blizzard_quest_navigation_loads_cleanly_during_eager_game_sweep(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_QuestNavigation")
                || message.contains("SuperTrackedFrame")
                || message.contains("SuperTrackedFrameMixin")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_QuestNavigation emitted addon-specific Lua errors during the \
         eager Game-screen sweep:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_quest_navigation_publishes_super_tracked_frame_mixin(env: &WowLuaEnv) {

    let mixin_kind: String = env
        .eval("return type(_G.SuperTrackedFrameMixin)")
        .expect("type probe should succeed");
    assert_eq!(
        mixin_kind, "table",
        "_G.SuperTrackedFrameMixin must publish as a table — the addon ships ONE \
         single mixin defined at file scope (`SuperTrackedFrameMixin = {{}};` at \
         SuperTrackedFrame.lua:3) and assigned via XML `mixin=\"SuperTrackedFrameMixin\"` \
         to the SuperTrackedFrame frame; this mixin owns the entire navigation \
         arrow/icon state machine: clamped-vs-onscreen tracking, alpha fade, \
         distance-text formatting, party-member portrait icon switching"
    );

    let methods_present: bool = env
        .eval(
            "return type(SuperTrackedFrameMixin.OnLoad) == 'function' \
                and type(SuperTrackedFrameMixin.OnEvent) == 'function' \
                and type(SuperTrackedFrameMixin.OnUpdate) == 'function' \
                and type(SuperTrackedFrameMixin.UpdateClampedState) == 'function' \
                and type(SuperTrackedFrameMixin.UpdatePosition) == 'function' \
                and type(SuperTrackedFrameMixin.UpdateArrow) == 'function' \
                and type(SuperTrackedFrameMixin.UpdateDistanceText) == 'function' \
                and type(SuperTrackedFrameMixin.UpdateAlpha) == 'function' \
                and type(SuperTrackedFrameMixin.UpdateIcon) == 'function' \
                and type(SuperTrackedFrameMixin.OnDestinationReached) == 'function' \
                and type(SuperTrackedFrameMixin.InitializeNavigationFrame) == 'function' \
                and type(SuperTrackedFrameMixin.ShutdownNavigationFrame) == 'function'",
        )
        .expect("mixin method probe should succeed");
    assert!(
        methods_present,
        "SuperTrackedFrameMixin must publish the canonical 12-method surface — \
         3 script handlers (OnLoad/OnEvent/OnUpdate) wired via XML <Scripts> + 9 \
         body methods that the OnUpdate sequence calls in order each tick: \
         UpdateClampedState → UpdatePosition → UpdateArrow → UpdateDistanceText \
         → UpdateAlpha (the per-tick render pipeline) plus UpdateIcon (called on \
         SUPER_TRACKING_CHANGED), OnDestinationReached (NAVIGATION_DESTINATION_REACHED), \
         and the InitializeNavigationFrame/ShutdownNavigationFrame pair (called \
         on NAVIGATION_FRAME_CREATED/DESTROYED)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_quest_navigation_publishes_super_tracked_frame_global(env: &WowLuaEnv) {

    let frame_kind: String = env
        .eval("return type(_G.SuperTrackedFrame)")
        .expect("type probe should succeed");
    assert_eq!(
        frame_kind, "table",
        "_G.SuperTrackedFrame must publish as a frame — declared in \
         SuperTrackedFrame.xml as `<Frame name=\"SuperTrackedFrame\" \
         parent=\"UIParent\" frameStrata=\"BACKGROUND\" \
         mixin=\"SuperTrackedFrameMixin\">`. The frame is parented to UIParent \
         (NOT WorldFrame) at BACKGROUND strata so it sits below all UI panels \
         but above the 3D world; the XML <Size> declares a square frame of 100 \
         by 100 with an Icon texture (atlas Navigation-Tracked-Icon) at center, \
         an Arrow texture (atlas Navigation-Tracked-Arrow, hidden by default) \
         above the icon, and a DistanceText FontString below"
    );

    let (strata, parent_name, width, height): (String, String, f64, f64) = env
        .eval(
            "return SuperTrackedFrame:GetFrameStrata(), \
                SuperTrackedFrame:GetParent():GetName(), \
                SuperTrackedFrame:GetWidth(), \
                SuperTrackedFrame:GetHeight()",
        )
        .expect("frame property probe should succeed");
    assert_eq!(strata, "BACKGROUND");
    assert_eq!(parent_name, "UIParent");
    assert_eq!(width, 100.0);
    assert_eq!(height, 100.0);
}
}

prefork_full_ui_case! {
fn blizzard_quest_navigation_super_tracked_frame_has_named_textures(env: &WowLuaEnv) {

    let textures_present: bool = env
        .eval(
            "return type(SuperTrackedFrame.Icon) == 'table' \
                and type(SuperTrackedFrame.Arrow) == 'table' \
                and type(SuperTrackedFrame.IconBorder) == 'table' \
                and type(SuperTrackedFrame.DistanceText) == 'table'",
        )
        .expect("texture/fontstring probe should succeed");
    assert!(
        textures_present,
        "SuperTrackedFrame must publish 3 named textures + 1 FontString as \
         parentKey children: Icon (BACKGROUND layer, atlas Navigation-Tracked-Icon, \
         the main quest/party-member portrait), Arrow (BACKGROUND layer, atlas \
         Navigation-Tracked-Arrow, hidden by default — toggled when frame is \
         clamped to screen edge), IconBorder (OVERLAY layer, hidden by default), \
         and DistanceText (BACKGROUND-layer FontString anchored below the Icon \
         showing yards-to-target). The parentKey publishing is verified by \
         `SyncChildrenKeys` after XML parsing"
    );
}
}

prefork_full_ui_case! {
fn blizzard_quest_navigation_registers_navigation_events_in_onload(env: &WowLuaEnv) {

    let registered: bool = env
        .eval(
            "return SuperTrackedFrame:IsEventRegistered('NAVIGATION_FRAME_CREATED') \
                and SuperTrackedFrame:IsEventRegistered('NAVIGATION_FRAME_DESTROYED') \
                and SuperTrackedFrame:IsEventRegistered('NAVIGATION_DESTINATION_REACHED') \
                and SuperTrackedFrame:IsEventRegistered('SUPER_TRACKING_CHANGED')",
        )
        .expect("event registration probe should succeed");
    assert!(
        registered,
        "OnLoad must register exactly 4 navigation events — \
         NAVIGATION_FRAME_CREATED (the C_Navigation backend allocated a target \
         frame), NAVIGATION_FRAME_DESTROYED (target frame torn down), \
         NAVIGATION_DESTINATION_REACHED (player walked into the tracked target), \
         and SUPER_TRACKING_CHANGED (the player switched what they're \
         super-tracking via the quest log or map)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_quest_navigation_consumes_c_super_track_namespace(env: &WowLuaEnv) {

    let api_present: bool = env
        .eval(
            "return type(C_SuperTrack.GetHighestPrioritySuperTrackingType) == 'function' \
                and type(C_SuperTrack.ClearAllSuperTracked) == 'function' \
                and type(C_SuperTrack.GetSuperTrackedMapPin) == 'function' \
                and type(C_Navigation.WasClampedToScreen) == 'function' \
                and type(C_Navigation.GetTargetState) == 'function' \
                and type(C_Navigation.HasValidScreenPosition) == 'function' \
                and type(C_Navigation.GetDistance) == 'function' \
                and type(C_Navigation.GetFrame) == 'function' \
                and type(C_Navigation.GetNearestPartyMemberToken) == 'function'",
        )
        .expect("namespace probe should succeed");
    assert!(
        api_present,
        "Blizzard_QuestNavigation consumes 3 C_SuperTrack methods + 6 C_Navigation \
         methods. C_SuperTrack must publish GetHighestPrioritySuperTrackingType \
         (returns nil-able SuperTrackingType — the active super-track kind), \
         ClearAllSuperTracked (noop in headless: clears all super-track state \
         when the player reaches their destination), and GetSuperTrackedMapPin \
         (returns pinType + typeID for map-pin super tracking). C_Navigation \
         must publish the 6 methods used by the per-tick UpdatePosition / \
         UpdateArrow / UpdateDistanceText / UpdateClampedState / UpdateIcon / \
         OnUpdate-init pipeline"
    );
}
}

prefork_full_ui_case! {
fn blizzard_quest_navigation_consumes_super_tracking_type_enum(env: &WowLuaEnv) {

    let enum_present: bool = env
        .eval(
            "return type(Enum.SuperTrackingType) == 'table' \
                and Enum.SuperTrackingType.Quest == 0 \
                and Enum.SuperTrackingType.UserWaypoint == 1 \
                and Enum.SuperTrackingType.Corpse == 2 \
                and Enum.SuperTrackingType.Scenario == 3 \
                and Enum.SuperTrackingType.Content == 4 \
                and Enum.SuperTrackingType.PartyMember == 5 \
                and Enum.SuperTrackingType.MapPin == 6 \
                and Enum.SuperTrackingType.Vignette == 7",
        )
        .expect("enum probe should succeed");
    assert!(
        enum_present,
        "Enum.SuperTrackingType must publish the 8-value sequential enum that \
         SuperTrackedFrameMixin:UpdateIcon dispatches on (Quest=0 baseline, \
         UserWaypoint=1, Corpse=2, Scenario=3, Content=4, PartyMember=5, \
         MapPin=6, Vignette=7). The mixin checks `superTrackingType == \
         Enum.SuperTrackingType.PartyMember` to decide whether to render the \
         party-member portrait icon, and falls back to Quest when \
         GetHighestPrioritySuperTrackingType returns nil"
    );
}
}

// The Mists/Era/Anniversary caches ship 218 addons each and Wrath ships 23;
// none contains Blizzard_QuestNavigation, so the pinned TOC facts above describe no file the
// classic loaders can read.
classic_cache_absent_toc_fixture! {
#[test]
fn blizzard_questnavigation_toc_is_absent_from_classic_caches() {
    let dir = quest_navigation_dir();
    assert!(
        !dir.is_dir(),
        "classic profile caches must not contain Blizzard_QuestNavigation"
    );
    assert!(wow_ui_sim::loader::find_toc_file(&dir).is_none());
}
}
