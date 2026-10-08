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

fn plunderstorm_basics_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_PlunderstormBasics")
}

fn plunderstorm_basics_toc() -> PathBuf {
    plunderstorm_basics_dir().join("Blizzard_PlunderstormBasics.toc")
}

const PLUNDERSTORM_BASICS_TOC_FILES: &[&str] = &[
    "Blizzard_PlunderstormBasics.lua",
    "Blizzard_PlunderstormBasics.xml",
];

const PUBLIC_MIXINS: &[&str] = &[
    "PlunderstormAccountStoreToggleMixin",
    "PlunderstormBasicsContainerFrameMixin",
];

const PUBLIC_NAMED_FRAMES: &[&str] = &["PlunderstormBasicsContainerFrame"];

const VIRTUAL_TEMPLATES_NOT_IN_GLOBALS: &[&str] = &["PlunderstormAccountStoreToggleTemplate"];

fn load_plunderstorm_basics_after_fork(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &plunderstorm_basics_toc())
        .expect("explicit load_addon for Blizzard_PlunderstormBasics succeeds");
}

#[cfg(not(feature = "client-retail"))]
fn load_full_game_ui_with_plunderstorm_basics() -> WowLuaEnv {
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

    load_addon(&env.loader_env(), &plunderstorm_basics_toc())
        .expect("explicit load_addon for Blizzard_PlunderstormBasics succeeds");

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);

    env
}

#[test]
fn blizzard_plunderstorm_basics_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&plunderstorm_basics_dir())
        .expect("Blizzard_PlunderstormBasics TOC resolves");
    assert_eq!(
        resolved,
        plunderstorm_basics_toc(),
        "Blizzard_PlunderstormBasics ships exactly one bare TOC — no `_Mainline.toc` variant. \
         The Plunderstorm-basics panel (the WoWLabs game-mode tutorial card with the swords \
         atlas at the top, `Game Basics` title, and the Plunder lifetime-currency display + \
         Plunderstore button at the bottom) is dual-flavor by design via `## AllowLoad: Both` \
         so a single bare TOC drives both the in-world UI and the glue-screen UI"
    );

    let mainline = plunderstorm_basics_dir().join("Blizzard_PlunderstormBasics_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {} — the bare TOC is the canonical entry point",
        mainline.display()
    );
}

#[test]
fn blizzard_plunderstorm_basics_toc_declares_load_on_demand_with_allow_load_both() {
    let toc = TocFile::from_file(&plunderstorm_basics_toc())
        .expect("Blizzard_PlunderstormBasics TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "TOC declares `## LoadOnDemand: 1` so `is_load_on_demand()` returns true — the \
         basics card is shown on demand when the player opens the Plunderstorm queue \
         dialog or returns to the lobby; lazy-loading defers the cost until the engine \
         fires `LoadAddOn('Blizzard_PlunderstormBasics')` from the WoWLabs lobby flow"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(!toc.is_game_type_restricted());

    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            toc.allows_screen(screen),
            "`## AllowLoad: Both` must enable {screen:?} — the basics card is shown both \
             in the in-world Plunderstorm hub AND on the glue-screen WoWLabs queue dialog \
             (the addon's OnShow branches on `C_Glue.IsOnGlueScreen()` to add the \
             Plunderstore button only on glue). `allows_screen` at src/toc.rs:306-313 \
             returns true for ALL screen kinds when AllowLoad value is `both` \
             (case-insensitive)"
        );
    }

    assert!(
        toc.dependencies().is_empty(),
        "Zero `## Dependencies:` — leaf addon. The basics card consumes \
         BigGoldRedThreeSliceButtonTemplate / NineSlicePanelTemplate / VerticalLayoutFrame \
         / SystemFont_Huge2 / Game16Font / WHITE_FONT_COLOR / NORMAL_FONT_COLOR — all part \
         of the always-loaded SharedXML core. AccountStoreUtil is loaded lazily via \
         `C_AddOns.LoadAddOn('Blizzard_AccountStore')` inside UpdatePlunderAmount when \
         needed, NOT declared as a hard dep so the basics card loads quickly without \
         pulling in the full account-store panel"
    );

    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` — no soft sibling addons"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — pure stateless mirror; lifetime plunder is fetched from \
         C_CurrencyInfo.GetCurrencyInfo(2922) (in-world) or \
         C_WoWLabsMatchmaking.GetCurrentParty (glue-screen), both server-authoritative"
    );
}

#[test]
fn blizzard_plunderstorm_basics_toc_declares_metadata_in_raw_bytes() {
    let raw = std::fs::read_to_string(plunderstorm_basics_toc())
        .expect("Blizzard_PlunderstormBasics TOC reads utf-8");
    assert!(
        raw.contains("## Title: Blizzard Plunderstorm Basics"),
        "TOC must declare `## Title: Blizzard Plunderstorm Basics` exactly — \
         space-and-prose human-readable label"
    );
    assert!(
        raw.contains("## AllowLoad: Both"),
        "TOC must declare `## AllowLoad: Both` exactly — the dual-flavor flag that opens \
         the addon to all 4 ScreenKind values"
    );
    assert!(
        raw.contains("## Author: Blizzard Entertainment"),
        "TOC must declare `## Author: Blizzard Entertainment` exactly — the standard \
         Blizzard-shipped author line"
    );
    assert!(
        raw.contains("## LoadOnDemand: 1"),
        "TOC must declare `## LoadOnDemand: 1` exactly"
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
        "TOC must NOT declare `## UseSecureEnvironment:` — display-only tutorial card"
    );
    assert!(
        !raw.contains("## Version"),
        "TOC must NOT declare `## Version:` — unversioned"
    );
    assert!(
        !raw.contains("## AllowLoadGameType"),
        "TOC must NOT declare `## AllowLoadGameType:` — the addon ships on every flavor \
         that has Plunderstorm; the AllowLoad: Both metadata is the screen-gate, not the \
         flavor-gate"
    );
}

#[test]
fn blizzard_plunderstorm_basics_toc_lists_two_files_in_canonical_order() {
    let toc = TocFile::from_file(&plunderstorm_basics_toc())
        .expect("Blizzard_PlunderstormBasics TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, PLUNDERSTORM_BASICS_TOC_FILES,
        "TOC body must list exactly 2 files in canonical Lua-then-XML pair order: \
         Blizzard_PlunderstormBasics.lua FIRST (declares the 2 mixins \
         PlunderstormAccountStoreToggleMixin / PlunderstormBasicsContainerFrameMixin so \
         the XML's `mixin=\"...\"` attributes resolve at parse time), then \
         Blizzard_PlunderstormBasics.xml SECOND (materializes the virtual button template \
         + the named container frame)"
    );
}

#[test]
fn blizzard_plunderstorm_basics_does_not_appear_in_eager_discovery_for_any_screen() {
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
            .any(|(name, _)| name == "Blizzard_PlunderstormBasics");
        assert!(
            !found,
            "Blizzard_PlunderstormBasics must NOT appear in eager discovery for \
             {screen:?} — LoadOnDemand: 1 keeps the addon in the lod_pool only despite \
             AllowLoad: Both opening it to all 4 screens; the AllowLoad flag gates which \
             screens the LoD load is permitted on, NOT which screens trigger eager \
             discovery"
        );
    }
}

#[test]
fn blizzard_plunderstorm_basics_appears_in_full_addon_inventory() {
    let inventory = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = inventory
        .iter()
        .any(|(name, _)| name == "Blizzard_PlunderstormBasics");
    assert!(
        found,
        "Blizzard_PlunderstormBasics must appear in `discover_all_blizzard_addons` — the \
         full inventory is a structural listing of every parseable TOC including LoD \
         addons"
    );
}

prefork_full_ui_case! {
    fn blizzard_plunderstorm_basics_is_addon_loaded_after_explicit_load(env: &WowLuaEnv) {
    load_plunderstorm_basics_after_fork(env);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_PlunderstormBasics')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_PlunderstormBasics') must return true after \
         explicit load_addon — LoadOnDemand: 1 means only the explicit load path makes \
         IsAddOnLoaded report true"
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn blizzard_plunderstorm_basics_is_addon_loaded_after_explicit_load() {
    let env = load_full_game_ui_with_plunderstorm_basics();
    blizzard_plunderstorm_basics_is_addon_loaded_after_explicit_load::run(&env);
}

prefork_full_ui_case! {
    fn blizzard_plunderstorm_basics_publishes_two_mixins(env: &WowLuaEnv) {
    load_plunderstorm_basics_after_fork(env);

    for mixin in PUBLIC_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{mixin})"))
            .unwrap_or_else(|err| panic!("type(_G.{mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table — the 2 mixins back the basics card's \
             behavior surface: PlunderstormAccountStoreToggleMixin (the OnClick that calls \
             AccountStoreUtil.ToggleAccountStore() guarded by GetPlayerPartyMemberInfo() != \
             nil; OnEnter shows the ACCOUNT_STORE_UNAVAILABLE tooltip when the button is \
             disabled), PlunderstormBasicsContainerFrameMixin (the container's OnShow that \
             registers ACCOUNT_STORE_CURRENCY_AVAILABLE_UPDATED + STORE_FRONT_STATE_UPDATED \
             events, lazily creates the Plunderstore toggle on glue screens via \
             `CreateFrame('BUTTON', nil, self, 'PlunderstormAccountStoreToggleTemplate')`, \
             and wires the PlunderDisplay tooltip OnEnter showing the lifetime-plunder \
             count)"
        );
    }
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn blizzard_plunderstorm_basics_publishes_two_mixins() {
    let env = load_full_game_ui_with_plunderstorm_basics();
    blizzard_plunderstorm_basics_publishes_two_mixins::run(&env);
}

prefork_full_ui_case! {
    fn blizzard_plunderstorm_basics_creates_named_non_virtual_frame(env: &WowLuaEnv) {
    load_plunderstorm_basics_after_fork(env);

    for frame in PUBLIC_NAMED_FRAMES {
        let kind: String = env
            .eval(&format!("return type(_G.{frame})"))
            .unwrap_or_else(|err| panic!("type(_G.{frame}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{frame} must publish as a frame userdata — PlunderstormBasicsContainerFrame \
             is the panel root (mixin=PlunderstormBasicsContainerFrameMixin, inherits \
             VerticalLayoutFrame, frameLevel=1000, anchored LEFT x=46 y=0 with implicit \
             UIParent relativeTo). It hosts the swords atlas + title + body text + 2 \
             horizontal-line separators in the ARTWORK Layer and the PlunderDisplay button \
             + Border in the Frames container, all driven by the VerticalLayoutFrame's \
             layoutIndex+topPadding/bottomPadding KeyValues"
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
fn blizzard_plunderstorm_basics_creates_named_non_virtual_frame() {
    let env = load_full_game_ui_with_plunderstorm_basics();
    blizzard_plunderstorm_basics_creates_named_non_virtual_frame::run(&env);
}

prefork_full_ui_case! {
    fn blizzard_plunderstorm_basics_does_not_leak_virtual_template_to_globals(env: &WowLuaEnv) {
    load_plunderstorm_basics_after_fork(env);

    for template in VIRTUAL_TEMPLATES_NOT_IN_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G.{template})"))
            .unwrap_or_else(|err| panic!("type(_G.{template}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{template} must be nil — virtual templates (`virtual=\"true\"` on the XML \
             element) live in the template registry only, not in globals. \
             PlunderstormAccountStoreToggleTemplate is the BigGoldRedThreeSliceButtonTemplate \
             specialization that the container's OnShow lazily instantiates via \
             CreateFrame('BUTTON', nil, self, 'PlunderstormAccountStoreToggleTemplate'); \
             leaking it to _G would let consumer addons mutate the template definition and \
             break every existing instance"
        );
    }
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn blizzard_plunderstorm_basics_does_not_leak_virtual_template_to_globals() {
    let env = load_full_game_ui_with_plunderstorm_basics();
    blizzard_plunderstorm_basics_does_not_leak_virtual_template_to_globals::run(&env);
}

prefork_full_ui_case! {
    fn blizzard_plunderstorm_basics_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_plunderstorm_basics_after_fork(env);

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_PlunderstormBasics")
                || message.contains("PlunderstormBasics")
                || message.contains("PlunderstormAccountStoreToggle")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_PlunderstormBasics emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn blizzard_plunderstorm_basics_loads_without_addon_specific_lua_errors() {
    let env = load_full_game_ui_with_plunderstorm_basics();
    blizzard_plunderstorm_basics_loads_without_addon_specific_lua_errors::run(&env);
}
