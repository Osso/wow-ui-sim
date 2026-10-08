use std::path::PathBuf;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path().expect("Blizzard UI cache should be available")
}

fn spell_diminish_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_SpellDiminishUI")
}

fn spell_diminish_toc() -> PathBuf {
    spell_diminish_dir().join("Blizzard_SpellDiminishUI.toc")
}

const PUBLISHED_MIXINS: &[&str] = &[
    "SpellDiminishStatusTrayItemMixin",
    "SpellDiminishStatusTrayMixin",
];

const TRAY_ITEM_METHODS: &[&str] = &[
    "OnLoad",
    "SetupImmunityIndicator",
    "SetCategoryInfo",
    "GetCategory",
    "GetCategoryName",
    "UpdateState",
    "Reset",
];

const TRAY_METHODS: &[&str] = &[
    "OnLoad",
    "OnShow",
    "OnHide",
    "OnEvent",
    "SetUnit",
    "InitializeTrayItemPool",
    "AddCategoryToOrder",
    "RemoveCategoryFromOrder",
    "GetActiveTrayItemForCategory",
    "TryUpdateOrAddTrayItem",
    "ShouldTrackSpellDiminishCategory",
    "UpdateOrAddTrayItem",
    "AddNewItemToTray",
    "CreateTrayItemForCategory",
    "OnTrayItemCooldownDone",
    "RefreshTrayLayout",
    "RemoveAllTrayItems",
    "UpdateTrayItemAnchoring",
    "AnchorFirstTrayItem",
    "AnchorNextTrayItem",
    "UpdateShownState",
    "SetIsInEditMode",
    "IsInEditMode",
    "PopulateEditModePreviewItems",
    "ClearEditModePreviewItems",
];

const VIRTUAL_TEMPLATES: &[&str] = &[
    "SpellDiminishStatusTrayItemTemplate",
    "SpellDiminishStatusTrayTemplate",
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
    let resolved = find_toc_file(&spell_diminish_dir()).expect("SpellDiminishUI TOC resolves");
    assert_eq!(
        resolved,
        spell_diminish_toc(),
        "Blizzard_SpellDiminishUI ships a single bare \
         `Blizzard_SpellDiminishUI.toc` (NO `_Mainline.toc` flavor variant) — \
         per-flavor gating is expressed via the `## AllowLoadGameType: mainline` \
         directive instead, because diminishing-returns category state \
         (UNIT_SPELL_DIMINISH_CATEGORY_STATE_UPDATED) and the C_SpellDiminish \
         namespace are mainline-only PvP combat infrastructure"
    );
}

#[test]
fn dependencies_returns_shared_xml_via_plural_dependencies_key() {
    let toc = TocFile::from_file(&spell_diminish_toc()).expect("TOC parses");

    assert_eq!(
        toc.dependencies(),
        vec!["Blizzard_SharedXML".to_string()],
        "SpellDiminishUI uses the plural `## Dependencies:` key honored by \
         `dependencies()` at src/toc.rs:210-217. SharedXML provides the \
         ResizeLayoutFrame template that SpellDiminishStatusTrayTemplate \
         inherits=\"ResizeLayoutFrame\" against (the auto-resizing layout \
         driver that recomputes width/height from child anchors with \
         widthPadding=2 / heightPadding=2 / minimumWidth=30 / \
         minimumHeight=30). Got: {:?}",
        toc.dependencies()
    );
}

#[test]
fn allow_load_game_lowercase_resolves_correctly() {
    let toc = TocFile::from_file(&spell_diminish_toc()).expect("TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: game` (lowercase) MUST allow Game — \
         allows_screen at src/toc.rs:307 uses eq_ignore_ascii_case so \
         the lowercase `game` token matches"
    );
    assert!(
        !toc.allows_screen(ScreenKind::Login),
        "`## AllowLoad: game` MUST exclude Login glue — diminishing returns \
         only fire mid-combat in arena/battleground frames, not on the \
         glue screens"
    );
    assert!(!toc.allows_screen(ScreenKind::CharacterSelect));
    assert!(!toc.allows_screen(ScreenKind::CharacterCreate));
}

#[test]
fn allow_load_game_type_mainline_is_not_restricted() {
    let toc = TocFile::from_file(&spell_diminish_toc()).expect("TOC parses");

    assert!(
        !toc.is_game_type_restricted(),
        "`## AllowLoadGameType: mainline` MUST resolve to \
         is_game_type_restricted() == false because mainline is in the \
         allowed game-type list at src/toc.rs:298-299 (alongside \
         `standard`). The directive documents that the addon is \
         mainline-only — Mists/classic do not have C_SpellDiminish or \
         the UNIT_SPELL_DIMINISH_CATEGORY_STATE_UPDATED event"
    );
}

#[test]
fn toc_is_eager_with_no_secure_env_or_saved_vars() {
    let toc = TocFile::from_file(&spell_diminish_toc()).expect("TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "SpellDiminishUI must NOT be LoadOnDemand — compact raid frames \
         and arena enemy frames inherit the SpellDiminishStatusTrayTemplate \
         template at frame-construction time, so the template registry must \
         already hold the virtual definition before any combat frame is built"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.optional_deps().is_empty());
    assert!(toc.default_enabled());
}

#[test]
fn toc_raw_bytes_pin_four_metadata_directives() {
    let raw = std::fs::read_to_string(spell_diminish_toc()).expect("TOC reads utf-8");

    let expected_directives = [
        "## Title: Blizzard Spell Diminish UI",
        "## AllowLoad: game",
        "## AllowLoadGameType: mainline",
        "## Dependencies: Blizzard_SharedXML",
    ];

    for directive in expected_directives {
        assert!(
            raw.contains(directive),
            "Raw bytes MUST pin the `{directive}` directive — \
             SpellDiminishUI's TOC is small (4 metadata lines + 2 body \
             entries) so each directive is load-bearing"
        );
    }

    assert!(!raw.contains("## Author"));
    assert!(!raw.contains("## DefaultState"));
    assert!(!raw.contains("## Version"));
    assert!(!raw.contains("## LoadOnDemand"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## RequiredDep"));
    assert!(!raw.contains("## OptionalDep"));
    assert!(!raw.contains("## UseSecureEnvironment"));
}

#[test]
fn body_lists_lua_before_xml() {
    let toc = TocFile::from_file(&spell_diminish_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let expected = [
        "Blizzard_SpellDiminishUITemplates.lua",
        "Blizzard_SpellDiminishUITemplates.xml",
    ];

    assert_eq!(
        body.len(),
        expected.len(),
        "Body must contain exactly 2 entries — the addon ships one .lua \
         + one .xml. Got: {body:?}"
    );

    for (i, want) in expected.iter().enumerate() {
        assert_eq!(
            &body[i], want,
            "Body entry {i}: expected {want}, got {}",
            body[i]
        );
    }

    assert!(
        body[0].ends_with(".lua") && body[1].ends_with(".xml"),
        "Templates.lua MUST load BEFORE Templates.xml — the XML \
         references SpellDiminishStatusTrayItemMixin (on \
         SpellDiminishStatusTrayItemTemplate) and \
         SpellDiminishStatusTrayMixin (on SpellDiminishStatusTrayTemplate) \
         via `mixin=\"…\"` attributes. The XML loader resolves the mixin \
         tables at template-registration time, so they MUST already be \
         tables in _G when the .xml chunk is processed"
    );
}

#[test]
fn appears_in_eager_discovery_on_game_screen_only() {
    let ui = blizzard_ui_dir();

    let game_addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let game_found = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_SpellDiminishUI");
    assert!(
        game_found,
        "`## AllowLoad: game` MUST surface SpellDiminishUI on the Game \
         screen eager discovery sweep — diminishing-returns trays must be \
         registered before any combat frame mounts a SpellDiminishStatusTray"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_SpellDiminishUI");
        assert!(
            !found,
            "`## AllowLoad: game` MUST exclude SpellDiminishUI from \
             {screen:?} eager discovery — diminishing returns are a \
             combat-only PvP feature, no glue-screen reachability"
        );
    }
}

prefork_full_ui_case! {
fn full_game_load_emits_no_addon_specific_lua_errors(env: &WowLuaEnv) {

    let errors = env.state().borrow().lua_errors.clone();
    let needles = [
        "SpellDiminishUI",
        "SpellDiminishStatusTray",
        "SpellDiminishStatusTrayItem",
        "SpellDiminishStatusTrayMixin",
        "SpellDiminishStatusTrayItemMixin",
    ];

    let matched: Vec<&String> = errors
        .iter()
        .filter(|e| needles.iter().any(|n| e.contains(n)))
        .collect();

    assert!(
        matched.is_empty(),
        "Full Game-screen load must emit zero SpellDiminishUI-specific \
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
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_SpellDiminishUI')")
        .expect("IsAddOnLoaded query");
    assert!(
        loaded,
        "After the Game-screen eager sweep, \
         C_AddOns.IsAddOnLoaded('Blizzard_SpellDiminishUI') must return \
         true — `## AllowLoad: game` with no `## LoadOnDemand` and \
         mainline-allowed game type means the addon is part of the \
         auto-loaded set"
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
             Blizzard_SpellDiminishUI loads — \
             Blizzard_SpellDiminishUITemplates.lua creates the two empty \
             mixin tables at file scope (lines 1 and 37) before binding \
             methods to them"
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
fn tray_item_mixin_carries_seven_canonical_methods(env: &WowLuaEnv) {

    for method in TRAY_ITEM_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(SpellDiminishStatusTrayItemMixin['{method}'])"
            ))
            .unwrap_or_else(|err| {
                panic!("SpellDiminishStatusTrayItemMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            kind, "function",
            "SpellDiminishStatusTrayItemMixin.{method} must publish as a \
             function — the per-tray-item mixin owns 7 methods covering \
             OnLoad (raises ImmunityIndicator above Cooldown), \
             SetupImmunityIndicator (frame-level dance for the immune \
             icon), SetCategoryInfo (binds icon texture from \
             categoryInfo.icon with the INV_Misc_QuestionMark fallback), \
             GetCategory / GetCategoryName accessors, UpdateState \
             (drives ImmunityIndicator visibility + CooldownFrame_Set), \
             and Reset (used by the FramePool resetter trayItemResetter)"
        );
    }
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn tray_item_mixin_carries_seven_canonical_methods() {
    let env = load_full_ui_for(ScreenKind::Game);
    tray_item_mixin_carries_seven_canonical_methods::run(&env);
}

prefork_full_ui_case! {
fn tray_mixin_carries_twenty_five_lifecycle_and_pool_methods(env: &WowLuaEnv) {

    for method in TRAY_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(SpellDiminishStatusTrayMixin['{method}'])"
            ))
            .unwrap_or_else(|err| {
                panic!("SpellDiminishStatusTrayMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            kind, "function",
            "SpellDiminishStatusTrayMixin.{method} must publish as a \
             function — the tray mixin owns 25 methods including the 4 \
             XML-wired script handlers (OnLoad/OnShow/OnHide/OnEvent), \
             pool plumbing (InitializeTrayItemPool builds a \
             CreateUnsecuredFramePool with trayItemResetter), \
             order-list bookkeeping (AddCategoryToOrder / \
             RemoveCategoryFromOrder via tInsertUnique / tDeleteItem), \
             event routing (TryUpdateOrAddTrayItem gates on unitToken \
             match + ShouldTrackSpellDiminishCategory), \
             cooldown lifecycle (OnTrayItemCooldownDone releases the \
             frame back to the pool), anchor chain \
             (AnchorFirstTrayItem / AnchorNextTrayItem rebuild the \
             LEFT-TO-RIGHT 2px-spaced layout), and edit-mode preview \
             (SetIsInEditMode / PopulateEditModePreviewItems / \
             ClearEditModePreviewItems iterate \
             C_SpellDiminish.GetAllSpellDiminishCategories)"
        );
    }
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn tray_mixin_carries_twenty_five_lifecycle_and_pool_methods() {
    let env = load_full_ui_for(ScreenKind::Game);
    tray_mixin_carries_twenty_five_lifecycle_and_pool_methods::run(&env);
}

prefork_full_ui_case! {
fn tray_item_template_materializes_with_cooldown_and_immunity_children(env: &WowLuaEnv) {

    let probe = "local f = CreateFrame('Frame', 'SpellDiminishItemProbe', UIParent, 'SpellDiminishStatusTrayItemTemplate') \
                 if not f then return 'frame nil' end \
                 local missing = {} \
                 if type(f.Icon) ~= 'table' then table.insert(missing, 'Icon:'..type(f.Icon)) end \
                 if type(f.Cooldown) ~= 'table' then table.insert(missing, 'Cooldown:'..type(f.Cooldown)) end \
                 if type(f.ImmunityIndicator) ~= 'table' then table.insert(missing, 'ImmunityIndicator:'..type(f.ImmunityIndicator)) end \
                 if #missing == 0 then return 'OK' else return table.concat(missing, ',') end";
    let report: String = env
        .eval(probe)
        .expect("SpellDiminishStatusTrayItemTemplate materialize");
    assert_eq!(
        report, "OK",
        "SpellDiminishStatusTrayItemTemplate must materialize via \
         CreateFrame with three parentKey children: Icon (OVERLAY-layer \
         Texture loading INV_Misc_QuestionMark.blp by default — \
         setAllPoints over the 26x26 frame), Cooldown (a Cooldown widget \
         with reverse=true setAllPoints + a SwipeTexture color override \
         to rgba 0,0,0,0.85 for the dark cooldown swipe), and \
         ImmunityIndicator (a Frame child with its own ARTWORK Texture \
         using the GM-icon-role-tank atlas as a placeholder, anchored \
         CENTER on the Icon's TOP edge, hidden=true at construction). \
         Report: {report}"
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn tray_item_template_materializes_with_cooldown_and_immunity_children() {
    let env = load_full_ui_for(ScreenKind::Game);
    tray_item_template_materializes_with_cooldown_and_immunity_children::run(&env);
}

prefork_full_ui_case! {
fn tray_template_materializes_with_resize_layout_keyvalues(env: &WowLuaEnv) {

    let probe = "local f = CreateFrame('Frame', 'SpellDiminishTrayProbe', UIParent, 'SpellDiminishStatusTrayTemplate') \
                 if not f then return 'frame nil' end \
                 local missing = {} \
                 if f.minimumWidth ~= 30 then table.insert(missing, 'minimumWidth:'..tostring(f.minimumWidth)) end \
                 if f.minimumHeight ~= 30 then table.insert(missing, 'minimumHeight:'..tostring(f.minimumHeight)) end \
                 if f.widthPadding ~= 2 then table.insert(missing, 'widthPadding:'..tostring(f.widthPadding)) end \
                 if f.heightPadding ~= 2 then table.insert(missing, 'heightPadding:'..tostring(f.heightPadding)) end \
                 if #missing == 0 then return 'OK' else return table.concat(missing, ',') end";
    let report: String = env
        .eval(probe)
        .expect("SpellDiminishStatusTrayTemplate materialize");
    assert_eq!(
        report, "OK",
        "SpellDiminishStatusTrayTemplate must materialize with the four \
         numeric KeyValues that drive ResizeLayoutFrame: minimumWidth=30, \
         minimumHeight=30, widthPadding=2, heightPadding=2 — these prevent \
         the auto-resize from collapsing to zero when no tray items are \
         active and provide 2px breathing room around the LEFT-anchored \
         tray-item chain. Report: {report}"
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn tray_template_materializes_with_resize_layout_keyvalues() {
    let env = load_full_ui_for(ScreenKind::Game);
    tray_template_materializes_with_resize_layout_keyvalues::run(&env);
}

prefork_full_ui_case! {
fn xml_registers_both_virtual_templates(env: &WowLuaEnv) {

    for template in VIRTUAL_TEMPLATES {
        let probe = format!(
            "local ok, frame = pcall(function() \
                return CreateFrame('Frame', nil, UIParent, {template:?}) \
             end) \
             return ok and frame ~= nil"
        );
        let result: bool = env
            .eval(&probe)
            .unwrap_or_else(|err| panic!("template probe ({template}): {err}"));
        assert!(
            result,
            "Virtual template {template} (registered by \
             Blizzard_SpellDiminishUITemplates.xml) must materialize via \
             CreateFrame as a Frame — both templates declare \
             virtual=\"true\" so they live in the template registry only \
             until consumed by inheritance from a containing frame's \
             buttonTemplate / inherits attribute"
        );
    }
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn xml_registers_both_virtual_templates() {
    let env = load_full_ui_for(ScreenKind::Game);
    xml_registers_both_virtual_templates::run(&env);
}

prefork_full_ui_case! {
fn tray_template_inherits_resize_layout_frame(env: &WowLuaEnv) {

    let probe = "local f = CreateFrame('Frame', 'SpellDiminishTrayResizeProbe', UIParent, 'SpellDiminishStatusTrayTemplate') \
                 if not f then return false end \
                 return type(f.Layout) == 'function' or \
                        type(f.GetMinimumSize) == 'function' or \
                        type(f.MarkDirty) == 'function'";
    let result: bool = env
        .eval(probe)
        .expect("SpellDiminishStatusTrayTemplate ResizeLayout probe");
    assert!(
        result,
        "SpellDiminishStatusTrayTemplate inherits=\"ResizeLayoutFrame\" so \
         it must expose the ResizeLayoutFrame API (Layout / GetMinimumSize \
         / MarkDirty — at least one of which the simulator provides). \
         RefreshTrayLayout / OnShow / RemoveAllTrayItems all call \
         self:Layout() to recompute size from the dynamic tray-item \
         children, so the Layout method MUST resolve through the template \
         inheritance chain to the SharedXML-provided ResizeLayoutFrame \
         template"
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn tray_template_inherits_resize_layout_frame() {
    let env = load_full_ui_for(ScreenKind::Game);
    tray_template_inherits_resize_layout_frame::run(&env);
}
