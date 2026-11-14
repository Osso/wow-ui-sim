use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::loader::{discover_all_blizzard_addons, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn quick_keybind_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_QuickKeybind")
}

fn quick_keybind_toc() -> PathBuf {
    quick_keybind_dir().join("Blizzard_QuickKeybind.toc")
}

const TOC_FILES: &[&str] = &["QuickKeybind.xml"];

const REQUIRED_DEPS: &[&str] = &["Blizzard_SettingsDefinitions_Frame"];

const PUBLIC_MIXIN_GLOBALS: &[&str] =
    &["QuickKeybindButtonTemplateMixin", "QuickKeybindFrameMixin"];

const PUBLIC_NAMED_FRAMES: &[&str] = &["QuickKeybindFrame", "QuickKeybindTooltip"];

const VIRTUAL_TEMPLATES: &[&str] = &["QuickKeybindButtonTemplate", "QuickKeybindFrameTemplate"];

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
        wow_ui_sim::loader::load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);

    env
}

// Literal cached inputs absent from Wrath manifest1-382/cache; behavior cases stay enabled.
post_wrath_ui_fixture! {
#[test]
fn blizzard_quick_keybind_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&quick_keybind_dir()).expect("Blizzard_QuickKeybind TOC resolves");
    assert_eq!(
        resolved,
        quick_keybind_toc(),
        "Blizzard_QuickKeybind ships a SINGLE bare `Blizzard_QuickKeybind.toc` \
         (NO `_Mainline.toc` / `_Mists.toc` / `_Classic.toc` variant). The TOC \
         carries NO `## AllowLoadGameType:` directive at all — meaning the \
         addon loads on EVERY game flavor (mainline / mists / classic / wrath \
         / cata) without a flavor gate. `find_toc_file` walks the \
         suffix-priority list `[_Mainline.toc, .toc]` and falls through to \
         the bare form because no Mainline-suffixed variant exists"
    );

    for variant_suffix in ["_Mainline.toc", "_Mists.toc", "_Wrath.toc", "_Classic.toc"] {
        let variant = quick_keybind_dir().join(format!("Blizzard_QuickKeybind{variant_suffix}"));
        assert!(
            !variant.exists(),
            "Blizzard_QuickKeybind must NOT ship a {variant_suffix} variant — \
             single bare TOC only with no flavor gate at all (cross-flavor \
             addon)"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_quick_keybind_toc_pins_eager_cross_flavor_with_default_state_enabled() {
    let toc = TocFile::from_file(&quick_keybind_toc()).expect("Blizzard_QuickKeybind TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "TOC must NOT declare `## LoadOnDemand:` — eager-loaded along with \
         Blizzard_SettingsDefinitions_Frame so the QuickKeybind dialog and \
         the per-action-button QuickKeybindButtonTemplate hover-binding \
         handlers are wired up at startup; the dialog itself stays \
         `hidden=true` until SettingsPanel opens it via the AdvancedOptions \
         `Settings.QuickKeybindInitializer` registered in \
         Blizzard_SettingsDefinitions_Frame/Mainline/KeybindingsOverrides.lua"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(!toc.is_ptr_only());

    assert!(
        !toc.is_game_type_restricted(),
        "TOC declares NO `## AllowLoadGameType:` directive — \
         `is_game_type_restricted()` at src/toc.rs:294-302 returns FALSE for \
         the empty/missing case (any flavor allowed). Distinct from \
         Blizzard_QuickJoin which pins `mainline` and from Blizzard_QuestTimer \
         which pins `classic`; QuickKeybind is genuinely cross-flavor — both \
         retail action bars and classic action bars expose the quick-keybind \
         affordance through the same dialog"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "TOC declares `## AllowLoad: Game` — `allows_screen` at src/toc.rs:308 \
         routes via `eq_ignore_ascii_case(\"game\")` so the capitalized form \
         resolves correctly. The dialog can ONLY exist in-world because it \
         iterates the live ActionButtonUtil pool"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Game-only screen gate must EXCLUDE {screen:?} — keybind \
             discovery requires the action button pool which only exists \
             post-PLAYER_ENTERING_WORLD"
        );
    }

    assert!(toc.optional_deps().is_empty());
    assert!(toc.load_with().is_empty());
    assert!(
        toc.saved_variables().is_empty(),
        "TOC must declare ZERO `## SavedVariables:` — keybinds persist via \
         the C_KeyBindings / SaveBindings backing store (game-side, not \
         addon-side). The dialog is purely a UI affordance over the live \
         GetCurrentBindingSet / GetBindingKeyForAction state"
    );
    assert!(toc.saved_variables_per_character().is_empty());
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_quick_keybind_toc_declares_one_dependency() {
    let toc = TocFile::from_file(&quick_keybind_toc()).expect("TOC parses");
    let dependencies = toc.dependencies();
    let deps: Vec<&str> = dependencies.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        deps, REQUIRED_DEPS,
        "TOC must declare exactly 1 hard dep: \
         Blizzard_SettingsDefinitions_Frame — that addon publishes the \
         `Settings.QuickKeybindInitializer` (set in \
         Mainline/KeybindingsOverrides.lua:65) which AdvancedOptions.lua then \
         mirrors into the SettingsPanel. The transitive chain also pulls in \
         Blizzard_SharedXML (where `KeybindFrames_InQuickKeybindMode()` is \
         defined at BindingUtil.lua:166) — the QuickKeybind dialog calls \
         that helper from every per-button OnEnter / OnClick / OnLeave / \
         OnMouseWheel script, so without the transitive Blizzard_SharedXML \
         load every script handler would fail with a nil global"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_quick_keybind_toc_declares_metadata_in_raw_bytes() {
    let raw = std::fs::read_to_string(quick_keybind_toc()).expect("TOC reads utf-8");

    assert!(raw.contains("## Title: Blizzard_QuickKeybind"));
    assert!(raw.contains("## Author: Blizzard Entertainment"));
    assert!(
        raw.contains("## DefaultState: enabled"),
        "TOC must declare `## DefaultState: enabled` — the AddOn list UI \
         shows the addon enabled-by-default for users who toggle it manually"
    );
    assert!(raw.contains("## Dependencies: Blizzard_SettingsDefinitions_Frame"));
    assert!(
        raw.contains("## AllowLoad: Game"),
        "TOC must declare `## AllowLoad: Game` (CAPITALIZED) — same shape as \
         Blizzard_QuickJoin. The parser at src/toc.rs:308 normalizes via \
         eq_ignore_ascii_case so capitalization is irrelevant at the gate"
    );

    assert!(
        !raw.contains("## AllowLoadGameType"),
        "TOC must NOT carry `## AllowLoadGameType:` at all — cross-flavor \
         addon, no game-type gate. This is the distinguishing trait that \
         separates QuickKeybind from QuickJoin (mainline-only) and \
         QuestTimer (classic-only) — all three sit in the same Blizzard_* \
         tree but only QuickKeybind ships with a missing game-type directive"
    );

    assert!(
        !raw.contains("## LoadOnDemand"),
        "TOC must NOT carry any LoadOnDemand directive (eager-loaded)"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT carry any SavedVariables directive"
    );
    assert!(
        !raw.contains("## OnlyBetaAndPTR"),
        "TOC must NOT carry OnlyBetaAndPTR — ships on live retail and live \
         classic"
    );
    assert!(
        !raw.contains("## Version"),
        "TOC must NOT carry a Version directive — one of the Blizzard_* \
         addons missing the canonical version line (same omission as \
         Blizzard_QuestTimer / Blizzard_QueueStatusFrame / Blizzard_QuickJoin)"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_quick_keybind_toc_lists_one_xml_file_with_companion_lua_pulled_in() {
    let toc = TocFile::from_file(&quick_keybind_toc()).expect("TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, TOC_FILES,
        "TOC body lists EXACTLY 1 file: QuickKeybind.xml. The companion \
         QuickKeybind.lua is NOT listed in the TOC body — instead it gets \
         pulled in by `<Script file=\"QuickKeybind.lua\"/>` at line 3 of \
         QuickKeybind.xml itself. This is the canonical XML-driven Lua \
         loading pattern (same shape as Blizzard_QuickJoin / \
         Blizzard_QuestNavigation / Blizzard_PVPMatch) and it differs from \
         the older eager-load form used by Blizzard_QuestTimer where both \
         .lua and .xml are listed explicitly in the TOC body"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_quick_keybind_appears_in_eager_game_discovery() {
    let ui = blizzard_ui_dir();

    let game_addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let game_found = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_QuickKeybind");
    assert!(
        game_found,
        "Blizzard_QuickKeybind MUST appear in eager Game-screen discovery: \
         no `## LoadOnDemand:` (so `is_load_on_demand()` false), no \
         `## AllowLoadGameType:` (so `is_game_type_restricted()` false on \
         all flavors), `## AllowLoad: Game` (so the Game screen gate \
         passes). All 3 loader filters at src/loader/mod.rs:527 admit it"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let glue_addons = discover_blizzard_addons_for_screen(&ui, screen);
        let glue_found = glue_addons
            .iter()
            .any(|(name, _)| name == "Blizzard_QuickKeybind");
        assert!(
            !glue_found,
            "Blizzard_QuickKeybind must NOT appear in eager discovery for \
             {screen:?} — the `## AllowLoad: Game` gate excludes glue-screen \
             load paths"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_quick_keybind_appears_in_full_addon_inventory() {
    let inventory = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = inventory
        .iter()
        .any(|(name, _)| name == "Blizzard_QuickKeybind");
    assert!(
        found,
        "Blizzard_QuickKeybind MUST appear in `discover_all_blizzard_addons`"
    );
}
}

prefork_full_ui_case! {
fn blizzard_quick_keybind_loads_in_eager_game_sweep_without_lua_errors(env: &WowLuaEnv) {

    let addon_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_QuickKeybind")
                || message.contains("QuickKeybindFrame")
                || message.contains("QuickKeybindButton")
                || message.contains("QuickKeybindTooltip")
        })
        .cloned()
        .collect();
    assert!(
        addon_errors.is_empty(),
        "Blizzard_QuickKeybind emitted addon-specific Lua errors during \
         eager load:\n  {}",
        addon_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_quick_keybind_publishes_two_mixin_globals(env: &WowLuaEnv) {

    for mixin in PUBLIC_MIXIN_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G[{mixin:?}])"))
            .unwrap_or_else(|err| panic!("type probe for {mixin} failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table — Blizzard_QuickKeybind ships \
             EXACTLY 2 public Mixin globals (no helper functions, no \
             namespace tables): QuickKeybindButtonTemplateMixin owns each \
             per-action-button hover/click affordance with the OnShow / \
             OnHide / OnClick / OnEnter / OnLeave script handlers — the \
             OnEnter handler swaps in the QuickKeybindHighlightTexture \
             alpha=1, sets up the QuickKeybindTooltip with a binding-name \
             header + escape-to-unbind instruction line, and re-routes the \
             button's OnUpdate to QuickKeybindButtonOnUpdate so the tooltip \
             follows the cursor when it would clip the GameTooltip; the \
             OnLeave handler restores the prior OnUpdate and dims the \
             highlight back to alpha=0.5; the OnClick handler routes \
             non-LeftButton/RightButton clicks into QuickKeybindFrame:OnKeyDown \
             so middle-mouse / mouse4 / mouse5 register as keybinds. \
             QuickKeybindFrameMixin owns the dialog itself with the OnLoad \
             handler wiring CancelButton/OkayButton/DefaultsButton/\
             UseCharacterBindingsButton click handlers + the \
             KeybindListener.UnbindFailed / RebindFailed / RebindSuccess \
             EventRegistry callbacks, and the OnKeyDown / OnMouseWheel \
             handlers routing input into KeybindListener:OnKeyDown / \
             :OnMouseWheel for the actual binding mutation. Source: \
             QuickKeybind.lua:2 and QuickKeybind.lua:104"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_quick_keybind_publishes_two_named_top_level_frames(env: &WowLuaEnv) {

    for frame_name in PUBLIC_NAMED_FRAMES {
        let kind: String = env
            .eval(&format!("return type(_G[{frame_name:?}])"))
            .unwrap_or_else(|err| panic!("type probe for {frame_name} failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{frame_name} must publish as a frame userdata — \
             Blizzard_QuickKeybind ships EXACTLY 2 named non-virtual \
             top-level frames in QuickKeybind.xml: QuickKeybindFrame is the \
             dialog (Button widget — Blizzard models the dialog as a Button \
             so it inherits the OnKeyDown / OnGamePadButtonDown / \
             OnMouseWheel script slots without an extra wrapper) at \
             frameStrata=DIALOG / hidden=true / movable=true / \
             clampedToScreen=true / dontSavePosition=true / protected=true / \
             registerForDrag=LeftButton inheriting QuickKeybindFrameTemplate \
             with mixin=QuickKeybindFrameMixin and parented to UIParent; \
             QuickKeybindTooltip is a SECOND GameTooltip (NOT the global \
             GameTooltip — a dedicated parallel tooltip) at toplevel=true \
             inheriting SharedTooltipTemplate, parented to UIParent, used \
             when the QuickKeybind dialog is open so the per-button tooltip \
             does NOT collide with the regular hover GameTooltip — the \
             QuickKeybindButtonOnUpdate handler explicitly checks for the \
             two overlapping and re-anchors `QuickKeybindTooltip` to \
             ANCHOR_TOP-of-GameTooltip when they would clip"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_quick_keybind_virtual_templates_not_in_global_env(env: &WowLuaEnv) {

    for template in VIRTUAL_TEMPLATES {
        let kind: String = env
            .eval(&format!("return type(_G[{template:?}])"))
            .unwrap_or_else(|err| panic!("type probe for {template} failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{template} must be nil — virtual templates live in the \
             template registry, NOT the global environment. The 2 virtual \
             templates Blizzard_QuickKeybind ships are: \
             QuickKeybindButtonTemplate — the per-action-button hover \
             affordance Button virtual template that publishes a single \
             OVERLAY-layer Texture (parentKey=QuickKeybindHighlightTexture, \
             atlas=UI-HUD-ActionBar-IconFrame-Mouseover, alphaMode=ADD, \
             alpha=0.4, hidden=true) and 5 script handlers \
             (OnShow/OnHide/OnClick/OnEnter/OnLeave) all dispatching to \
             QuickKeybindButtonTemplateMixin methods — instantiated by every \
             ActionButton via ActionButtonUtil.\
             ShowAllQuickKeybindButtonHighlights when the dialog opens; \
             QuickKeybindFrameTemplate — the dialog frame virtual template \
             (Button) carrying the 450-by-250 Size, the InstructionText / \
             CancelDescriptionText / OutputText FontStrings on the BORDER \
             layer, the BG (DialogBorderTemplate) + Header \
             (DialogHeaderTemplate with KeyValue textString=\
             QUICK_KEYBIND_MODE) + UseCharacterBindingsButton \
             (UICheckButtonTemplate) + DefaultsButton/CancelButton/\
             OkayButton (UIPanelButtonTemplate) child Frames, and 8 script \
             handlers including OnGamePadButtonDown which aliases to OnKeyDown"
        );
    }
}
}

#[test]
#[cfg(all(feature = "client-retail", feature = "optional-tests"))]
fn retail_quick_keybind_original_grid_and_highlights_survive_startup() {
    use wow_ui_sim::loader::{
        discover_blizzard_startup_addons_for_screen, register_load_with_addons,
    };

    let ui = blizzard_ui_dir();
    let env = WowLuaEnv::new().expect("create Retail Game environment");
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    env.exec(
        r#"
        assert(ActionButtonUtil.ShowAllActionButtonGrids == nil)
        assert(ActionButtonUtil.HideAllActionButtonGrids == nil)
        assert(ActionButtonUtil.SetAllQuickKeybindButtonHighlights == nil)
        assert(ActionButtonUtil.ShowAllQuickKeybindButtonHighlights == nil)
        assert(ActionButtonUtil.HideAllQuickKeybindButtonHighlights == nil)
    "#,
    )
    .expect("no simulator quick-keybind defaults");

    let addons = discover_blizzard_startup_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc) in [
        ("Blizzard_ActionBar", "Blizzard_ActionBar.toc"),
        ("Blizzard_QuickKeybind", "Blizzard_QuickKeybind.toc"),
    ] {
        assert_eq!(
            addons
                .iter()
                .find(|a| a.name == name)
                .unwrap()
                .toc_path
                .file_name()
                .unwrap(),
            toc
        );
    }
    register_load_with_addons(&env.loader_env(), &discover_all_blizzard_addons(&ui));
    crate::common::load_game_startup_inventory(&env);
    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);

    env.exec(r#"
        local function original(fn, path, line)
            local info = debug.getinfo(fn, "S")
            local source = info.source:gsub(string.char(92), '/')
            assert(source:find(path, 1, true), source)
            assert(info.linedefined == line, source .. ':' .. info.linedefined)
        end
        local owner = "Blizzard_ActionBar/Shared/ActionButtonUtil.lua"
        original(ActionButtonUtil.ShowAllActionButtonGrids, owner, 42)
        original(ActionButtonUtil.HideAllActionButtonGrids, owner, 47)
        original(ActionButtonUtil.ShowAllQuickKeybindButtonHighlights, owner, 56)
        original(ActionButtonUtil.HideAllQuickKeybindButtonHighlights, owner, 60)
        original(ActionButtonUtil.SetAllQuickKeybindButtonHighlights,
            "Blizzard_ActionBar/Mainline/ActionButtonUtilOverrides.lua", 16)
        local quickbind = "Blizzard_QuickKeybind/QuickKeybind.lua"
        original(QuickKeybindFrameMixin.OnShow, quickbind, 142)
        original(QuickKeybindFrameMixin.OnHide, quickbind, 159)
        local function hasGrid(bar)
            return bit.band(bar.showAllButtons or 0, ACTION_BUTTON_SHOW_GRID_REASON_EVENT) ~= 0
        end
        local function check(active)
            assert(hasGrid(MainActionBar) == active, "main bar grid")
            assert(hasGrid(MultiBarBottomLeft) == active, "multi bar grid")
            assert(ActionButton1.QuickKeybindHighlightTexture:IsShown() == active, "main button highlight")
            assert(MultiBarBottomLeftButton1.QuickKeybindHighlightTexture:IsShown() == active,
                "multi button highlight")
        end
        assert(not QuickKeybindFrame:IsShown(), "QuickKeybind.xml:89 starts hidden")
        check(false)
        QuickKeybindFrame:Show()
        assert(KeybindFrames_InQuickKeybindMode())
        check(true)
        QuickKeybindFrame:Hide()
        assert(not KeybindFrames_InQuickKeybindMode())
        check(false)
    "#).expect("the original QuickKeybind frame toggles grids and highlights");
}
