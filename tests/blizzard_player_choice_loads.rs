use std::path::PathBuf;

use wow_ui_sim::loader::{discover_all_blizzard_addons, discover_blizzard_addons_for_screen};
use wow_ui_sim::loader::{find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn player_choice_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_PlayerChoice")
}

fn player_choice_toc() -> PathBuf {
    player_choice_dir().join("Blizzard_PlayerChoice.toc")
}

const PLAYER_CHOICE_TOC_FILES: &[&str] = &[
    "Blizzard_PlayerChoice_Bootstrap.lua",
    "Blizzard_PlayerChoiceToggleButton.lua",
    "Blizzard_PlayerChoiceToggleButton.xml",
    "Blizzard_PlayerChoiceOptionBase.lua",
    "Blizzard_PlayerChoiceOptionBase.xml",
    "Blizzard_PlayerChoiceNormalOptionTemplate.lua",
    "Blizzard_PlayerChoiceNormalOptionTemplate.xml",
    "Blizzard_PlayerChoicePowerChoiceTemplate.lua",
    "Blizzard_PlayerChoicePowerChoiceTemplate.xml",
    "Blizzard_PlayerChoiceTorghastOptionTemplate.lua",
    "Blizzard_PlayerChoiceTorghastOptionTemplate.xml",
    "Blizzard_PlayerChoiceCovenantChoiceOptionTemplate.lua",
    "Blizzard_PlayerChoiceCovenantChoiceOptionTemplate.xml",
    "Blizzard_PlayerChoiceCypherOptionTemplate.lua",
    "Blizzard_PlayerChoiceCypherOptionTemplate.xml",
    "Blizzard_PlayerChoiceGenericPowerChoiceOptionTemplate.lua",
    "Blizzard_PlayerChoiceGenericPowerChoiceOptionTemplate.xml",
    "Blizzard_PlayerChoice.lua",
    "Blizzard_PlayerChoice.xml",
    "Blizzard_PlayerChoiceTimer.lua",
    "Blizzard_PlayerChoiceTimer.xml",
];

const REQUIRED_DEPS: &[&str] = &[
    "Blizzard_Colors",
    "Blizzard_GameMenuEsc",
    "Blizzard_UIWidgets",
];

const PUBLIC_MIXINS: &[&str] = &[
    "PlayerChoiceFrameMixin",
    "PlayerChoiceTimeRemainingMixin",
    "PlayerChoiceToggleButtonMixin",
    "TorghastPlayerChoiceToggleButtonMixin",
    "CypherPlayerChoiceToggleButtonMixin",
    "GenericPlayerChoiceToggleButtonMixin",
    "PlayerChoiceRerollButtonMixin",
    "PlayerChoiceBaseOptionTemplateMixin",
    "PlayerChoiceBaseOptionAlignedSectionMixin",
    "PlayerChoiceBaseOptionTextTemplateMixin",
    "PlayerChoiceBaseOptionButtonFrameTemplateMixin",
    "PlayerChoiceBaseOptionButtonTemplateMixin",
    "PlayerChoiceBaseOptionButtonsContainerMixin",
    "PlayerChoiceBaseOptionCurrencyRewardMixin",
    "PlayerChoiceBaseOptionItemRewardMixin",
    "PlayerChoiceBaseOptionCurrencyContainerRewardMixin",
    "PlayerChoiceBaseOptionReputationRewardMixin",
    "PlayerChoiceBaseOptionRewardsMixin",
    "PlayerChoiceWidgetContainerMixin",
    "PlayerChoiceNormalOptionTemplateMixin",
    "PlayerChoicePowerChoiceTemplateMixin",
    "PlayerChoiceTorghastOptionTemplateMixin",
    "PlayerChoiceCovenantChoiceOptionTemplateMixin",
    "PlayerChoiceNormalOptionGridTemplateMixin",
    "PlayerChoiceCypherOptionTemplateMixin",
    "PlayerChoiceGenericPowerChoiceOptionTemplateMixin",
];

const PUBLIC_NAMED_FRAMES: &[&str] = &[
    "PlayerChoiceFrame",
    "PlayerChoiceTimeRemaining",
    "TorghastPlayerChoiceToggleButton",
    "CypherPlayerChoiceToggleButton",
    "GenericPlayerChoiceToggleButton",
];

const VIRTUAL_TEMPLATES_NOT_IN_GLOBALS: &[&str] = &[
    "PlayerChoiceBaseCenteredFrame",
    "PlayerChoiceBaseOptionAlignedSection",
    "PlayerChoiceBaseOptionButtonFrameTemplate",
    "PlayerChoiceBaseOptionButtonTemplate",
    "PlayerChoiceBaseOptionButtonsContainer",
    "PlayerChoiceBaseOptionCurrencyContainerRewardTemplate",
    "PlayerChoiceBaseOptionCurrencyRewardTemplate",
    "PlayerChoiceBaseOptionItemRewardTemplate",
    "PlayerChoiceBaseOptionReputationRewardTemplate",
    "PlayerChoiceBaseOptionRewardsTemplate",
    "PlayerChoiceBaseOptionTemplate",
    "PlayerChoiceBaseOptionTextTemplate",
    "PlayerChoiceBaseSmallerOptionButtonTemplate",
    "PlayerChoiceCovenantChoiceOptionTemplate",
    "PlayerChoiceCypherOptionTemplate",
    "PlayerChoiceGenericPowerChoiceOptionTemplate",
    "PlayerChoiceNormalOptionTemplate",
    "PlayerChoicePowerChoiceTemplate",
    "PlayerChoiceSmallerOptionButtonFrameTemplate",
    "PlayerChoiceToggleButtonTemplate",
    "PlayerChoiceTorghastOptionTemplate",
];

fn load_full_game_ui_with_player_choice() -> WowLuaEnv {
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

    load_addon(&env.loader_env(), &player_choice_toc())
        .expect("explicit load_addon for Blizzard_PlayerChoice succeeds");

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);

    env
}

#[test]
fn blizzard_player_choice_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&player_choice_dir()).expect("Blizzard_PlayerChoice TOC resolves");
    assert_eq!(
        resolved,
        player_choice_toc(),
        "Blizzard_PlayerChoice ships exactly one bare TOC — no `_Mainline.toc` variant. \
         The PlayerChoice surface (covenant pick, Torghast power picks, Tinkers / Cypher \
         choices, generic dialog choices) is reused across mainline and earlier flavors, \
         so the addon stays bare-named without flavor-specific TOC splitting"
    );

    let mainline = player_choice_dir().join("Blizzard_PlayerChoice_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {} — the bare TOC is the canonical entry point",
        mainline.display()
    );
}

#[test]
fn blizzard_player_choice_toc_declares_load_on_demand_with_split_dependency_keys() {
    let toc = TocFile::from_file(&player_choice_toc()).expect("Blizzard_PlayerChoice TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "TOC declares `## LoadOnDemand: 1` so `is_load_on_demand()` returns true — the \
         PlayerChoice UI is a heavy panel with 20 source files (~3500 lines of Lua/XML) \
         covering 6 distinct option-template variants (Normal / PowerChoice / Torghast / \
         Covenant / Cypher / GenericPowerChoice); lazy-loading defers the cost until the \
         server fires the START_PLAYER_CHOICE event and the engine triggers \
         LoadAddOn('Blizzard_PlayerChoice')"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(!toc.is_game_type_restricted());

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Default-Game-only `allows_screen` at src/toc.rs:311 returns true for \
         ScreenKind::Game when AllowLoad is omitted"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Omitted `## AllowLoad:` must NOT enable {screen:?} — PlayerChoice is an \
             in-world quest/scenario surface; glue screens cannot trigger it"
        );
    }

    assert_eq!(
        toc.dependencies(),
        REQUIRED_DEPS,
        "TOC ships both `## Dependencies: Blizzard_Colors` and `## RequiredDeps: \
         Blizzard_UIWidgets`. The parser merges complementary dependency-key \
         variants and returns both hard dependencies in canonical parser order"
    );

    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` — no soft sibling addons"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — PlayerChoice is a pure stateless mirror of \
         server-authoritative state pulled via C_PlayerChoice.GetCurrentPlayerChoiceInfo \
         each time a START_PLAYER_CHOICE event fires. Pending pick state, timer state, \
         and reroll counts are all server-driven and never persisted client-side"
    );
}

#[test]
fn blizzard_player_choice_toc_declares_metadata_in_raw_bytes() {
    let raw = std::fs::read_to_string(player_choice_toc())
        .expect("Blizzard_PlayerChoice TOC reads utf-8");
    assert!(
        raw.contains("## Title: Blizzard_PlayerChoice"),
        "TOC must declare `## Title: Blizzard_PlayerChoice` exactly — underscore-namespace \
         spelling matching the Blizzard_PingUI / Blizzard_PerksProgram pattern"
    );
    assert!(
        raw.contains("## LoadOnDemand: 1"),
        "TOC must declare `## LoadOnDemand: 1` exactly — the canonical retail spelling \
         for explicit lazy loading"
    );
    assert!(
        raw.contains("## RequiredDeps: Blizzard_UIWidgets"),
        "TOC must declare `## RequiredDeps: Blizzard_UIWidgets` exactly — it ships \
         alongside the complementary `## Dependencies: Blizzard_Colors` entry, and \
         both lists are part of the addon's hard dependency set"
    );
    assert!(
        raw.contains("## Dependencies: Blizzard_Colors, Blizzard_GameMenuEsc"),
        "TOC must declare Blizzard_Colors and Blizzard_GameMenuEsc in its Dependencies list."
    );
    assert!(
        !raw.contains("## AllowLoad:"),
        "TOC must NOT declare `## AllowLoad:` — Game-only is the default behavior when \
         the key is omitted"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT declare any `## SavedVariables*` keys — pure stateless mirror"
    );
    assert!(
        !raw.contains("## OptionalDeps"),
        "TOC must NOT declare any `## OptionalDeps:` — zero soft siblings"
    );
    assert!(
        !raw.contains("## UseSecureEnvironment"),
        "TOC must NOT declare `## UseSecureEnvironment:` — PlayerChoice is a \
         display-only quest/scenario UI; the server-authoritative pick dispatch \
         (C_PlayerChoice.SendPlayerChoiceResponse) is gated server-side, not via \
         secureenv-fenv addon-side taint protection"
    );
    assert!(
        !raw.contains("## Author"),
        "TOC must NOT declare `## Author:` — minimal-metadata pattern matching \
         Blizzard_PerksProgram"
    );
    assert!(
        !raw.contains("## Version"),
        "TOC must NOT declare `## Version:` — unversioned, matching the bare-bones \
         metadata profile"
    );
}

#[test]
fn blizzard_player_choice_toc_lists_bootstrap_then_twenty_files_in_canonical_order() {
    let toc = TocFile::from_file(&player_choice_toc()).expect("Blizzard_PlayerChoice TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, PLAYER_CHOICE_TOC_FILES,
        "Retail 12.1.0.69497 lists its bootstrap first, then twenty PlayerChoice source files \
         in the established paired Lua-then-XML module order"
    );
}

#[test]
fn blizzard_player_choice_does_not_appear_in_eager_discovery_for_any_screen() {
    let ui = blizzard_ui_dir();

    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_PlayerChoice");
        assert!(
            !found,
            "Blizzard_PlayerChoice must NOT appear in eager discovery for {screen:?} — \
             LoadOnDemand: 1 keeps the addon in the lod_pool, not in the eager-discovery \
             set; the Game-screen pass would only auto-load it if LoadOnDemand was absent"
        );
    }
}

#[test]
fn blizzard_player_choice_appears_in_full_addon_inventory() {
    let inventory = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = inventory
        .iter()
        .any(|(name, _)| name == "Blizzard_PlayerChoice");
    assert!(
        found,
        "Blizzard_PlayerChoice must appear in `discover_all_blizzard_addons` — the full \
         inventory is a structural listing of every parseable TOC including LoD addons"
    );
}

#[test]
fn blizzard_player_choice_loads_without_addon_specific_lua_errors() {
    let env = load_full_game_ui_with_player_choice();

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_PlayerChoice")
                || message.contains("PlayerChoice")
                || message.contains("PlayerChoiceFrame")
                || message.contains("PlayerChoiceToggle")
                || message.contains("PlayerChoiceTimeRemaining")
        })
        .filter(|message| {
            let touches_three_d_model_gap = message.contains("ScriptAnimatedModelSceneTemplate")
                || message.contains("BorderLayerModelScene")
                || message.contains("ModelScene");
            !touches_three_d_model_gap
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_PlayerChoice emitted addon-specific Lua errors during load (excluding \
         the documented 3D ModelScene permanent gap — CLAUDE.md flags Model/ModelScene/ \
         PlayerModel/DressUpModel as intentional ~38-stub permanent gaps; the \
         BorderLayerModelScene child of PlayerChoiceFrame inherits \
         ScriptAnimatedModelSceneTemplate and the `:AddEffect` / actor-based methods are \
         all permanent stubs):\n  {}",
        load_errors.join("\n  ")
    );
}

#[test]
fn blizzard_player_choice_is_addon_loaded_after_explicit_load() {
    let env = load_full_game_ui_with_player_choice();

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_PlayerChoice')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_PlayerChoice') must return true after \
         explicit load_addon — the addon is LoadOnDemand so only the explicit load path \
         makes IsAddOnLoaded report true"
    );
}

#[test]
fn blizzard_player_choice_publishes_twenty_six_mixin_tables() {
    let env = load_full_game_ui_with_player_choice();

    for mixin in PUBLIC_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{mixin})"))
            .unwrap_or_else(|err| panic!("type(_G.{mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table — Blizzard_PlayerChoice declares 26 \
             mixins across 10 Lua files. Current retail replaces the former \
             Covenant preview-button mixin with \
             PlayerChoiceNormalOptionGridTemplateMixin; derived option mixins \
             remain real tables for CreateFromMixins inheritance"
        );
    }

    assert_eq!(
        PUBLIC_MIXINS.len(),
        26,
        "PUBLIC_MIXINS must contain exactly 26 entries — pinned so vendor TAG bumps that \
         add or remove a mixin surface here as a deliberate test update"
    );
}

#[test]
fn blizzard_player_choice_creates_named_non_virtual_frames() {
    let env = load_full_game_ui_with_player_choice();

    for frame in PUBLIC_NAMED_FRAMES {
        let kind: String = env
            .eval(&format!("return type(_G.{frame})"))
            .unwrap_or_else(|err| panic!("type(_G.{frame}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{frame} must publish as a frame userdata (FrameRef reports as `'table'` \
             via the custom __type metamethod). Blizzard_PlayerChoice ships 5 named \
             non-virtual top-level frames: PlayerChoiceFrame (the panel root, \
             frameStrata=HIGH, parent=UIParent, toplevel=true, inherits \
             HorizontalLayoutFrame, hidden=true at load — Blizzard_PlayerChoice.xml:4); \
             PlayerChoiceTimeRemaining (the round-timer countdown frame, \
             frameStrata=HIGH, parent=UIParent, toplevel=true, hidden=true — \
             Blizzard_PlayerChoiceTimer.xml); and the 3 toggle buttons \
             (TorghastPlayerChoiceToggleButton / CypherPlayerChoiceToggleButton / \
             GenericPlayerChoiceToggleButton) which all inherit from \
             PlayerChoiceToggleButtonTemplate (virtual=true, parent=UIParent, \
             frameStrata=DIALOG, hidden=true) so they ALL land at parent=UIParent via \
             template inheritance, NOT via per-frame parent= attributes"
        );

        let name: String = env
            .eval(&format!("return _G.{frame}:GetName()"))
            .unwrap_or_else(|err| panic!("_G.{frame}:GetName() probe failed: {err}"));
        assert_eq!(
            name, *frame,
            "_G.{frame}:GetName() must round-trip the same name — the XML-driven name \
             registration writes `frame.name = \"{frame}\"` and registers under the same \
             key in `_G`"
        );
    }
}

#[test]
fn blizzard_player_choice_does_not_leak_virtual_templates_to_globals() {
    let env = load_full_game_ui_with_player_choice();

    for template in VIRTUAL_TEMPLATES_NOT_IN_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G.{template})"))
            .unwrap_or_else(|err| panic!("type(_G.{template}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{template} must be nil — Blizzard_PlayerChoice ships 21 virtual \
             templates that live in the template registry, NOT in `_G`. Leaking would \
             let consumer addons mutate the template definition and break every existing \
             instance. The 21 templates split into: 1 layout helper \
             (PlayerChoiceBaseCenteredFrame), 11 base-option building blocks (the \
             AlignedSection / ButtonFrame / Button / ButtonsContainer / 4 reward \
             variants / RewardsContainer / TextTemplate + the BaseOptionTemplate root + \
             the SmallerOption variants), 6 option variants (Normal / PowerChoice / \
             Torghast / Covenant / Cypher / GenericPowerChoice), 1 toggle-button base \
             (PlayerChoiceToggleButtonTemplate), and 1 smaller-button-frame variant \
             (PlayerChoiceSmallerOptionButtonFrameTemplate). The 22nd virtual template \
             in the addon — `<Font name=\"PlayerChoiceTextFont\" virtual=\"true\"/>` — is \
             a Font, NOT a Frame template, and is excluded from this list because Fonts \
             register in the simulator's font registry under the same name and may also \
             land in `_G` depending on font-registration behavior; pinning it here would \
             couple this test to font-loader internals"
        );
    }

    assert_eq!(
        VIRTUAL_TEMPLATES_NOT_IN_GLOBALS.len(),
        21,
        "VIRTUAL_TEMPLATES_NOT_IN_GLOBALS must contain exactly 21 entries — pinned so \
         vendor TAG bumps that add or remove a Frame template surface here. \
         PlayerChoiceTextFont (a Font, not a Frame) is intentionally excluded"
    );
}

#[test]
fn blizzard_player_choice_frame_renders_as_hidden_after_load() {
    let env = load_full_game_ui_with_player_choice();

    let visible: bool = env
        .eval("return PlayerChoiceFrame:IsShown()")
        .expect("PlayerChoiceFrame:IsShown() probe succeeds");
    assert!(
        !visible,
        "PlayerChoiceFrame must be hidden at load — `<Frame ... hidden=\"true\">` in \
         Blizzard_PlayerChoice.xml:4 stamps the panel as hidden. The panel only shows \
         when the server fires START_PLAYER_CHOICE and \
         PlayerChoiceFrameMixin:OnEvent walks through the SetupFrame path"
    );

    let strata: String = env
        .eval("return PlayerChoiceFrame:GetFrameStrata()")
        .expect("PlayerChoiceFrame:GetFrameStrata() probe succeeds");
    assert_eq!(
        strata, "HIGH",
        "PlayerChoiceFrame:GetFrameStrata() must return `HIGH` — `frameStrata=\"HIGH\"` \
         in the XML. UNUSUAL choice: most modal-style player-facing panels run at DIALOG \
         strata; HIGH places PlayerChoice ABOVE general action UI but BELOW dialogs and \
         tooltips, which means a tooltip can render over a PlayerChoice option button"
    );
}

#[test]
fn blizzard_player_choice_uiwidgets_dep_loaded_via_eager_discovery() {
    let env = load_full_game_ui_with_player_choice();

    let widgets_loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_UIWidgets')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        widgets_loaded,
        "Blizzard_UIWidgets must be loaded — although the simulator's `dependencies()` \
         parser drops `RequiredDeps: Blizzard_UIWidgets` (it returns only \
         `Blizzard_Colors` from the higher-priority `Dependencies:` key), \
         Blizzard_UIWidgets is independently auto-discovered via its own \
         `## DefaultState: enabled` flag and loaded eagerly during the Game-screen sweep. \
         By the time the explicit load_addon('Blizzard_PlayerChoice') call fires after \
         the eager pass, UIWidgets globals (UIWidgetManager, UIWidgetTemplateBase, etc.) \
         are already published. PlayerChoiceWidgetContainerMixin in OptionBase.lua:775 \
         calls `UIWidgetManager:RegisterWidgetContainer(...)` — works because UIWidgets \
         loaded eagerly, NOT because the dep declaration in the TOC was honored"
    );

    let colors_loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_Colors')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        colors_loaded,
        "Blizzard_Colors must be loaded — the `## Dependencies: Blizzard_Colors` key is \
         the one the simulator's `dependencies()` parser actually reads. ColorManager \
         globals (BLACK_FONT_COLOR / NORMAL_FONT_COLOR / etc.) are referenced by \
         PlayerChoice XML `<Color color=\"BLACK_FONT_COLOR\"/>` directives \
         (Blizzard_PlayerChoice.xml:38) at parse time, so Colors must be loaded BEFORE \
         PlayerChoice or the color name resolution would fail"
    );
}
