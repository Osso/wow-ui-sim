#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn minimap_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_Minimap")
}

fn minimap_mainline_toc() -> PathBuf {
    minimap_dir().join("Blizzard_Minimap_Mainline.toc")
}

fn minimap_classic_toc() -> PathBuf {
    minimap_dir().join("Blizzard_Minimap_Classic.toc")
}

const MINIMAP_TOC_FILES: &[&str] = &[
    "Mainline/Minimap.lua",
    "Mainline/Minimap.xml",
    "Mainline/GameTime.lua",
    "Mainline/GameTime.xml",
    "Mainline/AddonCompartment.lua",
    "Mainline/AddonCompartment.xml",
];

const REQUIRED_DEPENDENCIES: &[&str] = &[
    "Blizzard_ActionBar",
    "Blizzard_EditMode",
    "Blizzard_FrameXMLUtil",
    "Blizzard_GameTooltip",
];

const MINIMAP_MIXINS: &[&str] = &[
    "MinimapZoneTextButtonMixin",
    "MinimapMixin",
    "MinimapZoomInButtonMixin",
    "MinimapZoomOutButtonMixin",
    "MinimapClusterMixin",
    "MiniMapMailFrameMixin",
    "MinimapMailAnimMixin",
    "MiniMapCraftingOrderFrameMixin",
    "MiniMapTrackingButtonMixin",
    "ExpansionLandingPageMinimapButtonMixin",
];

const ADDON_COMPARTMENT_METHODS: &[&str] = &[
    "OnLoad",
    "OnEvent",
    "RegisterAddons",
    "RegisterAddon",
    "UpdateDisplay",
];

const NAMED_FRAMES: &[&str] = &[
    "MinimapCluster",
    "Minimap",
    "MinimapBackdrop",
    "ExpansionLandingPageMinimapButton",
    "GameTimeFrame",
    "AddonCompartmentFrame",
];

const GAMETIME_FUNCTIONS: &[&str] = &[
    "GameTime_GetFormattedTime",
    "GameTime_ComputeMinutes",
    "GameTime_ComputeStandardTime",
    "GameTime_ComputeMilitaryTime",
    "GameTime_GetLocalTime",
    "GameTime_GetGameTime",
    "GameTime_GetTime",
    "GameTime_UpdateTooltip",
];

const MINIMAP_FUNCTIONS: &[&str] = &[
    "ToggleMinimap",
    "Minimap_Update",
    "Minimap_SetTooltip",
    "Minimap_OnUpdate",
    "Minimap_ZoomIn",
    "Minimap_ZoomOut",
];

const GARRISON_FUNCTIONS: &[&str] = &[
    "GarrisonLandingPage_Toggle",
    "GarrisonMinimapBuilding_ShowPulse",
    "GarrisonMinimapMission_ShowPulse",
    "GarrisonMinimapMission_HidePulse",
    "GarrisonMinimapInvasion_ShowPulse",
    "GarrisonMinimapShipmentCreated_ShowPulse",
    "GarrisonMinimap_CheckShowCovenantCallingsNotification",
    "GarrisonMinimap_OnCallingsUpdated",
    "GarrisonMinimap_SetQueuedHelpTip",
    "GarrisonMinimap_CheckQueuedHelpTip",
    "GarrisonMinimap_ClearQueuedHelpTip",
    "GarrisonMinimap_HideHelpTip",
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

#[test]
fn blizzard_minimap_find_toc_resolves_mainline_variant() {
    let resolved = find_toc_file(&minimap_dir()).expect("Blizzard_Minimap TOC should resolve");
    assert_eq!(
        resolved,
        minimap_mainline_toc(),
        "Blizzard_Minimap ships dual TOC variants (`_Mainline.toc` + `_Classic.toc`); the \
         resolver MUST prefer the Mainline copy on retail (the `_Mainline` lookup is the \
         first probe in `find_toc_file`). The Classic copy carries flavor-shifted file lists \
         under Cata/ + Mists/ subdirectories that the retail simulator must not pick up"
    );
    assert!(
        minimap_classic_toc().exists(),
        "The Classic variant TOC must coexist on disk — confirms this is a true dual-variant \
         addon, not a single bare TOC. The Mainline-preference behavior of `find_toc_file` is \
         only meaningful when both variants are present"
    );
}

#[test]
fn blizzard_minimap_toc_declares_default_state_with_four_required_deps() {
    let toc = TocFile::from_file(&minimap_mainline_toc()).expect("Blizzard_Minimap TOC parses");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_Minimap omits `## LoadOnDemand:` — `## DefaultState: enabled` makes it an \
         eager-load addon. The Minimap is a foundational HUD frame; every Game-screen boot \
         must publish MinimapCluster + GameTimeFrame + AddonCompartmentFrame at startup so \
         the EditMode system can register them"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());

    let deps = toc.dependencies();
    assert_eq!(
        deps.len(),
        REQUIRED_DEPENDENCIES.len(),
        "TOC must declare exactly {} `## Dependencies:` entries; got {deps:?}. The minimap \
         depends on Blizzard_ActionBar (XP/reputation bar anchoring), Blizzard_EditMode \
         (MinimapCluster registers as `EditModeMinimapSystemTemplate`), Blizzard_FrameXMLUtil \
         (shared GameTime utility helpers), and Blizzard_GameTooltip (zone-text + game-time \
         hover tooltips)",
        REQUIRED_DEPENDENCIES.len()
    );
    for required in REQUIRED_DEPENDENCIES {
        assert!(
            deps.contains(&required.to_string()),
            "Blizzard_Minimap must declare `## Dependencies: {required}`; got {deps:?}"
        );
    }

    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` — every consumer of minimap surfaces is loaded later in the \
         dependency graph (Blizzard_Calendar, Blizzard_GarrisonUI, etc. depend on Minimap, \
         not the other way around)"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — minimap state (zoom, position, rotation) is persisted via \
         CVars (`minimapZoom`, `rotateMinimap`) handled by the C client, not via TOC-level \
         saved variables"
    );
}

#[test]
fn blizzard_minimap_toc_declares_allow_load_game_lowercase_with_mainline_only() {
    let raw = std::fs::read_to_string(minimap_mainline_toc()).expect("Mainline TOC reads");
    assert!(
        raw.contains("## AllowLoad: game"),
        "TOC must declare `## AllowLoad: game` exactly with lowercase `game`. The minimap is \
         a HUD frame that only makes sense in-world; the glue screens (login + character \
         select / create) have no minimap. The case-insensitive matcher at src/toc.rs:308 \
         normalizes through `eq_ignore_ascii_case`, but the raw lowercase spelling matches \
         the live Blizzard convention"
    );
    assert!(
        raw.contains("## AllowLoadGameType: mainline"),
        "TOC must declare `## AllowLoadGameType: mainline` — the Classic variant carries a \
         different game-type marker so the retail-only Mainline copy is the one that \
         participates in mainline auto-discovery"
    );
}

#[test]
fn blizzard_minimap_toc_is_unrestricted_on_mainline_screens() {
    let toc = TocFile::from_file(&minimap_mainline_toc()).expect("Blizzard_Minimap TOC parses");
    assert!(
        !toc.is_game_type_restricted(),
        "TOC declares `## AllowLoadGameType: mainline`; the simulator's \
         `is_game_type_restricted` (src/toc.rs:294) treats `mainline` and `standard` as the \
         same retail target, so this MUST return false. A `true` return would silently skip \
         Minimap from retail discovery — every Game-screen frame downstream would break"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: game` must allow Game screen (the lowercase `game` literal \
         normalizes through `eq_ignore_ascii_case` at src/toc.rs:308)"
    );
    for glue in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(glue),
            "`## AllowLoad: game` must NOT allow glue screen {glue:?}. The minimap surface \
             is in-world only; loading it on the login / character-select panels would \
             waste setup work and break frame-strata assumptions"
        );
    }
}

#[test]
fn blizzard_minimap_toc_lists_six_files_all_under_mainline_subdir() {
    let toc = TocFile::from_file(&minimap_mainline_toc()).expect("Blizzard_Minimap TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, MINIMAP_TOC_FILES,
        "TOC body must list exactly 6 files in declaration order — three (lua, xml) pairs \
         under the `Mainline/` subdirectory: Minimap, GameTime, AddonCompartment. The \
         TOC parser normalizes backslash separators to forward slashes when constructing \
         the relative file paths. The `Mainline` prefix is the load-order isolator that \
         keeps retail-only sources separate from the parallel Cata and Mists trees that \
         the Classic variant TOC consumes"
    );
}

#[test]
fn blizzard_minimap_appears_only_on_game_screen_discovery() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_Minimap");
    assert!(
        in_game,
        "Blizzard_Minimap (`## AllowLoad: game`, `## DefaultState: enabled`, non-LOD) MUST \
         appear in Game-screen discovery — it's a foundational HUD addon"
    );

    for glue in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let glue_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), glue);
        let leaks_to_glue = glue_addons
            .iter()
            .any(|(name, _)| name == "Blizzard_Minimap");
        assert!(
            !leaks_to_glue,
            "Blizzard_Minimap MUST NOT appear in {glue:?} discovery — `## AllowLoad: game` \
             gates it out of every glue screen at the discovery layer (`allows_screen` \
             returns false for non-Game screens when AllowLoad is `game`)"
        );
    }
}

#[test]
fn blizzard_minimap_required_deps_appear_in_game_discovery() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    for required in REQUIRED_DEPENDENCIES {
        let found = addons.iter().any(|(name, _)| name == required);
        assert!(
            found,
            "Blizzard_Minimap declares `## Dependencies: {required}`; the dependency MUST \
             also be auto-discovered on the Game screen so the loader can satisfy the \
             dependency edge before booting Minimap"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_minimap_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let addon_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_Minimap")
                || message.contains("MinimapCluster")
                || message.contains("MinimapMixin")
                || message.contains("AddonCompartment")
                || message.contains("GameTimeFrame")
                || message.contains("ExpansionLandingPage")
        })
        .cloned()
        .collect();
    assert!(
        addon_errors.is_empty(),
        "Blizzard_Minimap emitted addon-specific Lua errors during load:\n  {}",
        addon_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_minimap_is_addon_loaded_after_auto_discovery(env: &WowLuaEnv) {
    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_Minimap')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_Minimap') must return true after the eager \
         auto-discovery sweep — proves the minimap addon registered with the loaded-set \
         during the standard Game-screen boot pipeline"
    );
}
}

prefork_full_ui_case! {
fn blizzard_minimap_publishes_ten_minimap_mixins_as_tables(env: &WowLuaEnv) {
    for mixin in MINIMAP_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{mixin})"))
            .unwrap_or_else(|err| panic!("type({mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "Mixin `{mixin}` must publish at `_G` as a table. Minimap.lua declares 10 \
             mixins as bare `MixinName = {{ }}` shells at file scope, then attaches methods \
             via `function MixinName:Method()` — every mixin is a public table that other \
             addons read via `CreateFromMixins`"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_minimap_publishes_addon_compartment_mixin_with_five_methods(env: &WowLuaEnv) {
    let kind: String = env
        .eval("return type(_G.AddonCompartmentMixin)")
        .expect("AddonCompartmentMixin probe");
    assert_eq!(
        kind, "table",
        "AddonCompartmentMixin must publish at `_G` as a table. AddonCompartment.lua line 3 \
         declares `AddonCompartmentMixin = {{ }};` then attaches 5 methods (OnLoad, OnEvent, \
         RegisterAddons, RegisterAddon, UpdateDisplay) — the mixin is the public surface \
         every addon uses to register an entry in the minimap-anchored compartment menu"
    );
    for method in ADDON_COMPARTMENT_METHODS {
        let method_kind: String = env
            .eval(&format!("return type(AddonCompartmentMixin.{method})"))
            .unwrap_or_else(|err| panic!("AddonCompartmentMixin.{method} probe failed: {err}"));
        assert_eq!(
            method_kind, "function",
            "AddonCompartmentMixin.{method} must be a function. The 5 methods form the \
             addon-compartment registration contract: OnLoad / OnEvent are the script \
             handlers wired in XML; RegisterAddons / RegisterAddon / UpdateDisplay are the \
             public registration API third-party addons call to add a compartment entry"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_minimap_publishes_module_level_constants(env: &WowLuaEnv) {
    let probe = "\
        return MINIMAPPING_TIMER == 5.5 \
            and MINIMAPPING_FADE_TIMER == 0.5 \
            and MINIMAP_BOTTOM_EDGE_EXTENT == 192 \
            and MINIMAP_RECORDING_INDICATOR_ON == false \
            and MINIMAP_EXPANDER_MAXSIZE == 28 \
            and HUNTER_TRACKING == 1 \
            and TOWNSFOLK_TRACKING == 2 \
            and type(MinimapPulseLock) == 'table' \
            and MinimapPulseLock.GarrisonBuilding == 1 \
            and MinimapPulseLock.GarrisonInvasion == 2 \
            and MinimapPulseLock.RunesOfPower == 8 \
            and type(ExpansionLandingPageMode) == 'table' \
            and ExpansionLandingPageMode.Garrison == 1";
    let ok: bool = env
        .eval(probe)
        .expect("Module-level constants probe succeeds");
    assert!(
        ok,
        "Module-level constants must publish at `_G` with their declared values. \
         Minimap.lua defines canonical timing budgets, tracking categories, and the eight-entry \
         MinimapPulseLock enum that consumers read by name to drive minimap pulse / fade \
         animations. GameTime.lua no longer exports GAMETIME_AM / GAMETIME_PM globals"
    );
}
}

prefork_full_ui_case! {
fn blizzard_minimap_publishes_named_frames_with_correct_widget_types(env: &WowLuaEnv) {
    for frame in NAMED_FRAMES {
        let exists: bool = env
            .eval(&format!("return type(_G.{frame}) == 'table'"))
            .unwrap_or_else(|err| panic!("type({frame}) probe failed: {err}"));
        assert!(
            exists,
            "Named frame `{frame}` must publish at `_G` as a table after Blizzard_Minimap \
             load. Minimap.xml + GameTime.xml + AddonCompartment.xml declare the 6 \
             top-level named frames (MinimapCluster + Minimap + MinimapBackdrop + \
             ExpansionLandingPageMinimapButton from Minimap.xml, GameTimeFrame from \
             GameTime.xml, AddonCompartmentFrame from AddonCompartment.xml) that every \
             EditMode + minimap-aware addon reads"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_minimap_publishes_gametime_helper_functions(env: &WowLuaEnv) {
    for func in GAMETIME_FUNCTIONS {
        let kind: String = env
            .eval(&format!("return type(_G.{func})"))
            .unwrap_or_else(|err| panic!("type({func}) probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "GameTime helper `{func}` must publish at `_G` as a function. GameTime.lua \
             exports 8 free-standing helpers covering the AM/PM ↔ military-time \
             conversion + local-time / game-time / formatted-time / tooltip surface — \
             every minimap clock display + calendar-event tooltip pulls from this set"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_minimap_publishes_minimap_helper_functions(env: &WowLuaEnv) {
    for func in MINIMAP_FUNCTIONS {
        let kind: String = env
            .eval(&format!("return type(_G.{func})"))
            .unwrap_or_else(|err| panic!("type({func}) probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "Minimap helper `{func}` must publish at `_G` as a function. Minimap.lua \
             exports 6 free-standing helpers covering the show/hide toggle, periodic \
             update tick, tooltip wiring, and zoom controls — these are the canonical \
             entry points for keybinding handlers and external addons"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_minimap_publishes_garrison_helper_functions(env: &WowLuaEnv) {
    for func in GARRISON_FUNCTIONS {
        let kind: String = env
            .eval(&format!("return type(_G.{func})"))
            .unwrap_or_else(|err| panic!("type({func}) probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "Garrison helper `{func}` must publish at `_G` as a function. Minimap.lua \
             carries the current 12-function garrison-pulse + covenant-callings surface, \
             including Mission_HidePulse and CheckShowCovenantCallingsNotification; these \
             exports drive pulse locks and queued help-tip routing on the minimap button"
        );
    }
}
}

#[test]
#[cfg(feature = "optional-tests")]
fn retail_original_minimap_owns_calendar_invite_initialization_and_refresh() {
    use wow_ui_sim::loader::{
        discover_all_blizzard_addons, discover_blizzard_startup_addons_for_screen,
        register_load_with_addons,
    };
    let env = WowLuaEnv::new().expect("create normal Retail Game environment");
    let ui = blizzard_ui_dir();
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    let startup = discover_blizzard_startup_addons_for_screen(&ui, ScreenKind::Game);
    let owner = startup
        .iter()
        .find(|a| a.name == "Blizzard_Minimap")
        .unwrap();
    assert_eq!(
        owner.toc_path.file_name().unwrap(),
        "Blizzard_Minimap_Mainline.toc"
    );
    register_load_with_addons(&env.loader_env(), &discover_all_blizzard_addons(&ui));
    crate::common::load_game_startup_inventory(&env);

    // Before post-load workarounds and PLAYER_ENTERING_WORLD, XML OnLoad has already run.
    env.exec(
        r#"
        local function original(fn, line)
            local info = debug.getinfo(fn, "S")
            assert(info.source:gsub(string.char(92), '/'):find(
                'Blizzard_Minimap/Mainline/GameTime.lua', 1, true), info.source)
            assert(info.linedefined == line)
        end
        original(GameTimeFrame:GetScript('OnLoad'), 36)
        original(GameTimeFrame:GetScript('OnEvent'), 60)
        assert(GameTimeFrame.pendingCalendarInvites == 0)
        assert(GameTimeFrame:IsEventRegistered('PLAYER_ENTERING_WORLD'))
        assert(GameTimeFrame:IsEventRegistered('CALENDAR_UPDATE_PENDING_INVITES'))
    "#,
    )
    .expect("selected original Minimap XML OnLoad owns the invite count before startup events");
    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);

    for _ in 0..2 {
        let error_count = env.state().borrow().lua_errors.len();
        env.fire_event("CALENDAR_UPDATE_PENDING_INVITES").unwrap();
        env.exec(
            r#"
            assert(C_Calendar.GetNumPendingInvites() == 0, "modeled no-invite state")
            assert(GameTimeFrame.pendingCalendarInvites == 0, "original counter")
            assert(GameTimeFrame.flashInvite == false, "original flash disabled")
            assert(not GameTimeCalendarInvitesTexture:IsShown(), "original invite texture hidden")
            assert(not GameTimeCalendarInvitesGlow:IsShown(), "original invite glow hidden")
        "#,
        )
        .expect("stock invite event preserves the modeled no-invite state");
        assert_eq!(env.state().borrow().lua_errors.len(), error_count);
    }
}
