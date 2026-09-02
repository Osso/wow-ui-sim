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

fn raid_frame_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_RaidFrame")
}

fn raid_frame_mainline_toc() -> PathBuf {
    raid_frame_dir().join("Blizzard_RaidFrame_Mainline.toc")
}

const TOC_FILES: &[&str] = &[
    "Mainline/RaidFrame.lua",
    "Mainline/RaidFrame.xml",
    "Mainline/RaidFrameSocialView.lua",
    "Mainline/RaidFrameSocialView.xml",
    "Mainline/RaidInfoFrameSocialView.lua",
    "Mainline/RaidInfoFrameSocialView.xml",
];

const REQUIRED_DEPS: &[&str] = &[
    "Blizzard_UIPanels_Game",
    "Blizzard_UnitFrame",
    "Blizzard_FriendsFrame",
];

const PUBLIC_NAMED_FRAMES: &[&str] = &["RaidParentFrame", "RaidFrame"];

const VIRTUAL_TEMPLATES: &[&str] = &["RaidInfoHeaderTemplate", "RaidInfoInstanceTemplate"];

const PUBLIC_GLOBAL_HELPERS: &[&str] = &[
    "RaidParentFrame_OnLoad",
    "RaidFrame_OnLoad",
    "RaidFrame_OnShow",
    "RaidFrame_OnEvent",
    "RaidParentFrame_SetView",
    "RaidFrame_Update",
    "RaidFrame_ConvertToRaid",
    "UpdateRaidAndPartyFrames",
    "RaidInfoFrame_InitButton",
    "RaidInfoFrame_SetButtonSelected",
    "RaidInfoFrame_Update",
    "RaidInfoInstance_OnMouseDown",
    "RaidInfoInstance_OnMouseUp",
    "RaidInfoInstance_OnClick",
    "RaidInfoInstance_OnEnter",
    "RaidInfoFrame_UpdateButtons",
    "RaidInfoExtendButton_OnClick",
    "RaidFrameAllAssistCheckButton_UpdateAvailable",
    "ClaimRaidFrame",
];

const ON_LOAD_REGISTERED_EVENTS: &[&str] = &[
    "PLAYER_LOGIN",
    "GROUP_ROSTER_UPDATE",
    "UPDATE_INSTANCE_INFO",
    "PARTY_LEADER_CHANGED",
    "PLAYER_ENTERING_WORLD",
    "READY_CHECK",
    "READY_CHECK_CONFIRM",
    "READY_CHECK_FINISHED",
    "PARTY_LFG_RESTRICTED",
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

#[test]
fn blizzard_raid_frame_find_toc_resolves_mainline_variant() {
    let resolved = find_toc_file(&raid_frame_dir()).expect("Blizzard_RaidFrame TOC resolves");
    assert_eq!(
        resolved,
        raid_frame_mainline_toc(),
        "Blizzard_RaidFrame ships a SINGLE `_Mainline.toc` variant (NO bare \
         `.toc`, NO Mists / Wrath / Classic variant). The retail RaidFrame is \
         distinct from the legacy Wrath RaidFrame which ships a separate \
         Wrath/RaidFrame.xml that the Mainline TOC does NOT reference. \
         `find_toc_file` walks the suffix-priority list `[_Mainline.toc, \
         .toc]` at src/loader/mod.rs:65-95 and picks the Mainline-suffixed \
         form first"
    );

    for variant_suffix in [".toc", "_Mists.toc", "_Wrath.toc", "_Classic.toc"] {
        let variant = raid_frame_dir().join(format!("Blizzard_RaidFrame{variant_suffix}"));
        assert!(
            !variant.exists(),
            "Blizzard_RaidFrame must NOT ship a {variant_suffix} variant — \
             single Mainline TOC only. The Wrath/ subdirectory ships a \
             separate RaidFrame.xml but no companion Wrath TOC; the Wrath \
             flavor builds use a different addon graph entirely"
        );
    }
}

#[test]
fn blizzard_raid_frame_toc_pins_eager_mainline_only_with_default_state_enabled() {
    let toc =
        TocFile::from_file(&raid_frame_mainline_toc()).expect("Blizzard_RaidFrame TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "TOC must NOT declare `## LoadOnDemand:` — eager-loaded so the raid \
         tab content panel and the RaidParentFrame ButtonFrameTemplate \
         container are wired up at startup; the OnLoad RegisterEvent calls \
         (PLAYER_LOGIN / GROUP_ROSTER_UPDATE / UPDATE_INSTANCE_INFO / \
         PLAYER_ENTERING_WORLD etc.) need to fire as soon as the player \
         enters the world so the saved-instance-info table is populated \
         before the user opens the FriendsFrame raid tab"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(!toc.is_ptr_only());

    assert!(
        !toc.is_game_type_restricted(),
        "TOC declares `## AllowLoadGameType: mainline` — \
         `is_game_type_restricted()` at src/toc.rs:294-302 returns FALSE \
         because `mainline` is in the cross-flavor allowlist at \
         src/toc.rs:299 alongside `standard`. The Wrath flavor sibling at \
         Wrath/RaidFrame.xml is NOT loaded by this Mainline TOC — that file \
         exists in the directory but is dead weight from this TOC's \
         perspective, only relevant to a separate Wrath-flavor addon graph"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "TOC declares NO `## AllowLoad:` directive at all — `allows_screen` \
         at src/toc.rs:305-313 falls into the `None` arm at line 311 which \
         defaults to `screen == ScreenKind::Game`. So the raid frame is \
         IMPLICITLY game-only via the missing-directive default rather than \
         via an explicit `## AllowLoad: Game` value"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Game-only screen gate must EXCLUDE {screen:?} — raid info / \
             saved instances only exist in-world via GetSavedInstanceInfo"
        );
    }

    assert_eq!(
        toc.optional_deps(),
        vec!["Blizzard_SocialUIShared".to_string()],
        "Retail 12.1.0.69497 declares Blizzard_SocialUIShared as an optional dependency"
    );
    assert!(toc.load_with().is_empty());
    assert!(
        toc.saved_variables().is_empty(),
        "TOC must declare ZERO `## SavedVariables:` — pure stateless display: \
         every saved instance pulls from live GetSavedInstanceInfo / \
         GetNumSavedInstances / GetNumSavedWorldBosses queries each \
         UPDATE_INSTANCE_INFO event tick, which the C-side raid-info backing \
         store rebuilds on every RequestRaidInfo()"
    );
    assert!(toc.saved_variables_per_character().is_empty());
}

#[test]
fn blizzard_raid_frame_toc_declares_three_dependencies() {
    let toc = TocFile::from_file(&raid_frame_mainline_toc()).expect("TOC parses");
    let dependencies = toc.dependencies();
    let deps: Vec<&str> = dependencies.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        deps, REQUIRED_DEPS,
        "TOC must declare exactly 3 hard deps in `## Dependencies:` (plural \
         form, comma-separated, in declaration order): \
         Blizzard_UIPanels_Game (publishes ButtonFrameTemplate which the \
         RaidParentFrame inherits — provides the title strip + close button \
         + portrait area common to all in-world panels), Blizzard_UnitFrame \
         (publishes UpdateMicroButtons + the role-related globals \
         IsEveryoneAssistant / UnitIsGroupLeader the \
         RaidFrameAllAssistCheckButton_UpdateAvailable handler calls), and \
         Blizzard_FriendsFrame (publishes the FriendsFrame container that \
         the raid tab parents into via ClaimRaidFrame, plus \
         ButtonFrameTemplate_ShowAttic / FriendsFrame_UpdateInsetVisibility \
         / ButtonFrameTemplate_ShowButtonBar / ButtonFrameTemplate_HideButtonBar \
         the RaidFrame_Update + RaidFrame_OnShow handlers route through). \
         Order matters: UIPanels_Game first because the FriendsFrame from \
         Blizzard_FriendsFrame inherits from ButtonFrameTemplate which \
         UIPanels_Game publishes, so the dep graph must walk \
         UIPanels_Game → UnitFrame → FriendsFrame → RaidFrame"
    );
}

#[test]
fn blizzard_raid_frame_toc_declares_metadata_in_raw_bytes() {
    let raw = std::fs::read_to_string(raid_frame_mainline_toc()).expect("TOC reads utf-8");

    assert!(raw.contains("## Title: Blizzard_RaidFrame"));
    assert!(raw.contains("## Author: Blizzard Entertainment"));
    assert!(
        raw.contains("## DefaultState: enabled"),
        "TOC must declare `## DefaultState: enabled`"
    );
    assert!(raw.contains(
        "## Dependencies: Blizzard_UIPanels_Game, Blizzard_UnitFrame, Blizzard_FriendsFrame"
    ));
    assert!(raw.contains("## AllowLoadGameType: mainline"));
    assert!(raw.contains("## OptionalDeps: Blizzard_SocialUIShared"));

    assert!(
        !raw.contains("## AllowLoad:"),
        "TOC must NOT carry any `## AllowLoad:` directive at all — the \
         Game-only restriction comes from the default behavior of \
         `allows_screen` at src/toc.rs:311 (the `None` arm returns \
         `screen == Game`). This is the IMPLICIT Game-only form, distinct \
         from QuickKeybind / QuickJoin which carry the explicit \
         `## AllowLoad: Game` directive"
    );
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
        "TOC must NOT carry OnlyBetaAndPTR — ships on live retail"
    );
    assert!(
        !raw.contains("## Version"),
        "TOC must NOT carry a Version directive — same missing-version \
         pattern as Blizzard_QuestTimer / Blizzard_QueueStatusFrame / \
         Blizzard_QuickJoin / Blizzard_QuickKeybind"
    );
}

#[test]
fn blizzard_raid_frame_toc_lists_two_files_lua_then_xml() {
    let toc = TocFile::from_file(&raid_frame_mainline_toc()).expect("TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, TOC_FILES,
        "Retail 12.1.0.69497 lists the core RaidFrame Lua/XML pair followed by \
         RaidFrameSocialView and RaidInfoFrameSocialView Lua/XML pairs"
    );
}

#[test]
fn blizzard_raid_frame_appears_in_eager_game_discovery() {
    let ui = blizzard_ui_dir();

    let game_addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let game_found = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_RaidFrame");
    assert!(
        game_found,
        "Blizzard_RaidFrame MUST appear in eager Game-screen discovery: no \
         `## LoadOnDemand:` (so `is_load_on_demand()` false), \
         `## AllowLoadGameType: mainline` (so `is_game_type_restricted()` \
         false), no explicit `## AllowLoad:` (defaults to Game per \
         src/toc.rs:311). All 3 loader filters at src/loader/mod.rs:527 \
         admit it"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let glue_addons = discover_blizzard_addons_for_screen(&ui, screen);
        let glue_found = glue_addons
            .iter()
            .any(|(name, _)| name == "Blizzard_RaidFrame");
        assert!(
            !glue_found,
            "Blizzard_RaidFrame must NOT appear in eager discovery for \
             {screen:?} — the implicit-Game default excludes glue-screen \
             load paths"
        );
    }
}

#[test]
fn blizzard_raid_frame_appears_in_full_addon_inventory() {
    let inventory = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = inventory
        .iter()
        .any(|(name, _)| name == "Blizzard_RaidFrame");
    assert!(
        found,
        "Blizzard_RaidFrame MUST appear in `discover_all_blizzard_addons`"
    );
}

prefork_full_ui_case! {
fn blizzard_raid_frame_loads_in_eager_game_sweep_without_lua_errors(env: &WowLuaEnv) {

    let addon_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_RaidFrame")
                || message.contains("RaidParentFrame")
                || message.contains("RaidFrame")
                || message.contains("RaidInfoFrame")
                || message.contains("RaidInfoInstance")
                || message.contains("RaidInfoExtendButton")
                || message.contains("ClaimRaidFrame")
        })
        .cloned()
        .collect();
    assert!(
        addon_errors.is_empty(),
        "Blizzard_RaidFrame emitted addon-specific Lua errors during eager \
         load:\n  {}",
        addon_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_raid_frame_publishes_nineteen_global_helper_functions(env: &WowLuaEnv) {

    for helper in PUBLIC_GLOBAL_HELPERS {
        let kind: String = env
            .eval(&format!("return type(_G[{helper:?}])"))
            .unwrap_or_else(|err| panic!("type probe for {helper} failed: {err}"));
        assert_eq!(
            kind, "function",
            "_G.{helper} must publish as a function — Blizzard_RaidFrame is \
             an OLDER-STYLE addon that PRE-DATES the mixin pattern: every \
             handler is a top-level free function dispatched via XML \
             `<OnLoad function=\"RaidFrame_OnLoad\"/>` rather than via a \
             `<OnLoad method=\"OnLoad\"/>` mixin method reference. There \
             are ZERO `*Mixin = {{}}` declarations in RaidFrame.lua. The 19 \
             public functions split into: 4 RaidFrame lifecycle handlers \
             (RaidFrame_OnLoad registers 9 events + ClaimRaidFrame initial \
             parent assign + ScrollBox view init + ScrollingDescription \
             setup; RaidFrame_OnShow toggles the attic + sets title text + \
             RaidFrame_Update + RequestRaidInfo + UpdateMicroButtons; \
             RaidFrame_OnEvent dispatches the 9 events to RaidFrame_Update \
             / RaidFrame_LoadUI / RaidPullout_RenewFrames / \
             RaidGroupFrame_Update / RaidInfoFrame_Update; \
             RaidFrame_Update toggles RaidFrameConvertToRaidButton + \
             RaidFrameNotInRaid visibility based on IsInRaid()), 2 \
             RaidParentFrame handlers (RaidParentFrame_OnLoad sets portrait \
             to UI-LFR-PORTRAIT + 2-tab template; RaidParentFrame_SetView \
             switches between tab 1 (Raid) and tab 2 (LFR) calling \
             ClaimRaidFrame), 1 conversion helper (RaidFrame_ConvertToRaid \
             routing into C_PartyInfo.ConvertToRaid), 1 cross-frame helper \
             (UpdateRaidAndPartyFrames calling PartyFrame:HidePartyFrames + \
             CompactRaidFrameManager_UpdateShown + \
             PartyFrame:UpdatePartyFrames), 8 RaidInfoFrame helpers \
             (RaidInfoFrame_InitButton populates per-row data from \
             GetSavedInstanceInfo / GetSavedWorldBossInfo with locked / \
             extended / reset formatting; RaidInfoFrame_SetButtonSelected \
             calls Lock/UnlockHighlight; RaidInfoFrame_Update rebuilds the \
             dataProvider from saved instances + saved world bosses; \
             RaidInfoInstance_OnMouseDown / OnMouseUp shift the name / \
             reset FontStrings by 2px to fake a press effect; \
             RaidInfoInstance_OnClick toggles selection or routes to \
             ChatFrameUtil.InsertLink on shift-click; RaidInfoInstance_OnEnter \
             sets up the GameTooltip via SetInstanceLockEncountersComplete; \
             RaidInfoFrame_UpdateButtons toggles the EXTEND_RAID_LOCK / \
             UNEXTEND_RAID_LOCK / REACTIVATE_RAID_LOCK button text and \
             enables/disables based on extendDisabled), 1 extend handler \
             (RaidInfoExtendButton_OnClick calls SetSavedInstanceExtend), 1 \
             everyone-assist toggle (RaidFrameAllAssistCheckButton_UpdateAvailable \
             reads IsEveryoneAssistant + UnitIsGroupLeader to enable/disable \
             the checkbox), and 1 reparent helper (ClaimRaidFrame — the \
             '4.3 Temp - Chaz' annotated function that sets RaidFrame's \
             parent then anchors TOPLEFT/BOTTOMRIGHT to fill the new parent \
             — the comment dates this as a Cataclysm-era hack that has \
             persisted to retail)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_raid_frame_publishes_two_named_top_level_frames(env: &WowLuaEnv) {

    for frame_name in PUBLIC_NAMED_FRAMES {
        let kind: String = env
            .eval(&format!("return type(_G[{frame_name:?}])"))
            .unwrap_or_else(|err| panic!("type probe for {frame_name} failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{frame_name} must publish as a frame userdata — \
             Blizzard_RaidFrame ships EXACTLY 2 named non-virtual top-level \
             frames in RaidFrame.xml: RaidParentFrame is the outer Frame \
             container at toplevel=true / parent=UIParent / movable=true / \
             enableMouse=true / hidden=true inheriting ButtonFrameTemplate \
             — it carries the 2 PanelTabButtonTemplate tabs ($parentTab1 \
             text=RAID id=1 routing to RaidParentFrame_SetView(1), \
             $parentTab2 text=LOOKING_FOR_RAID id=2 routing to \
             RaidParentFrame_SetView(2)) and the OnLoad handler that sets \
             the LFR portrait + 2-tab PanelTemplates state; RaidFrame is \
             the inner Frame whose parent is set DYNAMICALLY by \
             ClaimRaidFrame (defaulting to RaidParentFrame at OnLoad via \
             the explicit ClaimRaidFrame call at RaidFrame.lua:27, but \
             reparented to FriendsFrame when the user opens the FriendsFrame \
             raid tab — the `4.3 Temp - Chaz` comment in ClaimRaidFrame at \
             RaidFrame.lua:289 dates this dual-parent pattern back to the \
             Cataclysm 4.3 patch where the raid info was migrated from a \
             standalone window into the FriendsFrame tab strip while still \
             retaining the standalone RaidParentFrame for backwards \
             compatibility — it has persisted unchanged through every \
             expansion since)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_raid_frame_virtual_templates_not_in_global_env(env: &WowLuaEnv) {

    for template in VIRTUAL_TEMPLATES {
        let kind: String = env
            .eval(&format!("return type(_G[{template:?}])"))
            .unwrap_or_else(|err| panic!("type probe for {template} failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{template} must be nil — virtual templates live in the \
             template registry, NOT the global environment. The 2 virtual \
             templates Blizzard_RaidFrame ships are: RaidInfoHeaderTemplate \
             (Frame virtual template with 3 BACKGROUND-layer Textures \
             $parentLeft / $parentMiddle / $parentRight using the \
             Interface\\FriendsFrame\\WhoFrame-ColumnTabs sprite sheet to \
             render a horizontally-tiled column header — used by \
             RaidInfoInstanceLabel + RaidInfoIDLabel inside the \
             RaidInfoFrame to label the saved-instance scroll columns), \
             and RaidInfoInstanceTemplate (Button virtual template — the \
             per-row button used by the WowScrollBoxList view at \
             RaidInfoFrame.ScrollBox; carries 4 OVERLAY-layer FontStrings \
             $parentName / $parentDifficulty / $parentReset / $parentExtended \
             with parentKey=name/difficulty/reset/extended for the \
             RaidInfoFrame_InitButton handler to populate, plus 3 inline \
             scripts OnMouseDown / OnMouseUp / OnClick / OnEnter dispatching \
             to the matching free functions). Both are referenced by name \
             from CreateScrollBoxListLinearView():SetElementInitializer or \
             from <Frame inherits=> XML attributes inside RaidInfoFrame, \
             never instantiated as named singletons"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_raid_frame_registers_nine_events_in_onload(env: &WowLuaEnv) {

    for event in ON_LOAD_REGISTERED_EVENTS {
        let registered: bool = env
            .eval(&format!("return RaidFrame:IsEventRegistered({event:?})"))
            .unwrap_or_else(|err| panic!("event probe for {event} failed: {err}"));
        assert!(
            registered,
            "RaidFrame must register `{event}` in OnLoad — RaidFrame_OnLoad \
             at RaidFrame.lua:8-17 calls RegisterEvent for 9 distinct \
             game events: PLAYER_LOGIN (initial raid-state seed; if \
             IsInRaid() at login the handler eagerly calls RaidFrame_LoadUI \
             + RaidFrame_Update so the raid panel is ready before the user \
             opens the FriendsFrame), GROUP_ROSTER_UPDATE (party / raid \
             roster changed — drives RaidFrame_LoadUI + RaidFrame_Update + \
             RaidPullout_RenewFrames), UPDATE_INSTANCE_INFO (saved-instance \
             info refresh — drives RaidInfoFrame_Update; the first event \
             instance flips the `RaidFrame.hasRaidInfo` flag and returns \
             early to skip the redundant first refresh), \
             PARTY_LEADER_CHANGED (party leader changed — drives \
             RaidFrame_Update so the convert-to-raid button enabled state \
             reflects the new leader's permissions), PLAYER_ENTERING_WORLD \
             (zone-in or initial world entry — drives RequestRaidInfo so \
             the saved-instance backing store gets refreshed), READY_CHECK \
             (a ready check started — drives RaidGroupFrame_Update if the \
             RaidFrame is visible), READY_CHECK_CONFIRM (a player \
             confirmed/declined the ready check — drives \
             RaidGroupFrame_Update), READY_CHECK_FINISHED (ready check \
             ended — drives RaidGroupFrame_ReadyCheckFinished), and \
             PARTY_LFG_RESTRICTED (LFG dungeon restrictions changed — \
             drives RaidFrame_Update). NOTE: RaidParentFrame_OnLoad does \
             NOT register any events directly; all event handling routes \
             through the inner RaidFrame, while RaidParentFrame is purely \
             a visual container managing the tab strip"
        );
    }
}
}
