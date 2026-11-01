use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::paths::default_blizzard_ui_addons_path;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> std::path::PathBuf {
    default_blizzard_ui_addons_path().expect("Blizzard UI cache should be synced")
}

fn timerunning_util_dir() -> std::path::PathBuf {
    blizzard_ui_dir().join("Blizzard_TimerunningUtil")
}

fn timerunning_util_toc() -> std::path::PathBuf {
    timerunning_util_dir().join("Blizzard_TimerunningUtil.toc")
}

const ALL_FOUR_SCREENS: &[ScreenKind] = &[
    ScreenKind::Game,
    ScreenKind::Login,
    ScreenKind::CharacterSelect,
    ScreenKind::CharacterCreate,
];

const PUBLISHED_METHODS: &[&str] = &[
    "AddTinyIcon",
    "AddSmallIcon",
    "AddLargeIcon",
    "TimerunningEnabledForPlayer",
    "GetActiveTimerunningSeasonID",
    "GetTimerunningExpansion",
    "GetTimerunningChoiceDesc",
    "GetTimerunningBannerHeaderText",
];

const MAINLINE_DEPENDENTS: &[&str] = &[
    "Blizzard_ChatFrameBase",
    "Blizzard_Communities",
    "Blizzard_FriendsFrame",
    "Blizzard_GlueXML",
    "Blizzard_TimerunningCharacterCreate",
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

// Literal cached inputs absent from Wrath manifest1-382/cache; behavior cases stay enabled.
post_wrath_ui_fixture! {
#[test]
fn find_toc_file_resolves_bare_toc() {
    let resolved = find_toc_file(&timerunning_util_dir()).expect("TimerunningUtil TOC resolves");
    assert_eq!(
        resolved,
        timerunning_util_toc(),
        "Bare TOC — no flavor suffix; foundational helper-library \
         addon ships universally and is consumed by both glue and game"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_is_eager_with_single_shared_xml_dependency() {
    let toc = TocFile::from_file(&timerunning_util_toc()).expect("TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "No `## LoadOnDemand` — TimerunningUtil publishes the global \
         TimerunningUtil helper table consumed eagerly by ChatFrameBase \
         (timerunner-icon chat decoration), Communities, FriendsFrame, \
         GlueXML, and TimerunningCharacterCreate, so it must load \
         before any consumer's Lua executes"
    );
    assert_eq!(
        toc.dependencies(),
        vec!["Blizzard_SharedXML".to_string()],
        "`## Dependencies: Blizzard_SharedXML` — single hard dep on \
         SharedXML for CreateAtlasMarkup helper. Got: {:?}",
        toc.dependencies()
    );
    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.saved_variables_per_character().is_empty());
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(
        !toc.is_game_type_restricted(),
        "`## AllowLoadGameType: mainline` — toc.rs:294-302 permits \
         `mainline`/`standard` tokens so is_game_type_restricted=false"
    );
    assert!(toc.default_enabled());
}
}

post_wrath_ui_fixture! {
#[test]
fn allow_load_both_surfaces_on_all_four_screens() {
    let toc = TocFile::from_file(&timerunning_util_toc()).expect("TOC parses");

    for screen in ALL_FOUR_SCREENS {
        assert!(
            toc.allows_screen(*screen),
            "`## AllowLoad: both` (case-insensitive at toc.rs:307) \
             must permit screen {screen:?} — TimerunningUtil is needed \
             on glue (CharacterSelect timerunning banner + choice popup) \
             AND in Game (chat-name-prefix timerunner icon decoration)"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_raw_bytes_pin_six_metadata_directives() {
    let raw = std::fs::read_to_string(timerunning_util_toc()).expect("TOC reads utf-8");

    let expected_directives = [
        "## Title: Blizzard Timerunning Util",
        "## Author: Blizzard Entertainment",
        "## DefaultState: enabled",
        "## AllowLoad: both",
        "## AllowLoadGameType: mainline",
        "## Dependencies: Blizzard_SharedXML",
        "Blizzard_TimerunningUtil.lua",
    ];

    for directive in expected_directives {
        assert!(
            raw.contains(directive),
            "Raw TOC must pin `{directive}` — 6 metadata directives + \
             1 body file (lua only, no XML)"
        );
    }

    assert!(!raw.contains("## LoadOnDemand"));
    assert!(!raw.contains("## RequiredDep"));
    assert!(!raw.contains("## OptionalDep"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## UseSecureEnvironment"));
    assert!(!raw.contains("## LoadFirst"));
}
}

post_wrath_ui_fixture! {
#[test]
fn body_resolves_to_a_single_lua_file() {
    let toc = TocFile::from_file(&timerunning_util_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    assert_eq!(
        body,
        vec!["Blizzard_TimerunningUtil.lua".to_string()],
        "Body must be exactly 1 entry — TimerunningUtil is a Lua-only \
         helper library, no XML templates. Got: {body:?}"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn present_in_every_screen_eager_discovery() {
    for screen in ALL_FOUR_SCREENS {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), *screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_TimerunningUtil");
        assert!(
            found,
            "Blizzard_TimerunningUtil must appear in {screen:?} eager \
             discovery — AllowLoad: both + no LoadOnDemand + \
             AllowLoadGameType=mainline allowed"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn five_mainline_addons_declare_timerunning_util_as_dependency() {
    let mut declarers: Vec<String> = Vec::new();

    for entry in std::fs::read_dir(blizzard_ui_dir())
        .expect("BlizzardUI dir reads")
        .flatten()
    {
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
        let declared_hard = toc
            .dependencies()
            .iter()
            .any(|d| d == "Blizzard_TimerunningUtil");
        if declared_hard {
            let name = addon_dir.file_name().unwrap().to_string_lossy().to_string();
            declarers.push(name);
        }
    }

    declarers.sort();
    let mut expected: Vec<String> = MAINLINE_DEPENDENTS.iter().map(|s| s.to_string()).collect();
    expected.sort();

    assert_eq!(
        declarers, expected,
        "Exactly 5 Blizzard addons must declare \
         Blizzard_TimerunningUtil as a HARD dep — ChatFrameBase \
         (Dependencies, for timerunner chat-prefix icon), Communities \
         (RequiredDep singular, for member-list timerunner badge), \
         FriendsFrame (Dependencies, for friends-list timerunner \
         badge), GlueXML (Dependencies, for character-select \
         timerunning banner), TimerunningCharacterCreate (RequiredDep, \
         calls TimerunningUtil.AddLargeIcon + GetTimerunningChoiceDesc \
         + GetTimerunningBannerHeaderText). Got: {declarers:?}"
    );
}
}

prefork_full_ui_case! {
fn timerunning_util_table_publishes_with_eight_methods(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(TimerunningUtil)")
        .expect("TimerunningUtil probe");
    assert_eq!(
        kind, "table",
        "TimerunningUtil must be a table — Blizzard_TimerunningUtil.lua \
         line 1 publishes the global helper table consumed across glue+game"
    );

    let count_probe = "local n = 0 for _, v in pairs(TimerunningUtil) do if type(v) == 'function' then n = n + 1 end end return n";
    let count: i64 = env.eval(count_probe).expect("method count probe");
    assert_eq!(
        count, 8,
        "TimerunningUtil must publish exactly 8 methods. Methods: \
         AddTinyIcon (atlas-markup wrap, 9x12), AddSmallIcon \
         (atlas-markup wrap, 12x12), AddLargeIcon (atlas-markup + \
         space + text via format), TimerunningEnabledForPlayer (wraps \
         PlayerIsTimerunning), GetActiveTimerunningSeasonID \
         (dispatches between glue + game season-id sources via \
         C_Glue.IsOnGlueScreen), GetTimerunningExpansion \
         (LE_EXPANSION_* lookup from current config), \
         GetTimerunningChoiceDesc (locale string from current config), \
         GetTimerunningBannerHeaderText (locale string from current \
         config). Got: {count}"
    );

    for method in PUBLISHED_METHODS {
        let kind: String = env
            .eval(&format!("return type(TimerunningUtil.{method})"))
            .unwrap_or_else(|err| panic!("TimerunningUtil.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "TimerunningUtil.{method} must be a function"
        );
    }
}
}

prefork_full_ui_case! {
fn timerunning_constants_resolve_to_three_season_ids(env: &WowLuaEnv) {

    let none_id: i64 = env
        .eval("return Constants.TimerunningConsts.TIMERUNNING_SEASON_NONE")
        .expect("SEASON_NONE probe");
    let pandaria_id: i64 = env
        .eval("return Constants.TimerunningConsts.TIMERUNNING_SEASON_PANDARIA")
        .expect("SEASON_PANDARIA probe");
    let legion_id: i64 = env
        .eval("return Constants.TimerunningConsts.TIMERUNNING_SEASON_LEGION")
        .expect("SEASON_LEGION probe");

    assert_eq!(
        none_id, 0,
        "TIMERUNNING_SEASON_NONE = 0 — sentinel for off-season; \
         GetCurrTimerunningSeasonConfig falls back to this when \
         GetActiveTimerunningSeasonID returns nil"
    );
    assert_eq!(
        pandaria_id, 1,
        "TIMERUNNING_SEASON_PANDARIA = 1 — Mists of Pandaria \
         timerunner season"
    );
    assert_eq!(
        legion_id, 2,
        "TIMERUNNING_SEASON_LEGION = 2 — Legion timerunner season"
    );
}
}

prefork_full_ui_case! {
fn icon_helpers_emit_atlas_markup_wrapping_text(env: &WowLuaEnv) {

    let tiny: String = env
        .eval("return TimerunningUtil.AddTinyIcon('Hello')")
        .expect("AddTinyIcon probe");
    assert!(
        tiny.contains("timerunning-glues-icon-small") && tiny.contains("Hello"),
        "AddTinyIcon must concat CreateAtlasMarkup('timerunning-glues-icon-small', 9, 12) \
         + text — the small-variant icon used in compact contexts. Got: {tiny:?}"
    );

    let small: String = env
        .eval("return TimerunningUtil.AddSmallIcon('Hello')")
        .expect("AddSmallIcon probe");
    assert!(
        small.contains("timerunning-glues-icon") && small.contains("Hello"),
        "AddSmallIcon must concat CreateAtlasMarkup('timerunning-glues-icon', 12, 12) \
         + text — used inline by chat decoration. Got: {small:?}"
    );

    let large: String = env
        .eval("return TimerunningUtil.AddLargeIcon('Hello')")
        .expect("AddLargeIcon probe");
    assert!(
        large.contains("timerunning-glues-icon") && large.contains("Hello"),
        "AddLargeIcon must format `%s %s` with atlas-markup + space + \
         text — extra-space variant used in dialog headers. Got: {large:?}"
    );
}
}

prefork_full_ui_case! {
fn season_lookup_returns_classic_expansion_when_no_active_season(env: &WowLuaEnv) {

    env.state().borrow_mut().active_timerunning_season_id = 0;
    let probe = "return TimerunningUtil.GetTimerunningExpansion(), \
        TimerunningUtil.GetTimerunningChoiceDesc(), TimerunningUtil.GetTimerunningBannerHeaderText()";

    let (expansion, desc, header): (i64, String, String) = env.eval(probe).expect("season probe");

    assert_eq!(
        expansion, 0,
        "When no active season, GetTimerunningExpansion must fall back \
         to LE_EXPANSION_CLASSIC (=0, the vanilla expansion-id) — \
         TIMERUNNING_SEASON_NONE config pins expansion = \
         LE_EXPANSION_CLASSIC at lua line 29"
    );
    assert_eq!(
        desc, "",
        "When no active season, GetTimerunningChoiceDesc must return \
         empty string from the SEASON_NONE config — gluesTimerunningChoiceDesc = \"\""
    );
    assert_eq!(
        header, "",
        "When no active season, GetTimerunningBannerHeaderText must \
         return empty string from the SEASON_NONE config — \
         gluesTimerunningBannerHeaderText = \"\""
    );

    env.state().borrow_mut().active_timerunning_season_id = 1;
    let expansion: i64 = env
        .eval("return TimerunningUtil.GetTimerunningExpansion()")
        .expect("active season probe");
    assert_eq!(expansion, 4, "Pandaria season selects the Mists expansion");
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

    load_addon(&env.loader_env(), &timerunning_util_toc())
        .expect("Blizzard_TimerunningUtil must reload via Rust loader");

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let addon_specific: Vec<&String> = errors
        .iter()
        .filter(|e| e.contains("Blizzard_TimerunningUtil") || e.contains("TimerunningUtil"))
        .collect();

    assert!(
        addon_specific.is_empty(),
        "Re-loading TimerunningUtil over a fully-loaded game env must \
         emit zero addon-specific errors — pure Lua module with no \
         frames and no event handlers, just an 8-method table + \
         3-entry season-config table. Found {}: {:#?}",
        addon_specific.len(),
        addon_specific
    );
}
}
