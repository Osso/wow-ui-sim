use std::path::PathBuf;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn torghast_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_TorghastLevelPicker")
}

fn torghast_toc() -> PathBuf {
    torghast_dir().join("Blizzard_TorghastLevelPicker.toc")
}

const ALL_FOUR_SCREENS: &[ScreenKind] = &[
    ScreenKind::Game,
    ScreenKind::Login,
    ScreenKind::CharacterSelect,
    ScreenKind::CharacterCreate,
];

const PUBLISHED_MIXINS: &[(&str, usize)] = &[
    ("TorghastLevelPickerFrameMixin", 17),
    ("TorghastLevelPickerOptionButtonMixin", 10),
    ("TorghastPagingContainerMixin", 6),
    ("TorghastLevelPickerRewardCircleMixin", 7),
    ("TorghastLevelPickerOpenPortalButtonMixin", 3),
];

const GOSSIP_TEXTURE_KIT_HANDLERS: &[&str] = &[
    "skoldushall",
    "mortregar",
    "coldheartinterstitia",
    "fracturechambers",
    "soulforges",
    "theupperreaches",
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
retail_ptr_forever_toc_fixture! {
#[test]
fn find_toc_file_resolves_bare_toc() {
    let resolved = find_toc_file(&torghast_dir()).expect("TorghastLevelPicker TOC resolves");
    assert_eq!(
        resolved,
        torghast_toc(),
        "Bare TOC — no flavor suffix; Shadowlands-era LoD addon \
         resolved via the bare-TOC path in find_toc_file at \
         src/loader/mod.rs:65-95"
    );
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn toc_is_load_on_demand_with_single_blizzard_colors_dependency() {
    let toc = TocFile::from_file(&torghast_toc()).expect("TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "`## LoadOnDemand: 1` — only loads when CustomGossipFrameBase \
         dispatches a Torghast textureKit (skoldushall/mortregar/etc.) \
         via HandleTorghastLevelPickerGossipShow which calls \
         C_AddOns.LoadAddOn(\"Blizzard_TorghastLevelPicker\") at \
         CustomGossipFrameBase.lua:15-18"
    );
    assert_eq!(
        toc.dependencies(),
        vec!["Blizzard_Colors".to_string()],
        "`## Dependencies: Blizzard_Colors` — single hard dep on the \
         Blizzard_Colors palette (TorghastLevelPicker uses NORMAL_FONT_COLOR \
         / GREEN_FONT_COLOR / DISABLED_FONT_COLOR for level-button \
         label coloring). Got: {:?}",
        toc.dependencies()
    );
    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.saved_variables_per_character().is_empty());
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(
        !toc.is_game_type_restricted(),
        "AllowLoadGameType absent → not restricted (false)"
    );
    assert!(toc.default_enabled());
}
}

post_wrath_ui_fixture! {
#[test]
fn allow_load_absent_defaults_to_game_only_screen() {
    let toc = TocFile::from_file(&torghast_toc()).expect("TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "AllowLoad absent → toc.rs:305-313 None branch defaults to \
         Game-only — Torghast level picker only opens via in-world \
         gossip with NPCs at the 6 Torghast wing entrances"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Glue screen {screen:?} must be excluded — gossip-driven \
             in-world UI does not exist on glue"
        );
    }
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn toc_raw_bytes_pin_five_metadata_directives() {
    let raw = std::fs::read_to_string(torghast_toc()).expect("TOC reads utf-8");

    let expected_directives = [
        "## Title: Blizzard_TorghastLevelPicker",
        "## Author: Blizzard Entertainment",
        "## Version: 1.0",
        "## LoadOnDemand: 1",
        "## Dependencies: Blizzard_Colors",
        "Blizzard_TorghastLevelPicker.xml",
    ];

    for directive in expected_directives {
        assert!(
            raw.contains(directive),
            "Raw TOC must pin `{directive}` — 5 metadata directives + \
             1 body file (XML only — the .lua is loaded via \
             `<Script file=\"Blizzard_TorghastLevelPicker.lua\"/>` \
             at xml line 3, not directly listed in the TOC body)"
        );
    }

    assert!(!raw.contains("## DefaultState"));
    assert!(!raw.contains("## RequiredDep"));
    assert!(!raw.contains("## OptionalDep"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## AllowLoad"));
    assert!(!raw.contains("## AllowLoadGameType"));
    assert!(!raw.contains("## UseSecureEnvironment"));
    assert!(!raw.contains("## LoadFirst"));
}
}

post_wrath_ui_fixture! {
#[test]
fn body_resolves_to_single_xml_file_only() {
    let toc = TocFile::from_file(&torghast_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    assert_eq!(
        body,
        vec!["Blizzard_TorghastLevelPicker.xml".to_string()],
        "Body must be exactly 1 entry — only the XML file is listed; \
         the .lua is pulled in transitively by the XML's `<Script \
         file=...>` directive at xml line 3 (an alternative body \
         shape compared to most addons that list both files \
         explicitly). Got: {body:?}"
    );
}
}

#[test]
fn absent_from_every_screen_eager_discovery() {
    for screen in ALL_FOUR_SCREENS {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), *screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_TorghastLevelPicker");
        assert!(
            !found,
            "Blizzard_TorghastLevelPicker must be absent from \
             {screen:?} eager discovery — `## LoadOnDemand: 1` \
             excludes LoD addons from the eager sweep"
        );
    }
}

#[test]
fn no_addon_declares_torghast_level_picker_as_dependency() {
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
        let declared = toc
            .dependencies()
            .iter()
            .any(|d| d == "Blizzard_TorghastLevelPicker")
            || toc
                .optional_deps()
                .iter()
                .any(|d| d == "Blizzard_TorghastLevelPicker");
        if declared {
            let name = addon_dir.file_name().unwrap().to_string_lossy().to_string();
            declarers.push(name);
        }
    }

    assert!(
        declarers.is_empty(),
        "No Blizzard addon may declare Blizzard_TorghastLevelPicker \
         as a hard or optional dep — strictly LoD, triggered ONLY by \
         CustomGossipFrameBase.lua handlers when a Torghast NPC \
         gossip textureKit is detected. Found declarers: {declarers:?}"
    );
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_colors_dep_directory_exists_on_disk() {
    let colors_dir = blizzard_ui_dir().join("Blizzard_Colors");
    assert!(
        colors_dir.is_dir(),
        "Hard-dep directory `Blizzard_Colors` must exist on disk — \
         without it the dependency-resolution path can't find a TOC \
         and load_addon would fail at TorghastLevelPicker load time"
    );
    let toc = find_toc_file(&colors_dir);
    assert!(
        toc.is_some(),
        "Blizzard_Colors must have a discoverable TOC \
         (mainline-flavor or bare). Got: {toc:?}"
    );
}
}

prefork_full_ui_case! {
fn explicit_load_publishes_five_mixins_with_expected_method_counts(env: &WowLuaEnv) {

    load_addon(&env.loader_env(), &torghast_toc())
        .expect("Blizzard_TorghastLevelPicker must load via Rust loader");

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
            "{mixin} must publish {expected_methods} methods. \
             Counts cover: FrameMixin (OnLoad/OnEvent/OnShow/\
             CancelEffects/UpdatePortalButtonState/SetupOptions/\
             TryShow/OnHide/SetupGrid/SetupLevelButtons/SetStartingPage/\
             GetCurrentPage/ClearLevelSelection/SelectLevel/SetupBackground/\
             SetupDescription/ScrollAndSelectHighestAvailableLayer = 17), \
             OptionButtonMixin (SetDifficultyTexture/Setup/\
             ShouldOptionBeEnabled/SetState/UpdateSelectionState/\
             ClearSelection/OnClick/RefreshTooltip/OnEnter/OnLeave = 10), \
             PagingContainerMixin (Init/Setup/SetupPageNumberString/\
             SetupPagingButtonStates/PagePrevious/PageNext = 6), \
             RewardCircleMixin (SetSortedRewards/AddCurrencyToTooltip/\
             SetRewardIcon/Init/OnEnter/RefreshTooltip/OnLeave = 7), \
             OpenPortalButtonMixin (OnEnter/OnLeave/OnClick = 3). Got {actual}"
        );
    }
}
}

prefork_full_ui_case! {
fn explicit_load_creates_torghast_level_picker_frame(env: &WowLuaEnv) {

    load_addon(&env.loader_env(), &torghast_toc())
        .expect("Blizzard_TorghastLevelPicker must load via Rust loader");

    let exists: bool = env
        .eval("return TorghastLevelPickerFrame ~= nil")
        .expect("TorghastLevelPickerFrame probe");
    assert!(
        exists,
        "TorghastLevelPickerFrame must exist as a named global after \
         LoD load — declared at xml:187 as `<Frame \
         name=\"TorghastLevelPickerFrame\" \
         inherits=\"CustomGossipFrameBaseGridTemplate\" \
         mixin=\"TorghastLevelPickerFrameMixin\" hidden=\"true\">`. \
         The frame is the lone non-virtual top-level frame published \
         by this addon — everything else (TorghastPagingContainerTemplate \
         etc.) is a virtual template"
    );
}
}

prefork_full_ui_case! {
fn ui_panel_windows_entry_registered_at_boot(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(UIPanelWindows['TorghastLevelPickerFrame'])")
        .expect("UIPanelWindows entry probe");
    assert_eq!(
        kind, "table",
        "Blizzard_UIParentPanelManager/Mainline/UIPanelWindows.lua:63 \
         registers `UIPanelWindows[\"TorghastLevelPickerFrame\"]` at \
         boot with area=center, pushable=0, xoffset=-16, yoffset=12, \
         whileDead=0, allowOtherPanels=1 — the panel-manager entry \
         exists BEFORE Blizzard_TorghastLevelPicker itself loads, \
         so ShowUIPanel/HideUIPanel can immediately route the frame \
         once the LoD load materializes it"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn six_torghast_gossip_texture_kits_route_to_load_addon() {
    let raw = std::fs::read_to_string(
        blizzard_ui_dir().join("Blizzard_UIPanels_Game/Mainline/CustomGossipFrameBase.lua"),
    )
    .expect("CustomGossipFrameBase.lua reads utf-8");

    for kit in GOSSIP_TEXTURE_KIT_HANDLERS {
        let needle = format!("RegisterHandler(\"{kit}\", HandleTorghastLevelPickerGossipShow)");
        assert!(
            raw.contains(&needle),
            "CustomGossipFrameBase.lua must register the `{kit}` \
             gossip textureKit handler — the 6 Torghast wings each \
             have their own NPC textureKit (Skoldus Hall, Mort'regar, \
             Coldheart Interstitia, Fracture Chambers, Soulforges, \
             The Upper Reaches), and all 6 dispatch to \
             HandleTorghastLevelPickerGossipShow which calls \
             C_AddOns.LoadAddOn at line 16 then TryShow at line 17"
        );
    }
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

    load_addon(&env.loader_env(), &torghast_toc())
        .expect("Blizzard_TorghastLevelPicker must load via Rust loader");

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let addon_specific: Vec<&String> = errors
        .iter()
        .filter(|e| e.contains("Blizzard_TorghastLevelPicker") || e.contains("TorghastLevelPicker"))
        .collect();

    assert!(
        addon_specific.is_empty(),
        "Re-loading TorghastLevelPicker over a fully-loaded game env \
         must emit zero addon-specific errors — load creates frames + \
         registers mixins, no event handlers fire until TryShow is \
         called from gossip. Found {}: {:#?}",
        addon_specific.len(),
        addon_specific
    );
}
}

// The Mists/Era/Anniversary caches ship 218 addons each and Wrath ships 23;
// none contains Blizzard_TorghastLevelPicker, so the pinned TOC facts above describe no file the
// classic loaders can read.
classic_cache_absent_toc_fixture! {
#[test]
fn blizzard_torghastlevelpicker_toc_is_absent_from_classic_caches() {
    let dir = torghast_dir();
    assert!(
        !dir.is_dir(),
        "classic profile caches must not contain Blizzard_TorghastLevelPicker"
    );
    assert!(wow_ui_sim::loader::find_toc_file(&dir).is_none());
}
}
