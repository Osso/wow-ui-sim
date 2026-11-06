use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::loader::{discover_all_blizzard_addons, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

#[path = "common/toc_layout.rs"]
mod toc_layout;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn queue_status_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_QueueStatusFrame")
}

fn queue_status_mainline_toc() -> PathBuf {
    find_toc_file(&queue_status_dir()).expect("Blizzard_QueueStatusFrame TOC resolves")
}

const TOC_FILES: &[&str] = &[
    "Mainline/QueueStatusFrame.lua",
    "Mainline/QueueStatusFrame.xml",
];

// The unified PTR TOC also lists the Mainline override Lua.
const PTR_TOC_FILES: &[&str] = &[
    "Mainline/QueueStatusFrame.lua",
    "Mainline/QueueStatusFrameOverrides.lua",
    "Mainline/QueueStatusFrame.xml",
];

// The Forever TOC gates the Mainline override out and keeps the Camelot one.
const FOREVER_TOC_FILES: &[&str] = &[
    "Mainline/QueueStatusFrame.lua",
    "Camelot/QueueStatusFrameOverrides.lua",
    "Mainline/QueueStatusFrame.xml",
];

const CLASSIC_TOC_FILES: &[&str] = &[
    "Classic/QueueStatusFrame.lua",
    "Classic/QueueStatusFrame.xml",
];

fn queue_status_toc_files() -> &'static [&'static str] {
    if toc_layout::classic_cache() {
        CLASSIC_TOC_FILES
    } else if toc_layout::forever() {
        FOREVER_TOC_FILES
    } else if toc_layout::unified_toc_tree() {
        PTR_TOC_FILES
    } else {
        TOC_FILES
    }
}

const REQUIRED_DEPS: &[&str] = &["Blizzard_ActionBar", "Blizzard_LFGUtil"];

// The unified PTR TOC no longer depends on Blizzard_ActionBar; the Forever TOC
// keeps both hard dependencies.
const PTR_REQUIRED_DEPS: &[&str] = &["Blizzard_LFGUtil"];

fn queue_status_deps() -> &'static [&'static str] {
    if toc_layout::classic_cache() {
        &["Blizzard_ActionBar"]
    } else if toc_layout::unified_toc_tree() && !toc_layout::forever() {
        PTR_REQUIRED_DEPS
    } else {
        REQUIRED_DEPS
    }
}

const PUBLIC_MIXIN_GLOBALS: &[&str] = &[
    "EyeTemplateMixin",
    "QueueStatusButtonMixin",
    "QueueStatusFrameMixin",
];

const PUBLIC_NAMED_FRAMES: &[&str] = &["QueueStatusButton", "QueueStatusFrame"];

const VIRTUAL_TEMPLATES_SAMPLE: &[&str] = &[
    "QueueStatusRoleCountTemplate",
    "QueueStatusEntryTemplate",
    "EyeTemplate",
];

const PUBLIC_GLOBAL_HELPERS: &[&str] = &[
    "QueueStatusDropdown_AcceptQueuedPVPMatch",
    "QueueStatusDropdown_AddBattlefieldButtons",
    "QueueStatusDropdown_AddLFGButtons",
    "QueueStatusDropdown_AddLFGListApplicationButtons",
    "QueueStatusDropdown_AddLFGListButtons",
    "QueueStatusDropdown_AddPetBattleButtons",
    "QueueStatusDropdown_AddPlunderstormButtons",
    "QueueStatusDropdown_AddPVPRoleCheckButtons",
    "QueueStatusDropdown_AddWorldPvPButtons",
    "QueueStatusEntry_OnUpdate",
    "QueueStatusEntry_SetFullDisplay",
    "QueueStatusEntry_SetMinimalDisplay",
    "QueueStatusEntry_SetUpActiveWorldPVP",
    "QueueStatusEntry_SetUpBattlefield",
    "QueueStatusEntry_SetUpLFG",
    "QueueStatusEntry_SetUpLFGListActiveEntry",
    "QueueStatusEntry_SetUpLFGListApplication",
    "QueueStatusEntry_SetUpPetBattlePvP",
    "QueueStatusEntry_SetUpPlunderstorm",
    "QueueStatusEntry_SetUpPvPReadyCheck",
    "QueueStatusEntry_SetUpPVPRoleCheck",
    "QueueStatusEntry_SetUpWorldPvP",
    "QueueStatus_InActiveBattlefield",
    "TogglePVPScoreboardOrResults",
];

const ON_LOAD_REGISTERED_EVENTS: &[&str] = &[
    "PLAYER_ENTERING_WORLD",
    "GROUP_ROSTER_UPDATE",
    "LOBBY_MATCHMAKER_QUEUE_STATUS_UPDATE",
    "LOBBY_MATCHMAKER_QUEUE_ABANDONED",
    "LOBBY_MATCHMAKER_QUEUE_POPPED",
    "LOBBY_MATCHMAKER_QUEUE_EXPIRED",
    "LOBBY_MATCHMAKER_QUEUE_ERROR",
    "LFG_UPDATE",
    "LFG_ROLE_CHECK_UPDATE",
    "LFG_READY_CHECK_UPDATE",
    "LFG_PROPOSAL_UPDATE",
    "LFG_PROPOSAL_FAILED",
    "LFG_PROPOSAL_SUCCEEDED",
    "LFG_PROPOSAL_SHOW",
    "LFG_QUEUE_STATUS_UPDATE",
    "LFG_LIST_ACTIVE_ENTRY_UPDATE",
    "LFG_LIST_SEARCH_RESULTS_RECEIVED",
    "LFG_LIST_SEARCH_RESULT_UPDATED",
    "LFG_LIST_APPLICANT_UPDATED",
    "UPDATE_BATTLEFIELD_STATUS",
    "PVP_BRAWL_INFO_UPDATED",
    "ZONE_CHANGED_NEW_AREA",
    "ZONE_CHANGED",
    "PET_BATTLE_QUEUE_STATUS",
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
        wow_ui_sim::loader::load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);

    env
}

// Literal cached inputs absent from Wrath manifest1-382/cache; behavior cases stay enabled.
post_wrath_ui_fixture! {
#[test]
fn blizzard_queue_status_frame_find_toc_resolves_mainline_variant() {
    let resolved =
        find_toc_file(&queue_status_dir()).expect("Blizzard_QueueStatusFrame TOC resolves");
    assert_eq!(
        resolved.file_name().and_then(|name| name.to_str()),
        Some(toc_layout::expected_toc_name("Blizzard_QueueStatusFrame").as_str()),
        "find_toc_file resolves the active profile's QueueStatusFrame TOC: the Retail cache \
         ships a single `_Mainline.toc`, while the pinned PTR cache ships the bare unified TOC"
    );

    let absent_variants: &[&str] = if toc_layout::classic_cache() {
        &["_Mainline.toc", ".toc", "_Mists.toc", "_Wrath.toc"]
    } else if toc_layout::unified_toc_tree() {
        &["_Mainline.toc", "_Mists.toc", "_Wrath.toc", "_Classic.toc"]
    } else {
        &[".toc", "_Mists.toc", "_Wrath.toc", "_Classic.toc"]
    };
    for variant_suffix in absent_variants {
        let variant = queue_status_dir().join(format!("Blizzard_QueueStatusFrame{variant_suffix}"));
        assert!(
            !variant.exists(),
            "Blizzard_QueueStatusFrame must NOT ship a {variant_suffix} variant — \
             one TOC covers the queue-status button on the active profile"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_queue_status_frame_toc_pins_eager_mainline_only_with_default_state_enabled() {
    let toc = TocFile::from_file(&queue_status_mainline_toc())
        .expect("Blizzard_QueueStatusFrame TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "TOC must NOT declare `## LoadOnDemand:` — this is an eager-load addon \
         that materializes the queue-status button as soon as the player enters \
         the world; the button must be present before the LFG / Battlefield / \
         Plunderstorm queue events can fire"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(!toc.is_ptr_only());

    assert!(
        !toc.is_game_type_restricted(),
        "TOC declares `## AllowLoadGameType: mainline` — \
         `is_game_type_restricted()` at src/toc.rs:294-302 returns FALSE because \
         `mainline` is in the cross-flavor allowlist alongside `standard`. The \
         loader filter at src/loader/mod.rs:527 keeps this addon in the eager \
         pool on retail"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "TOC declares `## AllowLoad: game` (lowercase) — `allows_screen` at \
         src/toc.rs:308 routes via `eq_ignore_ascii_case(\"game\")` so the \
         lowercase form resolves the same as `## AllowLoad: Game`"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Game-only screen gate must EXCLUDE {screen:?} — the queue-status \
             button only matters in-world after character selection"
        );
    }

    assert!(toc.optional_deps().is_empty());
    assert!(toc.load_with().is_empty());
    assert!(
        toc.saved_variables().is_empty(),
        "TOC must declare ZERO `## SavedVariables:` — pure stateless display: \
         every queue entry pulls from live GetLFGQueueStats / \
         GetBattlefieldStatus / C_LobbyMatchmakerInfo queries each event tick"
    );
    assert!(toc.saved_variables_per_character().is_empty());
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_queue_status_frame_toc_declares_action_bar_and_lfg_util_dependencies() {
    let toc = TocFile::from_file(&queue_status_mainline_toc()).expect("TOC parses");
    let dependencies = toc.dependencies();
    let deps: Vec<&str> = dependencies.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        deps, queue_status_deps(),
        "QueueStatusFrame declares its published dependencies in order; the unified PTR TOC \
         drops Blizzard_ActionBar from the Retail pair, while the Forever TOC keeps it"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_queue_status_frame_toc_declares_metadata_in_raw_bytes() {
    let raw = std::fs::read_to_string(queue_status_mainline_toc()).expect("TOC reads utf-8");

    assert!(raw.contains("## Title: Blizzard_QueueStatusFrame"));
    if toc_layout::unified_toc_tree() {
        assert!(raw.contains("## Dep: Blizzard_LFGUtil"));
    } else {
        assert!(raw.contains("## Author: Blizzard Entertainment"));
        assert!(
            raw.contains("## DefaultState: enabled"),
            "TOC must declare `## DefaultState: enabled` — the AddOn list UI shows \
             the addon enabled by default for users who toggle it manually"
        );
        if toc_layout::classic_cache() {
            assert!(raw.contains("## Dependencies: Blizzard_ActionBar"));
            assert!(raw.contains("## AllowLoadGameType: classic"));
        } else {
            assert!(raw.contains("## Dependencies: Blizzard_ActionBar, Blizzard_LFGUtil"));
            assert!(raw.contains("## AllowLoadGameType: mainline"));
        }
        assert!(raw.contains("## AllowLoad: game"));
    }

    assert!(
        !raw.contains("## LoadOnDemand"),
        "TOC must NOT carry any LoadOnDemand directive (eager-loaded)"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT carry any SavedVariables directive"
    );
    assert!(
        !raw.contains("## OnlyBetaAndPTR"),
        "TOC must NOT carry OnlyBetaAndPTR — ships on live"
    );
    assert!(
        !raw.contains("## Version"),
        "TOC must NOT carry a Version directive — one of the Blizzard_* addons \
         missing the canonical version line"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_queue_status_frame_toc_lists_two_files_with_variant_subdirectory() {
    let toc = TocFile::from_file(&queue_status_mainline_toc()).expect("TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed,
        queue_status_toc_files(),
        "TOC body lists the Lua sources before the XML that declares the mixins, virtual \
         templates, and the QueueStatusButton + QueueStatusFrame named frames. The unified \
         PTR TOC also lists the Mainline override Lua; the Forever TOC lists its Camelot \
         override instead"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_queue_status_frame_appears_in_eager_game_discovery() {
    let ui = blizzard_ui_dir();

    let game_addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let game_found = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_QueueStatusFrame");
    assert!(
        game_found,
        "Blizzard_QueueStatusFrame MUST appear in eager Game-screen discovery: \
         no `## LoadOnDemand:` (so `is_load_on_demand()` false), \
         `## AllowLoadGameType: mainline` (so `is_game_type_restricted()` \
         false on the simulator's mainline emulation), and `## AllowLoad: game` \
         (so the Game screen gate passes). All 3 loader filters at \
         src/loader/mod.rs:527 admit it"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let glue_addons = discover_blizzard_addons_for_screen(&ui, screen);
        let glue_found = glue_addons
            .iter()
            .any(|(name, _)| name == "Blizzard_QueueStatusFrame");
        assert!(
            !glue_found,
            "Blizzard_QueueStatusFrame must NOT appear in eager discovery for \
             {screen:?} — the `## AllowLoad: game` gate excludes glue-screen \
             load paths"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_queue_status_frame_appears_in_full_addon_inventory() {
    let inventory = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = inventory
        .iter()
        .any(|(name, _)| name == "Blizzard_QueueStatusFrame");
    assert!(
        found,
        "Blizzard_QueueStatusFrame MUST appear in `discover_all_blizzard_addons`"
    );
}
}

prefork_full_ui_case! {
fn blizzard_queue_status_frame_loads_in_eager_game_sweep_without_lua_errors(env: &WowLuaEnv) {

    let addon_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_QueueStatusFrame")
                || message.contains("QueueStatusFrame")
                || message.contains("QueueStatusButton")
                || message.contains("QueueStatusEntry")
                || message.contains("QueueStatusDropdown")
                || message.contains("EyeTemplate")
        })
        .cloned()
        .collect();
    assert!(
        addon_errors.is_empty(),
        "Blizzard_QueueStatusFrame emitted addon-specific Lua errors during \
         eager load:\n  {}",
        addon_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_queue_status_frame_publishes_three_mixin_globals(env: &WowLuaEnv) {

    for mixin in PUBLIC_MIXIN_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G[{mixin:?}])"))
            .unwrap_or_else(|err| panic!("type probe for {mixin} failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table — Blizzard_QueueStatusFrame \
             ships exactly 3 public Mixin globals at file scope: \
             EyeTemplateMixin (owns the searching/found/poke/hover animation \
             state machine for the eye icon child of QueueStatusButton), \
             QueueStatusButtonMixin (owns the QueueStatusButton parented to \
             MicroMenuContainer with the OnClick / OnEnter / OnLeave / OnUpdate \
             / glow-pulse / context-menu state machine), and \
             QueueStatusFrameMixin (owns the TOOLTIP-strata QueueStatusFrame \
             that displays the per-queue list with the 24-event OnLoad \
             registration plus the OnEvent dispatcher routing each queue \
             system's events to the matching QueueStatusEntry_SetUp* helper)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_queue_status_frame_publishes_two_named_top_level_frames(env: &WowLuaEnv) {

    for frame_name in PUBLIC_NAMED_FRAMES {
        let kind: String = env
            .eval(&format!("return type(_G[{frame_name:?}])"))
            .unwrap_or_else(|err| panic!("type probe for {frame_name} failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{frame_name} must publish as a frame userdata — \
             QueueStatusButton is the 45x45 OVERLAY-layered Button parented \
             to MicroMenuContainer (NOT UIParent — comes from the \
             Blizzard_ActionBar→Blizzard_MicroMenu transitive dep chain) with \
             child Eye frame inheriting EyeTemplate plus a Highlight texture \
             driven by the EyeHighlightAnim AnimationGroup; QueueStatusFrame \
             is the 275x150 TOOLTIP-strata frame parented to UIParent that \
             inherits TooltipBackdropTemplate and anchors RIGHT to the \
             QueueStatusButton's LEFT, displaying the per-queue list when the \
             player hovers the button. Both frames start hidden and are \
             clamped to screen"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_queue_status_frame_virtual_templates_not_in_global_env(env: &WowLuaEnv) {

    for template in VIRTUAL_TEMPLATES_SAMPLE {
        let kind: String = env
            .eval(&format!("return type(_G[{template:?}])"))
            .unwrap_or_else(|err| panic!("type probe for {template} failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{template} must be nil — virtual templates live in the \
             template registry, NOT the global environment. The 3 virtual \
             templates QueueStatusFrame.xml ships are: \
             QueueStatusRoleCountTemplate (the per-role count badge with a \
             FontString $parentCount inheriting GameFontHighlight, used \
             repeatedly inside the LFG entry display for tank / healer / dps \
             slot counts), QueueStatusEntryTemplate (the per-queue-type entry \
             row template repeated inside QueueStatusFrame for each active \
             queue), and EyeTemplate (the animated groupfinder eye icon — \
             initial / searching / found / hover / poke animation states — \
             instantiated as the QueueStatusButton's $parentIcon child via \
             `inherits=\"EyeTemplate\"`)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_queue_status_frame_publishes_global_helper_functions(env: &WowLuaEnv) {

    for helper in PUBLIC_GLOBAL_HELPERS {
        let kind: String = env
            .eval(&format!("return type(_G[{helper:?}])"))
            .unwrap_or_else(|err| panic!("type probe for {helper} failed: {err}"));
        assert_eq!(
            kind, "function",
            "_G.{helper} must publish as a function — \
             Blizzard_QueueStatusFrame ships exactly 24 public helper \
             functions outside the 3 mixin tables. 9 QueueStatusDropdown_* \
             helpers populate the right-click context-menu MenuUtil \
             descriptions for each queue type (Battlefield / LFG / LFGList / \
             LFGListApplication / PetBattle / Plunderstorm / PVPRoleCheck / \
             WorldPvP), 13 QueueStatusEntry_SetUp* / QueueStatusEntry_Set* / \
             QueueStatusEntry_OnUpdate helpers shape per-queue-row content, \
             QueueStatus_InActiveBattlefield reports whether the player is in \
             an active battlefield (returning showScoreboard alongside), and \
             TogglePVPScoreboardOrResults is the global keybind handler that \
             flips the PVP scoreboard / results frame visibility"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_queue_status_frame_registers_twenty_four_events_in_onload(env: &WowLuaEnv) {

    for event in ON_LOAD_REGISTERED_EVENTS {
        let registered: bool = env
            .eval(&format!(
                "return QueueStatusFrame:IsEventRegistered({event:?})"
            ))
            .unwrap_or_else(|err| panic!("event probe for {event} failed: {err}"));
        assert!(
            registered,
            "QueueStatusFrame must register `{event}` in OnLoad — \
             QueueStatusFrameMixin:OnLoad calls RegisterEvent for 24 distinct \
             events spanning every queue / matchmaking subsystem the addon \
             tracks: PLAYER_ENTERING_WORLD + GROUP_ROSTER_UPDATE for state \
             refresh; LOBBY_MATCHMAKER_QUEUE_* (5 events: STATUS_UPDATE / \
             ABANDONED / POPPED / EXPIRED / ERROR) for the modern \
             C_LobbyMatchmakerInfo queue (Plunderstorm and similar lobby-based \
             modes); LFG_* (8 events: UPDATE / ROLE_CHECK_UPDATE / \
             READY_CHECK_UPDATE / PROPOSAL_UPDATE / PROPOSAL_FAILED / \
             PROPOSAL_SUCCEEDED / PROPOSAL_SHOW / QUEUE_STATUS_UPDATE) for the \
             classic Looking-For-Group dungeon/raid finder; LFG_LIST_* (4 \
             events: ACTIVE_ENTRY_UPDATE / SEARCH_RESULTS_RECEIVED / \
             SEARCH_RESULT_UPDATED / APPLICANT_UPDATED) for the modern Premade \
             Groups list; UPDATE_BATTLEFIELD_STATUS + PVP_BRAWL_INFO_UPDATED \
             for PVP queues; ZONE_CHANGED_NEW_AREA + ZONE_CHANGED for \
             world-pvp zone transitions; PET_BATTLE_QUEUE_STATUS for the pet \
             battle pvp queue. The `{event}` event being unregistered would \
             mean the addon misses queue updates for its corresponding \
             subsystem and the QueueStatusButton would not display correctly"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_queue_status_button_enters_searching_mode_after_lfd_join(env: &WowLuaEnv) {

    let result: String = env
        .eval(
            r#"
            PVEFrame_ShowFrame("GroupFinderFrame", LFDParentFrame)
            LFDQueueFrame_SetType("specific")
            local ok, err = pcall(LFDQueueFrame_Join)
            if not ok then
                return "join_error=" .. tostring(err)
            end

            local mode = GetLFGMode(LE_LFG_CATEGORY_LFD)
            if mode ~= "queued" then
                return "mode=" .. tostring(mode)
            end
            if not QueueStatusButton:IsShown() then
                return "hidden"
            end
            if QueueStatusButton.Eye:IsStaticMode() then
                return "static"
            end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "joining an LFD queue must notify QueueStatusFrame so the eye leaves static mode"
    );
}
}
