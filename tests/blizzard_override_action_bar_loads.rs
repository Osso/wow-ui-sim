#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn override_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_OverrideActionBar")
}

fn override_toc() -> PathBuf {
    override_dir().join("Blizzard_OverrideActionBar.toc")
}

const OVERRIDE_TOC_FILES: &[&str] = &["OverrideActionBar.lua", "OverrideActionBar.xml"];

const REQUIRED_DEPS: &[&str] = &[
    "Blizzard_ActionBar",
    "Blizzard_UnitFrame",
    "Blizzard_MicroMenu",
    "Blizzard_ManagedFrameSystem",
];

const PUBLIC_MIXINS: &[&str] = &["OverrideActionBarMixin", "OverrideActionBarButtonMixin"];

const NAMED_FRAMES: &[&str] = &["OverrideActionBar"];

const VIRTUAL_TEMPLATES_NOT_IN_GLOBALS: &[&str] = &["OverrideActionBarButtonTemplate"];

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
fn blizzard_override_action_bar_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&override_dir()).expect("Blizzard_OverrideActionBar TOC resolves");
    assert_eq!(
        resolved,
        override_toc(),
        "Blizzard_OverrideActionBar ships exactly one bare TOC — no `_Mainline.toc` variant. \
         The override-action-bar is the special-vehicle / quest-replacement action bar that \
         appears when the player is in a vehicle or controlling a quest-NPC pet (Wrathion's \
         dragon, fishing barge, etc.); it is a retail-only feature but the TOC stayed bare \
         because the override-bar mechanic predates the flavor-split TOC convention. \
         `find_toc_file` resolves the bare TOC after the `_Mainline.toc` lookup misses"
    );

    let mainline = override_dir().join("Blizzard_OverrideActionBar_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {} — the bare TOC is the canonical entry point",
        mainline.display()
    );
}

#[test]
fn blizzard_override_action_bar_toc_declares_eager_game_only_with_four_dependencies() {
    let toc = TocFile::from_file(&override_toc()).expect("Blizzard_OverrideActionBar TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "TOC OMITS `## LoadOnDemand:` so `is_load_on_demand()` returns false — the \
         override-bar must be eager-loaded because the engine swaps to it on \
         UNIT_ENTERED_VEHICLE (a high-frequency event during quest content); a LoD route \
         would force a delay-and-fade-in transition the very first time the player mounts a \
         quest vehicle, which would feel broken"
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
            "`## AllowLoad: Game` must NOT enable {screen:?} — vehicles only exist in-world; \
             glue screens have no UNIT_ENTERED_VEHICLE event"
        );
    }

    assert_eq!(
        toc.dependencies(),
        REQUIRED_DEPS,
        "Retail 12.1.0.69497 declares ActionBar, UnitFrame, MicroMenu, and \
         ManagedFrameSystem in that order"
    );

    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` — no soft sibling addons. The override-bar's behavior is \
         fully determined by the 3 hard deps + foundational FrameXML (SetVehicleBarLayout / \
         VehicleMenuBar)"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — the override-bar is a pure mirror of vehicle-state events \
         (UNIT_ENTERED_VEHICLE / UNIT_EXITED_VEHICLE / UPDATE_VEHICLE_ACTIONBAR). Persisting \
         would only stale the cache; the vehicle-bar refreshes on every state change"
    );
}

#[test]
fn blizzard_override_action_bar_toc_declares_metadata_in_raw_bytes() {
    let raw = std::fs::read_to_string(override_toc())
        .expect("Blizzard_OverrideActionBar TOC reads utf-8");
    assert!(
        raw.contains("## Title: Blizzard_OverrideActionBar"),
        "TOC must declare `## Title: Blizzard_OverrideActionBar` exactly. UNUSUAL: the title \
         uses the underscore-namespace spelling (rather than the space-and-prose form like \
         `Blizzard Order Hall UI`); this is the minority pattern, suggesting the addon was \
         scaffolded from a code template rather than authored from a designer-facing brief"
    );
    assert!(
        raw.contains("## Author: Blizzard Entertainment"),
        "TOC must declare `## Author: Blizzard Entertainment` exactly"
    );
    assert!(
        raw.contains("## DefaultState: enabled"),
        "TOC must declare `## DefaultState: enabled` exactly — the override-bar is enabled \
         by default in the addon-manager UI; disabling it would break vehicle UI completely \
         (the player would see a blank screen during quest-vehicle content). The explicit \
         `enabled` documents that this is foundational gameplay infrastructure, not an \
         optional cosmetic addon"
    );
    assert!(
        raw.contains(
            "## Dependencies: Blizzard_ActionBar, Blizzard_UnitFrame, Blizzard_MicroMenu, \
             Blizzard_ManagedFrameSystem"
        ),
        "Retail 12.1.0.69497 declares all four dependencies on one Dependencies line"
    );
    assert!(
        raw.contains("## AllowLoad: Game"),
        "TOC must declare `## AllowLoad: Game` exactly — case-sensitive Game-only screen lock"
    );
    assert!(
        !raw.contains("## LoadOnDemand"),
        "TOC must NOT declare `## LoadOnDemand` — the absence is what flags eager loading"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT declare any `## SavedVariables*` keys — pure event-driven mirror"
    );
    assert!(
        !raw.contains("## OptionalDeps"),
        "TOC must NOT declare any `## OptionalDeps:` — zero soft sibling addons"
    );
}

#[test]
fn blizzard_override_action_bar_toc_lists_two_files_lua_first_xml_after() {
    let toc = TocFile::from_file(&override_toc()).expect("Blizzard_OverrideActionBar TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, OVERRIDE_TOC_FILES,
        "TOC body must list exactly 2 files in canonical order: OverrideActionBar.lua first \
         (declares the OverrideActionBarMixin / OverrideActionBarButtonMixin globals + the \
         file-private `textureList` table at module top — these MUST exist before the XML \
         parses, since the XML's `mixin=\"OverrideActionBarMixin\"` attribute resolves the \
         mixin by name at parse time and the OnLoad script body iterates textureList to \
         compose the bar's child texture references), then OverrideActionBar.xml after \
         (instantiates the named OverrideActionBar frame inheriting the mixin and bound to \
         all the layered textures listed in textureList). Reversing the order would crash \
         at XML parse time when OverrideActionBarMixin resolves to nil"
    );
}

#[test]
fn blizzard_override_action_bar_appears_in_game_screen_eager_discovery_only() {
    let ui = blizzard_ui_dir();

    let game_addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_OverrideActionBar");
    assert!(
        in_game,
        "Blizzard_OverrideActionBar must auto-discover on Game screen — eager (no \
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
            .any(|(name, _)| name == "Blizzard_OverrideActionBar");
        assert!(
            !found,
            "Blizzard_OverrideActionBar must NOT auto-discover on screen {screen:?} — `## \
             AllowLoad: Game` restricts discovery to ScreenKind::Game only"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_override_action_bar_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_OverrideActionBar")
                || message.contains("OverrideActionBar")
                || message.contains("OverrideActionBarMixin")
                || message.contains("OverrideActionBarButtonMixin")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_OverrideActionBar emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_override_action_bar_is_addon_loaded_after_eager_sweep(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_OverrideActionBar')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_OverrideActionBar') must return true after the \
         eager Game-screen sweep — no LoadOnDemand puts the addon in the eager set"
    );
}
}

prefork_full_ui_case! {
fn blizzard_override_action_bar_publishes_two_mixin_tables(env: &WowLuaEnv) {

    for mixin in PUBLIC_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{mixin})"))
            .unwrap_or_else(|err| panic!("type(_G.{mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table — OverrideActionBar.lua declares 2 mixins at \
             module top: OverrideActionBarMixin (the bar's lifecycle owner — OnLoad / \
             OnEvent / OnShow / OnHide / UpdateMicroButtons / UpdateSkin / SetSkin / \
             CalcSize / GetMicroButtonAnchor / Leave; OnLoad iterates the file-private \
             `textureList` to wire up child texture references for the bar's nine-slice \
             background, end caps, dividers, micro-menu strip, and pitch overlay; OnEvent \
             handles UNIT_ENTERED_VEHICLE / UNIT_EXITED_VEHICLE / \
             UPDATE_VEHICLE_ACTIONBAR / ACTIONBAR_PAGE_CHANGED to swap skins and refresh \
             button bindings), OverrideActionBarButtonMixin (per-button — extends the \
             standard SecureActionButton conventions for vehicle ability buttons; covers \
             cooldown frames, count text, and the special vehicle-cast flash). Each mixin \
             is referenced by an XML `mixin=\"...\"` attribute"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_override_action_bar_creates_named_frame(env: &WowLuaEnv) {

    for frame_name in NAMED_FRAMES {
        let kind: String = env
            .eval(&format!("return type(_G.{frame_name})"))
            .unwrap_or_else(|err| panic!("type(_G.{frame_name}) probe failed: {err}"));
        assert!(
            kind == "table" || kind == "userdata",
            "_G.{frame_name} must be a frame (table or userdata) — the named non-virtual \
             `<Frame name=\"OverrideActionBar\">` at OverrideActionBar.xml line 14 \
             instantiates immediately at XML parse time. Sets `mixin=\
             \"OverrideActionBarMixin\"`, `frameStrata=\"MEDIUM\"`, `parent=\"UIParent\"`, \
             `enableMouse=\"true\"`, `toplevel=\"true\"`, `hidden=\"true\"`. The frame is \
             the only named non-virtual frame the addon publishes — vehicle ability \
             buttons are instantiated dynamically from OverrideActionBarButtonTemplate by \
             the OnLoad logic. Got type {kind} for {frame_name}"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_override_action_bar_does_not_leak_virtual_template_to_globals(env: &WowLuaEnv) {

    for template in VIRTUAL_TEMPLATES_NOT_IN_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G.{template})"))
            .unwrap_or_else(|err| panic!("type(_G.{template}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{template} must be nil — virtual templates (`virtual=\"true\"`) live in the \
             template registry, NOT in `_G`. OverrideActionBarButtonTemplate is the per-button \
             vehicle-ability template instantiated dynamically by the OnLoad layout code; \
             leaking it as a global would let addons mutate the template definition and \
             break every existing button instance"
        );
    }
}
}
