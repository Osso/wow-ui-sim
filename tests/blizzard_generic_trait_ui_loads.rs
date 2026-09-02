#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn generic_trait_ui_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_GenericTraitUI")
}

fn generic_trait_ui_toc() -> PathBuf {
    generic_trait_ui_dir().join("Blizzard_GenericTraitUI.toc")
}

fn shared_talent_ui_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_SharedTalentUI/Blizzard_SharedTalentUI.toc")
}

fn spell_search_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_SpellSearch/Blizzard_SpellSearch.toc")
}

fn load_generic_trait_ui_with_dependency(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &spell_search_toc())
        .expect("Blizzard_SpellSearch (LoD dep of Blizzard_SharedTalentUI) should load");
    load_addon(&env.loader_env(), &generic_trait_ui_toc())
        .expect("Blizzard_GenericTraitUI should load via Rust loader");
}

#[test]
fn blizzard_generic_trait_ui_resolves_bare_toc() {
    let resolved = find_toc_file(&generic_trait_ui_dir())
        .expect("Blizzard_GenericTraitUI directory must contain a discoverable TOC");
    let resolved_name = resolved
        .file_name()
        .expect("resolved TOC must have a filename")
        .to_str()
        .expect("resolved TOC filename must be utf-8");

    assert_eq!(
        resolved_name, "Blizzard_GenericTraitUI.toc",
        "Blizzard_GenericTraitUI ships only the bare `Blizzard_GenericTraitUI.toc` (no \
         `_Mainline.toc` variant); src/loader/mod.rs:65's `find_toc_file` falls through the \
         `_Mainline.toc` lookup and resolves the bare suffix"
    );
}

#[test]
fn blizzard_generic_trait_ui_toc_declares_lod_with_single_shared_talent_dep() {
    let toc =
        TocFile::from_file(&generic_trait_ui_toc()).expect("Blizzard_GenericTraitUI TOC parse");

    assert!(
        toc.is_load_on_demand(),
        "Blizzard_GenericTraitUI declares `## LoadOnDemand: 1` — the generic trait frame is \
         pulled in lazily by callers like Skyriding (treeID 672), DRIVE (1056), Visions \
         (1057), TitanConsole (1061), and ZulAmanLoaBlessing (1166), each of which calls \
         `LoadAddOn(\"Blizzard_GenericTraitUI\")` only when the player opens that specific \
         tree NPC interaction"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_GenericTraitUI does not declare `## UseSecureEnvironment` — trait \
         configuration commits go through C_Traits which routes its own protected actions, \
         the addon itself does not need a secure-environment surface"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_GenericTraitUI declares no `## SavedVariables` — trait selection state is \
         server-authoritative via C_Traits configIDs, never persisted client-side"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_GenericTraitUI declares no `## AllowLoadGameType` — \
         is_game_type_restricted() returns false because the absence of the directive means \
         the addon is allowed under all game types (src/toc.rs:299 only restricts when an \
         explicit non-`mainline`/`standard` game type is set)"
    );

    let deps = toc.dependencies();
    assert_eq!(
        deps,
        vec!["Blizzard_SharedTalentUI".to_string()],
        "Blizzard_GenericTraitUI declares exactly ONE dependency, \
         `## Dependencies: Blizzard_SharedTalentUI`, because GenericTraitFrameMixin extends \
         TalentFrameBaseMixin (Blizzard_SharedTalentUI/Blizzard_SharedTalentFrameBase.lua) \
         and consumes TalentFrameUtil + TalentFrameBaseTemplate from that addon. Got: {deps:?}"
    );
}

#[test]
fn blizzard_generic_trait_ui_toc_lists_bootstrap_two_lua_files_and_one_xml() {
    let toc_text = std::fs::read_to_string(generic_trait_ui_toc())
        .expect("Blizzard_GenericTraitUI TOC should read");
    let lua_count = toc_text.matches(".lua").count();
    let xml_count = toc_text.matches(".xml").count();
    assert_eq!(
        lua_count, 3,
        "Retail 12.1.0.69497 lists the bootstrap, Blizzard_GenericTraitUtil.lua, and \
         Blizzard_GenericTraitFrame.lua. Got: {lua_count}"
    );
    assert_eq!(
        xml_count, 1,
        "Blizzard_GenericTraitUI TOC enumerates exactly 1 .xml file \
         (Blizzard_GenericTraitFrame.xml — owns the toplevel `GenericTraitFrame` Frame \
         instance with mixin=GenericTraitFrameMixin, inherits TalentFrameBaseTemplate, plus \
         the FxModelScene / Currency button / Header / Inset / NineSlice / CloseButton \
         children). Got: {xml_count}"
    );
    assert!(
        toc_text.contains("Blizzard_GenericTraitUI_Bootstrap.lua [Bootstrap]"),
        "Retail 12.1.0.69497 lists the GenericTrait bootstrap before its regular files"
    );
    assert!(
        toc_text.contains("Blizzard_GenericTraitUtil.lua"),
        "TOC must list the Util .lua after the bootstrap and before GenericTraitFrame.lua — \
         its GenericTraitUtil namespace is consumed at line 119 of Blizzard_GenericTraitFrame.lua \
         (`GenericTraitUtil.GetFrameLayoutInfo(treeID)`) and at the XML KeyValue \
         `getEdgeTemplateType=GenericTraitUtil.GetEdgeTemplateType` (line 5 of \
         Blizzard_GenericTraitFrame.xml), so file order matters"
    );
}

#[test]
fn blizzard_generic_trait_ui_excluded_from_game_auto_discovery_due_to_lod() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_GenericTraitUI");
    assert!(
        !in_game,
        "Blizzard_GenericTraitUI declares `## LoadOnDemand: 1` — it must NOT appear in \
         Game-screen auto-discovery; callers (the Skyriding NPC, DRIVE module, Visions \
         interaction, TitanConsole UI, ZulAman Loa Blessing NPC) invoke \
         `LoadAddOn(\"Blizzard_GenericTraitUI\")` lazily when that specific tree opens"
    );

    let login_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Login);
    let in_login = login_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_GenericTraitUI");
    assert!(
        !in_login,
        "Blizzard_GenericTraitUI must also be absent from Login auto-discovery — without \
         `## AllowLoad: Glue/Both`, LoadOnDemand addons default to game-screen-only \
         eligibility"
    );
}

prefork_full_ui_case! {
fn blizzard_generic_trait_ui_loads_explicitly_via_load_addon_without_errors(env: &WowLuaEnv) {

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_addon(&env.loader_env(), &spell_search_toc())
        .expect("Blizzard_SpellSearch (LoD dep of Blizzard_SharedTalentUI) should load");
    load_addon(&env.loader_env(), &generic_trait_ui_toc())
        .expect("Blizzard_GenericTraitUI should load via Rust loader");

    let load_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let trait_errors: Vec<String> = load_errors
        .iter()
        .filter(|message| {
            message.contains("GenericTrait") || message.contains("Blizzard_GenericTraitUI")
        })
        .cloned()
        .collect();
    assert!(
        trait_errors.is_empty(),
        "Blizzard_GenericTraitUI emitted Lua errors during explicit load:\n  {}",
        trait_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_generic_trait_ui_is_addon_loaded_returns_true_after_explicit_load(env: &WowLuaEnv) {

    let before: bool = env
        .eval("return C_AddOns and C_AddOns.IsAddOnLoaded('Blizzard_GenericTraitUI') or false")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        !before,
        "IsAddOnLoaded should return false BEFORE explicit LoadAddOn — LoadOnDemand keeps \
         Blizzard_GenericTraitUI out of auto-discovery"
    );

    load_addon(&env.loader_env(), &spell_search_toc()).expect("Blizzard_SpellSearch should load");
    load_addon(&env.loader_env(), &generic_trait_ui_toc())
        .expect("Blizzard_GenericTraitUI should load");

    let after: bool = env
        .eval("return C_AddOns and C_AddOns.IsAddOnLoaded('Blizzard_GenericTraitUI') or false")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        after,
        "C_AddOns.IsAddOnLoaded('Blizzard_GenericTraitUI') must return true AFTER explicit \
         LoadAddOn (the loader registers the addon's name + state in the addon-info table \
         that backs IsAddOnLoaded)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_shared_talent_ui_is_non_lod_dep_loaded_by_game_auto_discovery(env: &WowLuaEnv) {
    let toc =
        TocFile::from_file(&shared_talent_ui_toc()).expect("Blizzard_SharedTalentUI TOC parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_SharedTalentUI declares `## LoadOnDemand: 0` — it loads automatically with \
         the Game-screen auto-discovery, so Blizzard_GenericTraitUI's only declared dep is \
         already present at the moment LoadAddOn(\"Blizzard_GenericTraitUI\") fires"
    );

    let shared_loaded: bool = env
        .eval("return C_AddOns and C_AddOns.IsAddOnLoaded('Blizzard_SharedTalentUI') or false")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        shared_loaded,
        "Blizzard_SharedTalentUI must be loaded by the standard Game-screen auto-discovery \
         pass before Blizzard_GenericTraitUI is loaded — that's why GenericTraitUI's TOC \
         only enumerates SharedTalentUI as a dep (the dep itself is non-LOD and pulled in \
         eagerly)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_generic_trait_ui_publishes_util_namespace_with_layout_accessors(env: &WowLuaEnv) {
    load_generic_trait_ui_with_dependency(env);

    let util: (String, bool, bool, bool, bool, bool) = env
        .eval(
            "return type(GenericTraitUtil), \
                    type(GenericTraitUtil.GetEdgeTemplateType) == 'function', \
                    type(GenericTraitUtil.GetFrameLayoutInfo) == 'function', \
                    type(GenericTraitUtil.AddFrameLayoutInfo) == 'function', \
                    type(GenericTraitUtil.GetFrameTutorialInfo) == 'function', \
                    type(GenericTraitUtil.GetCurrencyTutorialInfo) == 'function'",
        )
        .expect("GenericTraitUtil namespace probe should succeed");
    assert_eq!(
        util,
        ("table".to_string(), true, true, true, true, true),
        "Blizzard_GenericTraitUtil.lua publishes the GenericTraitUtil namespace (line 114) \
         with 5 functions backed by module-local tables: GetEdgeTemplateType (resolves \
         Enum.TraitEdgeVisualStyle.Straight to the TalentEdgeArrowTemplate string for the \
         GenericTraitFrame edge mesh), GetFrameLayoutInfo (returns a metatabled layout-info \
         struct that falls through to the Default layout via __index), AddFrameLayoutInfo \
         (lets external callers register layout overrides keyed by traitTreeID), \
         GetFrameTutorialInfo (returns the optional first-open tutorial copy keyed by \
         treeID), GetCurrencyTutorialInfo (returns the dragonriding/2563-style currency \
         tutorial keyed by treeID)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_generic_trait_ui_layout_accessor_returns_default_metatabled_struct(env: &WowLuaEnv) {
    load_generic_trait_ui_with_dependency(env);

    let layout: (bool, String, String, bool) = env
        .eval(
            "local layout = GenericTraitUtil.GetFrameLayoutInfo(672); \
             local mt = getmetatable(layout); \
             return mt ~= nil and type(mt.__index) == 'table', \
                    tostring(layout.PanelArea), \
                    tostring(layout.BackgroundAtlas), \
                    layout.HideCurrencyDisplay == false",
        )
        .expect("Layout-info probe should succeed");
    assert_eq!(
        layout.0, true,
        "GetFrameLayoutInfo must return a metatabled layout struct so unset fields fall \
         through to GenericTraitFrameLayoutOptions.Default — Blizzard_GenericTraitUtil.lua \
         line 133 wires `setmetatable(layoutInfo, {{__index = \
         GenericTraitFrameLayoutOptions.Default}})`"
    );
    assert_eq!(
        layout.1, "left",
        "The Default layout's PanelArea = 'left' (line 24 of Blizzard_GenericTraitUtil.lua) \
         must be visible through __index when reading from the Skyriding (672) layout, \
         which leaves PanelArea unset to inherit the Default value"
    );
    assert_eq!(
        layout.2, "ui-frame-dragonflight-backgroundtile",
        "Skyriding's own BackgroundAtlas = 'ui-frame-dragonflight-backgroundtile' (line 30) \
         must take precedence over Default.BackgroundAtlas = \
         'ui-frame-midnight-backgroundtile' (line 14) — direct fields shadow __index"
    );
    assert_eq!(
        layout.3, true,
        "The Default layout's HideCurrencyDisplay = false (line 25) must be visible \
         through __index when reading from the Skyriding (672) layout, proving false-valued \
         fields aren't accidentally treated as missing"
    );
}
}

prefork_full_ui_case! {
fn blizzard_generic_trait_ui_publishes_frame_mixin_with_full_lifecycle(env: &WowLuaEnv) {
    load_generic_trait_ui_with_dependency(env);

    let mixin: (bool, bool, bool, bool, bool, bool, bool, bool) = env
        .eval(
            "return type(GenericTraitFrameMixin) == 'table', \
                    type(GenericTraitFrameMixin.OnLoad) == 'function', \
                    type(GenericTraitFrameMixin.OnShow) == 'function', \
                    type(GenericTraitFrameMixin.OnHide) == 'function', \
                    type(GenericTraitFrameMixin.OnEvent) == 'function', \
                    type(GenericTraitFrameMixin.ApplyLayout) == 'function', \
                    type(GenericTraitFrameMixin.SetTreeID) == 'function', \
                    type(GenericTraitFrameMixin.SetConfigIDBySystemID) == 'function'",
        )
        .expect("GenericTraitFrameMixin probe should succeed");
    assert_eq!(
        mixin,
        (true, true, true, true, true, true, true, true),
        "Blizzard_GenericTraitFrame.lua publishes GenericTraitFrameMixin (line 14) which \
         extends TalentFrameBaseMixin via explicit calls (TalentFrameBaseMixin.OnLoad(self), \
         TalentFrameBaseMixin.OnShow(self), TalentFrameBaseMixin.OnHide(self), \
         TalentFrameBaseMixin.OnEvent(self, event, ...)). The 7 lifecycle methods drive the \
         canonical frame flow: OnLoad chains to TalentFrameBaseMixin.OnLoad; ApplyLayout \
         applies a GenericTraitUtil.GetFrameLayoutInfo(treeID) result to the frame; OnShow \
         registers GenericTraitFrameEvents (TRAIT_SYSTEM_NPC_CLOSED, \
         TRAIT_TREE_CURRENCY_INFO_UPDATED) via FrameUtil + triggers \
         EventRegistry:TriggerEvent('GenericTraitFrame.OnShow'); OnHide unregisters the \
         events + clears C_PlayerInteractionManager interaction; SetTreeID resolves the \
         layout and triggers GenericTraitFrame.SetTreeID; SetConfigIDBySystemID triggers \
         GenericTraitFrame.SetSystemID with the resolved configID"
    );
}
}

prefork_full_ui_case! {
fn blizzard_generic_trait_ui_purchase_and_selection_methods_have_current_owners(env: &WowLuaEnv) {
    load_generic_trait_ui_with_dependency(env);

    let methods: (bool, bool, bool, bool, bool, bool, bool, bool) = env
        .eval(
            "return type(AutoCommitTraitFrameMixin.PurchaseRank) == 'function', \
                    type(AutoCommitTraitFrameMixin.PurchaseRankCallback) == 'function', \
                    type(AutoCommitTraitFrameMixin.SetSelection) == 'function', \
                    type(AutoCommitTraitFrameMixin.SetSelectionCallback) == 'function', \
                    type(AutoCommitTraitFrameMixin.AttemptConfigOperation) == 'function', \
                    type(AutoCommitTraitFrameMixin.CheckAndReportCommitOperation) == 'function', \
                    type(TalentFrameBaseMixin.GetConfigCommitErrorString) == 'function', \
                    type(GenericTraitFrameMixin.UpdateTreeCurrencyInfo) == 'function'",
        )
        .expect("Purchase/selection method-owner probe should succeed");
    assert_eq!(
        methods,
        (true, true, true, true, true, true, true, true),
        "AutoCommitTraitFrameMixin owns the purchase, selection, and auto-commit methods \
         inherited by GenericTraitFrame through AutoCommitTraitFrameTemplate. \
         TalentFrameBaseMixin owns the overridable commit-error string method, while \
         GenericTraitFrameMixin directly owns its currency-display refresh override"
    );
}
}

prefork_full_ui_case! {
fn blizzard_generic_trait_ui_publishes_currency_frame_mixin(env: &WowLuaEnv) {
    load_generic_trait_ui_with_dependency(env);

    let currency: (bool, bool, bool, bool) = env
        .eval(
            "return type(GenericTraitFrameCurrencyFrameMixin) == 'table', \
                    type(GenericTraitFrameCurrencyFrameMixin.UpdateWidgetSet) == 'function', \
                    type(GenericTraitFrameCurrencyFrameMixin.Setup) == 'function', \
                    type(GenericTraitFrameCurrencyFrameMixin.OnEnter) == 'function'",
        )
        .expect("Currency frame mixin probe should succeed");
    assert_eq!(
        currency,
        (true, true, true, true),
        "Blizzard_GenericTraitFrame.lua publishes GenericTraitFrameCurrencyFrameMixin (line \
         346) — the mixin attached to the GenericTraitFrame.Currency button via \
         `mixin=GenericTraitFrameCurrencyFrameMixin` in the XML. UpdateWidgetSet pulls the \
         widget-set ID from C_Traits.GetTraitSystemWidgetSetID and configures the \
         UIWidgetContainer; Setup wires the unspent-points text + currency icon based on \
         the layout info; OnEnter routes the GameTooltip via GameTooltip_AddWidgetSet to \
         show the per-tree currency tooltip"
    );
}
}

prefork_full_ui_case! {
fn blizzard_generic_trait_ui_registers_uipanelwindows_metadata(env: &WowLuaEnv) {
    load_generic_trait_ui_with_dependency(env);

    let panel: (String, f64, String) = env
        .eval(
            "local meta = UIPanelWindows['GenericTraitFrame']; \
             return tostring(meta and meta.area or '<missing>'), \
                    (meta and meta.checkFit or 0), \
                    tostring(meta and meta.allowOtherPanels or '<missing>')",
        )
        .expect("UIPanelWindows registration probe should succeed");
    assert_eq!(
        panel.0, "left",
        "Blizzard_GenericTraitFrame.lua line 2 registers \
         UIPanelWindows['GenericTraitFrame'] with `area = 'left'` so the frame docks into \
         the left UIParent slot via UIParent_ManageFramePosition's left-area allowance"
    );
    assert_eq!(
        panel.1, 1.0,
        "UIPanelWindows['GenericTraitFrame'] must declare `checkFit = 1` so \
         UIParent_ManageFramePosition runs the fit-check pass and offsets the frame when \
         the bag bar / objective tracker would overlap"
    );
    assert!(
        !panel.2.is_empty(),
        "UIPanelWindows['GenericTraitFrame'] must publish an allowOtherPanels entry — the \
         exact value (truthy/falsy) reflects whether the trait frame allows the right-side \
         CharacterFrame to remain open. Got: {}",
        panel.2
    );
}
}

prefork_full_ui_case! {
fn blizzard_generic_trait_ui_publishes_toplevel_named_frame_via_xml(env: &WowLuaEnv) {
    load_generic_trait_ui_with_dependency(env);

    let frame: (bool, bool, bool) = env
        .eval(
            "return GenericTraitFrame ~= nil, \
                    type(GenericTraitFrame.SetTreeID) == 'function', \
                    type(GenericTraitFrame.Currency) == 'table'",
        )
        .expect("GenericTraitFrame XML instance probe should succeed");
    assert_eq!(
        frame,
        (true, true, true),
        "Blizzard_GenericTraitFrame.xml line 3 declares the toplevel `GenericTraitFrame` \
         Frame instance with `mixin=GenericTraitFrameMixin` + \
         `inherits=TalentFrameBaseTemplate` + `parent=UIParent`. The post-XML instance must \
         resolve as a global, expose GenericTraitFrameMixin methods directly via the mixin \
         metatable (SetTreeID), and expose the `Currency` parentKey child Button \
         (mixin=GenericTraitFrameCurrencyFrameMixin) as a sibling field"
    );
}
}

prefork_full_ui_case! {
fn blizzard_generic_trait_ui_does_not_leak_layout_options_table_as_global(env: &WowLuaEnv) {
    load_generic_trait_ui_with_dependency(env);

    let leaks: (bool, bool, bool, bool) = env
        .eval(
            "return _G.GenericTraitFrameLayoutOptions == nil, \
                    _G.GenericTraitFrameLayouts == nil, \
                    _G.GenericTraitFrameTutorials == nil, \
                    _G.GenericTraitCurrencyTutorials == nil",
        )
        .expect("Module-local leak probe should succeed");
    assert_eq!(
        leaks,
        (true, true, true, true),
        "Blizzard_GenericTraitUtil.lua keeps GenericTraitFrameLayoutOptions / \
         GenericTraitFrameLayouts / GenericTraitFrameTutorials / \
         GenericTraitCurrencyTutorials as `local` module-scoped tables — they must NOT leak \
         into _G. External callers go through the GenericTraitUtil.GetFrameLayoutInfo / \
         GetFrameTutorialInfo / GetCurrencyTutorialInfo accessors instead, so registering \
         new layouts via AddFrameLayoutInfo is the only supported path"
    );
}
}

prefork_full_ui_case! {
fn blizzard_generic_trait_ui_add_frame_layout_info_round_trip(env: &WowLuaEnv) {
    load_generic_trait_ui_with_dependency(env);

    let round_trip: (String, String, bool) = env
        .eval(
            "GenericTraitUtil.AddFrameLayoutInfo(99999, { Title = 'TestTree', \
                                                          PanelArea = 'right' }); \
             local layout = GenericTraitUtil.GetFrameLayoutInfo(99999); \
             local mt = getmetatable(layout); \
             return tostring(layout.Title), \
                    tostring(layout.PanelArea), \
                    mt ~= nil and type(mt.__index) == 'table'",
        )
        .expect("AddFrameLayoutInfo round-trip probe should succeed");
    assert_eq!(
        round_trip.0, "TestTree",
        "AddFrameLayoutInfo(treeID, info) must store the caller-provided info verbatim \
         under the treeID key so subsequent GetFrameLayoutInfo(treeID) reads return the \
         registered Title"
    );
    assert_eq!(
        round_trip.1, "right",
        "PanelArea overrides set by AddFrameLayoutInfo must shadow the Default layout's \
         PanelArea — the caller's table is the metatabled layer, Default is just the \
         __index fallback"
    );
    assert_eq!(
        round_trip.2, true,
        "GetFrameLayoutInfo must always return a metatabled struct, even for layouts \
         registered after-the-fact via AddFrameLayoutInfo, so unset fields still inherit \
         from GenericTraitFrameLayoutOptions.Default"
    );
}
}
