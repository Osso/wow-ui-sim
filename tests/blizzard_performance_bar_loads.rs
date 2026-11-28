#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::loader::{find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn performance_bar_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_PerformanceBar")
}

fn performance_bar_toc() -> PathBuf {
    performance_bar_dir().join("Blizzard_PerformanceBar.toc")
}

const PERFORMANCE_BAR_MAINLINE_FILES: &[&str] = &["PerformanceBar.lua"];

const PUBLIC_GLOBAL_FUNCTIONS: &[&str] = &[
    "MainMenu_GetMovieDownloadProgress",
    "MainMenuBarPerformanceBarFrame_UseDetailedTooltip",
    "MainMenuBarPerformanceBarFrame_OnEnter",
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
fn blizzard_performance_bar_find_toc_resolves_bare_variant() {
    let resolved =
        find_toc_file(&performance_bar_dir()).expect("Blizzard_PerformanceBar TOC resolves");
    assert_eq!(
        resolved,
        performance_bar_toc(),
        "Blizzard_PerformanceBar ships exactly one bare TOC — no `_Mainline.toc` variant. \
         The performance bar is the latency/FPS/memory tooltip-target on the main menu bar; \
         the Classic flavor differences are handled in-band via the per-file \
         `[AllowLoadGameType classic]` annotation on a single override file, not via a \
         separate flavor-split TOC"
    );

    let mainline = performance_bar_dir().join("Blizzard_PerformanceBar_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {} — the bare TOC is the canonical entry point",
        mainline.display()
    );
}

#[test]
fn blizzard_performance_bar_toc_declares_eager_game_only_with_no_dependencies() {
    let toc =
        TocFile::from_file(&performance_bar_toc()).expect("Blizzard_PerformanceBar TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "TOC OMITS `## LoadOnDemand:` so `is_load_on_demand()` returns false — the \
         performance bar is eager-loaded because its global functions \
         (MainMenuBarPerformanceBarFrame_OnEnter, etc.) are bound by name to the \
         MainMenuBarPerformanceBarFrame's OnEnter script handler in Blizzard_FrameXML's \
         MainMenuBar.xml; if PerformanceBar loaded later the script handler would resolve \
         to nil at the moment the user first hovers the bar"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: Game` must enable Game-screen discovery — `allows_screen` at \
         src/toc.rs:308 returns true for ScreenKind::Game when AllowLoad is `Game`"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "`## AllowLoad: Game` must NOT enable {screen:?} — the performance bar lives on \
             the in-game main menu bar; glue screens have no main menu bar to attach to"
        );
    }

    assert!(
        toc.dependencies().is_empty(),
        "Zero `## RequiredDep:` / `## Dependencies:` — the performance bar is a leaf in \
         the FrameXML graph; it consumes only Blizzard_FrameXML primitives (GameTooltip, \
         GetNetStats, GetFramerate, GetAvailableBandwidth, GetMovieDownloadProgress, \
         C_AddOns.GetNumAddOns, C_AddOnProfiler) which are all part of the SharedXML / \
         FrameXML core"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` — no soft sibling addons"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — the performance bar is a pure live-stats mirror; nothing \
         to persist across sessions"
    );
}

#[test]
fn blizzard_performance_bar_toc_declares_metadata_in_raw_bytes() {
    let raw = std::fs::read_to_string(performance_bar_toc())
        .expect("Blizzard_PerformanceBar TOC reads utf-8");
    assert!(
        raw.contains("## Title: Blizzard_PerformanceBar"),
        "TOC must declare `## Title: Blizzard_PerformanceBar` exactly. UNUSUAL: the title \
         uses the underscore-namespace spelling (rather than the space-and-prose form like \
         `Blizzard Performance Bar`); the minority pattern, suggesting a code-template \
         scaffold"
    );
    assert!(
        raw.contains("## Author: Blizzard Entertainment"),
        "TOC must declare `## Author: Blizzard Entertainment` exactly"
    );
    assert!(
        raw.contains("## DefaultState: enabled"),
        "TOC must declare `## DefaultState: enabled` exactly — disabling the performance \
         bar would break the main menu bar's tooltip on the latency icon (the OnEnter \
         script would resolve to nil); the explicit `enabled` documents that this is \
         foundational UI infrastructure"
    );
    assert!(
        raw.contains("## AllowLoad: Game"),
        "TOC must declare `## AllowLoad: Game` exactly — case-sensitive Game-only screen lock"
    );
    assert!(
        raw.contains("Classic\\PerformanceBarOverrides.lua [AllowLoadGameType classic]"),
        "TOC body must list the classic-only override file with the per-file \
         `[AllowLoadGameType classic]` annotation verbatim. The annotation routes through \
         `is_allowed_game_type` at src/toc.rs:45-57 which only accepts `mainline` or \
         `standard` — `classic` is filtered out, so the file is excluded from `toc.files` \
         on Mainline. The override exists because Classic re-enables the legacy \
         `showNewbieTips` CVar tooltip behavior that Mainline removed"
    );
    assert!(
        !raw.contains("## LoadOnDemand"),
        "TOC must NOT declare `## LoadOnDemand` — the absence is what flags eager loading"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT declare any `## SavedVariables*` keys — pure live-stats mirror"
    );
    assert!(
        !raw.contains("## Dependencies"),
        "TOC must NOT declare any `## Dependencies` keys — leaf addon"
    );
    assert!(
        !raw.contains("## OptionalDeps"),
        "TOC must NOT declare any `## OptionalDeps:` — zero soft siblings"
    );
}

#[test]
fn blizzard_performance_bar_toc_filters_classic_only_file_on_mainline() {
    let toc =
        TocFile::from_file(&performance_bar_toc()).expect("Blizzard_PerformanceBar TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, PERFORMANCE_BAR_MAINLINE_FILES,
        "TOC body lists 2 raw lines on disk (PerformanceBar.lua + \
         Classic/PerformanceBarOverrides.lua [AllowLoadGameType classic]), but \
         `is_allowed_game_type` at src/toc.rs:45-57 filters out the classic-only line. \
         The result on Mainline is exactly 1 file: PerformanceBar.lua. Pinning the \
         filtered list guards against a regression where the `[AllowLoadGameType ...]` \
         filter is removed and the Classic-only file would crash on Mainline because it \
         redefines MainMenuBarPerformanceBarFrame_UseDetailedTooltip to read the \
         `showNewbieTips` CVar that no longer exists in retail"
    );
}

#[test]
fn blizzard_performance_bar_appears_in_game_screen_eager_discovery_only() {
    let ui = blizzard_ui_dir();

    let game_addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_PerformanceBar");
    assert!(
        in_game,
        "Blizzard_PerformanceBar must auto-discover on Game screen — eager (no \
         LoadOnDemand) AND `## AllowLoad: Game` makes \
         `discover_blizzard_addons_for_screen(Game)` include the addon"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_PerformanceBar");
        assert!(
            !found,
            "Blizzard_PerformanceBar must NOT auto-discover on screen {screen:?} — `## \
             AllowLoad: Game` restricts discovery to ScreenKind::Game only"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_performance_bar_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_PerformanceBar")
                || message.contains("PerformanceBar")
                || message.contains("MainMenuBarPerformanceBarFrame")
                || message.contains("MainMenu_GetMovieDownloadProgress")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_PerformanceBar emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_performance_bar_is_addon_loaded_after_eager_sweep(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_PerformanceBar')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_PerformanceBar') must return true after the \
         eager Game-screen sweep — no LoadOnDemand puts the addon in the eager set"
    );
}
}

prefork_full_ui_case! {
fn blizzard_performance_bar_publishes_three_global_functions(env: &WowLuaEnv) {

    for func in PUBLIC_GLOBAL_FUNCTIONS {
        let kind: String = env
            .eval(&format!("return type(_G.{func})"))
            .unwrap_or_else(|err| panic!("type(_G.{func}) probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "_G.{func} must publish as a function. PerformanceBar.lua declares 3 \
             top-level global functions (no mixins, no namespace tables, no XML — the \
             entire addon is 192 lines of Lua publishing 3 free functions): \
             MainMenu_GetMovieDownloadProgress(id) (returns the aggregate \
             cinematic-download progress for the expansion-id-keyed MovieList — sums \
             GetMovieDownloadProgress over all movieIds in the list), \
             MainMenuBarPerformanceBarFrame_UseDetailedTooltip(self) (returns false on \
             Mainline — the Classic override file replaces this with the legacy \
             `showNewbieTips` CVar check), and MainMenuBarPerformanceBarFrame_OnEnter(self) \
             (the GameTooltip-building OnEnter handler bound by name to the \
             MainMenuBarPerformanceBarFrame in Blizzard_FrameXML's MainMenuBar.xml — \
             builds tooltip lines for latency, IPv4/IPv6 protocol, framerate, bandwidth, \
             download percent, in-progress cinematic downloads, top-3 addon CPU profiler \
             results from C_AddOnProfiler, and top-3 addon memory usage from \
             UpdateAddOnMemoryUsage / GetAddOnMemoryUsage)"
        );
    }
}
}

prefork_full_ui_case! {
fn performance_api_shims_are_safe_noops(env: &WowLuaEnv) {

    // 9fc0a48934 models the Update/Reset calls with zero returns (R88fg.3: Forever
    // VERIFIED zero Lua values; Retail return values UNKNOWN), not the retired 0 results.
    let result: (f64, f64, f64, f64, f64) = env
        .eval(
            r##"
            return GetFramerate(),
                   select("#", UpdateAddOnMemoryUsage()),
                   select("#", UpdateAddOnCPUUsage()),
                   select("#", ResetCPUUsage()),
                   GetAddOnMemoryUsage("missing")
            "##,
        )
        .expect("performance API shim probe should succeed");

    assert_eq!(result, (60.0, 0.0, 0.0, 0.0, 0.0));
}
}

prefork_full_ui_case! {
fn performance_bar_on_enter_builds_tooltip_without_format_errors(env: &WowLuaEnv) {

    let result: String = env
        .eval(
            r#"
            local frame = MainMenuMicroButton and MainMenuMicroButton.MainMenuBarPerformanceBar
            if not frame then return "missing_frame" end
            local ok, err = pcall(MainMenuBarPerformanceBarFrame_OnEnter, frame)
            if not ok then return tostring(err) end
            return "ok"
            "#,
        )
        .expect("PerformanceBar OnEnter probe should run");

    assert_eq!(result, "ok");
}
}
