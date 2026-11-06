use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::paths::default_blizzard_ui_addons_path;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::settle_headless_startup;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> std::path::PathBuf {
    default_blizzard_ui_addons_path().expect("Blizzard UI cache should be synced")
}

fn timerunning_dir() -> std::path::PathBuf {
    blizzard_ui_dir().join("Blizzard_TimerunningCharacterCreate")
}

fn timerunning_toc() -> std::path::PathBuf {
    timerunning_dir().join("Blizzard_TimerunningCharacterCreate.toc")
}

const ALL_FOUR_SCREENS: &[ScreenKind] = &[
    ScreenKind::Game,
    ScreenKind::Login,
    ScreenKind::CharacterSelect,
    ScreenKind::CharacterCreate,
];

const REQUIRED_DEPS: &[&str] = &[
    "Blizzard_CharacterCreate",
    "Blizzard_GlueXML",
    "Blizzard_TimerunningUtil",
];

const PUBLISHED_MIXINS: &[(&str, usize)] = &[
    ("TimerunningCreateCharacterButtonGlowMixin", 3),
    ("TimerunningFirstTimeDialogMixin", 7),
    ("TimerunningChoiceDialogMixin", 2),
    ("TimerunningChoicePopupMixin", 3),
    ("TimerunningEventBannerMixin", 7),
    ("TimerunningConversionButtonMixin", 6),
];

const NAMED_TOPLEVEL_FRAMES: &[&str] = &[
    "TimerunningCreateCharacterButtonGlow",
    "TimerunningFirstTimeDialog",
    "TimerunningChoicePopup",
    "TimerunningEventBanner",
];

const STATIC_POPUPS: &[&str] = &["TIMERUNNING_CHOICE_WARNING", "CONVERT_TIMERUNNER_EARLY"];

fn load_character_select_screen() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::CharacterSelect);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    wow_ui_sim::xml::register_intrinsic_templates();

    let addons =
        discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::CharacterSelect);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    settle_headless_startup(&env);
    env
}

// Literal cached inputs absent from Wrath manifest1-382/cache; behavior cases stay enabled.
retail_ptr_forever_toc_fixture! {
#[test]
fn find_toc_file_resolves_bare_toc() {
    let resolved =
        find_toc_file(&timerunning_dir()).expect("TimerunningCharacterCreate TOC resolves");
    assert_eq!(
        resolved,
        timerunning_toc(),
        "Bare TOC — no flavor suffix; glue-only LoD addon resolved \
         via the bare-TOC path in find_toc_file at \
         src/loader/mod.rs:65-95"
    );
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn toc_is_load_on_demand_with_three_required_deps() {
    let toc = TocFile::from_file(&timerunning_toc()).expect("TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "`## LoadOnDemand: 1` — only loads when \
         CharacterSelect_UpdateTimerunning calls \
         C_AddOns.LoadAddOn upon detecting an active timerunning \
         season"
    );
    assert_eq!(
        toc.dependencies(),
        REQUIRED_DEPS
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
        "`## RequiredDep: Blizzard_CharacterCreate, Blizzard_GlueXML, \
         Blizzard_TimerunningUtil` — toc.rs:209-217 reads RequiredDep \
         (singular) when Dependencies absent. 3 hard deps: \
         CharacterCreate (provides CharacterSelectBlockingFrameTemplate \
         + CharacterCreateFrame state probed by UpdateState), \
         GlueXML (GlueParent / GlueTooltip / glue-screen substrate), \
         TimerunningUtil (AddLargeIcon + GetTimerunningChoiceDesc + \
         GetTimerunningBannerHeaderText + IsTimerunningEnabled \
         helpers). Got: {:?}",
        toc.dependencies()
    );
    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.saved_variables_per_character().is_empty());
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(!toc.is_game_type_restricted());
    assert!(toc.default_enabled());
}
}

post_wrath_ui_fixture! {
#[test]
fn allow_load_glue_surfaces_on_three_glue_screens_only() {
    let toc = TocFile::from_file(&timerunning_toc()).expect("TOC parses");

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            toc.allows_screen(screen),
            "`## AllowLoad: Glue` (case-insensitive at toc.rs:309) → \
             screen.is_glue() must permit {screen:?} — timerunning \
             popup needs to surface during glue flows where players \
             actually create characters"
        );
    }
    assert!(
        !toc.allows_screen(ScreenKind::Game),
        "Game screen must be excluded — the addon attaches frames to \
         CharacterSelectUI / CharSelectCreateCharacterButton / \
         GlueParent which do not exist in-world"
    );
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn toc_raw_bytes_pin_five_metadata_directives() {
    let raw = std::fs::read_to_string(timerunning_toc()).expect("TOC reads utf-8");

    let expected_directives = [
        "## Title: Blizzard Timerunning Character Create",
        "## Author: Blizzard Entertainment",
        "## AllowLoad: Glue",
        "## LoadOnDemand: 1",
        "## RequiredDep: Blizzard_CharacterCreate, Blizzard_GlueXML, Blizzard_TimerunningUtil",
        "Blizzard_TimerunningCharacterCreate.lua",
        "Blizzard_TimerunningCharacterCreate.xml",
        "Localization.lua",
    ];

    for directive in expected_directives {
        assert!(
            raw.contains(directive),
            "Raw TOC must pin `{directive}` — 5 metadata directives + \
             3 body files (lua, xml, Localization) all addon-root, \
             no flavor subdir"
        );
    }

    assert!(!raw.contains("## Dependencies"));
    assert!(!raw.contains("## OptionalDep"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## AllowLoadGameType"));
    assert!(!raw.contains("## UseSecureEnvironment"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("## DefaultState"));
}
}

post_wrath_ui_fixture! {
#[test]
fn body_resolves_three_entries_at_addon_root() {
    let toc = TocFile::from_file(&timerunning_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let expected = vec![
        "Blizzard_TimerunningCharacterCreate.lua".to_string(),
        "Blizzard_TimerunningCharacterCreate.xml".to_string(),
        "Localization.lua".to_string(),
    ];

    assert_eq!(
        body, expected,
        "Body must resolve to 3 entries at addon root (no Mainline/ \
         subdir) in declared order: lua → xml → Localization. Got: \
         {body:?}"
    );
}
}

#[test]
fn absent_from_every_screen_eager_discovery() {
    for screen in ALL_FOUR_SCREENS {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), *screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_TimerunningCharacterCreate");
        assert!(
            !found,
            "Blizzard_TimerunningCharacterCreate must be absent from \
             {screen:?} eager discovery — `## LoadOnDemand: 1` \
             excludes LoD addons from the eager sweep"
        );
    }
}

#[test]
fn no_mainline_addon_declares_timerunning_as_dependency() {
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
        if toc
            .dependencies()
            .iter()
            .any(|d| d == "Blizzard_TimerunningCharacterCreate")
            || toc
                .optional_deps()
                .iter()
                .any(|d| d == "Blizzard_TimerunningCharacterCreate")
        {
            let name = addon_dir.file_name().unwrap().to_string_lossy().to_string();
            declarers.push(name);
        }
    }

    assert!(
        declarers.is_empty(),
        "No Blizzard addon may declare \
         Blizzard_TimerunningCharacterCreate as a Dependency or \
         OptionalDep — strictly LoD, triggered ONLY by \
         CharacterSelect_UpdateTimerunning + \
         CharacterSelect_ShowTimerunningChoiceWhenActive in \
         Blizzard_GlueXML/Mainline/CharacterSelect.lua via runtime \
         C_AddOns.LoadAddOn calls. Found declarers: {declarers:?}"
    );
}

post_wrath_ui_fixture! {
#[test]
fn explicit_load_publishes_six_mixins_with_expected_method_counts() {
    let env = load_character_select_screen();

    load_addon(&env.loader_env(), &timerunning_toc())
        .expect("Blizzard_TimerunningCharacterCreate must load via Rust loader");

    for (mixin, expected_methods) in PUBLISHED_MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} probe failed: {err}"));
        assert_eq!(kind, "table", "{mixin} must be a table after LoD load");

        let count_probe = format!(
            "local n = 0 for k, v in pairs({mixin}) do if type(v) == 'function' then n = n + 1 end end return n"
        );
        let actual: i64 = env
            .eval(&count_probe)
            .unwrap_or_else(|err| panic!("{mixin} method count probe failed: {err}"));
        assert_eq!(
            actual, *expected_methods as i64,
            "{mixin} must publish exactly {expected_methods} methods. \
             Counts cover: GlowMixin (OnLoad/OnSizeChanged/UpdateHeight \
             — 3), FirstTimeDialogMixin (OnLoad/OnShow/OnEvent/\
             UpdateState/ShowFromClick/Dismiss/OnEscapePressed — 7), \
             ChoiceDialogMixin (OnLoad/OnShow — 2), ChoicePopupMixin \
             (OnLoad/OnShow/OnEvent — 3), EventBannerMixin (OnLoad/\
             OnEvent/UpdateShown/UpdateTimeLeft/OnEnter/OnLeave/OnClick \
             — 7), ConversionButtonMixin (OnClick/OnMouseDown/OnMouseUp/\
             OnEnter/OnLeave/UpdateTextureStates — 6). Got {actual}"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn explicit_load_creates_four_named_toplevel_frames() {
    let env = load_character_select_screen();

    load_addon(&env.loader_env(), &timerunning_toc())
        .expect("Blizzard_TimerunningCharacterCreate must load via Rust loader");

    for frame_name in NAMED_TOPLEVEL_FRAMES {
        let exists: bool = env
            .eval(&format!("return _G['{frame_name}'] ~= nil"))
            .unwrap_or_else(|err| panic!("{frame_name} probe failed: {err}"));
        assert!(
            exists,
            "{frame_name} must exist as a named global after LoD load \
             — TimerunningCreateCharacterButtonGlow (parent=\
             CharSelectCreateCharacterButton, the rotating-glow halo \
             overlay), TimerunningFirstTimeDialog (the season-intro \
             splash dialog), TimerunningChoicePopup (the \
             standard-vs-timerunner choice splitter containing 2 \
             child dialogs), TimerunningEventBanner (parent=\
             GlueParent, the \"active event\" banner with countdown)"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn explicit_load_registers_two_static_popups() {
    let env = load_character_select_screen();

    load_addon(&env.loader_env(), &timerunning_toc())
        .expect("Blizzard_TimerunningCharacterCreate must load via Rust loader");

    for popup in STATIC_POPUPS {
        let kind: String = env
            .eval(&format!("return type(StaticPopupDialogs['{popup}'])"))
            .unwrap_or_else(|err| panic!("StaticPopupDialogs.{popup} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "StaticPopupDialogs.{popup} must be a registered popup \
             table — TIMERUNNING_CHOICE_WARNING (cover=true confirm \
             dialog before timerunner creation, OnAccept calls \
             CharacterSelectUtil.CreateNewCharacter with the active \
             timerunning seasonID); CONVERT_TIMERUNNER_EARLY \
             (fullscreen=1 acceptDelay=5 dialog asking the player to \
             confirm permanent timerunner→standard conversion via \
             TryConvertTimerunningCharacterToStandard)"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn explicit_load_has_no_addon_specific_lua_errors() {
    let env = load_character_select_screen();

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_addon(&env.loader_env(), &timerunning_toc())
        .expect("Blizzard_TimerunningCharacterCreate must load via Rust loader");

    let errors = env.state().borrow().lua_errors.clone();
    assert!(
        errors.is_empty(),
        "Timerunning legacy globals should let the addon load without Lua errors. Found: {errors:#?}"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn three_required_dep_directories_exist_on_disk() {
    for dep in REQUIRED_DEPS {
        let dep_dir = blizzard_ui_dir().join(dep);
        assert!(
            dep_dir.is_dir(),
            "RequiredDep `{dep}` directory must exist on disk — without \
             it the dependency-resolution path can't find a TOC and \
             load_addon would fail with a missing-dep error"
        );
        let toc = find_toc_file(&dep_dir);
        assert!(
            toc.is_some(),
            "RequiredDep `{dep}` must have a discoverable TOC via \
             find_toc_file (mainline-flavor or bare). Got: {toc:?}"
        );
    }
}
}

// The Mists/Era/Anniversary caches ship 218 addons each and Wrath ships 23;
// none contains Blizzard_TimerunningCharacterCreate, so the pinned TOC facts above describe no file the
// classic loaders can read.
classic_cache_absent_toc_fixture! {
#[test]
fn blizzard_timerunningcharactercreate_toc_is_absent_from_classic_caches() {
    let dir = timerunning_dir();
    assert!(
        !dir.is_dir(),
        "classic profile caches must not contain Blizzard_TimerunningCharacterCreate"
    );
    assert!(wow_ui_sim::loader::find_toc_file(&dir).is_none());
}
}
