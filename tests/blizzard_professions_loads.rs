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

fn professions_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_Professions")
}

fn professions_toc() -> PathBuf {
    professions_dir().join("Blizzard_Professions.toc")
}

const PROFESSIONS_TOC_FILES: &[&str] = &[
    "Blizzard_Professions_Bootstrap.lua",
    "Blizzard_ProfessionsRankBar.xml",
    "Blizzard_ProfessionsGuildMemberList.xml",
    "Blizzard_ProfessionsCraftingOutputLog.xml",
    "Blizzard_ProfessionsRecipeLevel.xml",
    "Blizzard_ProfessionsCrafting.xml",
    "Blizzard_ProfessionsInspectRecipe.xml",
    "Blizzard_ProfessionsSpecializationsTemplates.xml",
    "Blizzard_ProfessionsSpecializations.xml",
    "Blizzard_ProfessionsCrafterOrderView.xml",
    "Blizzard_ProfessionsCrafterOrderPage.xml",
    "Blizzard_ProfessionsFrame.xml",
    "Blizzard_ProfessionsRegistration.lua",
    "Localization.lua",
];

const REQUIRED_DEPS: &[&str] = &[
    "Blizzard_ProfessionsTemplates",
    "Blizzard_SharedTalentUI",
    "Blizzard_Colors",
    "Blizzard_HelpPlate",
    "Blizzard_GameMenuEsc",
];

const PUBLIC_MIXIN_GLOBALS: &[&str] = &[
    "ProfessionsMixin",
    "ProfessionsCraftingPageMixin",
    "ProfessionsCraftingOrderPageMixin",
    "ProfessionsCrafterOrderViewMixin",
    "ProfessionsCraftingOutputLogMixin",
    "ProfessionsRankBarMixin",
    "ProfessionsRecipeLevelBarMixin",
    "ProfessionsSpecFrameMixin",
    "ProfessionsSpecPathMixin",
    "InspectRecipeMixin",
];

const NAMED_NON_VIRTUAL_TOP_LEVEL_FRAMES: &[&str] = &["ProfessionsFrame", "InspectRecipeFrame"];

const SAVED_VARIABLES_PER_CHARACTER: &[&str] = &[
    "g_professionsSpecsSelectedTabs",
    "g_professionsSpecsSelectedPaths",
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
fn blizzard_professions_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&professions_dir()).expect("Blizzard_Professions TOC resolves");
    assert_eq!(
        resolved,
        professions_toc(),
        "Blizzard_Professions ships exactly one bare TOC — no `_Mainline.toc` variant. \
         The addon is gated by `## AllowLoadGameType: mainline` in the bare TOC body, \
         not by a flavor-suffixed TOC file"
    );

    let mainline = professions_dir().join("Blizzard_Professions_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {} — the bare TOC carries the mainline gate \
         via the AllowLoadGameType directive instead",
        mainline.display()
    );
}

#[test]
fn blizzard_professions_toc_declares_lod_mainline_game_only_addon() {
    let toc = TocFile::from_file(&professions_toc()).expect("Blizzard_Professions TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "TOC must declare `## LoadOnDemand: 1` — the Professions panel is heavyweight \
         (~9000 lines across 13 Lua files + 11 XML files, ~46KB Lua) and is loaded \
         explicitly via UIParentLoadAddOn / LoadAddOn('Blizzard_Professions') when the \
         player first opens a profession window"
    );
    assert!(!toc.is_load_first());
    assert!(
        !toc.is_secure_env(),
        "TOC must NOT declare `## UseSecureEnvironment:` — professions UI is non-protected"
    );

    assert!(
        !toc.is_game_type_restricted(),
        "TOC declares `## AllowLoadGameType: mainline` but `is_game_type_restricted` at \
         src/toc.rs:294-302 returns FALSE for `mainline` / `standard` (treated as the \
         unrestricted default). The flag only fires for non-mainline gates like \
         `plunderstorm` / `classic` / `wowhack`. So while the TOC IS technically \
         retail-gated in raw bytes, the simulator's restriction predicate considers \
         this a baseline-mainline addon — pinned in `toc_declares_metadata_in_raw_bytes` \
         via the `## AllowLoadGameType: mainline` substring check"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: game` (lowercase) must enable Game screen — the panel is in-world \
         only"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "`## AllowLoad: game` must EXCLUDE {screen:?} — no profession state outside \
             the game world"
        );
    }
}

#[test]
fn blizzard_professions_toc_declares_five_dependencies() {
    let toc = TocFile::from_file(&professions_toc()).expect("Blizzard_Professions TOC parses");

    let dependencies = toc.dependencies();
    let deps: Vec<&str> = dependencies.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        deps, REQUIRED_DEPS,
        "Retail 12.1.0.69497 declares ProfessionsTemplates, SharedTalentUI, Colors, \
         HelpPlate, and GameMenuEsc in published order"
    );

    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` — every dependency is hard-required"
    );
}

#[test]
fn blizzard_professions_toc_declares_two_per_character_saved_vars() {
    let toc = TocFile::from_file(&professions_toc()).expect("Blizzard_Professions TOC parses");

    let account_wide = toc.saved_variables();
    assert!(
        account_wide.is_empty(),
        "Zero account-wide `## SavedVariables:` — the parser's `saved_variables()` at \
         src/toc.rs:316-328 only collects `SavedVariables` / `SavedVariablesMachine`, \
         neither of which Blizzard_Professions declares"
    );

    let per_character = toc.saved_variables_per_character();
    let per_char: Vec<&str> = per_character.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        per_char, SAVED_VARIABLES_PER_CHARACTER,
        "TOC must declare exactly 2 `## SavedVariablesPerCharacter:` entries — \
         g_professionsSpecsSelectedTabs (last-selected spec tab per known profession, \
         keyed by skillLineID — restores the player's tab pick across reloads), and \
         g_professionsSpecsSelectedPaths (last-selected spec talent-tree path per \
         profession, used by ProfessionsSpecFrameMixin:RestoreSelectedPath to reseat \
         the path-of-investment view). Per-character (NOT per-account) because both \
         vary per alt: a player's BS spec on warrior differs from their JC spec on \
         rogue. Read via `saved_variables_per_character()` at src/toc.rs:331-340 \
         (separate accessor from `saved_variables()`)"
    );
}

#[test]
fn blizzard_professions_toc_declares_metadata_in_raw_bytes() {
    let raw =
        std::fs::read_to_string(professions_toc()).expect("Blizzard_Professions TOC reads utf-8");
    assert!(
        raw.contains("## Title: Blizzard Professions"),
        "TOC must declare `## Title: Blizzard Professions` — SPACE-AND-PROSE form (NOT \
         the underscored-camelcase `Blizzard_Professions` directory name) because this \
         is a player-facing panel, not a developer-facing library"
    );
    assert!(
        raw.contains("## Author: Blizzard Entertainment"),
        "TOC must declare `## Author: Blizzard Entertainment` exactly"
    );
    assert!(
        raw.contains("## LoadOnDemand: 1"),
        "TOC must declare `## LoadOnDemand: 1` exactly — eager-load would slow boot for \
         every player whether or not they ever open a profession panel"
    );
    assert!(
        raw.contains("## AllowLoad: game"),
        "TOC must declare `## AllowLoad: game` (lowercase) — case-insensitive matching \
         still routes through the Game-only screen gate"
    );
    assert!(
        raw.contains("## AllowLoadGameType: mainline"),
        "TOC must declare `## AllowLoadGameType: mainline` — retail-only gate"
    );
    assert!(
        raw.contains(
            "## Dependencies: Blizzard_ProfessionsTemplates, Blizzard_SharedTalentUI, \
             Blizzard_Colors, Blizzard_HelpPlate, Blizzard_GameMenuEsc"
        ),
        "Retail 12.1.0.69497 declares all five dependencies on one comma-separated line"
    );
    assert!(
        raw.contains(
            "## SavedVariablesPerCharacter: g_professionsSpecsSelectedTabs, \
             g_professionsSpecsSelectedPaths"
        ),
        "TOC must declare the 2 per-character saved vars on one comma-separated line"
    );

    assert!(
        !raw.contains("## SavedVariables:"),
        "TOC must NOT declare account-wide `## SavedVariables:` — both stored values are \
         per-character"
    );
    assert!(
        !raw.contains("## OptionalDeps"),
        "TOC must NOT declare `## OptionalDeps:` — every dep is hard-required"
    );
    assert!(
        !raw.contains("## RequiredDep"),
        "TOC must NOT declare `## RequiredDep:` / `## RequiredDeps:` — uses the \
         canonical `Dependencies:` form"
    );
    assert!(
        !raw.contains("## UseSecureEnvironment"),
        "TOC must NOT declare `## UseSecureEnvironment:` — non-protected UI"
    );
    assert!(
        !raw.contains("## DefaultState"),
        "TOC must NOT declare `## DefaultState:` — defaults to enabled when omitted, \
         and a LoadOnDemand addon being disabled would make the panel unreachable"
    );
    assert!(
        !raw.contains("## Version"),
        "TOC must NOT declare `## Version:` — unversioned"
    );
}

#[test]
fn blizzard_professions_toc_lists_bootstrap_then_thirteen_files_in_canonical_load_order() {
    let toc = TocFile::from_file(&professions_toc()).expect("Blizzard_Professions TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, PROFESSIONS_TOC_FILES,
        "Retail 12.1.0.69497 lists its bootstrap first, then eleven XML entries, the \
         registration Lua, and Localization. The XML entries continue to load paired Lua \
         files through Script directives"
    );
}

#[test]
fn blizzard_professions_does_not_appear_in_eager_discovery() {
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
            .any(|(name, _)| name == "Blizzard_Professions");
        assert!(
            !found,
            "Blizzard_Professions must NOT appear in eager discovery for {screen:?} — \
             `## LoadOnDemand: 1` excludes it from auto-load. discovered via explicit \
             LoadAddOn('Blizzard_Professions') when the player first opens a \
             profession window"
        );
    }
}

#[test]
fn blizzard_professions_appears_in_full_addon_inventory() {
    let inventory = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = inventory
        .iter()
        .any(|(name, _)| name == "Blizzard_Professions");
    assert!(
        found,
        "Blizzard_Professions must appear in `discover_all_blizzard_addons` — the \
         unfiltered inventory at src/loader/mod.rs lists every parseable Blizzard_* TOC \
         regardless of LoadOnDemand gating"
    );
}

prefork_full_ui_case! {
fn blizzard_professions_loads_explicitly_after_dependencies(env: &WowLuaEnv) {

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    let ui = blizzard_ui_dir();
    let templates_toc = ui
        .join("Blizzard_ProfessionsTemplates")
        .join("Blizzard_ProfessionsTemplates.toc");
    let shared_talent_toc = ui
        .join("Blizzard_SharedTalentUI")
        .join("Blizzard_SharedTalentUI.toc");
    load_addon(&env.loader_env(), &templates_toc)
        .expect("Blizzard_ProfessionsTemplates loads cleanly");
    load_addon(&env.loader_env(), &shared_talent_toc)
        .expect("Blizzard_SharedTalentUI loads cleanly");

    load_addon(&env.loader_env(), &professions_toc())
        .expect("Blizzard_Professions loads via Rust loader");

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_Professions") || message.contains("ProfessionsFrame")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_Professions emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_professions_publishes_ten_top_level_mixin_globals(env: &WowLuaEnv) {

    let ui = blizzard_ui_dir();
    let templates_toc = ui
        .join("Blizzard_ProfessionsTemplates")
        .join("Blizzard_ProfessionsTemplates.toc");
    let shared_talent_toc = ui
        .join("Blizzard_SharedTalentUI")
        .join("Blizzard_SharedTalentUI.toc");
    load_addon(&env.loader_env(), &templates_toc)
        .expect("Blizzard_ProfessionsTemplates loads cleanly");
    load_addon(&env.loader_env(), &shared_talent_toc)
        .expect("Blizzard_SharedTalentUI loads cleanly");
    load_addon(&env.loader_env(), &professions_toc()).expect("Blizzard_Professions loads cleanly");

    for name in PUBLIC_MIXIN_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G.{name})"))
            .unwrap_or_else(|err| panic!("type(_G.{name}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{name} must publish as a table — Blizzard_Professions exposes its 10 \
             page-level / panel-level mixins to global scope so the XML \
             `mixin=\"...\"` attributes on top-level frames resolve. ProfessionsMixin \
             owns the ProfessionsFrame container itself; ProfessionsCraftingPageMixin / \
             ProfessionsCraftingOrderPageMixin extend ProfessionsRecipeListPanelMixin \
             from Blizzard_ProfessionsTemplates; ProfessionsCrafterOrderViewMixin owns \
             the in-progress order detail view; ProfessionsCraftingOutputLogMixin \
             extends CallbackRegistryMixin for the crafting-result feed; \
             ProfessionsRankBarMixin / ProfessionsRecipeLevelBarMixin own the skill-up \
             progress bars; ProfessionsSpecFrameMixin / ProfessionsSpecPathMixin extend \
             TalentButtonSpendMixin from Blizzard_SharedTalentUI for the spec talent \
             tree; InspectRecipeMixin owns the popup-out InspectRecipeFrame inspector"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_professions_named_non_virtual_top_level_frames_are_in_global_env(env: &WowLuaEnv) {

    let ui = blizzard_ui_dir();
    let templates_toc = ui
        .join("Blizzard_ProfessionsTemplates")
        .join("Blizzard_ProfessionsTemplates.toc");
    let shared_talent_toc = ui
        .join("Blizzard_SharedTalentUI")
        .join("Blizzard_SharedTalentUI.toc");
    load_addon(&env.loader_env(), &templates_toc)
        .expect("Blizzard_ProfessionsTemplates loads cleanly");
    load_addon(&env.loader_env(), &shared_talent_toc)
        .expect("Blizzard_SharedTalentUI loads cleanly");
    load_addon(&env.loader_env(), &professions_toc()).expect("Blizzard_Professions loads cleanly");

    for frame in NAMED_NON_VIRTUAL_TOP_LEVEL_FRAMES {
        let kind: String = env
            .eval(&format!("return type(_G.{frame})"))
            .unwrap_or_else(|err| panic!("type(_G.{frame}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{frame} must exist as a frame table — Blizzard_Professions defines 2 \
             named non-virtual top-level frames: ProfessionsFrame (the main panel \
             inheriting PortraitFrameTemplateNoCloseButton + TabSystemOwnerTemplate, \
             parent=UIParent, toplevel=true, hidden=true initially, registered with \
             UIPanelLayout via Blizzard_ProfessionsRegistration.lua) and \
             InspectRecipeFrame (the popup-out recipe inspector, also registered as a \
             UI panel). Both are registered with UIPanelWindows via RegisterUIPanel \
             with attributes area=left / xoffset=35 / pushable=1 / allowOtherPanels=1 \
             / checkFit=1"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_professions_registers_two_ui_panels_with_uiparent(env: &WowLuaEnv) {

    let ui = blizzard_ui_dir();
    let templates_toc = ui
        .join("Blizzard_ProfessionsTemplates")
        .join("Blizzard_ProfessionsTemplates.toc");
    let shared_talent_toc = ui
        .join("Blizzard_SharedTalentUI")
        .join("Blizzard_SharedTalentUI.toc");
    load_addon(&env.loader_env(), &templates_toc)
        .expect("Blizzard_ProfessionsTemplates loads cleanly");
    load_addon(&env.loader_env(), &shared_talent_toc)
        .expect("Blizzard_SharedTalentUI loads cleanly");
    load_addon(&env.loader_env(), &professions_toc()).expect("Blizzard_Professions loads cleanly");

    let registration_present: bool = env
        .eval(
            "return UIPanelWindows ~= nil \
                and UIPanelWindows.ProfessionsFrame ~= nil \
                and UIPanelWindows.InspectRecipeFrame ~= nil",
        )
        .expect("UIPanelWindows query succeeds");
    assert!(
        registration_present,
        "UIPanelWindows.ProfessionsFrame and UIPanelWindows.InspectRecipeFrame must be \
         populated after load — Blizzard_ProfessionsRegistration.lua calls \
         RegisterUIPanel(ProfessionsFrame, attributes) and \
         RegisterUIPanel(InspectRecipeFrame, attributes) at the end of the addon load \
         sequence (it is the LAST file in the TOC body before Localization), wiring \
         both frames into the global UI-panel manager so /script ToggleProfessionsBook \
         + UIParent push/pop logic can reach them"
    );
}
}
