//! Foundation-lane analysis: Blizzard_SharedXMLBase → Blizzard_SharedXML →
//! Blizzard_SharedXMLGame → Blizzard_UIPanelTemplates.
//!
//! This file is a LANE audit — separate from the per-addon loadability suites
//! (`blizzard_shared_xml_base_loads.rs`, `blizzard_shared_xml_loads.rs`,
//! `blizzard_shared_xml_game_loads.rs`, `blizzard_ui_panel_templates_loads.rs`).
//! It pins the cross-cutting invariants that downstream addon plans must rely
//! on: the dep-chain ordering, the harness shape, and the simulator gaps that
//! deviate from real-WoW TOC semantics.
//!
//! Layered structure (top = base, bottom = consumer):
//!     SharedXMLBase   (AllowLoad: Both, Mainline+Mists agnostic, depends on Blizzard_ScriptErrors)
//!         └── SharedXML       (AllowLoad: Both, Mainline-only via AllowLoadGameType, depends on
//!                              Fonts_Shared / SharedXMLBase / PrintHandler / Menu / Colors / HelpPlate)
//!             └── SharedXMLGame   (AllowLoad: Game, singular `## Dep:` form for SharedXML + Colors)
//!                 └── UIPanelTemplates (AllowLoad: game, AllowLoadGameType: mainline,
//!                                       depends on SharedXMLGame)

use std::path::{Path, PathBuf};

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

const LANE_ADDONS_BASE_TO_CONSUMER: &[&str] = &[
    "Blizzard_SharedXMLBase",
    "Blizzard_SharedXML",
    "Blizzard_SharedXMLGame",
    "Blizzard_UIPanelTemplates",
];

fn lane_toc(name: &str) -> PathBuf {
    let dir = blizzard_ui_dir().join(name);
    find_toc_file(&dir).unwrap_or_else(|| panic!("TOC for `{name}` should resolve under {dir:?}"))
}

fn parse_lane_toc(name: &str) -> TocFile {
    TocFile::from_file(&lane_toc(name)).unwrap_or_else(|err| panic!("`{name}` TOC parses: {err}"))
}

fn position_in(haystack: &[String], needle: &str) -> Option<usize> {
    haystack.iter().position(|n| n == needle)
}

fn fresh_lane_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("WowLuaEnv constructs");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    env
}

#[test]
fn each_lane_addon_directory_resolves_with_a_toc() {
    for name in LANE_ADDONS_BASE_TO_CONSUMER {
        let dir = blizzard_ui_dir().join(name);
        assert!(
            dir.is_dir(),
            "Lane addon `{name}` directory must exist at \
             `Interface/BlizzardUI/{name}/` — downstream addon plans assume the four foundation \
             dirs are guaranteed present"
        );
        assert!(
            find_toc_file(&dir).is_some(),
            "Lane addon `{name}` must ship a TOC file resolvable via find_toc_file. The \
             foundation lane must publish via TOCs only — no `.toc.disabled`, no externalized \
             manifest"
        );
    }
}

#[test]
fn lane_dep_edges_pin_canonical_chain() {
    let base = parse_lane_toc("Blizzard_SharedXMLBase");
    let shared = parse_lane_toc("Blizzard_SharedXML");
    let _game = parse_lane_toc("Blizzard_SharedXMLGame");
    let panels = parse_lane_toc("Blizzard_UIPanelTemplates");

    let base_deps = base.dependencies();
    assert_eq!(
        base_deps,
        vec!["Blizzard_ScriptErrors".to_string()],
        "SharedXMLBase pins exactly one dep: Blizzard_ScriptErrors. The error-handler addon \
         must be live before any base utility loads because Mixin / TableUtil / FrameUtil rely \
         on the script-error pipeline to surface bad-mixin errors at file scope. Got: {base_deps:?}"
    );

    let shared_deps = shared.dependencies();
    let expected_shared_deps = [
        "Blizzard_ProjectConstants",
        "Blizzard_Fonts_Shared",
        "Blizzard_SharedXMLBase",
        "Blizzard_PrintHandler",
        "Blizzard_Menu",
        "Blizzard_Colors",
        "Blizzard_HelpPlate",
        "Blizzard_Narration",
    ];
    assert_eq!(
        shared_deps,
        expected_shared_deps
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
        "Retail 12.1.0.69497 pins 8 SharedXML dependencies in published order, including \
         ProjectConstants first and Narration last. Got: {shared_deps:?}"
    );

    let panels_deps = panels.dependencies();
    assert_eq!(
        panels_deps,
        vec!["Blizzard_SharedXMLGame".to_string()],
        "UIPanelTemplates pins exactly one dep: SharedXMLGame. Implicit chain: SharedXMLGame \
         pulls in SharedXML, SharedXML pulls in SharedXMLBase, so loading UIPanelTemplates \
         transitively pulls in the entire foundation lane via the dependency resolver. Got: \
         {panels_deps:?}"
    );
}

#[test]
fn shared_xml_game_repeated_singular_deps_are_accumulated() {
    let game = parse_lane_toc("Blizzard_SharedXMLGame");

    assert_eq!(
        game.dependencies(),
        vec![
            "Blizzard_SharedXML".to_string(),
            "Blizzard_Colors".to_string(),
        ],
        "SharedXMLGame's repeated `## Dep:` directives must remain ordered and complete"
    );

    let raw =
        std::fs::read_to_string(lane_toc("Blizzard_SharedXMLGame")).expect("SharedXMLGame reads");
    assert!(raw.contains("## Dep: Blizzard_SharedXML"));
    assert!(raw.contains("## Dep: Blizzard_Colors"));
}

#[test]
fn shared_xml_base_publishes_global_env_overrides_for_diagnostic_files() {
    let base = parse_lane_toc("Blizzard_SharedXMLBase");

    let body: Vec<String> = base
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    assert!(
        body.iter().any(|f| f == "Mixin.lua"),
        "SharedXMLBase body must include Mixin.lua — the foundational `Mixin(self, ...)` / \
         `CreateFromMixins(...)` global is what every consumer mixin in the lane (and every \
         addon downstream) calls at file scope to extend tables. Got: {body:?}"
    );
    assert!(body.iter().any(|f| f == "FrameUtil.lua"));
    assert!(body.iter().any(|f| f == "Pools.lua"));
    assert!(body.iter().any(|f| f == "CallbackRegistry.lua"));

    for diagnostic in &["Mixin.lua", "TableUtil.lua", "FrameUtil.lua"] {
        let idx = position_in(&body, diagnostic).unwrap_or_else(|| {
            panic!("`{diagnostic}` must appear in SharedXMLBase body");
        });
        assert!(
            base.file_use_secure_env(idx).is_none(),
            "`{diagnostic}` must NOT carry an `[AllowLoadEnvironment ...]` annotation — these \
             foundational utilities load with the default fenv. file_use_secure_env returns \
             None when no override is present"
        );
    }
}

#[test]
fn shared_xml_diagnostic_files_carry_allow_load_environment_global_in_raw_form() {
    let shared = parse_lane_toc("Blizzard_SharedXML");

    let raw = std::fs::read_to_string(lane_toc("Blizzard_SharedXML")).expect("SharedXML reads");

    for diagnostic in &[
        "Dump.lua [AllowLoadEnvironment Global]",
        "DebugBarManager.lua [AllowLoadEnvironment Global]",
        "HelpTip.lua [AllowLoadEnvironment Global]",
        "HelpTip.xml [AllowLoadEnvironment Global]",
        "SharedBasicControls.lua [AllowLoadEnvironment Global]",
    ] {
        assert!(
            raw.contains(diagnostic),
            "SharedXML raw bytes must contain `{diagnostic}` — these entries explicitly request \
             the global load pass even when another environment pass is being replayed"
        );
    }

    let body: Vec<String> = shared
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    let dump_idx = position_in(&body, "Dump.lua").expect("Dump.lua must appear in SharedXML body");
    assert_eq!(
        shared.file_use_secure_env(dump_idx),
        None,
        "`[AllowLoadEnvironment Global]` must not map to the same per-file fenv override as \
         `[LoadIntoEnvironment Global]`"
    );
    assert_eq!(
        shared.file_allow_load_environment(dump_idx),
        Some(false),
        "`[AllowLoadEnvironment Global]` marks a global-only load pass"
    );
}

#[test]
fn shared_xml_game_body_substitutes_family_placeholder_to_mainline() {
    let game = parse_lane_toc("Blizzard_SharedXMLGame");

    let body: Vec<String> = game
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    assert!(
        body.iter().any(|f| f == "Mainline/Localization.lua"),
        "SharedXMLGame body uses `[Family]\\Localization.lua [AllowLoadGameType mainline]` \
         in raw form. The loader at src/toc.rs:144-146 substitutes `[Family]` → `Mainline` \
         (and `[Game]` → `Standard`) on every body line. The vanilla-only \
         `[Game]\\Localization.lua [AllowLoadGameType vanilla]` line is filtered out by the \
         AllowLoadGameType gate at src/toc.rs:141-143. Got: {body:?}"
    );
    assert!(
        !body
            .iter()
            .any(|f| f.contains("[Family]") || f.contains("[Game]")),
        "Substituted body MUST NOT retain any `[Family]` / `[Game]` placeholder strings — if \
         either survives, the substitution table at src/toc.rs:144-146 has regressed. Got: \
         {body:?}"
    );
    assert!(
        body.iter().any(|f| f == "Tooltip/TooltipDataHandler.lua"),
        "TooltipDataHandler.lua carries `[ExcludeLoadGameType vanilla]` — the simulator does \
         NOT explicitly handle ExcludeLoadGameType (only AllowLoadGameType, see \
         is_allowed_game_type at src/toc.rs:43-57). The line survives because no \
         `[AllowLoadGameType` substring is present, so the early-return at \
         src/toc.rs:141-143 doesn't fire. THIS IS A LATENT GAP: a future TOC line carrying \
         `[ExcludeLoadGameType mainline]` would NOT be stripped on mainline as it should be. \
         Currently no foundation TOC exercises this path, so the gap is dormant"
    );
}

#[test]
fn lane_appears_in_eager_discovery_in_dep_first_order() {
    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let names: Vec<String> = addons.iter().map(|(name, _)| name.clone()).collect();

    let positions: Vec<(&str, usize)> = LANE_ADDONS_BASE_TO_CONSUMER
        .iter()
        .map(|name| {
            let pos = position_in(&names, name).unwrap_or_else(|| {
                panic!(
                    "Lane addon `{name}` must appear in Game eager discovery — every foundation \
                     addon is non-LoD and AllowLoad covers Game"
                );
            });
            (*name, pos)
        })
        .collect();

    for window in positions.windows(2) {
        let (prev_name, prev_pos) = window[0];
        let (next_name, next_pos) = window[1];
        assert!(
            prev_pos < next_pos,
            "Eager discovery must list `{prev_name}` BEFORE `{next_name}` (got positions \
             {prev_pos} and {next_pos}). The dep resolver topologically sorts so each consumer \
             follows its deps. If this regresses, downstream addon load_addon calls will fail \
             because the consumer's CreateFromMixins / template-inherit reference resolves to \
             nil before the dep populates the registry"
        );
    }
}

#[test]
fn shared_xml_base_loads_on_glue_screens_others_do_not() {
    let ui = blizzard_ui_dir();
    let glue_screens = [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ];

    let base = parse_lane_toc("Blizzard_SharedXMLBase");
    let shared = parse_lane_toc("Blizzard_SharedXML");
    let game = parse_lane_toc("Blizzard_SharedXMLGame");
    let panels = parse_lane_toc("Blizzard_UIPanelTemplates");

    for screen in glue_screens {
        assert!(
            base.allows_screen(screen),
            "SharedXMLBase `## AllowLoad: Both` must include glue screen {screen:?} — the \
             foundation utilities (Mixin / TableUtil / Color) are required by glue-screen UIs \
             too (CharacterSelect, etc.)"
        );
        assert!(
            shared.allows_screen(screen),
            "SharedXML `## AllowLoad: Both` must include glue screen {screen:?} — \
             SharedXML's UI primitives (UIDropDownMenu, button templates, ScrollBox) are \
             consumed by glue-screen addons like Blizzard_GlueXML"
        );
        assert!(
            !game.allows_screen(screen),
            "SharedXMLGame `## AllowLoad: Game` must EXCLUDE glue screen {screen:?} — the \
             tooltip-data layer requires an in-game character context (transmog, equipped \
             items, hovered units) that doesn't exist on glue screens"
        );
        assert!(
            !panels.allows_screen(screen),
            "UIPanelTemplates `## AllowLoad: game` must EXCLUDE glue screen {screen:?} — \
             AutoCastTemplates / UIPanelSpellButtonFrame are spell-bar specific UI"
        );

        let names: Vec<String> = discover_blizzard_addons_for_screen(&ui, screen)
            .into_iter()
            .map(|(n, _)| n)
            .collect();
        assert!(
            names.contains(&"Blizzard_SharedXMLBase".to_string()),
            "SharedXMLBase MUST appear in {screen:?} eager discovery — without the base \
             utilities, glue-screen addons cannot Mixin or build pools"
        );
        assert!(
            !names.contains(&"Blizzard_SharedXMLGame".to_string()),
            "SharedXMLGame MUST be ABSENT from {screen:?} eager discovery"
        );
        assert!(
            !names.contains(&"Blizzard_UIPanelTemplates".to_string()),
            "UIPanelTemplates MUST be ABSENT from {screen:?} eager discovery"
        );
    }
}

prefork_full_ui_case! {
    fn lane_publishes_foundational_globals_after_full_game_load(env: &WowLuaEnv) {

        let foundational_globals = [
            ("Mixin", "function"),
            ("CreateFromMixins", "function"),
            ("CallbackRegistryMixin", "table"),
            ("FrameUtil", "table"),
            ("AnchorUtil", "table"),
            ("TableUtil", "table"),
            ("CreateColor", "function"),
            ("ColorMixin", "table"),
            ("HelpTip", "table"),
            ("ScrollUtil", "table"),
            ("EventRegistry", "table"),
        ];

        for (global, expected_kind) in foundational_globals {
            let actual: String = env
                .eval(&format!("return type({global})"))
                .unwrap_or_else(|err| panic!("type({global}) probe failed: {err}"));
            assert_eq!(
                actual, expected_kind,
                "Foundation global `{global}` must publish as `{expected_kind}` after the lane \
                 loads. If this regresses, downstream addon plans cannot rely on the global being \
                 live at file scope. `Mixin` is from SharedXMLBase/Mixin.lua; `CallbackRegistryMixin` \
                 from SharedXMLBase/CallbackRegistry.lua; `FrameUtil`/`AnchorUtil`/`TableUtil` \
                 from SharedXMLBase; `HelpTip` from SharedXML/HelpTip.lua (carries \
                 `[AllowLoadEnvironment Global]`, which the TOC parser maps to a global-env \
                 override); \
                 `ScrollUtil` from SharedXML/Shared/Scroll/ScrollUtil.lua; `CreateColor` \
                 constructor + `ColorMixin` table from SharedXMLBase/Color.lua (the global is the \
                 constructor function `CreateColor`, not a `Color` global — `ColorMixin` is the \
                 reusable mixin table that `CreateColor` returns instances of via \
                 `CreateFromMixins(ColorMixin)`)"
            );
        }
    }
}

prefork_full_ui_case! {
    fn lane_xml_templates_register_for_downstream_inheritance(env: &WowLuaEnv) {
        let _env = env;

        let foundational_templates = [
            (
                "CallbackRegistrantTemplate",
                "Blizzard_SharedXMLBase/CallbackRegistrant.xml",
            ),
            (
                "ColorSwatchTemplate",
                "Blizzard_SharedXMLBase/ColorSwatch.xml",
            ),
            (
                "UIPanelButtonTemplate",
                "Blizzard_SharedXML/Mainline/SharedUIPanelTemplates.xml",
            ),
            (
                "ManagedHorizontalLayoutFrameTemplate",
                "Blizzard_SharedXML/ManagedLayoutFrame.xml",
            ),
            (
                "ManagedVerticalLayoutFrameTemplate",
                "Blizzard_SharedXML/ManagedLayoutFrame.xml",
            ),
        ];

        for (template, source) in foundational_templates {
            assert!(
                wow_ui_sim::xml::get_template(template).is_some(),
                "Foundation template `{template}` (defined in {source}) must register in \
                 `wow_ui_sim::xml::get_template`. Downstream addon XML files use \
                 `inherits=\"{template}\"` and the parser resolves the inheritance at element- \
                 instantiation time. If this regresses, downstream addons fail to instantiate \
                 their named frames"
            );
        }
    }
}

prefork_full_ui_case! {
    fn lane_emits_no_addon_specific_lua_errors_during_full_load(env: &WowLuaEnv) {

        let load_errors: Vec<String> = env
            .state()
            .borrow()
            .lua_errors
            .iter()
            .filter(|message| {
                LANE_ADDONS_BASE_TO_CONSUMER
                    .iter()
                    .any(|name| message.contains(name))
            })
            .cloned()
            .collect();

        assert!(
            load_errors.is_empty(),
            "Foundation lane MUST load without lane-specific Lua errors — the lane is the floor \
             for everything else, so any error here cascades. Got:\n  {}",
            load_errors.join("\n  ")
        );
    }
}

prefork_full_ui_case! {
    fn lane_addons_are_loaded_after_eager_sweep(env: &WowLuaEnv) {

        for name in LANE_ADDONS_BASE_TO_CONSUMER {
            let loaded: bool = env
                .eval(&format!("return C_AddOns.IsAddOnLoaded('{name}')"))
                .unwrap_or_else(|err| panic!("IsAddOnLoaded({name}) probe failed: {err}"));
            assert!(
                loaded,
                "C_AddOns.IsAddOnLoaded('{name}') must return true after the eager Game sweep. \
                 The required harness shape for any downstream test is therefore: \
                 (1) `WowLuaEnv::new()`, (2) `set_screen_mode(ScreenKind::Game)`, \
                 (3) `state.addon_base_paths = vec![blizzard_ui_dir()]`, \
                 (4) `register_intrinsic_templates()`, \
                 (5) `discover_blizzard_addons_for_screen(&ui, ScreenKind::Game)` + per-result \
                 `load_addon`, (6) `apply_post_load_workarounds()`, \
                 (7) `fire_startup_events_for_screen(&env, ScreenKind::Game)`. The eager sweep \
                 brings in the full lane plus all other AllowLoad: Game eager addons, so \
                 downstream tests do not need to call load_addon for any lane addon directly"
            );
        }
    }
}

#[test]
fn lane_required_harness_shape_helpers_exist() {
    fn assert_path_resolves(path: &Path) -> bool {
        path.exists()
    }

    assert!(
        assert_path_resolves(&blizzard_ui_dir()),
        "Harness invariant: blizzard_ui_dir() (Interface/BlizzardUI) must resolve. Every lane \
         test sets `state.addon_base_paths = vec![blizzard_ui_dir()]` and the loader probes \
         this path for both eager-discovery and explicit load_addon calls"
    );

    let env = fresh_lane_env();
    let kind: String = env
        .eval("return type(_G)")
        .expect("fresh env must expose _G");
    assert_eq!(
        kind, "table",
        "Harness invariant: a fresh WowLuaEnv must expose `_G` even before any addon loads. \
         If this regresses, downstream tests cannot probe globals via env.eval"
    );
}
