use std::path::PathBuf;

use wow_ui_sim::loader::{discover_all_blizzard_addons, discover_blizzard_addons_for_screen};
use wow_ui_sim::loader::{find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
#[cfg(not(feature = "client-retail"))]
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn prematch_ui_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_PlunderstormPrematchUI")
}

fn prematch_ui_toc() -> PathBuf {
    prematch_ui_dir().join("Blizzard_PlunderstormPrematchUI.toc")
}

const PREMATCH_UI_TOC_FILES: &[&str] = &[
    "Blizzard_PlunderstormPrematchUI.lua",
    "Blizzard_PlunderstormPrematchUI.xml",
];

const PUBLIC_MIXINS: &[&str] = &[
    "PrematchHeaderMixin",
    "PrematchHeaderBaseButtonMixin",
    "HeaderPlunderstoreButtonMixin",
    "HeaderCustomizeButtonMixin",
    "TrainingLobbyQueueSelectButtonMixin",
    "PlunderstormDropMapButtonMixin",
    "TrainingLobbyQueueMixin",
    "StartQueueButtonMixin",
];

const PUBLIC_NAMED_FRAMES: &[&str] = &["PrematchHeaderFrame"];

const VIRTUAL_TEMPLATES_NOT_IN_GLOBALS: &[&str] = &[
    "TrainingLobbyQueueFrameTemplate",
    "PrematchHeaderButtonTemplate",
];

fn load_plunderstorm_prematch_ui_after_fork(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &prematch_ui_toc())
        .expect("explicit load_addon for Blizzard_PlunderstormPrematchUI succeeds");
}

#[cfg(not(feature = "client-retail"))]
fn load_full_game_ui_with_prematch_ui() -> WowLuaEnv {
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

    load_addon(&env.loader_env(), &prematch_ui_toc())
        .expect("explicit load_addon for Blizzard_PlunderstormPrematchUI succeeds");

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);

    env
}

#[test]
fn blizzard_plunderstorm_prematch_ui_find_toc_resolves_bare_variant() {
    let resolved =
        find_toc_file(&prematch_ui_dir()).expect("Blizzard_PlunderstormPrematchUI TOC resolves");
    assert_eq!(
        resolved,
        prematch_ui_toc(),
        "Blizzard_PlunderstormPrematchUI ships exactly one bare TOC — no `_Mainline.toc` \
         variant. The prematch header is a Plunderstorm-game-type-only feature (the small \
         top-of-screen header bar with the Plunderstore / customize / queue-select / drop- \
         map buttons that shows during the Plunderstorm pre-match lobby phase) so no \
         flavor-split is meaningful — the entire addon is excluded from non-Plunderstorm \
         flavors via `## AllowLoadGameType: plunderstorm`"
    );

    let mainline = prematch_ui_dir().join("Blizzard_PlunderstormPrematchUI_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {} — the bare TOC is the canonical entry point",
        mainline.display()
    );
}

#[test]
fn blizzard_plunderstorm_prematch_ui_toc_declares_eager_plunderstorm_only() {
    let toc =
        TocFile::from_file(&prematch_ui_toc()).expect("Blizzard_PlunderstormPrematchUI TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "TOC must NOT declare `## LoadOnDemand:` — eager-load on Plunderstorm flavor so \
         the prematch header is wired up before the player drops into the lobby"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());

    assert!(
        toc.is_game_type_restricted(),
        "TOC declares `## AllowLoadGameType: plunderstorm` — `is_game_type_restricted` at \
         src/toc.rs:294-302 returns TRUE because `plunderstorm` is neither `mainline` nor \
         `standard`. The simulator runs as Mainline by default so this addon is filtered \
         out from eager-discovery on every screen — explicit `load_addon` is the only \
         entry point for testing the load contract"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: Game` must enable Game screen — the prematch header is in-world UI"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "`## AllowLoad: Game` must NOT enable {screen:?} — the Plunderstorm prematch \
             header is bound to in-world game state (C_WoWLabsMatchmaking party state, \
             C_WowLabsDataManager match phase); glue screens have no match context"
        );
    }

    assert!(
        toc.dependencies().is_empty(),
        "Zero `## Dependencies:` — leaf addon. The prematch header consumes \
         PortraitFrameTemplate / QueueTypeSettingsFrameTemplate / SharedButtonSmallTemplate \
         / GameFontHighlight which are part of the always-loaded SharedXML core, plus \
         AccountStoreUtil / GameTooltip / EventRegistry / FrameUtil which are global \
         FrameXML utilities"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` — no soft sibling addons"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — pure stateless mirror; match phase is server-driven via \
         WOW_LABS_MATCH_STATE_UPDATED events; superTracking state is global"
    );
}

#[test]
fn blizzard_plunderstorm_prematch_ui_toc_declares_metadata_in_raw_bytes() {
    let raw = std::fs::read_to_string(prematch_ui_toc())
        .expect("Blizzard_PlunderstormPrematchUI TOC reads utf-8");
    assert!(
        raw.contains("## Title: Blizzard Plunderstorm Prematch UI"),
        "TOC must declare `## Title: Blizzard Plunderstorm Prematch UI` exactly — \
         space-and-prose form"
    );
    assert!(
        raw.contains("## Author: Blizzard Entertainment"),
        "TOC must declare `## Author: Blizzard Entertainment` exactly"
    );
    assert!(
        raw.contains("## DefaultState: enabled"),
        "TOC must declare `## DefaultState: enabled` exactly — the prematch header is \
         foundational Plunderstorm UI; documenting `enabled` makes it clear that disabling \
         would remove the prematch lobby header entirely"
    );
    assert!(
        raw.contains("## AllowLoadGameType: plunderstorm"),
        "TOC must declare `## AllowLoadGameType: plunderstorm` exactly — single-token \
         Plunderstorm-only gametype lock; routes through `is_game_type_restricted` at \
         src/toc.rs:294-302 which splits on `,` and returns TRUE (restricted) when no \
         token matches `mainline` or `standard`"
    );
    assert!(
        raw.contains("## AllowLoad: Game"),
        "TOC must declare `## AllowLoad: Game` exactly — in-world UI only, not glue"
    );
    assert!(
        !raw.contains("## LoadOnDemand"),
        "TOC must NOT declare `## LoadOnDemand:` — eager-load on Plunderstorm flavor"
    );
    assert!(
        !raw.contains("## Dependencies"),
        "TOC must NOT declare `## Dependencies:` — leaf addon"
    );
    assert!(
        !raw.contains("## RequiredDep"),
        "TOC must NOT declare `## RequiredDep:` or `## RequiredDeps:`"
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
        "TOC must NOT declare `## UseSecureEnvironment:` — display-only header bar"
    );
    assert!(
        !raw.contains("## Version"),
        "TOC must NOT declare `## Version:` — unversioned"
    );
}

#[test]
fn blizzard_plunderstorm_prematch_ui_toc_lists_two_files_in_canonical_order() {
    let toc =
        TocFile::from_file(&prematch_ui_toc()).expect("Blizzard_PlunderstormPrematchUI TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, PREMATCH_UI_TOC_FILES,
        "TOC body must list exactly 2 files in canonical Lua-then-XML pair order: \
         Blizzard_PlunderstormPrematchUI.lua FIRST (declares the 8 mixins so the XML's \
         `mixin=\"...\"` attributes resolve at parse time), then \
         Blizzard_PlunderstormPrematchUI.xml SECOND (materializes 2 virtual templates + \
         the named PrematchHeaderFrame)"
    );
}

#[test]
fn blizzard_plunderstorm_prematch_ui_does_not_appear_in_eager_discovery_for_any_screen() {
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
            .any(|(name, _)| name == "Blizzard_PlunderstormPrematchUI");
        assert!(
            !found,
            "Blizzard_PlunderstormPrematchUI must NOT appear in eager discovery for \
             {screen:?} — the simulator runs as Mainline by default and \
             `discover_blizzard_addon_toc_pools_for_screen` (src/loader/mod.rs:527) filters \
             out addons whose `is_game_type_restricted` returns true. \
             AllowLoadGameType: plunderstorm restricts the addon to Plunderstorm flavor; \
             on Mainline, ALL screens skip it"
        );
    }
}

#[test]
fn blizzard_plunderstorm_prematch_ui_appears_in_full_addon_inventory() {
    let inventory = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = inventory
        .iter()
        .any(|(name, _)| name == "Blizzard_PlunderstormPrematchUI");
    assert!(
        found,
        "Blizzard_PlunderstormPrematchUI must appear in `discover_all_blizzard_addons` — \
         the full inventory at src/loader/mod.rs:309-343 ignores screen / game-type / \
         LoadOnDemand filters and lists every parseable Blizzard_* TOC, so \
         flavor-restricted addons are still visible to inventory tools"
    );
}

prefork_full_ui_case! {
    fn blizzard_plunderstorm_prematch_ui_is_addon_loaded_after_explicit_load(env: &WowLuaEnv) {
    load_plunderstorm_prematch_ui_after_fork(env);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_PlunderstormPrematchUI')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_PlunderstormPrematchUI') must return true after \
         explicit load_addon — `load_addon` does NOT enforce game-type filtering (no \
         `is_game_type_restricted` check in src/loader/addon.rs), so explicit loads \
         succeed even on Mainline. This is intentional: testing the load contract for \
         Plunderstorm-restricted addons requires the explicit-load path"
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn blizzard_plunderstorm_prematch_ui_is_addon_loaded_after_explicit_load() {
    let env = load_full_game_ui_with_prematch_ui();
    blizzard_plunderstorm_prematch_ui_is_addon_loaded_after_explicit_load::run(&env);
}

prefork_full_ui_case! {
    fn blizzard_plunderstorm_prematch_ui_publishes_eight_mixins(env: &WowLuaEnv) {
    load_plunderstorm_prematch_ui_after_fork(env);

    for mixin in PUBLIC_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{mixin})"))
            .unwrap_or_else(|err| panic!("type(_G.{mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table — the 8 mixins back the prematch header's \
             behavior surface: PrematchHeaderMixin (the panel-root with OnLoad calling \
             self:Show() + RegisterFrameForEvents(WOW_LABS_MATCH_STATE_UPDATED), OnShow / \
             OnHide registering EventRegistry callbacks for PlunderstormCountdown.\
             TimerFinished, UpdateShown calling SetShown(IsInPrematch())); \
             PrematchHeaderBaseButtonMixin (the shared base for the 4 header buttons — \
             OnLoad calling UpdateTextures, OnShow/OnHide registering optional \
             selectedStateEvent / alternateSelectedStateEvent / selectedStateFrameEvent \
             callbacks, UpdateTextures formatting `plunderstorm-menu-{{kit}}` / \
             `plunderstorm-menu-{{kit}}-selected` atlas names); HeaderPlunderstoreButtonMixin \
             (Plunderstore button OnClick calling AccountStoreUtil.ToggleAccountStore, \
             ShouldShowSelectedState querying AccountStoreFrame:IsShown()); \
             HeaderCustomizeButtonMixin (Customize button OnClick setting a hardcoded \
             UiMapPoint at coords 2257, 0.8846, 0.7777, 10.64 for Da'kash's location in \
             Brew Bay via C_Map.SetUserWaypoint + C_SuperTrack.SetSuperTrackedUserWaypoint); \
             TrainingLobbyQueueSelectButtonMixin (toggles QueueFrame visibility — only \
             shown in TrainingGameMode); PlunderstormDropMapButtonMixin (toggles WorldMap — \
             hidden in TrainingGameMode and gated by C_GameRules.IsGameRuleActive(\
             Enum.GameRule.PlunderstormAreaSelection)); TrainingLobbyQueueMixin (the \
             modal queue dialog — uses QueueTypeSettingsFrameMixin's OnLoad/OnShow/OnHide \
             plus custom anchoring and TrainingLobbyQueue.ShownState event triggering); \
             StartQueueButtonMixin (the bottom Start Queue button — text varies based on \
             party leader, OnClick calls C_WoWLabsMatchmaking.SetAutoQueueOnLogout + \
             ForceLogout to re-enter queue)"
        );
    }
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn blizzard_plunderstorm_prematch_ui_publishes_eight_mixins() {
    let env = load_full_game_ui_with_prematch_ui();
    blizzard_plunderstorm_prematch_ui_publishes_eight_mixins::run(&env);
}

prefork_full_ui_case! {
    fn blizzard_plunderstorm_prematch_ui_creates_named_non_virtual_frame(env: &WowLuaEnv) {
    load_plunderstorm_prematch_ui_after_fork(env);

    for frame in PUBLIC_NAMED_FRAMES {
        let kind: String = env
            .eval(&format!("return type(_G.{frame})"))
            .unwrap_or_else(|err| panic!("type(_G.{frame}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{frame} must publish as a frame userdata — PrematchHeaderFrame is the \
             single panel-root (mixin=PrematchHeaderMixin, frameStrata=DIALOG, \
             setAllPoints=true, parent=UIParent, hidden=true initially but OnLoad calls \
             self:Show() so OnShow fires, anchored TOP x=0 y=-20). Hosts the \
             plunderstorm-top-menu-frame background atlas + 4 header buttons \
             (PlunderstoreButton / CustomizeButton / QueueSelect / DropMapButton) + the \
             nested TrainingLobbyQueueFrameTemplate-inheriting QueueFrame"
        );

        let name: String = env
            .eval(&format!("return _G.{frame}:GetName()"))
            .unwrap_or_else(|err| panic!("_G.{frame}:GetName() probe failed: {err}"));
        assert_eq!(
            name, *frame,
            "_G.{frame}:GetName() must round-trip the same name"
        );
    }
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn blizzard_plunderstorm_prematch_ui_creates_named_non_virtual_frame() {
    let env = load_full_game_ui_with_prematch_ui();
    blizzard_plunderstorm_prematch_ui_creates_named_non_virtual_frame::run(&env);
}

prefork_full_ui_case! {
    fn blizzard_plunderstorm_prematch_ui_does_not_leak_virtual_templates_to_globals(env: &WowLuaEnv) {
    load_plunderstorm_prematch_ui_after_fork(env);

    for template in VIRTUAL_TEMPLATES_NOT_IN_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G.{template})"))
            .unwrap_or_else(|err| panic!("type(_G.{template}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{template} must be nil — virtual templates (`virtual=\"true\"` on the XML \
             element) live in the template registry only, not in globals. \
             TrainingLobbyQueueFrameTemplate (PortraitFrameTemplate + \
             QueueTypeSettingsFrameTemplate hybrid with mixin=TrainingLobbyQueueMixin, \
             instantiated as PrematchHeaderFrame.QueueFrame); \
             PrematchHeaderButtonTemplate (the shared button base with mixin=\
             PrematchHeaderBaseButtonMixin used by all 4 header buttons via \
             `inherits=\"PrematchHeaderButtonTemplate\"`)"
        );
    }
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn blizzard_plunderstorm_prematch_ui_does_not_leak_virtual_templates_to_globals() {
    let env = load_full_game_ui_with_prematch_ui();
    blizzard_plunderstorm_prematch_ui_does_not_leak_virtual_templates_to_globals::run(&env);
}

prefork_full_ui_case! {
    fn blizzard_plunderstorm_prematch_ui_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_plunderstorm_prematch_ui_after_fork(env);

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_PlunderstormPrematchUI")
                || message.contains("PrematchHeader")
                || message.contains("HeaderPlunderstore")
                || message.contains("HeaderCustomize")
                || message.contains("TrainingLobbyQueue")
                || message.contains("PlunderstormDropMapButton")
                || message.contains("StartQueueButton")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_PlunderstormPrematchUI emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn blizzard_plunderstorm_prematch_ui_loads_without_addon_specific_lua_errors() {
    let env = load_full_game_ui_with_prematch_ui();
    blizzard_plunderstorm_prematch_ui_loads_without_addon_specific_lua_errors::run(&env);
}
