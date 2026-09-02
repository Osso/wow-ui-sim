use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::loader::{discover_all_blizzard_addons, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn raid_ui_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_RaidUI")
}

fn raid_ui_toc() -> PathBuf {
    find_toc_file(&raid_ui_dir()).expect("Blizzard_RaidUI active TOC resolves")
}

const TOC_FILES: &[&str] = &[
    "Blizzard_RaidUI_Bootstrap.lua",
    "Mainline/Blizzard_RaidUI.xml",
    "Localization.lua",
];

const TOP_LEVEL_FUNCTIONS: &[&str] = &[
    "RaidClassButton_OnLoad",
    "RaidClassButton_Update",
    "RaidClassButton_OnEnter",
    "RaidGroupFrame_OnLoad",
    "RaidGroupFrame_OnHide",
    "RaidGroupFrame_OnEvent",
    "RaidGroup_ResetSlotButtons",
    "RaidGroupFrame_Update",
    "RaidGroupFrame_UpdateLevel",
    "RaidGroupFrame_UpdateHealth",
    "RaidGroupFrame_OnUpdate",
    "RaidGroupFrame_ReadyCheckFinished",
    "RaidGroupButton_OpenMenu",
    "RaidGroupButton_OnLoad",
    "RaidGroupButton_OnDragStart",
    "RaidGroupButton_OnDragStop",
    "RaidGroupButton_OnEnter",
    "RaidButton_OnClick",
    "RaidPullout_OnEvent",
    "RaidPullout_ReadyCheckFinished",
    "RaidPullout_ReadyCheckFinishFunc",
    "RaidPullout_GeneratePulloutFrame",
    "RaidPullout_UpdateTarget",
    "RaidPullout_OnUpdate",
    "RaidPullout_Update",
    "RaidPulloutButton_OnEvent",
    "RaidPulloutButton_UpdateSwapFrames",
    "RaidPulloutButton_UpdateDead",
    "RaidPulloutButton_OnLoad",
    "RaidPulloutButton_OnDragStart",
    "RaidPulloutStopMoving",
    "RaidPullout_SaveFrames",
    "RaidPullout_RenewFrames",
    "RaidPullout_MatchName",
    "RaidPullout_GetFrame",
];

const VIRTUAL_TEMPLATES: &[&str] = &[
    "RaidClassButtonTemplate",
    "RaidRoleIconTemplate",
    "RaidGroupButtonTemplate",
    "RaidGroupSlotTemplate",
    "RaidGroupTemplate",
    "RaidAuraFrameTemplate",
    "RaidPulloutButtonTemplate",
    "RaidPulloutFrameTemplate",
];

const ON_LOAD_REGISTERED_EVENTS: &[&str] = &[
    "UNIT_PET",
    "UNIT_NAME_UPDATE",
    "UNIT_LEVEL",
    "UNIT_HEALTH",
    "PLAYER_ENTERING_WORLD",
    "VARIABLES_LOADED",
    "RAID_ROSTER_UPDATE",
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
fn blizzard_raid_ui_find_toc_resolves_active_mainline_variant() {
    assert_eq!(
        raid_ui_toc(),
        raid_ui_dir().join("Blizzard_RaidUI_Mainline.toc"),
        "Retail must select Blizzard_RaidUI_Mainline.toc through find_toc_file."
    );
}

#[test]
fn blizzard_raid_ui_toc_pins_load_on_demand_with_per_character_saved_vars() {
    let toc = TocFile::from_file(&raid_ui_toc()).expect("Blizzard_RaidUI TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "TOC must declare `## LoadOnDemand: 1` — the raid pullout windows \
         (RaidPulloutFrame1..N) and the 16 RaidClassButton + 8 RaidGroup + \
         40 RaidGroupButton frame array allocate hundreds of frames that \
         only need to exist while the user is actively raid-leading. \
         Triggered via `RaidFrame_LoadUI()` at \
         Blizzard_UIParent/Shared/UIParent.lua:296-298 which calls \
         `UIParentLoadAddOn(\"Blizzard_RaidUI\")` from the \
         Blizzard_RaidFrame OnEvent handler at RaidFrame.lua:57-62 only when \
         `IsInRaid()` is true at PLAYER_LOGIN or on GROUP_ROSTER_UPDATE"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(!toc.is_ptr_only());

    assert!(
        toc.saved_variables().is_empty(),
        "TOC must declare ZERO `## SavedVariables:` (account-wide) — all \
         raid pullout state is per-character because the player's class / \
         spec / role determines which pullout frames are useful and where \
         the user wants them docked"
    );
    let per_char = toc.saved_variables_per_character();
    assert_eq!(
        per_char,
        vec![
            "RAID_PULLOUT_POSITIONS".to_string(),
            "RAID_SINGLE_POSITIONS".to_string(),
        ],
        "TOC must declare exactly TWO `## SavedVariablesPerCharacter:` \
         globals in declaration order: RAID_PULLOUT_POSITIONS (per-class \
         pullout-frame positions keyed by filterID — DRUID / HUNTER / \
         WARLOCK / ... / PETS / MAINTANK / MAINASSIST), RAID_SINGLE_POSITIONS \
         (the single-target pullout positions array — uses `tinsert(_, 1, _)` \
         to push to the front so the most-recently-saved frame is restored \
         first by RaidPullout_RenewFrames). Both globals are declared at \
         Blizzard_RaidUI.lua:9-10 as empty tables, populated by \
         RaidPullout_SaveFrames (lua:1067), and consumed by \
         RaidPullout_RenewFrames (lua:1119)"
    );

    assert!(
        !toc.is_game_type_restricted(),
        "TOC must NOT declare `## AllowLoadGameType:` at all — without the \
         directive, `is_game_type_restricted()` at src/toc.rs:294-302 \
         returns FALSE via the `unwrap_or(false)` at line 301. The addon \
         loads on every flavor that supports raids"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Without an `## AllowLoad:` directive the `None` arm at \
         src/toc.rs:311 defaults to Game-only — same implicit form as \
         Blizzard_RaidFrame"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(!toc.allows_screen(screen));
    }
}

#[test]
fn blizzard_raid_ui_toc_declares_no_dependencies() {
    let toc = TocFile::from_file(&raid_ui_toc()).expect("TOC parses");

    assert!(
        toc.dependencies().is_empty(),
        "TOC must declare ZERO `## Dependencies:` entries — even though \
         every numbered button frame (RaidClassButton1..16, \
         RaidGroupButton1..40 inside RaidGroup1..8) parents to RaidFrame \
         from Blizzard_RaidFrame. The dep is satisfied implicitly: the only \
         caller of LoadOnDemand path (Blizzard_RaidFrame's RaidFrame_OnEvent \
         at PLAYER_LOGIN/GROUP_ROSTER_UPDATE) ALREADY has Blizzard_RaidFrame \
         loaded, so RaidFrame is guaranteed to exist as a global by the \
         time `UIParentLoadAddOn(\"Blizzard_RaidUI\")` fires. Declaring \
         Blizzard_RaidFrame as a hard dep would create a circular eager \
         load on first install"
    );
    assert!(toc.optional_deps().is_empty());
    assert!(toc.load_with().is_empty());
}

#[test]
fn blizzard_raid_ui_toc_declares_metadata_in_raw_bytes() {
    let raw = std::fs::read_to_string(raid_ui_toc()).expect("TOC reads utf-8");

    assert!(
        raw.contains("## Title: Blizzard Raid UI"),
        "TOC must declare the SPACED title `## Title: Blizzard Raid UI` \
         (with whitespace separators, NOT the underscored form \
         `Blizzard_RaidUI`). Distinct from sibling Blizzard_RaidFrame which \
         uses `## Title: Blizzard_RaidFrame` (underscored). The user-facing \
         AddOns list shows the spaced form; the internal addon name is \
         still Blizzard_RaidUI"
    );
    assert!(raw.contains("## LoadOnDemand: 1"));
    assert!(raw
        .contains("## SavedVariablesPerCharacter: RAID_PULLOUT_POSITIONS, RAID_SINGLE_POSITIONS"));

    assert!(!raw.contains("## Author"));
    assert!(!raw.contains("## DefaultState"));
    assert!(!raw.contains("## Dependencies"));
    assert!(
        !raw.contains("## OptionalDeps"),
        "TOC must NOT carry OptionalDeps"
    );
    assert!(
        raw.contains("## AllowLoad: game"),
        "Retail 12.1.0.69497 explicitly gates Blizzard_RaidUI to game screens"
    );
    assert!(
        raw.contains("## AllowLoadGameType: mainline"),
        "Retail 12.1.0.69497 restricts this TOC to mainline"
    );
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("## Version"));
    assert!(!raw.contains("## OnlyBetaAndPTR"));
    assert!(
        !raw.contains("## SavedVariables:"),
        "TOC must NOT carry the regular `## SavedVariables:` directive — \
         only the per-character form. Regex-anchored to the colon to avoid \
         matching the SavedVariablesPerCharacter prefix"
    );
}

#[test]
fn blizzard_raid_ui_toc_lists_bootstrap_xml_then_localization() {
    let toc = TocFile::from_file(&raid_ui_toc()).expect("TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, TOC_FILES,
        "TOC body lists EXACTLY 2 files in this order: \
         Blizzard_RaidUI.xml FIRST (publishes 8 virtual templates + the 64 \
         numbered named frames RaidClassButton1..16 + RaidGroup1..8 + \
         RaidGroupButton1..40, AND pulls in Blizzard_RaidUI.lua via \
         `<Script file=\"Blizzard_RaidUI.lua\"/>` at xml:3 — the modern \
         XML-driven Lua-loading pattern), Localization.lua SECOND (a 50-byte \
         file that is just a single comment `-- This file is executed at \
         the end of addon load`, kept around as a marker hook for any \
         locale-specific override addon to inject extra strings via overlay \
         loading). The companion .lua file is NOT listed in the TOC body \
         directly — distinct from Blizzard_RaidFrame which lists \
         Mainline/RaidFrame.lua then Mainline/RaidFrame.xml"
    );
}

#[test]
fn blizzard_raid_ui_is_pulled_into_game_discovery_by_raid_frame() {
    let ui = blizzard_ui_dir();
    let game_addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    assert!(
        game_addons
            .iter()
            .any(|(name, _)| name == "Blizzard_RaidUI"),
        "Retail game discovery includes LoadOnDemand Blizzard_RaidUI through the explicit \
         Blizzard_RaidUI -> Blizzard_RaidFrame startup dependency"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !discover_blizzard_addons_for_screen(&ui, screen)
                .iter()
                .any(|(name, _)| name == "Blizzard_RaidUI"),
            "Blizzard_RaidUI remains absent from {screen:?} discovery"
        );
    }
}

#[test]
fn blizzard_raid_ui_appears_in_full_addon_inventory() {
    let inventory = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = inventory.iter().any(|(name, _)| name == "Blizzard_RaidUI");
    assert!(
        found,
        "Blizzard_RaidUI MUST appear in `discover_all_blizzard_addons` — \
         the full inventory walk includes LOD addons so tooling and audit \
         scripts can enumerate every addon shipped, even those that are \
         only loaded at runtime"
    );
}

prefork_full_ui_case! {
fn blizzard_raid_ui_is_loaded_via_runtime_loadaddon_on_group_roster_update(env: &WowLuaEnv) {

    let already_loaded: bool = env
        .eval(
            "return type(C_AddOns) == 'table' \
                and type(C_AddOns.IsAddOnLoaded) == 'function' \
                and (C_AddOns.IsAddOnLoaded('Blizzard_RaidUI') and true or false)",
        )
        .expect("IsAddOnLoaded query should succeed");
    assert!(
        already_loaded,
        "Blizzard_RaidUI MUST be loaded after a normal Game-screen startup \
         — same pattern as Blizzard_CombatLog. The trigger is \
         Blizzard_RaidFrame's RaidFrame_OnEvent at RaidFrame.lua:61-64 \
         which UNCONDITIONALLY calls `RaidFrame_LoadUI()` on \
         GROUP_ROSTER_UPDATE (NO `IsInRaid()` guard on the \
         GROUP_ROSTER_UPDATE branch — only the PLAYER_LOGIN branch at \
         lua:56-60 has that guard). Since the simulator's startup sequence \
         fires GROUP_ROSTER_UPDATE during fire_startup_events_for_screen, \
         the LoadAddOn cascade walks UIParentLoadAddOn -> LoadAddOn -> \
         the loader's runtime LOD path and Blizzard_RaidUI ends up loaded \
         even though the player is solo. Distinct from \
         Blizzard_Commentator (also LOD) which has NO eager addon \
         triggering its load — Commentator stays unloaded until the \
         spectator client explicitly triggers it"
    );
}
}

prefork_full_ui_case! {
fn blizzard_raid_ui_loads_without_errors_during_startup(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_RaidUI")
                || message.contains("RaidPullout")
                || message.contains("RaidGroupFrame")
                || message.contains("RaidGroupButton")
                || message.contains("RaidClassButton")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_RaidUI emitted addon-specific Lua errors during runtime \
         load via GROUP_ROSTER_UPDATE handler:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_raid_ui_publishes_module_constants_after_load(env: &WowLuaEnv) {

    let constants_present: bool = env
        .eval(
            "return MAX_RAID_GROUPS == 8 \
                and RAID_RANGE_ALPHA == 0.5 \
                and RAID_PULLOUT_BUTTON_HEIGHT == 33 \
                and MAX_RAID_AURAS == 4 \
                and MOVING_RAID_MEMBER == nil \
                and TARGET_RAID_SLOT == nil \
                and MOVING_RAID_PULLOUT == nil \
                and NUM_RAID_PULLOUT_FRAMES == 0 \
                and type(RAID_SUBGROUP_LISTS) == 'table' \
                and type(RAID_PULLOUT_POSITIONS) == 'table' \
                and type(RAID_SINGLE_POSITIONS) == 'table' \
                and type(RAID_CLASS_BUTTONS) == 'table' \
                and type(RAID_PULLOUT_SAVED_SETTINGS) == 'table' \
                and RAID_PULLOUT_SAVED_SETTINGS.showTarget == true \
                and RAID_PULLOUT_SAVED_SETTINGS.showBuffs == true \
                and RAID_PULLOUT_SAVED_SETTINGS.showTargetTarget == true \
                and RAID_PULLOUT_SAVED_SETTINGS.showDebuffs == true \
                and RAID_PULLOUT_SAVED_SETTINGS.showBG == true \
                and MAX_RAID_CLASS_BUTTONS == MAX_CLASSES + 3",
        )
        .expect("constants probe should succeed");
    assert!(
        constants_present,
        "Blizzard_RaidUI must publish its module-level constants and tables \
         declared at lua:1-31. MAX_RAID_GROUPS=8 (the engine cap on raid \
         subgroups — drives the RaidGroup1..8 named frame array). \
         RAID_RANGE_ALPHA=0.5 (out-of-range alpha for pullout-button \
         portraits). RAID_PULLOUT_BUTTON_HEIGHT=33 (per-row pixel height for \
         dynamic pullout-frame layout). MAX_RAID_AURAS=4 (auras-per-row cap \
         drives the RaidAuraFrameTemplate $parentAura1..4 children). \
         RAID_CLASS_BUTTONS is populated in the do-block at lua:14-22 \
         keyed by class-token (DRUID / WARLOCK / ... / PETS=11 / \
         MAINTANK=12 / MAINASSIST=13) to button=1..13 + coords entry, so \
         RaidClassButton_OnLoad can lookup which button index to set up. \
         MAX_RAID_CLASS_BUTTONS=MAX_CLASSES+3 (the +3 covers PETS / \
         MAINTANK / MAINASSIST pseudo-classes appended after real classes). \
         RAID_PULLOUT_SAVED_SETTINGS is the default per-pullout settings \
         table merged into RAID_PULLOUT_POSITIONS[filterID].settings on save"
    );
}
}

prefork_full_ui_case! {
fn blizzard_raid_ui_publishes_top_level_handler_functions(env: &WowLuaEnv) {

    for func_name in TOP_LEVEL_FUNCTIONS {
        let kind: String = env
            .eval(&format!("return type(_G[{func_name:?}])"))
            .unwrap_or_else(|err| panic!("type probe for {func_name} failed: {err}"));
        assert_eq!(
            kind, "function",
            "_G.{func_name} must publish as a function — Blizzard_RaidUI is \
             an OLDER-STYLE addon (pre-mixin pattern, same shape as \
             sibling Blizzard_RaidFrame) where every handler is a top-level \
             free function dispatched via XML \
             `<OnLoad function=\"X\"/>` rather than via mixin methods. The \
             35.4K Lua file declares 35 such free functions across 4 logical \
             clusters: 3 RaidClassButton_* (top-strip class-icon buttons \
             showing PERS / MAINTANK / MAINASSIST counts), 9 RaidGroupFrame_* \
             + RaidGroup_ResetSlotButtons (the persistent in-world raid \
             panel — RaidGroupFrame_OnLoad is the EXTRA event registration \
             on RaidFrame from this addon, registering UNIT_PET / \
             UNIT_NAME_UPDATE / UNIT_LEVEL / UNIT_HEALTH / \
             PLAYER_ENTERING_WORLD / VARIABLES_LOADED / RAID_ROSTER_UPDATE \
             on TOP of the events RaidFrame_OnLoad already registers from \
             Blizzard_RaidFrame), 6 RaidGroupButton_* + RaidButton_OnClick \
             (per-slot buttons inheriting SecureUnitButtonTemplate — the \
             :1 raid member buttons inside each RaidGroup1..8 frame), and \
             18 RaidPullout_* / RaidPulloutButton_* / RaidPulloutStopMoving \
             (the detached class-pullout windows the user can drag out of \
             the main raid panel — RaidPullout_GeneratePulloutFrame at \
             lua:649 dynamically reuses pooled RaidPulloutFrame1..N \
             instances, RaidPullout_SaveFrames at lua:1067 captures the \
             positions into RAID_PULLOUT_POSITIONS / RAID_SINGLE_POSITIONS \
             on each drag-stop, RaidPullout_RenewFrames at lua:1119 \
             restores them after VARIABLES_LOADED)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_raid_ui_publishes_named_numbered_frame_arrays(env: &WowLuaEnv) {

    let class_buttons_present: bool = env
        .eval(
            "for i = 1, 16 do \
                if type(_G['RaidClassButton'..i]) ~= 'table' then return false end \
            end \
            return true",
        )
        .expect("RaidClassButton1..16 probe should succeed");
    assert!(
        class_buttons_present,
        "RaidClassButton1..16 must all publish as named globals — the \
         16-button strip across the top of the raid panel showing per-class \
         counts. IDs 1..10 map to the real classes via CLASS_SORT_ORDER, \
         11=PETS, 12=MAINTANK, 13=MAINASSIST, 14..16 are reserved padding \
         slots for future class additions (the array is sized MAX_CLASSES+3 \
         but historically Blizzard pre-allocated extra frames for forward \
         compatibility). All 16 inherit RaidClassButtonTemplate and parent \
         to RaidFrame from Blizzard_RaidFrame"
    );

    let groups_present: bool = env
        .eval(
            "for i = 1, 8 do \
                if type(_G['RaidGroup'..i]) ~= 'table' then return false end \
            end \
            return true",
        )
        .expect("RaidGroup1..8 probe should succeed");
    assert!(
        groups_present,
        "RaidGroup1..8 must all publish as named globals — one frame per \
         raid subgroup (the engine cap MAX_RAID_GROUPS=8). Each inherits \
         RaidGroupTemplate, parents to RaidFrame, and contains 5 \
         RaidGroupSlotTemplate child buttons ($parentSlot1..5) + a \
         $parentLabel for the group number — laid out as 8 columns of 5 \
         rows = 40 max member slots"
    );

    let group_buttons_present: bool = env
        .eval(
            "for i = 1, 40 do \
                if type(_G['RaidGroupButton'..i]) ~= 'table' then return false end \
            end \
            return true",
        )
        .expect("RaidGroupButton1..40 probe should succeed");
    assert!(
        group_buttons_present,
        "RaidGroupButton1..40 must all publish as named globals — the 40 \
         per-member buttons inheriting RaidGroupButtonTemplate (which itself \
         inherits SecureUnitButtonTemplate so the protected-frame \
         `unit=raid<i>` attribute can route clicks to TargetUnit). 8 groups \
         × 5 slots = 40, matching the engine's MAX_RAID_MEMBERS=40 cap. \
         The numbered globals exist for legacy `_G[\"RaidGroupButton\"..i]` \
         lookups — modern code accesses them via the RaidGroup1..8 / \
         $parentSlot1..5 hierarchy"
    );
}
}

prefork_full_ui_case! {
fn blizzard_raid_ui_virtual_templates_not_in_global_env(env: &WowLuaEnv) {

    for template in VIRTUAL_TEMPLATES {
        let kind: String = env
            .eval(&format!("return type(_G[{template:?}])"))
            .unwrap_or_else(|err| panic!("type probe for {template} failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{template} must be nil — virtual templates live in the \
             template registry, NOT the global environment. The 8 virtual \
             templates Blizzard_RaidUI ships are: RaidClassButtonTemplate \
             (top-strip class-count buttons with $parentIconTexture + \
             $parentCount NumberFontNormalSmall), RaidRoleIconTemplate \
             (the rank/role icon used by $parentRank + $parentRole inside \
             RaidGroupButtonTemplate), RaidGroupButtonTemplate (per-member \
             SecureUnitButtonTemplate-derived button — parent=RaidFrame, \
             movable=true, clampedToScreen=true), RaidGroupSlotTemplate \
             (the 5 per-group placeholder slots that RaidGroupButton1..40 \
             snap into), RaidGroupTemplate (Frame parent=RaidFrame holding \
             $parentLabel + $parentSlot1..5), RaidAuraFrameTemplate (the \
             $parentAura1..4 children inside RaidPulloutButtonTemplate \
             showing buffs/debuffs), RaidPulloutButtonTemplate (per-row \
             button inside a pullout window — health bar / mana bar / \
             target / target-of-target / 4 auras / clear button — \
             enableMouse=true, hidden=true), RaidPulloutFrameTemplate \
             (Button toplevel=true parent=UIParent movable=true \
             clampedToScreen=true hidden=true — the detachable pullout \
             window itself). None of these are instantiated as named \
             singletons in this addon"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_raid_ui_registers_extra_events_on_raid_frame(env: &WowLuaEnv) {

    for event in ON_LOAD_REGISTERED_EVENTS {
        let registered: bool = env
            .eval(&format!("return RaidFrame:IsEventRegistered({event:?})"))
            .unwrap_or_else(|err| panic!("event probe for {event} failed: {err}"));
        assert!(
            registered,
            "RaidFrame must have `{event}` registered after Blizzard_RaidUI \
             loads — RaidGroupFrame_OnLoad at lua:145-158 calls \
             RegisterEvent on the EXISTING RaidFrame from Blizzard_RaidFrame \
             for 7 EXTRA events on TOP of the 9 events already registered \
             by Blizzard_RaidFrame's own RaidFrame_OnLoad. The 7 added \
             events: UNIT_PET (raid pet roster changed — drives \
             RaidGroupFrame_Update), UNIT_NAME_UPDATE (raid member name \
             refresh), UNIT_LEVEL (raid member level changed — drives \
             RaidGroupFrame_UpdateLevel), UNIT_HEALTH (drives \
             RaidGroupFrame_UpdateHealth), PLAYER_ENTERING_WORLD (zone-in — \
             drives full RaidGroupFrame_Update), VARIABLES_LOADED (saved \
             vars ready — drives RaidPullout_RenewFrames to restore the \
             saved RAID_PULLOUT_POSITIONS / RAID_SINGLE_POSITIONS), \
             RAID_ROSTER_UPDATE (full raid roster changed — drives \
             RaidGroupFrame_Update + RaidClassButton_Update). The OnEvent \
             handler is also rebound to RaidGroupFrame_OnEvent via \
             SetScript at lua:154"
        );
    }
}
}

#[test]
fn blizzard_raid_ui_xml_pulls_lua_via_script_directive() {
    let xml_path = raid_ui_dir().join("Mainline/Blizzard_RaidUI.xml");
    let xml = std::fs::read_to_string(&xml_path).expect("XML reads utf-8");
    assert!(
        xml.contains(r#"<Script file="Blizzard_RaidUI.lua"/>"#),
        "Blizzard_RaidUI.xml must include `<Script file=\"Blizzard_RaidUI.lua\"/>` \
         at the top — the modern XML-driven Lua-loading pattern. The \
         companion .lua file is NOT listed in the TOC body directly; it is \
         loaded transitively when the XML parser hits the Script directive. \
         Distinct from the older-style sibling Blizzard_RaidFrame which \
         lists Mainline/RaidFrame.lua AND Mainline/RaidFrame.xml \
         explicitly in the TOC body"
    );
}
