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

fn ui_panel_templates_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_UIPanelTemplates")
}

fn ui_panel_templates_toc() -> PathBuf {
    find_toc_file(&ui_panel_templates_dir()).expect("UIPanelTemplates TOC resolves")
}

const GLUE_SCREENS: &[ScreenKind] = &[
    ScreenKind::Login,
    ScreenKind::CharacterSelect,
    ScreenKind::CharacterCreate,
];

const TOC_DEPENDENCIES: &[&str] = &["Blizzard_SharedXMLGame"];

const MIXINS: &[&str] = &[
    "UIPanelSpellButtonFrameMixin",
    "AutoCastOverlayMixin",
    "RoleCountMixin",
    "CurrencyTemplateMixin",
    "CurrencyDisplayMixin",
    "CurrencyDisplayGroupMixin",
    "CurrencyLayoutFrameIconMixin",
    "CurrencyHorizontalLayoutFrameMixin",
    "UIExpandingButtonMixin",
    "TalentRankDisplayMixin",
    "ButtonWithDisableMixin",
    "AnimatedShineMixin",
];

const FREE_FUNCTIONS: &[&str] = &[
    "BagSearch_OnHide",
    "BagSearch_OnTextChanged",
    "BagSearch_OnChar",
    "SquareButton_SetIcon",
    "CapProgressBar_SetNotches",
    "CapProgressBar_Update",
    "SetCheckButtonIsRadio",
    "InlineHyperlinkFrame_OnEnter",
    "InlineHyperlinkFrame_OnLeave",
    "InlineHyperlinkFrame_OnClick",
];

const REPRESENTATIVE_VIRTUAL_TEMPLATES: &[&str] = &[
    "BagSearchBoxTemplate",
    "GameMenuButtonTemplate",
    "AnimatedShineTemplate",
    "UIPanelBorderedButtonTemplate",
    "RoleCountNoScriptsTemplate",
    "RoleCountTemplate",
    "BaseBasicFrameTemplate",
    "BasicFrameTemplate",
    "EtherealFrameTemplate",
    "HorizontalBarTemplate",
    "TranslucentFrameTemplate",
    "ShadowOverlaySmallTemplate",
    "CapProgressBarTemplate",
    "ThinGoldEdgeTemplate",
    "InlineHyperlinkFrameTemplate",
    "UIExpandingButtonTemplate",
    "TalentRankDisplayTemplate",
    "ButtonWithDisableTooltipTemplate",
    "CurrencyDisplayTemplate",
];

const REPRESENTATIVE_GLOBAL_TABLES: &[&str] = &["ITEM_SEARCHBAR_LIST", "SQUARE_BUTTON_TEXCOORDS"];

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
fn find_toc_file_resolves_mainline_variant() {
    let resolved = find_toc_file(&ui_panel_templates_dir()).expect("UIPanelTemplates TOC resolves");
    assert_eq!(
        resolved.file_name().and_then(|name| name.to_str()),
        Some(toc_layout::expected_toc_name("Blizzard_UIPanelTemplates").as_str()),
        "find_toc_file resolves the active profile's UIPanelTemplates TOC: the Retail cache \
         ships only the `_Mainline.toc` variant, while the pinned PTR cache ships the bare \
         unified TOC"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_is_eager_with_one_dependency() {
    let toc = TocFile::from_file(&ui_panel_templates_toc()).expect("TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "No `## LoadOnDemand` directive → eagerly loaded on Game. The \
         template library must register its ~100 virtual templates \
         BEFORE any consumer tries to inherit from them"
    );

    let deps = toc.dependencies();
    assert_eq!(
        deps.len(),
        TOC_DEPENDENCIES.len(),
        "Mainline TOC must declare exactly {} hard dep. Got {}: {:?}",
        TOC_DEPENDENCIES.len(),
        deps.len(),
        deps
    );
    for expected in TOC_DEPENDENCIES {
        assert!(
            deps.iter().any(|d| d == expected),
            "TOC must declare `{expected}` — UIPanelTemplates only \
             leans on Blizzard_SharedXMLGame for in-world utility \
             functions (CreateFromMixins, ResizeLayoutFrame, \
             EventRegistry plumbing) since the templates are themselves \
             the lowest-level UI shared building blocks. Got: {deps:?}"
        );
    }

    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.saved_variables_per_character().is_empty());
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(toc.default_enabled());
}
}

post_wrath_ui_fixture! {
#[test]
fn allow_load_game_restricts_to_in_world() {
    let toc = TocFile::from_file(&ui_panel_templates_toc()).expect("TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: game` (lowercase) hits the `eq_ignore_ascii_case` \
         branch at toc.rs:308 → Game-only. The bag-search box, \
         CapProgressBar, currency-display widgets are all in-world \
         constructs"
    );
    for screen in GLUE_SCREENS {
        assert!(
            !toc.allows_screen(*screen),
            "Glue screen {screen:?} must be excluded — `AllowLoad: \
             game` matches only the Game variant via toc.rs:308"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn allow_load_game_type_mainline_is_not_restricted() {
    let toc = TocFile::from_file(&ui_panel_templates_toc()).expect("TOC parses");

    assert!(
        !toc.is_game_type_restricted(),
        "`## AllowLoadGameType: mainline` lists `mainline` which is \
         recognised as a non-restricting flavor at toc.rs:294-302 \
         (standard|mainline). Despite having a _Mainline suffix in the \
         filename, the gametype field is what gates loading at runtime"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_raw_bytes_pin_six_directives_and_six_body_files() {
    let raw = std::fs::read_to_string(ui_panel_templates_toc()).expect("TOC reads utf-8");

    let expected_lines: &[&str] = if toc_layout::classic_cache() {
        &[
            "## Title: Blizzard_UIPanelTemplates",
            "## Author: Blizzard Entertainment",
            "## DefaultState: enabled",
            "## Dependencies: Blizzard_SharedXMLGame",
            "## AllowLoad: game",
            "## AllowLoadGameType: classic",
            "Classic\\AutoCastTemplates.lua",
            "Classic\\AutoCastTemplates.xml",
            "Shared\\UIPanelSpellButtonFrame.lua",
            "Shared\\UIPanelSpellButtonFrame.xml",
            "Classic\\UIPanelTemplates.lua",
            "Classic\\UIPanelTemplates.xml",
        ]
    } else if toc_layout::unified_toc_tree() {
        &[
            "## Title: Blizzard_UIPanelTemplates",
            "## Dep: Blizzard_SharedXMLGame",
            "[Family]\\AutoCastTemplates.lua",
            "[Family]\\AutoCastTemplates.xml",
            "Shared\\UIPanelSpellButtonFrame.lua",
            "Shared\\UIPanelSpellButtonFrame.xml",
            "Shared\\UIPanelTemplatesShared.lua",
            "[Family]\\UIPanelTemplates.lua",
            "[Family]\\UIPanelTemplates.xml",
            "[Family]\\UIPanelTemplatesOverrides.xml",
            "[Game]\\UIPanelTemplatesOverrides.xml [AllowLoadGameType camelot]",
        ]
    } else {
        &[
            "## Title: Blizzard_UIPanelTemplates",
            "## Author: Blizzard Entertainment",
            "## DefaultState: enabled",
            "## Dependencies: Blizzard_SharedXMLGame",
            "## AllowLoad: game",
            "## AllowLoadGameType: mainline",
            "Mainline\\AutoCastTemplates.lua",
            "Mainline\\AutoCastTemplates.xml",
            "Shared\\UIPanelSpellButtonFrame.lua",
            "Shared\\UIPanelSpellButtonFrame.xml",
            "Mainline\\UIPanelTemplates.lua",
            "Mainline\\UIPanelTemplates.xml",
        ]
    };

    for line in expected_lines {
        assert!(
            raw.contains(line),
            "Raw TOC must pin `{line}` — body order is the lua/xml pairs \
             with the heaviest pair last: AutoCastTemplates → \
             UIPanelSpellButtonFrame (Shared/) → UIPanelTemplates. The \
             lua-before-xml ordering within each pair lets the XML's \
             `mixin=` attributes resolve against published mixin tables"
        );
    }

    assert!(!raw.contains("## LoadOnDemand"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## RequiredDep"));
    if toc_layout::unified_toc_tree() {
        assert!(
            raw.contains("[ExcludeLoadGameType classic, camelot]"),
            "The unified PTR TOC excludes the shared overrides XML from the classic and \
             camelot flavors"
        );
    } else {
        assert!(!raw.contains("## LoadFirst"));
        assert!(!raw.contains("## OptionalDeps"));
        assert!(!raw.contains("[Family]"));
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn appears_in_game_eager_discovery() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let found = addons
        .iter()
        .any(|(name, _)| name == "Blizzard_UIPanelTemplates");
    assert!(
        found,
        "Blizzard_UIPanelTemplates must appear in Game eager discovery \
         — non-LoD with AllowLoad:game. Many sibling addons declare it \
         as a hard dep: Blizzard_FrameXML, Blizzard_EditMode, \
         Blizzard_UIPanels_Game, Blizzard_GroupFinder, Blizzard_Transmog, \
         Blizzard_ChatFrame (Mists). Without this addon the entire UI \
         dependency graph collapses"
    );
}
}

#[test]
fn absent_from_glue_screens_eager_discovery() {
    for screen in GLUE_SCREENS {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), *screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_UIPanelTemplates");
        assert!(
            !found,
            "Blizzard_UIPanelTemplates must NOT appear on {screen:?} \
             — AllowLoad:game restricts to in-world via toc.rs:308, \
             checked at loader/mod.rs:527 BEFORE pool partitioning"
        );
    }
}

#[test]
fn dep_directories_exist_on_disk() {
    if cfg!(feature = "client-wrath") {
        let toc = crate::common::native_framexml_toc();
        for file in [
            "BasicControls.xml",
            "UIPanelTemplates.lua",
            "UIPanelTemplates.xml",
        ] {
            assert!(
                toc.files
                    .iter()
                    .any(|path| path == std::path::Path::new(file)),
                "native dependency absent from committed FrameXML order: {file}"
            );
            assert!(
                toc.addon_dir.join(file).is_file(),
                "native dependency missing on disk: {file}"
            );
        }
        return;
    }
    for dep in TOC_DEPENDENCIES {
        let dir = blizzard_ui_dir().join(dep);
        assert!(
            dir.is_dir(),
            "Hard-dep directory `{dep}` must exist on disk"
        );
        assert!(
            find_toc_file(&dir).is_some(),
            "{dep} must have a discoverable TOC"
        );
    }
}

prefork_full_ui_case! {
fn full_game_load_publishes_mixins(env: &WowLuaEnv) {

    for mixin in MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must be a global table after load. \
             UIPanelSpellButtonFrameMixin (Shared/UIPanelSpellButtonFrame.lua) \
             + AutoCastOverlayMixin (Mainline/AutoCastTemplates.lua) are \
             the focused-feature mixins. The rest live in \
             Mainline/UIPanelTemplates.lua and back the major shared \
             widgets: RoleCount (tank/healer/dps), Currency* family \
             (Display/Group/LayoutFrameIcon/HorizontalLayoutFrame all \
             chained via CreateFromMixins), UIExpandingButton (collapse \
             toggles), TalentRankDisplay (talent points), \
             ButtonWithDisable (motionScriptsWhileDisabled tooltip \
             support), AnimatedShine (the shine effect overlay)"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_publishes_free_functions(env: &WowLuaEnv) {

    for func in FREE_FUNCTIONS {
        let kind: String = env
            .eval(&format!("return type({func})"))
            .unwrap_or_else(|err| panic!("{func} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "{func} must be a global function. BagSearch_* are the \
             free OnHide/OnTextChanged/OnChar handlers wired by the XML \
             EditBox `<Scripts>` blocks for BagSearchBoxTemplate. \
             SquareButton_SetIcon swaps texcoords on the SquareButton \
             family via SQUARE_BUTTON_TEXCOORDS. CapProgressBar_* are \
             the public API for the cap-progress widget. \
             SetCheckButtonIsRadio swaps the check texture for a radio \
             texture. InlineHyperlinkFrame_* handle hover/click on \
             inline hyperlinks rendered inside text"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_publishes_global_tables(env: &WowLuaEnv) {

    for global in REPRESENTATIVE_GLOBAL_TABLES {
        let kind: String = env
            .eval(&format!("return type({global})"))
            .unwrap_or_else(|err| panic!("{global} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{global} must be a global table. ITEM_SEARCHBAR_LIST is \
             the searchbar-instance registry — siblings push their \
             EditBox onto it via `tinsert(ITEM_SEARCHBAR_LIST, self)` so \
             `BagItemSearchBox_OnTextChanged` can broadcast filter \
             changes across all bag UIs. SQUARE_BUTTON_TEXCOORDS maps \
             button-state names (UP/DOWN/DELETE/CHECK/etc.) to \
             4-element texcoord tables consumed by SquareButton_SetIcon"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_registers_representative_virtual_templates(env: &WowLuaEnv) {
    let _env = env;

    for template in REPRESENTATIVE_VIRTUAL_TEMPLATES {
        let entry = wow_ui_sim::xml::get_template(template);
        assert!(
            entry.is_some(),
            "{template} must be a registered virtual template. The \
             addon publishes ~100 virtual templates across 3 XML files \
             (1490+63+42 lines); this probe pins a representative \
             cross-section: search/menu/shine/bordered-button widgets, \
             role-count pair (RoleCountNoScripts is the no-OnLoad \
             chassis, RoleCount adds the mixin), basic frame chain \
             (BaseBasicFrame ← BasicFrame ← EtherealFrame ← \
             PortraitFrame), bar widgets (HorizontalBar, CapProgressBar, \
             ThinGoldEdge), Talent/CurrencyDisplay/UIExpandingButton/\
             ButtonWithDisableTooltip/InlineHyperlinkFrame/\
             TranslucentFrame/ShadowOverlaySmall/AnimatedShine"
        );
    }
}
}

prefork_full_ui_case! {
fn no_named_non_virtual_top_level_frames_published(env: &WowLuaEnv) {

    for global in &[
        "UIPanelTemplatesFrame",
        "AutoCastTemplatesFrame",
        "UIPanelSpellButtonFrame",
    ] {
        let exists: bool = env
            .eval(&format!("return _G[{global:?}] ~= nil"))
            .unwrap_or_else(|err| panic!("{global} probe failed: {err}"));
        assert!(
            !exists,
            "{global} must NOT exist — UIPanelTemplates publishes ONLY \
             virtual templates and mixins, no top-level non-virtual \
             named frames. Every consumer instantiates the templates \
             via inheritance"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_emits_no_addon_specific_errors(env: &WowLuaEnv) {

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let addon_specific: Vec<&String> = errors
        .iter()
        .filter(|e| e.contains("Blizzard_UIPanelTemplates/"))
        .collect();

    assert!(
        addon_specific.is_empty(),
        "Full game-screen load with UIPanelTemplates in dependency \
         order must emit zero UIPanelTemplates-body errors. The 6 body \
         files (3 lua + 3 xml) include the 1490-line UIPanelTemplates.xml \
         which registers ~100 virtual templates; all must register \
         cleanly without raising on the simulator's stub primitives. \
         Found: {addon_specific:?}"
    );
}
}
