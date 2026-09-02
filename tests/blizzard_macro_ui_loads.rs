#![cfg(any(feature = "client-retail", feature = "client-ptr"))]
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

fn macro_ui_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_MacroUI")
}

fn macro_ui_toc() -> PathBuf {
    macro_ui_dir().join("Blizzard_MacroUI.toc")
}

const MACRO_UI_TOC_FILES: &[&str] = &[
    "Blizzard_MacroUI_Bootstrap.lua",
    "Blizzard_MacroDefine.lua",
    "Blizzard_MacroScrollFrame.xml",
    "Blizzard_MacroUI.xml",
    "Blizzard_MacroIconSelector.xml",
    "Localization.lua",
];

const MACRO_DEFINE_CONSTANTS: &[(&str, i64)] = &[
    ("MACRO_SCROLL_BAR_OFFSET_X", -14),
    ("MACRO_SCROLL_BAR_OFFSET_TOP", -7),
    ("MACRO_SCROLL_BAR_OFFSET_BOTTOM", 3),
    ("MACRO_TAB2_WIDTH", 150),
];

const MACRO_FRAME_MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "OnShow",
    "OnHide",
    "OnEvent",
    "RefreshIconDataProvider",
    "SelectTab",
    "ChangeTab",
    "SetAccountMacros",
    "SetCharacterMacros",
    "Update",
    "UpdateButtons",
    "GetMacroDataIndex",
    "SelectMacro",
    "GetSelectedIndex",
    "DeleteMacro",
    "HideDetails",
    "ShowDetails",
    "SaveMacro",
];

const MACRO_BUTTON_MIXIN_METHODS: &[&str] = &["OnLoad", "OnClick", "OnDragStart"];

const MACRO_POPUP_FRAME_MIXIN_METHODS: &[&str] = &[
    "OnShow",
    "OnHide",
    "Update",
    "CancelButton_OnClick",
    "OkayButton_OnClick",
    "GetMacroFrame",
    "UpdateMacroFramePanelWidth",
];

const MACRO_FREE_FUNCTIONS: &[&str] = &[
    "MacroFrame_Show",
    "MacroFrame_SaveMacro",
    "MacroFrameSaveButton_OnClick",
    "MacroFrameCancelButton_OnClick",
    "MacroNewButton_OnClick",
    "MacroEditButton_OnClick",
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

fn load_macro_ui(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &macro_ui_toc())
        .expect("Blizzard_MacroUI should load via explicit Rust loader call");
}

#[test]
fn blizzard_macro_ui_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&macro_ui_dir()).expect("Blizzard_MacroUI TOC should resolve");
    assert_eq!(
        resolved,
        macro_ui_toc(),
        "Blizzard_MacroUI ships a single bare TOC. The macro editor is retail-only via \
         `## AllowLoadGameType: mainline` but there is no `_Mainline.toc` filename variant — \
         `find_toc_file` resolves it via the bare-name fallback after the `_Mainline.toc` \
         lookup misses (the gating is encoded inside the TOC body, not in the filename)"
    );
}

#[test]
fn blizzard_macro_ui_toc_declares_load_on_demand_with_plunderstorm_optional_dep() {
    let toc = TocFile::from_file(&macro_ui_toc()).expect("Blizzard_MacroUI TOC parses");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_MacroUI declares `## LoadOnDemand: 1`. The macro editor is fetched on \
         demand by ShowUIPanel(MacroFrame) — typically triggered when the player opens the \
         macro micro-button or types /macro. Outside that flow the addon stays unloaded — \
         keeps the icon selector + scroll-box machinery off the boot path"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(
        toc.dependencies().is_empty(),
        "Blizzard_MacroUI declares ZERO `## Dependencies:`. The macro editor relies only on \
         the global runtime surface — UIPanelWindows / StaticPopupDialogs / ShowUIPanel / \
         PanelTemplates_* / GetNumMacros / GetMacroInfo / EditMacro / DeleteMacro / \
         CreateMacro / PickupMacro / C_Macro.GetMacroName / C_Macro.GetSelectedMacroIcon / \
         IconDataProviderMixin / IconSelectorPopupFrameTemplateMixin / ScrollBoxConstants / \
         GenerateClosure / EventRegistry. The XML inheritance (ButtonFrameTemplate / \
         SelectorButtonTemplate / IconSelectorPopupFrameTemplate / ScrollFrameTemplate / \
         TooltipBackdropTemplate / UIPanelButtonTemplate / PanelTopTabButtonTemplate / \
         ScrollBoxSelectorTemplate) is supplied by Blizzard_SharedXML which is itself eager-loaded"
    );
    assert_eq!(
        toc.optional_deps(),
        vec!["Blizzard_Plunderstorm".to_string()],
        "Blizzard_MacroUI declares `## OptionalDeps: Blizzard_Plunderstorm`"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables. Macro state lives in the engine-side macro tables (account vs \
         character split surfaced via GetNumMacros / GetMacroInfo), not in addon SVs"
    );
}

#[test]
fn blizzard_macro_ui_toc_declares_mainline_only_with_default_game_screen() {
    let toc = TocFile::from_file(&macro_ui_toc()).expect("Blizzard_MacroUI TOC parses");
    assert!(
        !toc.is_game_type_restricted(),
        "TOC declares `## AllowLoadGameType: mainline` — `is_game_type_restricted` \
         (src/toc.rs:294-302) returns false because `mainline` is one of the two values \
         (mainline / standard) that the simulator considers non-restricted (the simulator \
         targets retail mainline). Classic flavors carry their own macro UI in a different \
         addon"
    );
    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Blizzard_MacroUI omits `## AllowLoad:` — `allows_screen` (src/toc.rs:305-313) \
         returns true for Game when the metadata key is missing. The macro editor is an \
         in-game UI panel docked into UIPanelWindows[\"MacroFrame\"]"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Blizzard_MacroUI must NOT publish on glue screens. With no `## AllowLoad:` \
             declared, the default Game-only behavior keeps the macro editor out of the \
             login / character-select / character-create flow. (Screen tested: {screen:?})"
        );
    }
}

#[test]
fn blizzard_macro_ui_toc_raw_bytes_declare_load_on_demand_mainline_with_plunderstorm_optional_dep()
{
    let raw = std::fs::read_to_string(macro_ui_toc()).expect("Blizzard_MacroUI TOC reads");
    assert!(
        raw.contains("## LoadOnDemand: 1"),
        "TOC must declare `## LoadOnDemand: 1` exactly — keeps the macro editor out of the \
         Game-screen auto-discovery sweep until ShowUIPanel(MacroFrame) flips the load"
    );
    assert!(
        raw.contains("## AllowLoadGameType: mainline"),
        "TOC must declare `## AllowLoadGameType: mainline` exactly — the new macro editor \
         (with the IconSelectorPopupFrameTemplate-driven icon picker + the \
         ScrollBoxSelectorTemplate-driven macro list) is retail-only. Classic flavors ship \
         the legacy macro UI through a separate addon path"
    );
    assert!(
        !raw.contains("## Dependencies:"),
        "TOC must NOT declare required `## Dependencies:`"
    );
    assert!(
        raw.contains("## OptionalDeps: Blizzard_Plunderstorm"),
        "TOC must declare Blizzard_Plunderstorm as an optional dependency"
    );
    assert!(
        !raw.contains("## AllowLoad:"),
        "TOC must NOT declare `## AllowLoad:` — Game-screen-only is the implicit default. \
         Note: `## AllowLoadGameType:` (game-type filter) is a different metadata key from \
         `## AllowLoad:` (screen-mode filter); the addon declares the former, not the latter"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT declare `## SavedVariables:` — macro state is engine-managed"
    );
}

#[test]
fn blizzard_macro_ui_toc_lists_bootstrap_then_five_ui_files_in_dependency_order() {
    let toc = TocFile::from_file(&macro_ui_toc()).expect("Blizzard_MacroUI TOC parses");
    assert_eq!(
        toc.files
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        MACRO_UI_TOC_FILES,
        "TOC body must list the bootstrap, MacroDefine.lua, three XML files, then \
         Localization.lua. The bootstrap publishes MacroFrame_LoadUI/ShowMacroFrame before \
         the on-demand UI body, while XML Script directives load the mixin Lua files"
    );
}

#[test]
fn blizzard_macro_ui_directory_holds_nine_entries_with_bootstrap() {
    let entries = std::fs::read_dir(macro_ui_dir())
        .expect("Blizzard_MacroUI directory reads")
        .count();
    assert_eq!(
        entries, 9,
        "Directory must hold exactly 9 entries — the bare TOC, Bootstrap.lua, four remaining \
         Lua files, and three XML files"
    );
}

#[test]
fn blizzard_macro_ui_is_game_startup_publisher_only() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    assert!(
        game_addons
            .iter()
            .any(|(name, _)| name == "Blizzard_MacroUI"),
        "Blizzard_MacroUI remains `## LoadOnDemand: 1` but is selected on Game so its \
         bootstrap publishes MacroFrame_LoadUI before Blizzard_ClickBindingUI OnLoad"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        assert!(
            !addons.iter().any(|(name, _)| name == "Blizzard_MacroUI"),
            "Blizzard_MacroUI must remain absent from non-game discovery ({screen:?})"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_macro_ui_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_macro_ui(env);

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_MacroUI")
                || message.contains("Blizzard_MacroDefine")
                || message.contains("Blizzard_MacroIconSelector")
                || message.contains("MacroFrameMixin")
                || message.contains("MacroButtonMixin")
                || message.contains("MacroPopupFrameMixin")
                || message.contains("MacroFrameScrollFrameTemplate")
                || message.contains("MacroButtonTemplate")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_MacroUI emitted addon-specific Lua errors during explicit LoD load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_macro_ui_is_addon_loaded_after_explicit_lod_call(env: &WowLuaEnv) {
    load_macro_ui(env);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_MacroUI')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_MacroUI') must return true after the explicit \
         load_addon call — proves the LoD addon registers with the loaded-set after \
         ShowUIPanel(MacroFrame)-equivalent load_addon path runs"
    );
}
}

prefork_full_ui_case! {
fn blizzard_macro_ui_publishes_macro_define_constants(env: &WowLuaEnv) {
    load_macro_ui(env);

    for (name, expected) in MACRO_DEFINE_CONSTANTS {
        let value: i64 = env
            .eval(&format!("return {name}"))
            .unwrap_or_else(|err| panic!("constant {name} probe failed: {err}"));
        assert_eq!(
            value, *expected,
            "Blizzard_MacroDefine.lua must publish `{name} = {expected}` at `_G`. The 4 \
             constants drive layout: MACRO_SCROLL_BAR_OFFSET_X/_TOP/_BOTTOM are passed to \
             self.MacroSelector:AdjustScrollBarOffsets in MacroFrameMixin:OnLoad, and \
             MACRO_TAB2_WIDTH is passed to PanelTemplates_TabResize on the per-character tab"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_macro_ui_publishes_macro_frame_mixin_with_all_methods(env: &WowLuaEnv) {
    load_macro_ui(env);

    let kind: String = env
        .eval("return type(MacroFrameMixin)")
        .expect("MacroFrameMixin probe succeeds");
    assert_eq!(
        kind, "table",
        "Blizzard_MacroUI.lua must publish `MacroFrameMixin = {{}}` at `_G` — the mixin \
         table that MacroFrame's XML binds via `mixin=MacroFrameMixin`. 18 methods total: \
         4 lifecycle hooks (OnLoad / OnShow / OnHide / OnEvent), tab-management \
         (SelectTab / ChangeTab / SetAccountMacros / SetCharacterMacros), data-flow \
         (Update / UpdateButtons / GetMacroDataIndex / SelectMacro / GetSelectedIndex), \
         editing (DeleteMacro / SaveMacro), detail panel (HideDetails / ShowDetails), and \
         the icon-data-provider hook (RefreshIconDataProvider)"
    );

    for method in MACRO_FRAME_MIXIN_METHODS {
        let has_method: bool = env
            .eval(&format!(
                "return type(MacroFrameMixin.{method}) == 'function'"
            ))
            .expect("MacroFrameMixin method probe succeeds");
        assert!(
            has_method,
            "MacroFrameMixin.{method} must be a function — one of the 18 documented methods"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_macro_ui_publishes_macro_button_mixin_with_three_methods(env: &WowLuaEnv) {
    load_macro_ui(env);

    let kind: String = env
        .eval("return type(MacroButtonMixin)")
        .expect("MacroButtonMixin probe succeeds");
    assert_eq!(
        kind, "table",
        "Blizzard_MacroUI.lua must publish `MacroButtonMixin = {{}}` at `_G` — the mixin \
         that MacroButtonTemplate (a virtual Button inheriting SelectorButtonTemplate) \
         binds via `mixin=MacroButtonMixin`. 3 methods total: OnLoad calls \
         self:RegisterForDrag(\"LeftButton\"); OnClick chains SelectorButtonMixin.OnClick + \
         the click-binding-mode AddNewAction handshake; OnDragStart resolves the actual \
         macro index and calls PickupMacro"
    );

    for method in MACRO_BUTTON_MIXIN_METHODS {
        let has_method: bool = env
            .eval(&format!(
                "return type(MacroButtonMixin.{method}) == 'function'"
            ))
            .expect("MacroButtonMixin method probe succeeds");
        assert!(has_method, "MacroButtonMixin.{method} must be a function");
    }
}
}

prefork_full_ui_case! {
fn blizzard_macro_ui_publishes_macro_popup_frame_mixin_with_seven_methods(env: &WowLuaEnv) {
    load_macro_ui(env);

    let kind: String = env
        .eval("return type(MacroPopupFrameMixin)")
        .expect("MacroPopupFrameMixin probe succeeds");
    assert_eq!(
        kind, "table",
        "Blizzard_MacroIconSelector.lua must publish `MacroPopupFrameMixin = {{}}` at `_G` \
         — bound by MacroPopupFrame XML via `mixin=MacroPopupFrameMixin` on the \
         IconSelectorPopupFrameTemplate-inheriting Frame. 7 own methods total — the mixin \
         delegates to IconSelectorPopupFrameTemplateMixin for the heavy lifting via \
         explicit super-calls (Mixin:OnShow / OnHide / CancelButton_OnClick / \
         OkayButton_OnClick all chain to the template equivalents)"
    );

    for method in MACRO_POPUP_FRAME_MIXIN_METHODS {
        let has_method: bool = env
            .eval(&format!(
                "return type(MacroPopupFrameMixin.{method}) == 'function'"
            ))
            .expect("MacroPopupFrameMixin method probe succeeds");
        assert!(
            has_method,
            "MacroPopupFrameMixin.{method} must be a function"
        );
    }
}
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_macro_save_placeholder_is_replaced_by_lod_delegate() {
    let env = load_full_game_ui();

    let placeholder_is_noop: bool = env
        .eval(
            r#"
            local before = MacroFrame_SaveMacro
            MacroFrame = { calls = 0, SaveMacro = function(self) self.calls = self.calls + 1 end }
            before()
            return type(before) == "function" and MacroFrame.calls == 0
            "#,
        )
        .expect("startup MacroFrame_SaveMacro placeholder probe should succeed");
    assert!(placeholder_is_noop);

    load_addon(&env.loader_env(), &macro_ui_toc())
        .expect("Blizzard_MacroUI should replace the startup placeholder");

    let lod_delegate_called: bool = env
        .eval(
            r#"
            MacroFrame.calls = 0
            MacroFrame.SaveMacro = function(self) self.calls = self.calls + 1 end
            MacroFrame_SaveMacro()
            return MacroFrame.calls == 1
            "#,
        )
        .expect("LoD MacroFrame_SaveMacro delegate probe should succeed");
    assert!(lod_delegate_called);
}

prefork_full_ui_case! {
fn blizzard_macro_ui_publishes_six_free_functions_at_global_scope(env: &WowLuaEnv) {
    load_macro_ui(env);

    for fn_name in MACRO_FREE_FUNCTIONS {
        let kind: String = env
            .eval(&format!("return type({fn_name})"))
            .expect("free function probe succeeds");
        assert_eq!(
            kind, "function",
            "Blizzard_MacroUI.lua must publish `{fn_name}` at `_G` as a function. The 6 \
             free functions are wired as XML `<Script function=\"...\"/>` callbacks (the \
             save / cancel / new / edit click handlers) plus the 2 helper entry points \
             (MacroFrame_Show — used by /macro chat handler — and MacroFrame_SaveMacro — \
             used by other UI paths to flush pending edits)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_macro_ui_registers_static_popup_dialog_for_delete_confirmation(env: &WowLuaEnv) {
    load_macro_ui(env);

    let kind: String = env
        .eval("return type(StaticPopupDialogs and StaticPopupDialogs['CONFIRM_DELETE_SELECTED_MACRO'])")
        .expect("CONFIRM_DELETE_SELECTED_MACRO probe succeeds");
    assert_eq!(
        kind, "table",
        "Blizzard_MacroUI.lua must register `StaticPopupDialogs['CONFIRM_DELETE_SELECTED_MACRO']` \
         — the confirmation dialog wired to MacroDeleteButton's OnClick via \
         StaticPopup_Show. Carries 6 keys (text=CONFIRM_DELETE_MACRO, button1=OKAY, \
         button2=CANCEL, OnAccept that calls MacroFrame:DeleteMacro, timeout=0, \
         whileDead=1, showAlert=1)"
    );

    let on_accept_kind: String = env
        .eval("return type(StaticPopupDialogs['CONFIRM_DELETE_SELECTED_MACRO'].OnAccept)")
        .expect("OnAccept probe succeeds");
    assert_eq!(
        on_accept_kind, "function",
        "OnAccept must be a function — runs MacroFrame:DeleteMacro on confirmation"
    );
}
}

prefork_full_ui_case! {
fn blizzard_macro_ui_registers_ui_panel_window_entry_for_macro_frame(env: &WowLuaEnv) {
    load_macro_ui(env);

    let kind: String = env
        .eval("return type(UIPanelWindows and UIPanelWindows['MacroFrame'])")
        .expect("UIPanelWindows MacroFrame probe succeeds");
    assert_eq!(
        kind, "table",
        "Blizzard_MacroUI.lua line 2 must register `UIPanelWindows['MacroFrame']` — the \
         entry that ShowUIPanel(MacroFrame) reads to determine docking area + width. \
         Carries area=\"left\", pushable=1, whileDead=1, width=PANEL_DEFAULT_WIDTH"
    );

    let area: String = env
        .eval("return UIPanelWindows['MacroFrame'].area")
        .expect("area probe succeeds");
    assert_eq!(
        area, "left",
        "UIPanelWindows['MacroFrame'].area must be \"left\" — the macro editor docks into \
         the left UIPanel slot (the same slot character / spellbook / talent panels use)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_macro_ui_publishes_macro_frame_global_with_button_frame_template(env: &WowLuaEnv) {
    load_macro_ui(env);

    let frame_present: bool = env
        .eval("return MacroFrame ~= nil and type(MacroFrame.IsShown) == 'function'")
        .expect("MacroFrame frame probe succeeds");
    assert!(
        frame_present,
        "Blizzard_MacroUI.xml line 23 declares `<Frame name=\"MacroFrame\" \
         inherits=\"ButtonFrameTemplate\">` so the frame publishes as a global with the \
         standard frame-method surface (IsShown / Show / Hide / GetParent etc.)"
    );

    let parent_name: String = env
        .eval("return MacroFrame:GetParent():GetName()")
        .expect("MacroFrame parent probe succeeds");
    assert_eq!(
        parent_name, "UIParent",
        "MacroFrame XML sets `parent=\"UIParent\"` — the macro editor docks into UIParent"
    );

    let starts_hidden: bool = env
        .eval("return not MacroFrame:IsShown()")
        .expect("MacroFrame visibility probe succeeds");
    assert!(
        starts_hidden,
        "MacroFrame XML declares `hidden=\"true\"` — the macro editor must start hidden \
         and only show when ShowUIPanel(MacroFrame) is invoked (typically via the macro \
         micro-button or /macro chat handler)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_macro_ui_publishes_macro_popup_frame_global_parented_to_macro_frame(env: &WowLuaEnv) {
    load_macro_ui(env);

    let frame_present: bool = env
        .eval("return MacroPopupFrame ~= nil and type(MacroPopupFrame.IsShown) == 'function'")
        .expect("MacroPopupFrame frame probe succeeds");
    assert!(
        frame_present,
        "Blizzard_MacroIconSelector.xml line 5 declares `<Frame name=\"MacroPopupFrame\" \
         inherits=\"IconSelectorPopupFrameTemplate\">` so the icon-selector popup publishes \
         as a global"
    );

    let parent_name: String = env
        .eval("return MacroPopupFrame:GetParent():GetName()")
        .expect("MacroPopupFrame parent probe succeeds");
    assert_eq!(
        parent_name, "MacroFrame",
        "MacroPopupFrame XML sets `parent=\"MacroFrame\"` — the icon picker docks to the \
         right of the macro editor (TOPLEFT-anchored to TOPRIGHT of MacroFrame with x=0 \
         y=5)"
    );

    let starts_hidden: bool = env
        .eval("return not MacroPopupFrame:IsShown()")
        .expect("MacroPopupFrame visibility probe succeeds");
    assert!(
        starts_hidden,
        "MacroPopupFrame XML declares `hidden=\"true\"` — the icon picker only shows when \
         the player clicks New / Edit (which sets MacroPopupFrame.mode + calls Show)"
    );
}
}
