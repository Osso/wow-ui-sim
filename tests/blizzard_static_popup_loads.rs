use std::path::PathBuf;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path().expect("Blizzard UI cache should be available")
}

fn static_popup_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_StaticPopup")
}

fn static_popup_toc() -> PathBuf {
    static_popup_dir().join("Blizzard_StaticPopup.toc")
}

const PUBLISHED_MIXINS: &[&str] = &["StaticPopupElementMixin", "StaticPopupEditBoxMixin"];

const ELEMENT_MIXIN_METHODS: &[&str] = &[
    "SetOwningDialog",
    "GetOwningDialog",
    "GetOwningDialogInfo",
    "GetOwningDialogData",
];

const EDITBOX_MIXIN_METHODS: &[&str] = &[
    "OnAttributeChanged",
    "OnEnterPressed",
    "OnEscapePressed",
    "OnTextChanged",
    "ClearText",
];

const SHOW_HIDE_GLOBALS: &[&str] = &[
    "StaticPopup_Show",
    "StaticPopup_Hide",
    "StaticPopup_HideAll",
    "StaticPopup_HideAllExcept",
    "StaticPopup_HideExclusive",
    "StaticPopup_HideNotification",
    "StaticPopup_FindVisible",
    "StaticPopup_Visible",
    "StaticPopup_IsAnyDialogShown",
    "StaticPopup_IsLastDisplayedFrame",
    "StaticPopup_IsCustomGenericConfirmationShown",
    "StaticPopup_IsSpecial",
];

const QUEUE_GLOBALS: &[&str] = &[
    "StaticPopup_Queue",
    "StaticPopup_CheckQueuedDialogs",
    "StaticPopup_AddDialog",
    "StaticPopup_RemoveDialog",
    "StaticPopup_AddDefinition",
    "StaticPopup_AddShowCondition",
    "StaticPopup_SetButtonText",
];

const HANDLER_GLOBALS: &[&str] = &[
    "StaticPopup_OnUpdate",
    "StaticPopup_OnShow",
    "StaticPopup_OnHide",
    "StaticPopup_OnClick",
    "StaticPopup_OnKeyDown",
    "StaticPopup_OnHyperlinkClick",
    "StaticPopup_OnHyperlinkEnter",
    "StaticPopup_OnHyperlinkLeave",
    "StaticPopup_OnCloseButtonClicked",
    "StaticPopup_OnAcceptWithSpinner",
];

const LAYOUT_GLOBALS: &[&str] = &[
    "StaticPopup_SetUpPosition",
    "StaticPopup_ReparentDialogs",
    "StaticPopup_ResizeShownDialogs",
    "StaticPopup_SetFullScreenFrame",
    "StaticPopup_ClearFullScreenFrame",
    "StaticPopup_UpdateProgressBar",
    "StaticPopup_UpdateAll",
    "StaticPopup_UpdateSubText",
    "StaticPopup_ReleaseInsertedFrame",
    "StaticPopup_SetTimeLeft",
    "StaticPopup_SetProgressBarTime",
    "StaticPopup_ForEachShownDialog",
    "StaticPopup_EscapePressed",
];

const SPECIAL_GLOBALS: &[&str] = &[
    "StaticPopupSpecial_Show",
    "StaticPopupSpecial_Hide",
    "StaticPopupSpecial_Toggle",
];

const SHOW_HELPER_GLOBALS: &[&str] = &[
    "StaticPopup_ShowNotification",
    "StaticPopup_ShowGenericConfirmation",
    "StaticPopup_ShowCustomGenericConfirmation",
    "StaticPopup_ShowCustomGenericInputBox",
    "StaticPopup_ShowGenericDropdown",
    "StaticPopup_StandardConfirmationTextHandler",
    "StaticPopup_StandardNonEmptyTextHandler",
    "StaticPopup_StandardEditBoxOnEscapePressed",
];

fn fresh_env(screen: ScreenKind) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(screen);
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    env
}

fn load_full_ui_for(screen: ScreenKind) -> WowLuaEnv {
    let env = fresh_env(screen);

    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    env
}

#[test]
fn find_toc_file_resolves_bare_toc() {
    let resolved = find_toc_file(&static_popup_dir()).expect("StaticPopup TOC resolves");
    assert_eq!(
        resolved,
        static_popup_toc(),
        "Blizzard_StaticPopup ships a single bare \
         `Blizzard_StaticPopup.toc` (NO `_Mainline.toc` flavor variant) — \
         the addon is shared between mainline + classic + glue, with \
         flavor-specific dialog definitions delegated downstream to \
         Blizzard_StaticPopup_Game / Blizzard_StaticPopup_Glue siblings"
    );
}

#[test]
fn dependencies_are_empty() {
    let toc = TocFile::from_file(&static_popup_toc()).expect("TOC parses");

    assert!(
        toc.dependencies().is_empty(),
        "StaticPopup must declare ZERO `## Dependencies` — it is the \
         baseline dialog primitive that downstream addons \
         (Blizzard_StaticPopup_Game, Blizzard_StaticPopup_Glue, \
         Blizzard_FrameXML) consume, not the other way around. The \
         engine-level globals it touches at file scope (CreateFrame, \
         C_Glue.IsOnGlueScreen, SecureTypes.CreateSecureMap, \
         EventRegistry) are eager-FrameXML primitives, not addons. \
         Got: {:?}",
        toc.dependencies()
    );
}

#[test]
fn allow_load_both_resolves_to_all_four_screens() {
    let toc = TocFile::from_file(&static_popup_toc()).expect("TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: Both` MUST allow Game — allows_screen at \
         src/toc.rs:307 short-circuits to true for the literal `Both` \
         token (case-insensitive)"
    );
    assert!(
        toc.allows_screen(ScreenKind::Login),
        "`## AllowLoad: Both` MUST allow Login glue — StaticPopup is \
         consumed by GlueDialog as the underlying confirmation \
         primitive on the login screen"
    );
    assert!(toc.allows_screen(ScreenKind::CharacterSelect));
    assert!(toc.allows_screen(ScreenKind::CharacterCreate));
}

#[test]
fn no_game_type_restriction() {
    let toc = TocFile::from_file(&static_popup_toc()).expect("TOC parses");

    assert!(
        !toc.is_game_type_restricted(),
        "StaticPopup declares NO `## AllowLoadGameType` directive so \
         is_game_type_restricted() must fall through to the None-branch \
         at src/toc.rs:301 returning false. The dialog primitive is \
         shared across all flavors (mainline, classic, plunderstorm, \
         standard) — flavor-specific dialog definitions live in the \
         downstream Blizzard_StaticPopup_Game / Blizzard_StaticPopup_Glue \
         addons, not here"
    );
}

#[test]
fn toc_is_eager_with_no_secure_env_or_saved_vars() {
    let toc = TocFile::from_file(&static_popup_toc()).expect("TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "StaticPopup must NOT be LoadOnDemand — every other addon \
         (game-side AND glue-side) calls StaticPopup_Show / \
         StaticPopup_Hide / StaticPopupDialogs[…] = {{}} at file scope, \
         so the dispatcher table and Show/Hide globals must be \
         registered before any consumer addon's body executes"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.optional_deps().is_empty());
    assert!(toc.default_enabled());
}

#[test]
fn toc_raw_bytes_pin_three_metadata_directives() {
    let raw = std::fs::read_to_string(static_popup_toc()).expect("TOC reads utf-8");

    let expected_directives = [
        "## Title: Blizzard_StaticPopup",
        "## DefaultState: enabled",
        "## AllowLoad: Both",
    ];

    for directive in expected_directives {
        assert!(
            raw.contains(directive),
            "Raw bytes MUST pin the `{directive}` directive — \
             StaticPopup's TOC is small (3 metadata lines + 3 body \
             entries) so each directive is load-bearing"
        );
    }

    assert!(!raw.contains("## Author"));
    assert!(!raw.contains("## Version"));
    assert!(!raw.contains("## LoadOnDemand"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## RequiredDep"));
    assert!(!raw.contains("## OptionalDep"));
    assert!(!raw.contains("## Dependencies"));
    assert!(!raw.contains("## AllowLoadGameType"));
    assert!(!raw.contains("## UseSecureEnvironment"));
}

#[test]
fn body_lists_three_lua_files_in_dependency_order() {
    let toc = TocFile::from_file(&static_popup_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let expected = [
        "StaticPopup.lua",
        "SharedTemplates.lua",
        "SharedDialogDefs.lua",
    ];

    assert_eq!(
        body.len(),
        expected.len(),
        "Body must contain exactly 3 entries — the addon ships three \
         pure-Lua files (no XML, all dialog frames are constructed in \
         downstream consumer XML). Got: {body:?}"
    );

    for (i, want) in expected.iter().enumerate() {
        assert_eq!(
            &body[i], want,
            "Body entry {i}: expected {want}, got {}",
            body[i]
        );
    }

    assert!(
        body.iter().all(|f| f.ends_with(".lua")),
        "All three body entries MUST be .lua — StaticPopup ships zero \
         XML because it is a pure dispatcher library: \
         StaticPopup.lua publishes the dispatcher, SharedTemplates.lua \
         publishes the editbox/element mixins consumed by downstream \
         XML, and SharedDialogDefs.lua seeds the GENERIC_CONFIRMATION \
         dialog into StaticPopupDialogs"
    );
}

#[test]
fn body_orders_dispatcher_before_templates_before_definitions() {
    let toc = TocFile::from_file(&static_popup_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    assert_eq!(
        body[0], "StaticPopup.lua",
        "StaticPopup.lua MUST load FIRST — it publishes \
         StaticPopupDialogs (the empty dispatcher table) and \
         StaticPopup_AddDefinition. SharedDialogDefs.lua then writes \
         StaticPopupDialogs[\"GENERIC_CONFIRMATION\"] which would error \
         on nil-index if the dispatcher table were not already present"
    );
    assert_eq!(
        body[1], "SharedTemplates.lua",
        "SharedTemplates.lua MUST load BEFORE SharedDialogDefs.lua — \
         it publishes StaticPopupElementMixin / StaticPopupEditBoxMixin \
         (the mixin tables consumed by downstream XML's \
         mixin=\"StaticPopupEditBoxMixin\" attribute on dialog \
         template editboxes). Order of {{}} = {{}} reuses unordered, \
         but the file order pins the publication sequence"
    );
    assert_eq!(
        body[2], "SharedDialogDefs.lua",
        "SharedDialogDefs.lua MUST load LAST — it injects the \
         GENERIC_CONFIRMATION dialog definition with OnShow / \
         OnAccept / OnCancel handlers, requiring StaticPopupDialogs \
         from StaticPopup.lua to already be declared"
    );
}

#[test]
fn appears_in_eager_discovery_on_all_four_screens() {
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
            .any(|(name, _)| name == "Blizzard_StaticPopup");
        assert!(
            found,
            "`## AllowLoad: Both` MUST surface StaticPopup on the \
             {screen:?} screen eager discovery sweep — the dispatcher \
             must be registered before any glue OR game dialog \
             consumer attempts StaticPopup_Show"
        );
    }
}

prefork_full_ui_case! {
fn full_game_load_emits_no_addon_specific_lua_errors(env: &WowLuaEnv) {

    let errors = env.state().borrow().lua_errors.clone();
    let needles = [
        "StaticPopup.lua",
        "SharedTemplates.lua",
        "SharedDialogDefs.lua",
        "StaticPopup_Show",
        "StaticPopup_Hide",
        "StaticPopupElementMixin",
        "StaticPopupEditBoxMixin",
        "GENERIC_CONFIRMATION",
    ];

    let matched: Vec<&String> = errors
        .iter()
        .filter(|e| needles.iter().any(|n| e.contains(n)))
        .collect();

    assert!(
        matched.is_empty(),
        "Full Game-screen load must emit zero StaticPopup-specific \
         Lua errors. Found {} matching errors: {:#?}",
        matched.len(),
        matched
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn full_game_load_emits_no_addon_specific_lua_errors() {
    let env = load_full_ui_for(ScreenKind::Game);
    full_game_load_emits_no_addon_specific_lua_errors::run(&env);
}

prefork_full_ui_case! {
fn is_addon_loaded_reports_true_after_eager_sweep(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_StaticPopup')")
        .expect("IsAddOnLoaded query");
    assert!(
        loaded,
        "After the Game-screen eager sweep, \
         C_AddOns.IsAddOnLoaded('Blizzard_StaticPopup') must return \
         true — `## AllowLoad: Both` with no `## LoadOnDemand` and no \
         `## AllowLoadGameType` restriction means the addon is part \
         of the auto-loaded set"
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn is_addon_loaded_reports_true_after_eager_sweep() {
    let env = load_full_ui_for(ScreenKind::Game);
    is_addon_loaded_reports_true_after_eager_sweep::run(&env);
}

prefork_full_ui_case! {
fn publishes_two_mixin_tables_at_global_scope(env: &WowLuaEnv) {

    for mixin in PUBLISHED_MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} type probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must publish at `_G` as a table after \
             Blizzard_StaticPopup loads — SharedTemplates.lua creates \
             StaticPopupElementMixin = {{}} (line 1) and \
             StaticPopupEditBoxMixin = CreateFromMixins(…) (line 21) \
             at file scope"
        );
    }
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn publishes_two_mixin_tables_at_global_scope() {
    let env = load_full_ui_for(ScreenKind::Game);
    publishes_two_mixin_tables_at_global_scope::run(&env);
}

prefork_full_ui_case! {
fn element_mixin_carries_four_owning_dialog_accessors(env: &WowLuaEnv) {

    for method in ELEMENT_MIXIN_METHODS {
        let kind: String = env
            .eval(&format!("return type(StaticPopupElementMixin['{method}'])"))
            .unwrap_or_else(|err| panic!("StaticPopupElementMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "StaticPopupElementMixin.{method} must publish as a \
             function — the base mixin carries 4 owning-dialog \
             accessors that propagate the dialog reference upward \
             through GetParent chains so per-element script handlers \
             can reach dialog.data without re-walking the parent tree: \
             SetOwningDialog (assigns self.owningDialog), \
             GetOwningDialog (reads it back), GetOwningDialogInfo \
             (returns dialog.dialogInfo via dialog and-shortcut), and \
             GetOwningDialogData (returns dialog.data via dialog \
             and-shortcut)"
        );
    }
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn element_mixin_carries_four_owning_dialog_accessors() {
    let env = load_full_ui_for(ScreenKind::Game);
    element_mixin_carries_four_owning_dialog_accessors::run(&env);
}

prefork_full_ui_case! {
fn editbox_mixin_inherits_from_element_mixin_via_create_from_mixins(env: &WowLuaEnv) {

    for method in ELEMENT_MIXIN_METHODS {
        let kind: String = env
            .eval(&format!("return type(StaticPopupEditBoxMixin['{method}'])"))
            .unwrap_or_else(|err| panic!("StaticPopupEditBoxMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "StaticPopupEditBoxMixin.{method} must inherit from \
             StaticPopupElementMixin via \
             CreateFromMixins(StaticPopupElementMixin) at \
             SharedTemplates.lua:21. CreateFromMixins shallow-copies \
             every key from each parent into the new table, so the \
             editbox subtype must surface the 4 owning-dialog \
             accessors from the parent mixin in addition to its own 5 \
             editbox-specific methods"
        );
    }
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn editbox_mixin_inherits_from_element_mixin_via_create_from_mixins() {
    let env = load_full_ui_for(ScreenKind::Game);
    editbox_mixin_inherits_from_element_mixin_via_create_from_mixins::run(&env);
}

prefork_full_ui_case! {
fn editbox_mixin_carries_five_editbox_specific_methods(env: &WowLuaEnv) {

    for method in EDITBOX_MIXIN_METHODS {
        let kind: String = env
            .eval(&format!("return type(StaticPopupEditBoxMixin['{method}'])"))
            .unwrap_or_else(|err| panic!("StaticPopupEditBoxMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "StaticPopupEditBoxMixin.{method} must publish as a \
             function — the editbox mixin owns 5 methods covering \
             OnAttributeChanged (intercepts the `clear-editbox` \
             attribute by calling SetText('') + SetSecureText(false)), \
             OnEnterPressed (routes through \
             AutoCompleteEditBox_OnEnterPressed and \
             StaticPopupDialogs[which].EditBoxOnEnterPressed dispatch), \
             OnEscapePressed (StaticPopupDialogs[which].\
             EditBoxOnEscapePressed dispatch), OnTextChanged (gates on \
             AutoCompleteEditBox_OnTextChanged and dispatches \
             EditBoxOnTextChanged + toggles Instructions visibility), \
             and ClearText (sets the `clear-editbox` attribute that \
             OnAttributeChanged intercepts)"
        );
    }
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn editbox_mixin_carries_five_editbox_specific_methods() {
    let env = load_full_ui_for(ScreenKind::Game);
    editbox_mixin_carries_five_editbox_specific_methods::run(&env);
}

prefork_full_ui_case! {
fn static_popup_dialogs_dispatcher_table_is_published(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(StaticPopupDialogs)")
        .expect("StaticPopupDialogs probe");
    assert_eq!(
        kind, "table",
        "StaticPopupDialogs must publish as a global table at \
         StaticPopup.lua:21 — it is the central dispatcher map keyed \
         by `which` strings (eg \"GENERIC_CONFIRMATION\", \
         \"DELETE_GOOD_ITEM\") that all StaticPopup_* helpers walk to \
         find the dialog definition"
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn static_popup_dialogs_dispatcher_table_is_published() {
    let env = load_full_ui_for(ScreenKind::Game);
    static_popup_dialogs_dispatcher_table_is_published::run(&env);
}

prefork_full_ui_case! {
fn static_popup_timeout_sec_default_is_sixty_seconds(env: &WowLuaEnv) {

    let secs: f64 = env
        .eval("return StaticPopupTimeoutSec")
        .expect("StaticPopupTimeoutSec probe");
    assert_eq!(
        secs, 60.0,
        "StaticPopupTimeoutSec must default to 60 — set at \
         StaticPopup.lua:22 as the engine-wide timeout for dialogs \
         that don't override timeout in their definition table. The \
         dispatcher uses dialogInfo.timeout || 0 (StaticPopup.lua:377), \
         so this global is informational rather than load-bearing"
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn static_popup_timeout_sec_default_is_sixty_seconds() {
    let env = load_full_ui_for(ScreenKind::Game);
    static_popup_timeout_sec_default_is_sixty_seconds::run(&env);
}

prefork_full_ui_case! {
fn generic_confirmation_dialog_is_seeded_with_handlers(env: &WowLuaEnv) {

    let report: String = env
        .eval(
            "local d = StaticPopupDialogs['GENERIC_CONFIRMATION'] \
             if type(d) ~= 'table' then return 'not-table:'..type(d) end \
             local missing = {} \
             if type(d.OnShow) ~= 'function' then table.insert(missing, 'OnShow:'..type(d.OnShow)) end \
             if type(d.OnAccept) ~= 'function' then table.insert(missing, 'OnAccept:'..type(d.OnAccept)) end \
             if type(d.OnCancel) ~= 'function' then table.insert(missing, 'OnCancel:'..type(d.OnCancel)) end \
             if d.hideOnEscape ~= 1 then table.insert(missing, 'hideOnEscape:'..tostring(d.hideOnEscape)) end \
             if d.timeout ~= 0 then table.insert(missing, 'timeout:'..tostring(d.timeout)) end \
             if d.multiple ~= 1 then table.insert(missing, 'multiple:'..tostring(d.multiple)) end \
             if d.whileDead ~= 1 then table.insert(missing, 'whileDead:'..tostring(d.whileDead)) end \
             if d.wide ~= 1 then table.insert(missing, 'wide:'..tostring(d.wide)) end \
             if #missing == 0 then return 'OK' else return table.concat(missing, ',') end",
        )
        .expect("GENERIC_CONFIRMATION probe");
    assert_eq!(
        report, "OK",
        "StaticPopupDialogs['GENERIC_CONFIRMATION'] must be seeded by \
         SharedDialogDefs.lua with three handlers (OnShow / OnAccept / \
         OnCancel) and five behavior flags (hideOnEscape=1, timeout=0, \
         multiple=1, whileDead=1, wide=1). This is the catch-all \
         confirmation used by StaticPopup_ShowGenericConfirmation / \
         StaticPopup_ShowCustomGenericConfirmation when no purpose-\
         specific dialog has been declared yet. Report: {report}"
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn generic_confirmation_dialog_is_seeded_with_handlers() {
    let env = load_full_ui_for(ScreenKind::Game);
    generic_confirmation_dialog_is_seeded_with_handlers::run(&env);
}

prefork_full_ui_case! {
fn show_hide_globals_are_functions(env: &WowLuaEnv) {

    for name in SHOW_HIDE_GLOBALS {
        let kind: String = env
            .eval(&format!("return type({name})"))
            .unwrap_or_else(|err| panic!("{name} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "{name} must publish as a global function — the \
             show/hide/visibility surface comprises the \
             dispatcher's primary public API"
        );
    }
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn show_hide_globals_are_functions() {
    let env = load_full_ui_for(ScreenKind::Game);
    show_hide_globals_are_functions::run(&env);
}

prefork_full_ui_case! {
fn queue_management_globals_are_functions(env: &WowLuaEnv) {

    for name in QUEUE_GLOBALS {
        let kind: String = env
            .eval(&format!("return type({name})"))
            .unwrap_or_else(|err| panic!("{name} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "{name} must publish as a global function — the \
             queue/registry surface manages the queuedDialogInfo + \
             dialogFrames + showConditions tables that the dispatcher \
             reads on every Show call"
        );
    }
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn queue_management_globals_are_functions() {
    let env = load_full_ui_for(ScreenKind::Game);
    queue_management_globals_are_functions::run(&env);
}

prefork_full_ui_case! {
fn script_handler_globals_are_functions(env: &WowLuaEnv) {

    for name in HANDLER_GLOBALS {
        let kind: String = env
            .eval(&format!("return type({name})"))
            .unwrap_or_else(|err| panic!("{name} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "{name} must publish as a global function — XML script \
             handlers in downstream dialog templates wire OnUpdate / \
             OnShow / OnHide / OnClick / OnKeyDown directly to these \
             dispatcher globals via `script function=\"…\"` attributes"
        );
    }
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn script_handler_globals_are_functions() {
    let env = load_full_ui_for(ScreenKind::Game);
    script_handler_globals_are_functions::run(&env);
}

prefork_full_ui_case! {
fn layout_and_position_globals_are_functions(env: &WowLuaEnv) {

    for name in LAYOUT_GLOBALS {
        let kind: String = env
            .eval(&format!("return type({name})"))
            .unwrap_or_else(|err| panic!("{name} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "{name} must publish as a global function — the layout/\
             reposition surface drives the per-screen anchor chain \
             rebuild that stacks shown dialogs vertically from the \
             topOffset and reparents them on \
             UI.AlternateTopLevelParentChanged events"
        );
    }
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn layout_and_position_globals_are_functions() {
    let env = load_full_ui_for(ScreenKind::Game);
    layout_and_position_globals_are_functions::run(&env);
}

prefork_full_ui_case! {
fn special_dialog_globals_are_functions(env: &WowLuaEnv) {

    for name in SPECIAL_GLOBALS {
        let kind: String = env
            .eval(&format!("return type({name})"))
            .unwrap_or_else(|err| panic!("{name} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "{name} must publish as a global function — \
             StaticPopupSpecial_* operate on dialogs that are not \
             registered via StaticPopupDialogs (eg the Tabard frame, \
             ChannelInvite frame) and use AssignDialogFallbackID + \
             StaticPopup_SetUpPosition for placement"
        );
    }
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn special_dialog_globals_are_functions() {
    let env = load_full_ui_for(ScreenKind::Game);
    special_dialog_globals_are_functions::run(&env);
}

prefork_full_ui_case! {
fn show_helper_globals_are_functions(env: &WowLuaEnv) {

    for name in SHOW_HELPER_GLOBALS {
        let kind: String = env
            .eval(&format!("return type({name})"))
            .unwrap_or_else(|err| panic!("{name} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "{name} must publish as a global function — the \
             convenience helper surface short-circuits the common \
             show patterns: ShowGenericConfirmation/\
             ShowCustomGenericConfirmation route through the \
             pre-seeded GENERIC_CONFIRMATION definition; \
             ShowNotification dynamically synthesizes \
             NOTIFICATION_<type> definitions; the Standard*Handler \
             helpers attach to dialog editbox OnTextChanged \
             callbacks to gate the accept button on input validity"
        );
    }
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn show_helper_globals_are_functions() {
    let env = load_full_ui_for(ScreenKind::Game);
    show_helper_globals_are_functions::run(&env);
}
