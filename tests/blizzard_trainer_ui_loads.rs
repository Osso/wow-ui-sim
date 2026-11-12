use std::path::PathBuf;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
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

fn trainer_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_TrainerUI")
}

fn trainer_toc() -> PathBuf {
    find_toc_file(&trainer_dir()).expect("Blizzard_TrainerUI TOC should resolve")
}

const ALL_FOUR_SCREENS: &[ScreenKind] = &[
    ScreenKind::Game,
    ScreenKind::Login,
    ScreenKind::CharacterSelect,
    ScreenKind::CharacterCreate,
];

const PUBLISHED_GLOBAL_FUNCTIONS: &[&str] = &[
    "ClassTrainerFrame_OnLoad",
    "ClassTrainerFrame_OnShow",
    "ClassTrainerFrame_OnHide",
    "ClassTrainerFrame_OnEvent",
    "ClassTrainerFrame_SetTrainButtonEnabled",
    "ClassTrainerFrame_Update",
    "ClassTrainerFrame_InitServiceButton",
    "ClassTrainer_SelectNearestLearnableSkill",
    "ClassTrainer_SetSelection",
    "ClassTrainerSkillButton_OnClick",
    "ClassTrainerTrainButton_OnClick",
];

const PUBLISHED_CONSTANTS: &[(&str, i64)] = &[
    ("CLASS_TRAINER_SKILLS_DISPLAYED", 7),
    ("CLASS_TRAINER_SCROLL_HEIGHT", 330),
    ("CLASS_TRAINER_SKILL_BUTTON_WIDTH", 318),
    ("CLASS_TRAINER_SKILL_BARBUTTON_WIDTH", 298),
    ("CLASS_TRAINER_SKILL_HEIGHT", 47),
    ("MAX_LEARNABLE_PROFESSIONS", 2),
];

fn fresh_game_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    env
}

fn load_full_game_ui() -> WowLuaEnv {
    let env = fresh_game_env();

    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn find_toc_file_resolves_mainline_toc() {
    let resolved = trainer_toc();
    assert_eq!(
        resolved.file_name().and_then(|name| name.to_str()),
        Some(toc_layout::expected_toc_name("Blizzard_TrainerUI").as_str()),
        "find_toc_file resolves the active profile's TrainerUI TOC: the Retail cache ships \
         `_Mainline.toc`, while the pinned PTR cache ships the bare unified TOC"
    );
}

#[test]
fn toc_is_load_on_demand_with_no_dependencies() {
    let toc = TocFile::from_file(&trainer_toc()).expect("TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "`## LoadOnDemand: 1` — only loads when the player engages an \
         NPC trainer; PlayerInteractionFrameManager.lua:29-35 maps \
         Enum.PlayerInteractionType.Trainer to loadFunc=\
         ClassTrainerFrame_LoadUI, which calls \
         UIParentLoadAddOn(\"Blizzard_TrainerUI\") at \
         UIParent.lua:265-267"
    );
    assert!(
        toc.dependencies().is_empty(),
        "No `## Dependencies:` directive — TrainerUI is a small classic-\
         era addon that relies only on the always-loaded Blizzard \
         FrameXML core (ButtonFrameTemplate, MagicButtonTemplate, \
         SmallMoneyFrameTemplate, WowScrollBoxList, MinimalScrollBar, \
         WowStyle1FilterDropdownTemplate, InsetFrameTemplate). Got: {:?}",
        toc.dependencies()
    );
    assert!(toc.optional_deps().is_empty());
    if toc_layout::wrath() {
        assert!(toc.saved_variables().is_empty());
    } else if toc_layout::classic_cache() {
        assert_eq!(
            toc.saved_variables(),
            vec![
                "TRAINER_FILTER_AVAILABLE_BOOL".to_string(),
                "TRAINER_FILTER_UNAVAILABLE_BOOL".to_string(),
                "TRAINER_FILTER_USED_BOOL".to_string(),
            ],
            "the classic Blizzard_TrainerUI.toc declares the three trainer filter flags"
        );
    } else {
        assert!(toc.saved_variables().is_empty());
    }
    assert!(toc.saved_variables_per_character().is_empty());
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(
        !toc.is_game_type_restricted(),
        "the active profile's TrainerUI TOC carries no restricting game-type directive"
    );
    assert!(toc.default_enabled());
}

#[test]
fn allow_load_game_restricts_to_in_world_screen() {
    let toc = TocFile::from_file(&trainer_toc()).expect("TOC parses");

    assert!(toc.allows_screen(ScreenKind::Game));
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "AllowLoad: game excludes TrainerUI from {screen:?}"
        );
    }
}

#[test]
fn toc_raw_bytes_pin_current_mainline_contract() {
    let raw = std::fs::read_to_string(trainer_toc()).expect("TOC reads utf-8");
    let expected_lines: &[&str] = if toc_layout::wrath() {
        &[
            "## Interface: 30300",
            "## Title: Blizzard Trainer UI",
            "## Secure: 1",
            "## LoadOnDemand: 1",
            "Blizzard_TrainerUI.xml",
            "Localization.lua",
        ]
    } else if toc_layout::classic_cache() {
        &[
            "## Title: Blizzard Trainer UI",
            "## LoadOnDemand: 1",
            "## SavedVariables: TRAINER_FILTER_AVAILABLE_BOOL, TRAINER_FILTER_UNAVAILABLE_BOOL, TRAINER_FILTER_USED_BOOL",
            "Blizzard_TrainerUI.xml",
            "Localization.lua",
        ]
    } else if toc_layout::unified_toc_tree() {
        &[
            "## Title: Blizzard_TrainerUI",
            "## LoadOnDemand: 1",
            "## SavedVariables: TRAINER_FILTER_AVAILABLE_BOOL, TRAINER_FILTER_UNAVAILABLE_BOOL, TRAINER_FILTER_USED_BOOL [AllowLoadGameType classic]",
            "",
            "Blizzard_TrainerUI_Bootstrap.lua [Bootstrap]",
            "[Family]\\Blizzard_TrainerUI.lua",
            "[Family]\\Blizzard_TrainerUI_Camelot.lua [AllowLoadGameType camelot]",
            "[Family]\\Blizzard_TrainerUI.xml",
            "Localization.lua",
        ]
    } else {
        &[
            "## Title: Blizzard Trainer UI",
            "## LoadOnDemand: 1",
            "## AllowLoad: game",
            "## AllowLoadGameType: mainline",
            "Blizzard_TrainerUI_Bootstrap.lua [Bootstrap]",
            "Mainline\\Blizzard_TrainerUI.xml",
            "Localization.lua",
        ]
    };
    assert_eq!(
        raw.lines().collect::<Vec<_>>(),
        expected_lines,
        "TrainerUI TOC must retain its current directives, bootstrap, and Mainline body: the \
         unified PTR TOC lists the family sources with per-entry game-type gates instead of \
         top-level AllowLoad directives"
    );
}

#[test]
fn body_resolves_profile_lua_xml_and_localization_order() {
    let toc = TocFile::from_file(&trainer_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let expected = if toc_layout::unified_toc_tree() {
        vec![
            "Blizzard_TrainerUI_Bootstrap.lua",
            "Mainline/Blizzard_TrainerUI.lua",
            "Mainline/Blizzard_TrainerUI.xml",
            "Localization.lua",
        ]
    } else {
        vec![
            "Blizzard_TrainerUI_Bootstrap.lua",
            "Mainline/Blizzard_TrainerUI.xml",
            "Localization.lua",
        ]
    };
    assert_eq!(
        body,
        expected.into_iter().map(str::to_string).collect::<Vec<_>>(),
        "TrainerUI body follows the active profile TOC. The unified PTR/Forever TOC line 6 \
         selects the Mainline Lua file directly before its XML entry. Got: {body:?}"
    );
}

#[test]
fn trainer_ui_is_game_startup_publisher_only() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    assert!(
        game_addons
            .iter()
            .any(|(name, _)| name == "Blizzard_TrainerUI"),
        "Blizzard_TrainerUI remains `## LoadOnDemand: 1` but is selected on Game so its \
         bootstrap publishes ClassTrainerFrame_LoadUI before startup interaction registration"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        assert!(
            !addons.iter().any(|(name, _)| name == "Blizzard_TrainerUI"),
            "Blizzard_TrainerUI must remain absent from non-game discovery ({screen:?})"
        );
    }
}

#[test]
fn no_addon_declares_trainer_ui_as_dependency() {
    let entries = std::fs::read_dir(blizzard_ui_dir()).expect("BlizzardUI dir reads");
    let mut declarers: Vec<String> = Vec::new();

    for entry in entries.flatten() {
        let addon_dir = entry.path();
        if !addon_dir.is_dir() {
            continue;
        }
        let Some(toc_path) = find_toc_file(&addon_dir) else {
            continue;
        };
        let Ok(toc) = TocFile::from_file(&toc_path) else {
            continue;
        };
        let declared = toc.dependencies().iter().any(|d| d == "Blizzard_TrainerUI")
            || toc
                .optional_deps()
                .iter()
                .any(|d| d == "Blizzard_TrainerUI");
        if declared {
            let name = addon_dir.file_name().unwrap().to_string_lossy().to_string();
            declarers.push(name);
        }
    }

    assert!(
        declarers.is_empty(),
        "No Blizzard addon may declare Blizzard_TrainerUI as a hard or \
         optional dep — strictly LoD, triggered ONLY by \
         PlayerInteractionFrameManager when the player engages a \
         trainer NPC. Found declarers: {declarers:?}"
    );
}

prefork_full_ui_case! {
fn explicit_load_publishes_constants(env: &WowLuaEnv) {

    load_addon(&env.loader_env(), &trainer_toc())
        .expect("Blizzard_TrainerUI must load via Rust loader");

    for (name, expected) in PUBLISHED_CONSTANTS {
        let actual: i64 = env
            .eval(&format!("return {name}"))
            .unwrap_or_else(|err| panic!("{name} probe failed: {err}"));
        assert_eq!(
            actual, *expected,
            "Global constant `{name}` must equal {expected} after LoD \
             load — declared at Blizzard_TrainerUI.lua lines 2-7. \
             These pin layout dimensions for the scroll list and the \
             profession-cap MAX_LEARNABLE_PROFESSIONS=2. Got {actual}"
        );
    }
}
}

prefork_full_ui_case! {
fn explicit_load_publishes_global_functions(env: &WowLuaEnv) {

    load_addon(&env.loader_env(), &trainer_toc())
        .expect("Blizzard_TrainerUI must load via Rust loader");

    for fn_name in PUBLISHED_GLOBAL_FUNCTIONS {
        let kind: String = env
            .eval(&format!("return type({fn_name})"))
            .unwrap_or_else(|err| panic!("{fn_name} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "Global function `{fn_name}` must be defined after LoD load. \
             These cover the frame scripts, train-button enable wrapper, data-provider rebuild, \
             per-row initialization, auto-selection, selection state, and button click handlers. \
             ClassTrainerFrame_Show and ClassTrainerFrame_Hide are local bootstrap callbacks, not \
             addon globals. Got type={kind} for {fn_name}"
        );
    }
}
}

prefork_full_ui_case! {
fn explicit_load_creates_class_trainer_frame_global(env: &WowLuaEnv) {

    load_addon(&env.loader_env(), &trainer_toc())
        .expect("Blizzard_TrainerUI must load via Rust loader");

    let exists: bool = env
        .eval("return ClassTrainerFrame ~= nil")
        .expect("ClassTrainerFrame probe");
    assert!(
        exists,
        "ClassTrainerFrame must exist as a named global after LoD load \
         — declared at xml:110 as `<Frame name=\"ClassTrainerFrame\" \
         inherits=\"ButtonFrameTemplate\" toplevel=\"true\" \
         movable=\"true\" parent=\"UIParent\" enableMouse=\"true\" \
         hidden=\"true\">`. The frame is the lone non-virtual top-\
         level frame published by this addon — \
         ClassTrainerSkillButtonTemplate at xml:28 is virtual and \
         instantiated by the WowScrollBoxList view"
    );
}
}

prefork_full_ui_case! {
fn explicit_load_registers_ui_panel_windows_entry(env: &WowLuaEnv) {

    load_addon(&env.loader_env(), &trainer_toc())
        .expect("Blizzard_TrainerUI must load via Rust loader");

    let kind: String = env
        .eval("return type(UIPanelWindows['ClassTrainerFrame'])")
        .expect("UIPanelWindows entry probe");
    assert_eq!(
        kind, "table",
        "Blizzard_TrainerUI.lua:9 must register \
         `UIPanelWindows[\"ClassTrainerFrame\"]` with area=left, \
         pushable=0, allowOtherPanels=1 — UNLIKE many panels (e.g. \
         TorghastLevelPickerFrame) the entry is NOT pre-registered at \
         boot; it appears only after the LoD addon executes its body, \
         so any caller that wants to ShowUIPanel(ClassTrainerFrame) \
         must first call UIParentLoadAddOn(\"Blizzard_TrainerUI\")"
    );

    let area: String = env
        .eval("return UIPanelWindows['ClassTrainerFrame'].area")
        .expect("area field probe");
    assert_eq!(
        area, "left",
        "UIPanelWindows entry must have area=\"left\" so trainer dialogs \
         dock on the left edge of the screen alongside other primary \
         interaction windows (merchant, banker, mailbox)"
    );
}
}

#[test]
fn bootstrap_registers_trainer_interaction() {
    let raw = std::fs::read_to_string(
        trainer_toc()
            .parent()
            .expect("Trainer TOC has an addon directory")
            .join("Blizzard_TrainerUI_Bootstrap.lua"),
    )
    .expect("Trainer bootstrap reads utf-8");

    assert!(
        raw.contains("function ClassTrainerFrame_LoadUI()")
            && raw.contains("RegisterPlayerInteraction(Enum.PlayerInteractionType.Trainer")
            && raw.contains("frame = \"ClassTrainerFrame\"")
            && raw.contains("loadFunc = ClassTrainerFrame_LoadUI"),
        "Retail 12.1 TrainerUI bootstrap owns its lazy-load wrapper and PlayerInteraction registration"
    );
}

prefork_full_ui_case! {
fn class_trainer_frame_load_ui_published_at_boot(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(ClassTrainerFrame_LoadUI)")
        .expect("ClassTrainerFrame_LoadUI probe");
    assert_eq!(
        kind, "function",
        "ClassTrainerFrame_LoadUI must be defined at boot — \
         UIParent.lua:265-267 declares it BEFORE Blizzard_TrainerUI \
         loads, so PlayerInteractionFrameManager.lua:34 can capture \
         the reference via `loadFunc = ClassTrainerFrame_LoadUI` and \
         the trainer interaction can lazily LoadAddOn the panel on \
         first engagement (without this boot-time wrapper, the \
         loadFunc reference would be nil at the time the \
         playerInteractionToFrameInfo table is constructed)"
    );
}
}

prefork_full_ui_case! {
fn explicit_load_emits_no_addon_specific_errors(env: &WowLuaEnv) {

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_addon(&env.loader_env(), &trainer_toc())
        .expect("Blizzard_TrainerUI must load via Rust loader");

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let addon_specific: Vec<&String> = errors
        .iter()
        .filter(|e| e.contains("Blizzard_TrainerUI") || e.contains("ClassTrainer"))
        .collect();

    assert!(
        addon_specific.is_empty(),
        "Re-loading TrainerUI over a fully-loaded game env must emit \
         zero addon-specific errors — load creates ClassTrainerFrame, \
         registers the StaticPopupDialogs[\"CONFIRM_PROFESSION\"] entry, \
         and sets UIPanelWindows[\"ClassTrainerFrame\"]; no event \
         handlers fire until the player actually engages a trainer. \
         Found {}: {:#?}",
        addon_specific.len(),
        addon_specific
    );
}
}
