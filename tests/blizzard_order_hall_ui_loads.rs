#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{
    discover_all_blizzard_addons, discover_blizzard_addons_for_screen, find_toc_file, load_addon,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn order_hall_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_OrderHallUI")
}

fn order_hall_toc() -> PathBuf {
    order_hall_dir().join("Blizzard_OrderHallUI.toc")
}

const ORDER_HALL_TOC_FILES: &[&str] = &[
    "Blizzard_OrderHallUI_Bootstrap.lua",
    "Blizzard_OrderHallTalents.xml",
    "Localization.lua",
];

const PUBLIC_MIXINS: &[&str] = &[
    "OrderHallTalentFrameMixin",
    "GarrisonTalentButtonMixin",
    "GarrisonTalentButtonAnimationMixin",
    "CypherEquipmentLevelMixin",
];

const NAMED_FRAMES: &[&str] = &["OrderHallTalentFrame"];

const VIRTUAL_TEMPLATES_NOT_IN_GLOBALS: &[&str] = &[
    "GarrisonTalentButtonTemplate",
    "GarrisonTalentArrowTemplate",
    "GarrisonTalentPrerequisiteArrowTemplate",
    "GarrisonTalentChoiceTemplate",
    "GarrisonTalentTrackTemplate",
    "GarrisonTalentButtonAnimationTemplate",
];

const PUBLIC_GLOBAL_FUNCTIONS: &[&str] = &["OrderHallTalentFrame_ToggleFrame"];

fn load_full_game_ui_then_request_order_hall() -> WowLuaEnv {
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

    load_addon(&env.loader_env(), &order_hall_toc())
        .expect("Blizzard_OrderHallUI load_addon succeeds after eager Game-screen sweep");

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);

    env
}

#[test]
fn blizzard_order_hall_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&order_hall_dir()).expect("Blizzard_OrderHallUI TOC resolves");
    assert_eq!(
        resolved,
        order_hall_toc(),
        "Blizzard_OrderHallUI ships exactly one bare TOC — no `_Mainline.toc` variant. The \
         Order Hall talent UI is Legion-era (patch 7.0) class-hall content; the TOC stayed \
         bare across expansions because the same Garrison-talent infrastructure is reused \
         for Torghast / Cypher Equipment / Covenant systems (the mixins + Lua all live here). \
         `find_toc_file` resolves the bare TOC after the `_Mainline.toc` lookup misses"
    );

    let mainline = order_hall_dir().join("Blizzard_OrderHallUI_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {} — flavor split is unnecessary because the \
         Garrison-talent system the addon owns has been retroactively reused for newer \
         expansions (Torghast in Shadowlands, Cypher in 9.2). The bare TOC is the canonical \
         entry point",
        mainline.display()
    );
}

#[test]
fn blizzard_order_hall_toc_declares_load_on_demand_with_game_menu_dependency() {
    let toc = TocFile::from_file(&order_hall_toc()).expect("Blizzard_OrderHallUI TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "TOC declares `## LoadOnDemand: 1` — the order hall talent UI is summoned via \
         OrderHallTalentFrame_ToggleFrame() called from class-hall NPC gossip / advisor-table \
         interaction, so eager-loading would waste resources on every login. The `## \
         LoadOnDemand: 1` route at src/loader/mod.rs:530-534 keeps the addon out of the eager \
         Game-screen discovery sweep until something explicitly calls load_addon"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());

    assert_eq!(
        toc.dependencies(),
        vec!["Blizzard_GameMenuEsc".to_string()],
        "Retail 12.1 declares Blizzard_GameMenuEsc as the Order Hall UI's single hard dependency."
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` — no soft sibling addons. Every surface the talent UI \
         touches is either foundational FrameXML (PortraitFrameTemplate, ShowUIPanel / \
         HideUIPanel, ScriptedAnimationEffects) or a built-in C_* namespace (C_Garrison.* / \
         Enum.GarrisonTalentAvailability)"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — the talent UI mirrors live server-side talent state \
         (researched / available / locked). Persisting would only stale the cache; the \
         engine reissues data via GARRISON_TALENT_* events on login"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "TOC OMITS `## AllowLoad:` so `allows_screen` at src/toc.rs:311 returns true for the \
         Game screen by default — the omitted-key default is Game-only. Order halls only \
         exist in-world; glue screens have no Garrison state"
    );
}

#[test]
fn blizzard_order_hall_toc_declares_metadata_in_raw_bytes() {
    let raw =
        std::fs::read_to_string(order_hall_toc()).expect("Blizzard_OrderHallUI TOC reads utf-8");
    assert!(
        raw.contains("## Title: Blizzard Order Hall UI"),
        "TOC must declare `## Title: Blizzard Order Hall UI` exactly — the space-and-prose \
         human-readable label"
    );
    assert!(
        raw.contains("## Author: Blizzard Entertainment"),
        "TOC must declare `## Author: Blizzard Entertainment` exactly. UNUSUAL: most \
         Blizzard-shipped addons OMIT the `## Author:` key (relying on the implicit \
         `Blizzard` ownership of the Blizzard_ namespace prefix); this one explicitly states \
         it. Pinning the exact spelling guards against a refactor that drops the key"
    );
    assert!(
        raw.contains("## Version: 1.0"),
        "TOC must declare `## Version: 1.0` exactly. UNUSUAL: most Blizzard-shipped addons \
         OMIT the `## Version:` key (relying on the engine's `## Interface:` for version \
         tracking); this one stubs `1.0` despite the addon having seen multiple expansions \
         of code changes. Likely a vestigial scaffold from the addon's original Legion-era \
         template that nobody updates"
    );
    assert!(
        raw.contains("## LoadOnDemand: 1"),
        "TOC must declare `## LoadOnDemand: 1` exactly — the explicit `1` form, the canonical \
         retail spelling for LoD addons"
    );
    assert!(
        raw.contains("## Dependencies: Blizzard_GameMenuEsc"),
        "Retail 12.1.0.69497 declares Blizzard_GameMenuEsc as its hard dependency"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT declare any `## SavedVariables*` keys — pure server-state mirror"
    );
    assert!(
        !raw.contains("## AllowLoad:"),
        "TOC must NOT declare `## AllowLoad:` — the absence is the canonical retail spelling \
         for Game-only addons (default)"
    );
}

#[test]
fn blizzard_order_hall_toc_lists_bootstrap_xml_then_localization() {
    let toc = TocFile::from_file(&order_hall_toc()).expect("Blizzard_OrderHallUI TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, ORDER_HALL_TOC_FILES,
        "Retail 12.1.0.69497 lists its bootstrap first, then Blizzard_OrderHallTalents.xml \
         and Localization.lua. The XML loads Blizzard_OrderHallTalents.lua through its Script \
         directive before the trailing localization file"
    );
}

#[test]
fn blizzard_order_hall_does_not_appear_in_eager_discovery_for_any_screen() {
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
            .any(|(name, _)| name == "Blizzard_OrderHallUI");
        assert!(
            !found,
            "Blizzard_OrderHallUI must NOT auto-discover on screen {screen:?} — `## \
             LoadOnDemand: 1` excludes the addon from every eager-discovery sweep. The LoD \
             addon must be summoned via an explicit `load_addon` call (typically via \
             `C_AddOns.LoadAddOn('Blizzard_OrderHallUI')` triggered by class-hall gossip)"
        );
    }
}

#[test]
fn blizzard_order_hall_appears_in_full_addon_inventory() {
    let ui = blizzard_ui_dir();
    let all_addons = discover_all_blizzard_addons(&ui);
    let found = all_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_OrderHallUI");
    assert!(
        found,
        "Blizzard_OrderHallUI must appear in `discover_all_blizzard_addons` — LoD addons \
         must be visible in the full inventory probe so the addon-manager UI can list them"
    );
}

#[test]
fn blizzard_order_hall_loads_without_addon_specific_lua_errors() {
    let env = load_full_game_ui_then_request_order_hall();

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_OrderHallUI")
                || message.contains("OrderHallTalentFrame")
                || message.contains("OrderHallTalentFrameMixin")
                || message.contains("GarrisonTalentButton")
                || message.contains("CypherEquipmentLevel")
                || message.contains("TalentTreeLayoutOptions")
                || message.contains("TalentUnavailableReasons")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_OrderHallUI emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}

#[test]
fn blizzard_order_hall_is_addon_loaded_after_explicit_load() {
    let env = load_full_game_ui_then_request_order_hall();

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_OrderHallUI')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_OrderHallUI') must return true after the explicit \
         load_addon call — the LoD-routed addon is bookkept by the simulator's addon \
         registry once load_addon completes"
    );
}

#[test]
fn blizzard_order_hall_publishes_four_mixin_tables() {
    let env = load_full_game_ui_then_request_order_hall();

    for mixin in PUBLIC_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{mixin})"))
            .unwrap_or_else(|err| panic!("type(_G.{mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table — Blizzard_OrderHallTalents.lua declares 4 \
             mixins at module top: OrderHallTalentFrameMixin (the parent frame's lifecycle — \
             OnLoad / OnEvent / OnShow / OnHide / Refresh / SetUseThemedTextures / etc.; \
             registers GARRISON_TALENT_* events and drives the talent-tree layout via \
             TalentTreeLayoutOptions[treeType] dispatch table), GarrisonTalentButtonMixin \
             (per-talent button — owns Refresh / OnClick / OnEnter / OnLeave / Update; \
             reads talent.talentAvailability and dispatches to TalentUnavailableReasons \
             lookup table for tooltip text; routes click → C_Garrison.ResearchTalent), \
             GarrisonTalentButtonAnimationMixin (button-level glow/sheen animation owner \
             — Init / Play / Stop hooks for the orderhalltalents-done-glow / \
             orderhalltalents-spellborder-yellow atlas effects), CypherEquipmentLevelMixin \
             (the Shadowlands 9.2 Cypher Equipment system that retroactively reused the \
             OrderHall talent tree skeleton — owns OnLoad / SetCypherLevel / GetCypherLevel; \
             the file's presence is the most visible artifact of the cross-expansion code \
             reuse). Each mixin is referenced by an XML `mixin=\"...\"` attribute"
        );
    }
}

#[test]
fn blizzard_order_hall_creates_named_frame_after_load() {
    let env = load_full_game_ui_then_request_order_hall();

    for frame_name in NAMED_FRAMES {
        let kind: String = env
            .eval(&format!("return type(_G.{frame_name})"))
            .unwrap_or_else(|err| panic!("type(_G.{frame_name}) probe failed: {err}"));
        assert!(
            kind == "table" || kind == "userdata",
            "_G.{frame_name} must be a frame (table or userdata) — the named non-virtual \
             `<Frame name=\"OrderHallTalentFrame\">` at Blizzard_OrderHallTalents.xml line \
             177 instantiates immediately at XML parse time. It inherits \
             `PortraitFrameTemplate`, sets `parent=\"UIParent\"`, `toplevel=\"true\"`, \
             `enableMouse=\"true\"`, `mixin=\"OrderHallTalentFrameMixin\"`, `hidden=\"true\"`. \
             It is the only named non-virtual frame the addon publishes — every talent \
             button / arrow / track is instantiated dynamically from a virtual template by \
             the talent-tree layout code at runtime. Got type {kind} for {frame_name}"
        );
    }
}

#[test]
fn blizzard_order_hall_publishes_toggle_global_function() {
    let env = load_full_game_ui_then_request_order_hall();

    for fname in PUBLIC_GLOBAL_FUNCTIONS {
        let kind: String = env
            .eval(&format!("return type(_G.{fname})"))
            .unwrap_or_else(|err| panic!("type(_G.{fname}) probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "_G.{fname} must be a function — Blizzard_OrderHallTalents.lua line 11 declares \
             `function OrderHallTalentFrame_ToggleFrame()` at module top; it is the public \
             entry point that class-hall NPC gossip handlers call to open/close the talent \
             UI. The body checks `OrderHallTalentFrame:IsShown()` and routes through \
             `ShowUIPanel` / `HideUIPanel` to integrate with the UIParent panel-management \
             system. The legacy underscore-naming pattern (`Foo_Bar`) is the pre-namespace \
             FrameXML convention — modern Blizzard addons would expose this as a method on a \
             namespace table"
        );
    }
}

#[test]
fn blizzard_order_hall_does_not_leak_virtual_templates_to_globals() {
    let env = load_full_game_ui_then_request_order_hall();

    for template in VIRTUAL_TEMPLATES_NOT_IN_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G.{template})"))
            .unwrap_or_else(|err| panic!("type(_G.{template}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{template} must be nil — virtual templates (`virtual=\"true\"`) live in the \
             template registry, NOT in `_G`. GarrisonTalentButtonTemplate / \
             GarrisonTalentArrowTemplate / GarrisonTalentPrerequisiteArrowTemplate / \
             GarrisonTalentChoiceTemplate / GarrisonTalentTrackTemplate are top-level \
             `<Button virtual=\"true\">` / `<Frame virtual=\"true\">` definitions; \
             GarrisonTalentButtonAnimationTemplate is the animation-overlay template. They \
             must NOT publish as globals — leaking would let addons mutate the template \
             definition and break every existing instance"
        );
    }
}
