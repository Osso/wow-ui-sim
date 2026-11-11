use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
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

fn rpe_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_RPE_TurnStrafe")
}

fn rpe_toc() -> PathBuf {
    rpe_dir().join("Blizzard_RPE_TurnStrafe.toc")
}

const TOC_FILES: &[&str] = &["Blizzard_RPE_TurnStrafe.lua", "Blizzard_RPE_TurnStrafe.xml"];

const MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "OnShow",
    "OnHide",
    "OnEvent",
    "SetActiveStyle",
    "Refresh",
];

const VIRTUAL_TEMPLATES: &[&str] = &[
    "RPETurnStrafeStyleTypeTemplate",
    "RPETurnStrafeStyleFrameTemplate",
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

// Literal cached inputs absent from Wrath manifest1-382/cache; behavior cases stay enabled.
retail_ptr_forever_toc_fixture! {
#[test]
fn find_toc_file_resolves_bare_toc() {
    let resolved = find_toc_file(&rpe_dir()).expect("Blizzard_RPE_TurnStrafe TOC should resolve");
    assert_eq!(
        resolved,
        rpe_toc(),
        "Blizzard_RPE_TurnStrafe ships exactly one TOC at the bare \
         `Blizzard_RPE_TurnStrafe.toc` path — no `_Mainline` flavor split. find_toc_file at \
         src/loader/mod.rs:65 tries `_Mainline.toc` first (miss), then bare (hit). The \
         AllowLoadGameType=standard directive in the bare TOC restricts to mainline retail \
         already, so a separate `_Mainline.toc` would be redundant"
    );
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn toc_declares_eager_standard_only_with_zero_dependencies() {
    let toc = TocFile::from_file(&rpe_toc()).expect("Blizzard_RPE_TurnStrafe TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_RPE_TurnStrafe declares `## LoadOnDemand: 0` (explicit 0, not absence) — \
         is_load_on_demand at src/toc.rs:259-264 must return FALSE because the lua chunk \
         registers EventUtil.ContinueAfterAllEvents on PLAYER_ENTERING_WORLD/VARIABLES_LOADED/ \
         BINDINGS_LOADED and the StaticPopupDialogs[\"RPE_TURNSTRAFE_CHANGED\"] table must \
         publish before any addon can call StaticPopup_Show on that key. LOD would defer the \
         entire turn-strafe migration prompt past initial bindings load and break the prompt's \
         trigger chain"
    );
    assert_eq!(
        toc.is_game_type_restricted(),
        !toc_layout::admits_game_type("standard"),
        "## AllowLoadGameType: standard — `standard` is in the Retail/PTR allow-list, so \
         is_game_type_restricted returns FALSE there. The pinned Forever allow-list is \
         `camelot, mainline`, so the same directive restricts the addon on that profile"
    );
    assert!(
        toc.dependencies().is_empty(),
        "Blizzard_RPE_TurnStrafe declares ZERO hard dependencies — no `## Dependencies:`, no \
         `## RequiredDep:`, no `## RequiredDeps:`. The lua chunk DOES reference \
         GameDialogBaseMixin (defined in Blizzard_StaticPopup_Game), Settings.* (defined in \
         Blizzard_Settings), Enum.TurnStrafeStyle/BindingSet (generated stubs), and \
         EventRegistry/EventUtil (defined in SharedXML), but Blizzard intentionally omits the \
         declaration because all of those load into the global scope before this addon's \
         deferred OnNotify/UpdateBindings closures fire — the references resolve lazily at \
         event-callback time, not at module load"
    );
    assert!(
        toc.metadata.get("DefaultState").is_none(),
        "Blizzard_RPE_TurnStrafe omits `## DefaultState` — default_enabled at src/toc.rs:251-256 \
         returns TRUE when the metadata is absent. The addon must always be enabled because \
         the C_KeyBindings.UpdateTurnStrafeBindingsForCharacter call inside UpdateBindings is \
         load-bearing for the turn/strafe migration flow on every login"
    );
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn toc_lists_two_files_in_root_directory() {
    let toc = TocFile::from_file(&rpe_toc()).expect("Blizzard_RPE_TurnStrafe TOC should parse");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(
        listed, TOC_FILES,
        "TOC body must list exactly these 2 files in this order: Blizzard_RPE_TurnStrafe.lua \
         loads FIRST so RPETurnStrafeStyleMixin = CreateFromMixins(GameDialogBaseMixin) and \
         StaticPopupDialogs[\"RPE_TURNSTRAFE_CHANGED\"] publish before \
         Blizzard_RPE_TurnStrafe.xml's `mixin=\"RPETurnStrafeStyleMixin\"` attribute on \
         RPETurnStrafeStyleFrameTemplate resolves the mixin lookup"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn allows_screen_returns_true_only_for_game_when_allowload_absent() {
    let toc = TocFile::from_file(&rpe_toc()).expect("Blizzard_RPE_TurnStrafe TOC should parse");
    assert!(
        toc.metadata.get("AllowLoad").is_none(),
        "Blizzard_RPE_TurnStrafe omits `## AllowLoad` entirely — allows_screen at \
         src/toc.rs:305-313 falls through to the `None` arm which returns \
         (screen == ScreenKind::Game). Equivalent semantics to AllowLoad=Game without the \
         explicit declaration"
    );
    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Game screen must allow the addon — the implicit AllowLoad=Game default applies"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Glue screen {screen:?} must REJECT the addon — the absent AllowLoad falls into \
             the `None => screen == ScreenKind::Game` branch which is false for every glue \
             screen. Turn/strafe binding migration only happens in-game"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn eager_discovery_matches_game_type_and_screen_filters() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_RPE_TurnStrafe");
    assert_eq!(
        in_game,
        toc_layout::admits_game_type("standard"),
        "RPE TurnStrafe is admitted only when the active profile allows standard"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_RPE_TurnStrafe");
        assert!(
            !found,
            "Blizzard_RPE_TurnStrafe must NOT appear on glue screen {screen:?} — \
             allows_screen returns false because the absent AllowLoad falls through to \
             Game-only semantics"
        );
    }
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn root_directory_holds_lua_and_xml_next_to_toc() {
    let mut entries: Vec<String> = std::fs::read_dir(rpe_dir())
        .expect("Blizzard_RPE_TurnStrafe directory should read")
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| name != "Blizzard_RPE_TurnStrafe.toc")
        .collect();
    entries.sort();
    assert_eq!(
        entries,
        vec![
            "Blizzard_RPE_TurnStrafe.lua".to_string(),
            "Blizzard_RPE_TurnStrafe.xml".to_string(),
        ],
        "Blizzard_RPE_TurnStrafe/ root must hold exactly the lua/xml pair next to the TOC — no \
         localization stub and no per-flavor subdirectory"
    );
}
}

prefork_full_ui_case! {
fn loads_without_lua_errors(env: &WowLuaEnv) {
    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("RPE_TurnStrafe")
                || message.contains("RPETurnStrafeStyleMixin")
                || message.contains("RPETurnStrafeStyleFrameTemplate")
                || message.contains("RPETurnStrafeStyleTypeTemplate")
                || message.contains("RPE_TURNSTRAFE_CHANGED")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_RPE_TurnStrafe emitted Lua errors during eager Game-screen load. The addon \
         is small (107 lines lua + 87 lines xml) but the OnLoad path touches multiple subsystems \
         (StaticPopupDialogs registration, GameDialogBaseMixin call-chain, SettingsPanel \
         category lookup, C_KeyBindings.GetTurnStrafeStyle, EventUtil.ContinueAfterAllEvents \
         deferred dispatch) so any error is a real load failure:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_after_eager_sweep(env: &WowLuaEnv) {
    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_RPE_TurnStrafe')")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_RPE_TurnStrafe') must return true after the eager \
         Game sweep — confirms the loader registers the explicit-LOD-0 addon as part of the \
         eager discovery set"
    );
}
}

prefork_full_ui_case! {
fn rpe_turn_strafe_style_mixin_publishes_with_full_method_surface(env: &WowLuaEnv) {
    let kind: String = env
        .eval("return type(RPETurnStrafeStyleMixin)")
        .expect("type(RPETurnStrafeStyleMixin) probe should succeed");
    assert_eq!(
        kind, "table",
        "RPETurnStrafeStyleMixin must publish at `_G` as a table — line 19 declares \
         `RPETurnStrafeStyleMixin = CreateFromMixins(GameDialogBaseMixin)` so the mixin starts \
         as a shallow-copy of GameDialogBaseMixin (defined in Blizzard_StaticPopup_Game/ \
         GameDialog.lua) and then receives 6 method overrides (OnLoad/OnShow/OnHide/OnEvent/ \
         SetActiveStyle/Refresh) bound directly via `function RPETurnStrafeStyleMixin:Method()` \
         syntax"
    );
    for method in MIXIN_METHODS {
        let mtype: String = env
            .eval(&format!("return type(RPETurnStrafeStyleMixin.{method})"))
            .unwrap_or_else(|err| panic!("type(RPETurnStrafeStyleMixin.{method}) failed: {err}"));
        assert_eq!(
            mtype, "function",
            "RPETurnStrafeStyleMixin.{method} must be a function — OnLoad chains \
             GameDialogBaseMixin.OnLoad(self) explicitly (line 22, the call-chain pattern \
             rather than the CreateFromMixins-only inheritance) before wiring the close \
             button, legacy/modern card buttons, the character-vs-account subtitle, and \
             calling Refresh. OnShow/OnHide register/unregister UPDATE_BINDINGS so the dialog \
             refreshes when keybinds change while open. SetActiveStyle wraps \
             C_KeyBindings.SetTurnStrafeStyle + SaveBindings(GetCurrentBindingSet()). \
             Refresh reads C_KeyBindings.GetTurnStrafeStyle and toggles the ActiveLabel/ \
             ActivateButton visibility on the LegacyFrame and ModernFrame children, hiding \
             the popup entirely if the style is Custom (the migration-prompt sentinel value)"
        );
    }
}
}

prefork_full_ui_case! {
fn static_popup_dialog_registers_under_rpe_turnstrafe_changed_key(env: &WowLuaEnv) {
    let kind: String = env
        .eval("return type(StaticPopupDialogs and StaticPopupDialogs['RPE_TURNSTRAFE_CHANGED'])")
        .expect("StaticPopupDialogs probe should succeed");
    assert_eq!(
        kind, "table",
        "StaticPopupDialogs[\"RPE_TURNSTRAFE_CHANGED\"] must publish as a table — registered at \
         lines 1-17 with text=RPE_TURNSTRAFE_CHANGED, button1=RPE_TURNSTRAFE_REVIEW, \
         button2=CLOSE, OnAccept that opens the SettingsPanel keybinds category and expands the \
         Movement section, and whileDead=1 so the prompt remains accessible on death. The \
         OnNotify closure (line 89) calls StaticPopup_Show(\"RPE_TURNSTRAFE_CHANGED\") when \
         C_KeyBindings.GetTurnStrafeStyle returns Enum.TurnStrafeStyle.Custom — the migration \
         prompt path. The non-Custom path creates a fresh \
         RPETurnStrafeStyleFrameTemplate-instanced frame instead"
    );
    let on_accept_kind: String = env
        .eval("return type(StaticPopupDialogs['RPE_TURNSTRAFE_CHANGED'].OnAccept)")
        .expect("OnAccept probe should succeed");
    assert_eq!(
        on_accept_kind, "function",
        "StaticPopupDialogs[\"RPE_TURNSTRAFE_CHANGED\"].OnAccept must be a function — clicking \
         RPE_TURNSTRAFE_REVIEW invokes it to walk SettingsPanel:GetCategory(KEYBINDINGS_CATEGORY_ID) \
         then GetLayout, EnumerateInitializers, find the BINDING_HEADER_MOVEMENT initializer, \
         set its `data.expanded=true` flag, and call Settings.OpenToCategory to focus the \
         Keybinds → Movement section"
    );
}
}

prefork_full_ui_case! {
fn virtual_templates_stay_off_global_scope(env: &WowLuaEnv) {
    for template in VIRTUAL_TEMPLATES {
        let kind: String = env
            .eval(&format!("return type(_G['{template}'])"))
            .unwrap_or_else(|err| panic!("type(_G.{template}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "{template} must NOT publish at `_G` — declared as `virtual=\"true\"` in \
             Blizzard_RPE_TurnStrafe.xml so the loader keeps it in the template registry only, \
             never as a global. RPETurnStrafeStyleTypeTemplate is the 280×128 inner card with \
             Title/SubTitle/ActiveLabel FontStrings plus an ActivateButton (UIPanelButtonTemplate); \
             RPETurnStrafeStyleFrameTemplate is the 666×260 popup with mixin=RPETurnStrafeStyleMixin \
             inheriting from StaticPopupBaseTemplate. The actual frame is created on-demand at \
             OnNotify time via CreateFrame(\"FRAME\", nil, UIParent, \
             \"RPETurnStrafeStyleFrameTemplate\") — there is no named non-virtual frame in the XML"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn xml_declares_two_virtual_templates_and_zero_named_frames() {
    let xml_text = std::fs::read_to_string(rpe_dir().join("Blizzard_RPE_TurnStrafe.xml"))
        .expect("Blizzard_RPE_TurnStrafe.xml should read");
    assert!(
        xml_text.contains("name=\"RPETurnStrafeStyleTypeTemplate\"")
            && xml_text.contains("name=\"RPETurnStrafeStyleFrameTemplate\""),
        "Blizzard_RPE_TurnStrafe.xml must declare both virtual templates by name"
    );
    let virtual_count = xml_text.matches("virtual=\"true\"").count();
    assert_eq!(
        virtual_count, 2,
        "Both <Frame> declarations must carry `virtual=\"true\"` — the file is purely a \
         template registry, no concrete frames. Counted virtual=\"true\" occurrences must be \
         exactly 2 to detect accidental concrete-frame additions"
    );
    assert!(
        xml_text.contains("mixin=\"RPETurnStrafeStyleMixin\"")
            && xml_text.contains("inherits=\"StaticPopupBaseTemplate\""),
        "RPETurnStrafeStyleFrameTemplate must declare both `mixin=\"RPETurnStrafeStyleMixin\"` \
         and `inherits=\"StaticPopupBaseTemplate\"` — the mixin attribute is what triggers \
         RPETurnStrafeStyleMixin:OnLoad on instantiation; StaticPopupBaseTemplate provides the \
         base popup frame infrastructure (CloseButton, popup-special show/hide hooks)"
    );
    assert!(
        xml_text.contains("<OnShow method=\"OnShow\"/>")
            && xml_text.contains("<OnHide method=\"OnHide\"/>")
            && xml_text.contains("<OnEvent method=\"OnEvent\"/>"),
        "RPETurnStrafeStyleFrameTemplate must wire OnShow/OnHide/OnEvent via mixin-method \
         dispatch — these are the bindings that register/unregister UPDATE_BINDINGS so the \
         popup refreshes when keybinds change while it is visible"
    );
}
}

// The Mists/Era/Anniversary caches ship 218 addons each and Wrath ships 23;
// none contains Blizzard_RPE_TurnStrafe, so the pinned TOC facts above describe no file the
// classic loaders can read.
classic_cache_absent_toc_fixture! {
#[test]
fn blizzard_rpe_turnstrafe_toc_is_absent_from_classic_caches() {
    let dir = rpe_dir();
    assert!(
        !dir.is_dir(),
        "classic profile caches must not contain Blizzard_RPE_TurnStrafe"
    );
    assert!(wow_ui_sim::loader::find_toc_file(&dir).is_none());
}
}
