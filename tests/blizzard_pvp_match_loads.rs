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

fn pvp_match_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_PVPMatch")
}

fn pvp_match_toc() -> PathBuf {
    pvp_match_dir().join("Blizzard_PVPMatch.toc")
}

const PVP_MATCH_TOC_FILES: &[&str] = &[
    "PVPMatchUtil.lua",
    "PVPMatchTable.xml",
    "PVPMatchResults.xml",
    "PVPMatchScoreboard.xml",
];

const REQUIRED_DEPS: &[&str] = &["Blizzard_UIWidgets", "Blizzard_LFGUtil"];

const PUBLIC_MIXIN_GLOBALS: &[&str] = &[
    "PVPRowMixin",
    "PVPHeaderMixin",
    "PVPHeaderIconMixin",
    "PVPCellClassMixin",
    "PVPCellHonorLevelMixin",
    "PVPHeaderStringMixin",
    "PVPCellStringMixin",
    "PVPCellNameMixin",
    "PVPSoloShuffleCellNameMixin",
    "PVPCellStatMixin",
    "PVPSoloShuffleCellStatMixin",
    "PVPNewRatingMixin",
    "PVPMatchScoreboardMixin",
    "PVPMatchResultsCurrencyRewardMixin",
    "PVPMatchResultsMixin",
    "PVPMatchResultsRatingMixin",
];

const VIRTUAL_TEMPLATES_SAMPLE: &[&str] = &[
    "PVPTableRowTemplate",
    "PVPStringTemplate",
    "PVPIconTemplate",
    "PVPCellHonorLevelTemplate",
    "PVPCellClassTemplate",
    "PVPCellNameTemplate",
    "PVPSoloShuffleCellNameTemplate",
    "PVPCellStringTemplate",
    "PVPCellStatTemplate",
    "PVPSoloShuffleCellStatTemplate",
    "PVPNewRatingTemplate",
    "PVPHeaderStringTemplate",
    "PVPHeaderIconTemplate",
    "PVPMatchResultsCurrencyRewardTemplate",
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
fn blizzard_pvp_match_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&pvp_match_dir()).expect("Blizzard_PVPMatch TOC resolves");
    assert_eq!(
        resolved,
        pvp_match_toc(),
        "Blizzard_PVPMatch ships exactly one bare TOC — no `_Mainline.toc` variant"
    );

    let mainline = pvp_match_dir().join("Blizzard_PVPMatch_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {}",
        mainline.display()
    );
}

#[test]
fn blizzard_pvp_match_toc_declares_explicit_load_on_demand_zero() {
    let toc = TocFile::from_file(&pvp_match_toc()).expect("Blizzard_PVPMatch TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "TOC must NOT count as LoadOnDemand — `## LoadOnDemand: 0` is the rare \
         EXPLICIT-zero form (almost every Blizzard_* addon either declares \
         `## LoadOnDemand: 1` for lazy load or omits the directive entirely for \
         eager load). The explicit zero is semantically equivalent to omission \
         here: `is_load_on_demand()` at src/toc.rs reads `LoadOnDemand` and only \
         returns true when the value is `1` (or `true`), so `0` resolves to false"
    );
    assert!(!toc.is_load_first());
    assert!(
        !toc.is_secure_env(),
        "TOC must NOT declare `## UseSecureEnvironment:` — insecure addon"
    );
    assert!(
        !toc.is_ptr_only(),
        "TOC has no `## OnlyBetaAndPTR:` directive — ships on live realms"
    );
}

#[test]
fn blizzard_pvp_match_toc_pins_standard_game_type_as_unrestricted() {
    let toc = TocFile::from_file(&pvp_match_toc()).expect("Blizzard_PVPMatch TOC parses");

    assert!(
        !toc.is_game_type_restricted(),
        "TOC declares `## AllowLoadGameType: standard` — `is_game_type_restricted()` \
         at src/toc.rs:294-302 returns FALSE because `standard` (alongside `mainline`) \
         is in the cross-flavor allowlist. The two values are treated as the modern \
         retail flavor; only values like `classic`, `wrath`, `cata`, `mists`, \
         `plunderstorm`, `wowhack` flip the restriction true. PVP scoreboard / \
         results UI ships on retail (`standard`) only — not on classic flavors \
         where the older PVPFrameTemplate predates this addon"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "TOC has no `## AllowLoad:` directive — `allows_screen` defaults to Game-only"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Default Game-only screen gate must EXCLUDE {screen:?} — PVP scoreboard \
             only matters in-world"
        );
    }
}

#[test]
fn blizzard_pvp_match_toc_uses_singular_required_dep_form() {
    let toc = TocFile::from_file(&pvp_match_toc()).expect("Blizzard_PVPMatch TOC parses");

    let dependencies = toc.dependencies();
    let deps: Vec<&str> = dependencies.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        deps, REQUIRED_DEPS,
        "TOC must declare exactly 1 dep: Blizzard_UIWidgets — note this addon uses \
         the SINGULAR `## RequiredDep:` form rather than the plural \
         `## Dependencies:` form. The `dependencies()` accessor at src/toc.rs:210-217 \
         tries `RequiredDep` FIRST, falling back to `Dependencies` then \
         `RequiredDeps`. Blizzard_UIWidgets publishes the UIWidgetTemplateMixin \
         family used by the PVP results screen's Reward and Bonus widget rows"
    );

    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` declared"
    );
    assert!(toc.load_with().is_empty(), "Zero `## LoadWith:` declared");
}

#[test]
fn blizzard_pvp_match_toc_declares_no_saved_variables() {
    let toc = TocFile::from_file(&pvp_match_toc()).expect("Blizzard_PVPMatch TOC parses");

    assert!(
        toc.saved_variables().is_empty(),
        "TOC must declare zero `## SavedVariables:` — pure stateless transient \
         scoreboard/results UI; every score pulls from the live battlefield score \
         via C_PvP queries each frame"
    );
    assert!(
        toc.saved_variables_per_character().is_empty(),
        "TOC must declare zero `## SavedVariablesPerCharacter:`"
    );
}

#[test]
fn blizzard_pvp_match_toc_declares_metadata_in_raw_bytes() {
    let raw = std::fs::read_to_string(pvp_match_toc()).expect("Blizzard_PVPMatch TOC reads utf-8");

    assert!(
        raw.contains("## Title: Blizzard_PVPMatch"),
        "TOC must declare `## Title: Blizzard_PVPMatch` exactly — note this is the \
         underscore-separated module name, NOT the space-separated player-facing \
         form (`Blizzard PVP Match`) most other addons use. PVPMatch is internal \
         infrastructure and was never given a player-facing title polish"
    );
    assert!(
        raw.contains("## LoadOnDemand: 0"),
        "TOC must declare `## LoadOnDemand: 0` exactly — the rare explicit-zero form"
    );
    assert!(
        raw.contains("## RequiredDep: Blizzard_UIWidgets"),
        "TOC must declare `## RequiredDep:` (singular) — NOT `## Dependencies:`"
    );
    assert!(
        raw.contains("## AllowLoadGameType: standard"),
        "TOC must declare `## AllowLoadGameType: standard` exactly"
    );

    assert!(
        !raw.contains("## Author"),
        "TOC must NOT declare `## Author:` — one of the few Blizzard_* addons \
         WITHOUT the canonical author line (PVPMatch was authored before the \
         author-line convention solidified)"
    );
    assert!(
        !raw.contains("## Dependencies"),
        "TOC must NOT use the plural `## Dependencies:` form — uses RequiredDep"
    );
    assert!(
        !raw.contains("## OnlyBetaAndPTR"),
        "TOC must NOT declare `## OnlyBetaAndPTR:` — ships on live"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT declare any SavedVariables directive"
    );
    assert!(
        !raw.contains("## UseSecureEnvironment"),
        "TOC must NOT declare `## UseSecureEnvironment:`"
    );
    assert!(
        !raw.contains("## DefaultState"),
        "TOC must NOT declare `## DefaultState:`"
    );
}

#[test]
fn blizzard_pvp_match_toc_lists_four_files_with_xml_pulling_companion_lua() {
    let toc = TocFile::from_file(&pvp_match_toc()).expect("Blizzard_PVPMatch TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, PVP_MATCH_TOC_FILES,
        "TOC must list exactly 4 files: PVPMatchUtil.lua FIRST (declares the \
         PVPMatchUtil global table with RowColors / CellColors palette tables, \
         the MatchTimeFormatter via `CreateFromMixins(SecondsFormatterMixin)`, \
         and ~20 helper functions InSoloShuffleBrawl / ModeUsesPvpRatingTiers / \
         IsActiveMatchComplete / GetColorIndex / GetRowColor / GetCellColor / \
         InitScrollBox / UpdateDataProvider / etc.), then 3 XML files in canonical \
         order: PVPMatchTable.xml (the row/cell template family — 14 virtual \
         templates), PVPMatchResults.xml (the post-match results screen with the \
         3-tab layout), PVPMatchScoreboard.xml (the in-match scoreboard overlay \
         with the 3-tab layout). The 3 XML files each declare \
         `<Script file=\"PVPMatchTable.lua\"/>` / `\"PVPMatchResults.lua\"` / \
         `\"PVPMatchScoreboard.lua\"` at the top of the XML body — the companion \
         Lua files are pulled in via XML rather than being listed in the TOC. \
         This is the canonical XML-driven Lua-loading pattern: only PVPMatchUtil.lua \
         (which has no companion XML) appears in the TOC body. Verify by reading \
         the XML files directly: PVPMatchTable.xml line 3 = `<Script file=\
         \"PVPMatchTable.lua\"/>`, etc."
    );
}

#[test]
fn blizzard_pvp_match_appears_in_eager_game_discovery() {
    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let found = addons.iter().any(|(name, _)| name == "Blizzard_PVPMatch");
    assert!(
        found,
        "Blizzard_PVPMatch MUST appear in eager Game-screen discovery — \
         `## LoadOnDemand: 0` is explicitly NOT lazy, `## AllowLoadGameType: \
         standard` does NOT restrict (allowlist hit), and absent `## AllowLoad:` \
         defaults to Game-only. All 3 gates pass for ScreenKind::Game"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let glue_addons = discover_blizzard_addons_for_screen(&ui, screen);
        let glue_found = glue_addons
            .iter()
            .any(|(name, _)| name == "Blizzard_PVPMatch");
        assert!(
            !glue_found,
            "Blizzard_PVPMatch must NOT appear in eager discovery for {screen:?} \
             — default Game-only gate excludes glue screens"
        );
    }
}

#[test]
fn blizzard_pvp_match_appears_in_full_addon_inventory() {
    let inventory = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = inventory
        .iter()
        .any(|(name, _)| name == "Blizzard_PVPMatch");
    assert!(
        found,
        "Blizzard_PVPMatch MUST appear in `discover_all_blizzard_addons`"
    );
}

prefork_full_ui_case! {
fn blizzard_pvp_match_loads_via_eager_game_sweep_without_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_PVPMatch")
                || message.contains("PVPMatchUtil")
                || message.contains("PVPMatchScoreboard")
                || message.contains("PVPMatchResults")
                || message.contains("PVPMatchTable")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_PVPMatch emitted addon-specific Lua errors during the eager \
         Game-screen sweep:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_pvp_match_publishes_util_global_with_color_tables(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(_G.PVPMatchUtil)")
        .expect("PVPMatchUtil type probe succeeds");
    assert_eq!(
        kind, "table",
        "_G.PVPMatchUtil must publish as a table — PVPMatchUtil.lua line 1 \
         declares the global table with RowColors and CellColors palette tables \
         at file scope. The table holds the 4-entry color palettes for \
         Horde/Alliance row + alternate variants, the MatchTimeFormatter \
         seconds-formatter helper, and the helper-function set used by the \
         scoreboard / results panels"
    );

    let row_colors_count: i64 = env
        .eval("return #PVPMatchUtil.RowColors")
        .expect("RowColors count probe succeeds");
    assert_eq!(
        row_colors_count, 4,
        "PVPMatchUtil.RowColors must hold exactly 4 entries: \
         PVP_SCOREBOARD_HORDE_ROW_COLOR + HORDE_ALT + ALLIANCE + ALLIANCE_ALT — \
         the 2-faction × 2-alternation matrix that GetRowColor indexes via \
         `(useAlternateColor and 2 or 1) + (factionIndex * 2)`"
    );

    let cell_colors_count: i64 = env
        .eval("return #PVPMatchUtil.CellColors")
        .expect("CellColors count probe succeeds");
    assert_eq!(
        cell_colors_count, 4,
        "PVPMatchUtil.CellColors must hold exactly 4 entries — same 2x2 matrix \
         as RowColors but for cell-level coloring"
    );

    let formatter_kind: String = env
        .eval("return type(PVPMatchUtil.MatchTimeFormatter)")
        .expect("MatchTimeFormatter type probe succeeds");
    assert_eq!(
        formatter_kind, "table",
        "PVPMatchUtil.MatchTimeFormatter must publish as a table — created via \
         `CreateFromMixins(SecondsFormatterMixin)` then initialized with \
         `:Init(0, SecondsFormatter.Abbreviation.Truncate, true)`"
    );
}
}

prefork_full_ui_case! {
fn blizzard_pvp_match_publishes_sixteen_mixin_globals(env: &WowLuaEnv) {

    for mixin in PUBLIC_MIXIN_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G[{mixin:?}])"))
            .unwrap_or_else(|err| panic!("type probe for {mixin} failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table — Blizzard_PVPMatch ships exactly \
             16 public Mixin globals across 3 Lua files: 12 in PVPMatchTable.lua \
             (the table-builder Row/Header/Cell families that extend \
             TableBuilderRowMixin / TableBuilderElementMixin / TableBuilderCellMixin), \
             1 in PVPMatchScoreboard.lua (PVPMatchScoreboardMixin owning the \
             in-match scoreboard frame), 3 in PVPMatchResults.lua \
             (PVPMatchResultsMixin owning the post-match results frame, plus \
             PVPMatchResultsCurrencyRewardMixin for currency-reward rows and \
             PVPMatchResultsRatingMixin for the rating-change-display panel)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_pvp_match_publishes_two_named_top_level_frames(env: &WowLuaEnv) {

    for frame_name in ["PVPMatchScoreboard", "PVPMatchResults"] {
        let kind: String = env
            .eval(&format!("return type(_G[{frame_name:?}])"))
            .unwrap_or_else(|err| panic!("type probe for {frame_name} failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{frame_name} must publish as a frame — Blizzard_PVPMatch ships \
             exactly 2 named non-virtual top-level frames: PVPMatchScoreboard \
             (DIALOG-strata toplevel scoreboard overlay shown via \
             ShowUIPanel during a match) and PVPMatchResults (HIGH-strata \
             toplevel results screen shown after match completion). Both are \
             parented to UIParent and start hidden=true. Note neither is \
             registered with RegisterUIPanel — they're managed externally by \
             FrameXML's PVP show/hide logic"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_pvp_match_virtual_templates_not_in_global_env(env: &WowLuaEnv) {

    for template_name in VIRTUAL_TEMPLATES_SAMPLE {
        let kind: String = env
            .eval(&format!("return type(_G[{template_name:?}])"))
            .unwrap_or_else(|err| panic!("type probe for {template_name} failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{template_name} must be nil — virtual templates live in the \
             template registry, NOT the global environment. Blizzard_PVPMatch \
             ships ~14 virtual templates across the 3 XML files (Row/String/Icon \
             base templates + 8 Cell variants in PVPMatchTable.xml + Header \
             variants + the CurrencyReward template in PVPMatchResults.xml). \
             Sampled here is a representative subset"
        );
    }
}
}
