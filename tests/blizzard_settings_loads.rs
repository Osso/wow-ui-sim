use std::path::PathBuf;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn settings_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_Settings")
}

fn settings_toc() -> PathBuf {
    settings_dir().join("Blizzard_Settings.toc")
}

fn settings_lua() -> PathBuf {
    settings_dir().join("Blizzard_Settings.lua")
}

// Literal cached inputs absent from Wrath manifest1-382/cache; behavior cases stay enabled.
retail_ptr_forever_toc_fixture! {
#[test]
fn find_toc_file_resolves_bare_toc() {
    let resolved = find_toc_file(&settings_dir()).expect("Blizzard_Settings TOC resolves");
    assert_eq!(
        resolved,
        settings_toc(),
        "Blizzard_Settings ships exactly one bare `Blizzard_Settings.toc` — no \
         flavor variants. The settings panel implementation lives entirely in \
         the upstream Blizzard_Settings_Shared / Blizzard_SettingsDefinitions_* \
         addons; this LoD shell exists only as a tracking marker"
    );
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn toc_declares_load_on_demand_with_blizzard_author_no_deps() {
    let toc = TocFile::from_file(&settings_toc()).expect("Blizzard_Settings TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "TOC must declare `## LoadOnDemand: 1` — the addon exists only to be \
         loaded once via `C_AddOns.LoadAddOn('Blizzard_Settings')` from \
         `SettingsPanelMixin:Open()` (Blizzard_Settings_Shared/Blizzard_SettingsPanel.lua:275, \
         comment: '-- Loaded for tracking'). The LoadOnDemand=1 flag keeps it out \
         of eager discovery so the global flag flip is observable: \
         `SettingsAddonLoaded` is nil at startup, then transitions to true on the \
         first settings-panel open"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(!toc.is_glue_only());

    assert!(
        toc.dependencies().is_empty(),
        "TOC must declare zero hard Dependencies — the 1-line Lua body does not \
         reference any other addon's globals; it merely sets one global flag"
    );
    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.saved_variables_per_character().is_empty());
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn toc_lacks_allow_load_so_falls_through_to_game_only() {
    let toc = TocFile::from_file(&settings_toc()).expect("Blizzard_Settings TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Without `## AllowLoad`, src/toc.rs:311 None arm restricts the addon to \
         the Game screen — combined with LoadOnDemand=1 the addon never enters \
         eager discovery, so the screen restriction only matters when something \
         attempts an explicit C_AddOns.LoadAddOn from a glue screen (which would \
         be rejected before any Lua runs)"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Glue screen {screen:?} must NOT be allowed — settings-panel tracking \
             is an in-game concern only; the glue-screen options dialog is a \
             separate code path"
        );
    }
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn toc_raw_bytes_pin_three_metadata_lines_with_single_lua_body() {
    let raw = std::fs::read_to_string(settings_toc()).expect("Blizzard_Settings TOC reads utf-8");

    assert!(raw.contains("## Title: Blizzard Settings"));
    assert!(raw.contains("## Author: Blizzard Entertainment"));
    assert!(raw.contains("## LoadOnDemand: 1"));
    assert!(
        !raw.contains("## Dependencies"),
        "TOC must NOT declare Dependencies — 1-line tracking flag has nothing to \
         depend on. The consumer Blizzard_Settings_Shared depends on this only \
         indirectly via the C_AddOns.LoadAddOn call site"
    );
    assert!(!raw.contains("## RequiredDep"));
    assert!(!raw.contains("## AllowLoad"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## DefaultState"));

    let body_lines: Vec<&str> = raw
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        })
        .collect();
    assert_eq!(
        body_lines,
        vec!["Blizzard_Settings.lua"],
        "TOC body must list exactly 1 file: Blizzard_Settings.lua"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn lua_body_is_a_single_global_flag_assignment() {
    let raw = std::fs::read_to_string(settings_lua()).expect("Blizzard_Settings.lua reads utf-8");
    let trimmed = raw.trim();
    assert_eq!(
        trimmed, "SettingsAddonLoaded = true;",
        "Blizzard_Settings.lua body must be exactly one statement: \
         `SettingsAddonLoaded = true;`. The upstream Blizzard_Settings_Shared \
         (Blizzard_SettingsPanel.lua:274-276) uses `if not SettingsAddonLoaded \
         then C_AddOns.LoadAddOn('Blizzard_Settings')` to load this addon \
         exactly once per session, on the first SettingsPanel:Open(). Adding any \
         other code here would defeat the tracking semantics: the addon must be \
         a thin lazy-load sentinel, NOT a code container"
    );
}
}

#[test]
fn lod_addon_excluded_from_eager_discovery_on_every_screen() {
    let ui = blizzard_ui_dir();

    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let found = addons.iter().any(|(name, _)| name == "Blizzard_Settings");
        assert!(
            !found,
            "Blizzard_Settings must be excluded from eager discovery on \
             {screen:?} — `## LoadOnDemand: 1` keeps it out of every eager \
             sweep. The addon loads exactly once per session via explicit \
             C_AddOns.LoadAddOn, gated on `not SettingsAddonLoaded`"
        );
    }
}

retail_ptr_forever_toc_fixture! {
#[test]
fn root_directory_holds_only_the_toc_and_one_lua_file() {
    let dir = settings_dir();
    assert!(dir.join("Blizzard_Settings.toc").is_file());
    assert!(dir.join("Blizzard_Settings.lua").is_file());

    let entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("read addon dir")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        entries.len(),
        2,
        "Blizzard_Settings directory must contain exactly 2 entries (1 toc + 1 \
         lua) — no XML, no helpers, no nested directories. Got: {entries:?}"
    );
}
}

#[test]
fn settings_addon_loaded_global_is_nil_before_explicit_load() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);

    let kind: String = env
        .eval("return type(SettingsAddonLoaded)")
        .expect("pre-load SettingsAddonLoaded probe succeeds");
    assert_eq!(
        kind, "nil",
        "SettingsAddonLoaded must be nil before the LoD addon loads — this is \
         the gate condition the consumer relies on (`if not SettingsAddonLoaded \
         then ... LoadAddOn(...)`). If startup pre-populates this global, the \
         consumer would skip its first-open LoadAddOn call and the tracking \
         signal would never fire"
    );
}

#[test]
fn explicit_load_emits_no_lua_errors_and_publishes_global_flag() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }
    wow_ui_sim::xml::register_intrinsic_templates();

    load_addon(&env.loader_env(), &settings_toc())
        .expect("Blizzard_Settings explicit load_addon must succeed");

    let load_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    assert!(
        load_errors.is_empty(),
        "Blizzard_Settings explicit load must emit zero Lua errors — the body \
         is a single trivial assignment. Got:\n  {}",
        load_errors.join("\n  ")
    );

    let kind: String = env
        .eval("return type(SettingsAddonLoaded)")
        .expect("post-load SettingsAddonLoaded probe succeeds");
    assert_eq!(
        kind, "boolean",
        "_G.SettingsAddonLoaded must be a boolean after load — the body \
         executes `SettingsAddonLoaded = true;` at top scope, publishing the \
         flag globally"
    );

    let value: bool = env
        .eval("return SettingsAddonLoaded")
        .expect("post-load SettingsAddonLoaded value probe succeeds");
    assert!(
        value,
        "_G.SettingsAddonLoaded must be true after load — the assignment is \
         hardcoded `= true`. Anything else would break the consumer's `if not \
         SettingsAddonLoaded` gate"
    );
}

post_wrath_ui_fixture! {
#[test]
fn is_addon_loaded_transitions_false_to_true_after_explicit_load() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }
    wow_ui_sim::xml::register_intrinsic_templates();

    let before: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_Settings')")
        .expect("pre-load IsAddOnLoaded probe succeeds");
    assert!(
        !before,
        "C_AddOns.IsAddOnLoaded('Blizzard_Settings') must be false before \
         explicit load — LoadOnDemand=1 keeps it out of eager discovery"
    );

    load_addon(&env.loader_env(), &settings_toc())
        .expect("Blizzard_Settings explicit load_addon must succeed");

    let after: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_Settings')")
        .expect("post-load IsAddOnLoaded probe succeeds");
    assert!(
        after,
        "C_AddOns.IsAddOnLoaded('Blizzard_Settings') must be true after \
         explicit load — the AddOn state machine flips loaded=true after \
         executing the body, regardless of how trivial the body is"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn settings_addon_loaded_consumer_lives_in_settings_shared_addon() {
    let consumer = blizzard_ui_dir().join("Blizzard_Settings_Shared/Blizzard_SettingsPanel.lua");
    let raw = std::fs::read_to_string(&consumer).expect("Blizzard_SettingsPanel.lua reads utf-8");

    assert!(
        raw.contains("if not SettingsAddonLoaded then"),
        "Blizzard_Settings_Shared/Blizzard_SettingsPanel.lua must contain the \
         `if not SettingsAddonLoaded then` gate — this is the call site that \
         drives the tracking semantic. If the consumer's gate disappears, the \
         entire reason for this LoD shell ceases to exist and the addon should \
         be removed"
    );
    assert!(
        raw.contains("C_AddOns.LoadAddOn(\"Blizzard_Settings\")"),
        "The consumer must lazy-load `Blizzard_Settings` via \
         `C_AddOns.LoadAddOn(\"Blizzard_Settings\")` inside the gate. The \
         comment `-- Loaded for tracking` documents intent: this is a one-shot \
         load-time event that downstream code can key off"
    );
}
}

// The Mists/Era/Anniversary caches ship 218 addons each and Wrath ships 23;
// none contains Blizzard_Settings, so the pinned TOC facts above describe no file the
// classic loaders can read.
classic_cache_absent_toc_fixture! {
#[test]
fn blizzard_settings_toc_is_absent_from_classic_caches() {
    let dir = settings_dir();
    assert!(
        !dir.is_dir(),
        "classic profile caches must not contain Blizzard_Settings"
    );
    assert!(wow_ui_sim::loader::find_toc_file(&dir).is_none());
}
}
