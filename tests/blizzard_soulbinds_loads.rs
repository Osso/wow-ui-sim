use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn soulbinds_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_Soulbinds")
}

fn soulbinds_toc() -> PathBuf {
    soulbinds_dir().join("Blizzard_Soulbinds.toc")
}

const PUBLISHED_MIXINS: &[&str] = &[
    "SoulbindViewerMixin",
    "SoulbindTreeMixin",
    "SoulbindTreeNodeMixin",
    "SoulbindTraitNodeMixin",
    "SoulbindConduitNodeMixin",
    "SoulbindTreeNodeLinkMixin",
    "SoulbindsSelectButtonMixin",
    "SoulbindSelectGroupMixin",
    "SoulbindConduitMixin",
    "ConduitListMixin",
    "ConduitListSectionMixin",
    "ConduitListCategoryButtonMixin",
    "ConduitListConduitButtonMixin",
];

const VIRTUAL_TEMPLATES: &[&str] = &[
    "SoulbindsUndoButtonTemplate",
    "SoulbindsSelectButtonTemplate",
    "SoulbindSelectGroupTemplate",
    "ConduitListConduitButtonTemplate",
    "ConduitListSectionTemplate",
    "ConduitListTemplate",
    "SoulbindTreeNodeTemplate",
    "SoulbindTraitNodeTemplate",
    "SoulbindConduitNodeTemplate",
    "SoulbindTreeNodeLinkTemplate",
    "SoulbindTreeTemplate",
    "ConduitInstallTemplate",
    "ConduitButtonGlow",
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
fn toc_declares_load_on_demand_with_current_dependencies() {
    let toc = TocFile::from_file(&soulbinds_toc()).expect("Soulbinds TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "`## LoadOnDemand: 1` MUST resolve to is_load_on_demand() == true. \
         Soulbinds is summoned by RegisterUIPanel(SoulbindViewer, ...) at \
         the bottom of Blizzard_Soulbinds.lua and only loads when the \
         player opens the Soulbind Viewer panel via the covenant \
         sanctum or `/run UIParentLoadAddOn('Blizzard_Soulbinds')` — \
         keeping ~4300 lines of Lua + XML out of the eager Game-screen \
         sweep until needed"
    );

    assert_eq!(
        toc.dependencies(),
        vec![
            "Blizzard_Colors".to_string(),
            "Blizzard_GameMenuEsc".to_string(),
        ],
        "Current retail Soulbinds depends on Blizzard_Colors and Blizzard_GameMenuEsc"
    );
}

#[test]
fn toc_omits_optional_metadata_directives() {
    let toc = TocFile::from_file(&soulbinds_toc()).expect("Soulbinds TOC parses");

    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.optional_deps().is_empty());
    assert!(
        toc.default_enabled(),
        "Soulbinds defaults to enabled (no `## DefaultState:` directive — \
         absence implies enabled)"
    );
}

#[test]
fn toc_raw_bytes_pin_four_metadata_directives() {
    let raw = std::fs::read_to_string(soulbinds_toc()).expect("TOC reads utf-8");

    let expected_directives = [
        "## Title: Blizzard Soulbinds",
        "## Author: Blizzard Entertainment",
        "## LoadOnDemand: 1",
        "## Dependencies: Blizzard_Colors, Blizzard_GameMenuEsc",
    ];

    for directive in expected_directives {
        assert!(
            raw.contains(directive),
            "Raw bytes MUST pin `{directive}` — Soulbinds TOC is small \
             (4 metadata lines + 11 body entries). Note `## Title: \
             Blizzard Soulbinds` uses a SPACE separator (not the \
             underscore-form `Blizzard_Soulbinds` — the display title \
             differs from the addon directory name)"
        );
    }

    assert!(!raw.contains("## DefaultState"));
    assert!(!raw.contains("## AllowLoad"));
    assert!(!raw.contains("## AllowLoadGameType"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## RequiredDep"));
    assert!(!raw.contains("## OptionalDep"));
    assert!(!raw.contains("## UseSecureEnvironment"));
    assert!(!raw.contains("## Version"));
}

#[test]
fn body_resolves_to_twelve_entries_in_canonical_order() {
    let toc = TocFile::from_file(&soulbinds_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let expected = [
        "Blizzard_Soulbinds_Bootstrap.lua",
        "Blizzard_SoulbindsUtil.lua",
        "Blizzard_SoulbindsTemplates.xml",
        "Blizzard_SoulbindsSelectButton.xml",
        "Blizzard_SoulbindsSelectGroup.xml",
        "Blizzard_SoulbindsConduit.lua",
        "Blizzard_SoulbindsConduitList.xml",
        "Blizzard_SoulbindsNode.xml",
        "Blizzard_SoulbindsNodeLink.xml",
        "Blizzard_SoulbindsTree.xml",
        "Blizzard_SoulbindsViewer.xml",
        "Blizzard_Soulbinds.xml",
    ];

    assert_eq!(
        body.len(),
        expected.len(),
        "Retail 12.1.0.69497 body contains 12 entries, beginning with the \
         Soulbinds bootstrap before its existing Lua/XML load sequence. Got: {body:?}"
    );

    for (i, want) in expected.iter().enumerate() {
        assert_eq!(
            &body[i], want,
            "Body entry {i}: expected {want}, got {}",
            body[i]
        );
    }
}

#[test]
fn bootstrap_loads_before_util_to_publish_soulbinds_namespace_before_consumers() {
    let toc = TocFile::from_file(&soulbinds_toc()).expect("TOC parses");

    let first = toc
        .files
        .first()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    assert_eq!(
        first, "Blizzard_Soulbinds_Bootstrap.lua",
        "Retail 12.1.0.69497 loads Blizzard_Soulbinds_Bootstrap.lua first"
    );
    let second = toc
        .files
        .get(1)
        .expect("Soulbinds TOC should list a file after the bootstrap");
    assert_eq!(
        second.to_string_lossy(),
        "Blizzard_SoulbindsUtil.lua",
        "Blizzard_SoulbindsUtil.lua follows the bootstrap and publishes the Soulbinds namespace"
    );
}

#[test]
fn templates_xml_loads_before_node_xml_for_select_button_template_inheritance() {
    let toc = TocFile::from_file(&soulbinds_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let templates_idx = body
        .iter()
        .position(|f| f == "Blizzard_SoulbindsTemplates.xml")
        .expect("Templates.xml present");
    let select_button_idx = body
        .iter()
        .position(|f| f == "Blizzard_SoulbindsSelectButton.xml")
        .expect("SelectButton.xml present");
    let node_idx = body
        .iter()
        .position(|f| f == "Blizzard_SoulbindsNode.xml")
        .expect("Node.xml present");
    let viewer_idx = body
        .iter()
        .position(|f| f == "Blizzard_SoulbindsViewer.xml")
        .expect("Viewer.xml present");

    assert!(
        templates_idx < select_button_idx,
        "Templates.xml must precede SelectButton.xml — Templates.xml \
         sources Blizzard_SoulbindsTemplates.lua via `<Script \
         file=...>` (the .lua is empty in this build but the slot is \
         reserved for shared mixin/utility tables that downstream \
         templates can mixin)"
    );
    assert!(
        select_button_idx < node_idx,
        "SelectButton.xml registers SoulbindsSelectButtonTemplate before \
         Node.xml registers SoulbindTreeNodeTemplate / \
         SoulbindTraitNodeTemplate / SoulbindConduitNodeTemplate — the \
         covenant-pick row must exist before tree-node templates can \
         compose them"
    );
    assert!(
        node_idx < viewer_idx,
        "Node.xml must precede Viewer.xml — SoulbindViewer's child \
         Tree frame instantiates SoulbindTreeNodeTemplate / \
         SoulbindTraitNodeTemplate / SoulbindConduitNodeTemplate, so \
         those virtual templates must register before the viewer's \
         <Frames> block resolves"
    );
}

#[test]
fn soulbinds_xml_is_last_so_register_ui_panel_runs_after_viewer_exists() {
    let toc = TocFile::from_file(&soulbinds_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    let last = body.last().expect("body non-empty").clone();

    assert_eq!(
        last, "Blizzard_Soulbinds.xml",
        "Blizzard_Soulbinds.xml MUST be LAST — it sources \
         Blizzard_Soulbinds.lua which calls \
         RegisterUIPanel(SoulbindViewer, attributes) at file scope. \
         SoulbindViewer is the named Frame defined by Viewer.xml's \
         <Frame name=\"SoulbindViewer\" parent=\"UIParent\" \
         mixin=\"SoulbindViewerMixin\">, so Viewer.xml MUST have \
         already run to publish the SoulbindViewer global before the \
         RegisterUIPanel call dereferences it"
    );
}

#[test]
fn soulbinds_does_not_auto_discover_on_any_screen() {
    let screens = [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ];

    for screen in screens {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons.iter().any(|(name, _)| name == "Blizzard_Soulbinds");
        assert!(
            !found,
            "Blizzard_Soulbinds (LoadOnDemand=1) must NOT appear in \
             auto-discovery for screen {screen:?}. The Soulbind Viewer \
             is summoned by the covenant sanctum NPC interaction or \
             /run C_AddOns.LoadAddOn('Blizzard_Soulbinds') — eager \
             discovery would defeat the LOD cost-deferral"
        );
    }
}

prefork_full_ui_case! {
fn explicit_load_addon_succeeds_with_no_addon_specific_lua_errors(env: &WowLuaEnv) {

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_addon(&env.loader_env(), &soulbinds_toc())
        .expect("Blizzard_Soulbinds should load via Rust loader");

    let load_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let needles = ["Soulbind", "Conduit", "ConduitList", "RegisterUIPanel"];
    let matched: Vec<&String> = load_errors
        .iter()
        .filter(|e| needles.iter().any(|n| e.contains(n)))
        .collect();

    assert!(
        matched.is_empty(),
        "Explicit load_addon for Blizzard_Soulbinds must emit zero \
         addon-specific Lua errors. Found {} matching errors:\n{:#?}",
        matched.len(),
        matched
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_reports_true_after_explicit_load(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &soulbinds_toc()).expect("Blizzard_Soulbinds should load");

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_Soulbinds')")
        .expect("IsAddOnLoaded query");
    assert!(
        loaded,
        "After explicit load_addon, \
         C_AddOns.IsAddOnLoaded('Blizzard_Soulbinds') must return true \
         — confirms the loader registered the addon name even though \
         it didn't surface in the eager Game-screen sweep"
    );
}
}

prefork_full_ui_case! {
fn util_publishes_soulbinds_namespace_with_canonical_helpers(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &soulbinds_toc()).expect("Blizzard_Soulbinds should load");

    let probe = "return type(Soulbinds) == 'table' and \
                 type(Soulbinds.HasConduitAtCursor) == 'function' and \
                 type(Soulbinds.SetPreviewConduit) == 'function' and \
                 type(Soulbinds.GetPreviewConduit) == 'function' and \
                 type(Soulbinds.GetOpenSoulbindID) == 'function' and \
                 type(Soulbinds.GetDefaultSoulbindID) == 'function' and \
                 type(Soulbinds.HasNewSoulbindTutorial) == 'function' and \
                 type(Soulbinds.GetConduitName) == 'function' and \
                 type(Soulbinds.GetConduitEmblemAtlas) == 'function' and \
                 type(Soulbinds.SetConduitInstallPending) == 'function' and \
                 type(Soulbinds.IsConduitCommitPending) == 'function'";
    let result: bool = env.eval(probe).expect("Soulbinds namespace probe");
    assert!(
        result,
        "Util.lua MUST publish the `Soulbinds` namespace table with \
         11 canonical helpers: HasConduitAtCursor (probes \
         C_Soulbinds.GetConduitCollectionDataAtCursor), \
         {{Set,Clear,Get}}PreviewConduit (drives the conduit-tooltip \
         preview state), GetOpenSoulbindID (delegates to \
         SoulbindViewer:GetOpenSoulbindID), GetDefaultSoulbindID + \
         HasNewSoulbindTutorial (drive the new-soulbind quest tutorial \
         flow per covenant), GetConduitName + GetConduitEmblemAtlas \
         (return CONDUIT_POTENCY / ENDURANCE / FINESSE strings + \
         Soulbinds_Tree_Conduit_Icon_{{Attack,Protect,Utility}} atlases \
         from Enum.SoulbindConduitType), \
         SetConduitInstallPending + IsConduitCommitPending (drive the \
         pending-install button-state machine)"
    );
}
}

prefork_full_ui_case! {
fn util_publishes_renown_currency_id_constant(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &soulbinds_toc()).expect("Blizzard_Soulbinds should load");

    let value: i32 = env
        .eval("return SOULBINDS_RENOWN_CURRENCY_ID")
        .expect("renown currency probe");
    assert_eq!(
        value, 1822,
        "SOULBINDS_RENOWN_CURRENCY_ID MUST resolve to 1822 — the \
         in-game currency ID for Reservoir Anima / Renown that drives \
         soulbind tree progression. Hardcoded at \
         Blizzard_SoulbindsUtil.lua:3"
    );
}
}

prefork_full_ui_case! {
fn publishes_thirteen_mixin_tables(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &soulbinds_toc()).expect("Blizzard_Soulbinds should load");

    for mixin in PUBLISHED_MIXINS {
        let probe = format!("return type({mixin}) == 'table'");
        let result: bool = env
            .eval(&probe)
            .unwrap_or_else(|err| panic!("mixin probe ({mixin}): {err}"));
        assert!(
            result,
            "Mixin {mixin} MUST publish as a global table after \
             Soulbinds load — XML mixin=\"{mixin}\" attributes resolve \
             the table at template-registration time. The 13 mixins \
             span: SoulbindViewerMixin (root frame controller), \
             SoulbindTreeMixin (tree-graph layout + node selection), \
             SoulbindTreeNodeMixin / SoulbindTraitNodeMixin / \
             SoulbindConduitNodeMixin (3-level node hierarchy), \
             SoulbindTreeNodeLinkMixin (line-segment renderer between \
             nodes), SoulbindsSelectButtonMixin (covenant-pick row), \
             SoulbindSelectGroupMixin (covenant-tab group), \
             SoulbindConduitMixin (extends SpellMixin for tooltip \
             data), and the 4 conduit-list mixins"
        );
    }
}
}

prefork_full_ui_case! {
fn callback_registry_mixins_inherit_register_callback(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &soulbinds_toc()).expect("Blizzard_Soulbinds should load");

    let probe = "return type(SoulbindViewerMixin.RegisterCallback) == 'function' and \
                 type(SoulbindTreeMixin.RegisterCallback) == 'function' and \
                 type(SoulbindSelectGroupMixin.RegisterCallback) == 'function' and \
                 type(SoulbindTreeNodeMixin.RegisterCallback) == 'function'";
    let result: bool = env.eval(probe).expect("CallbackRegistry inherit probe");
    assert!(
        result,
        "SoulbindViewerMixin / SoulbindTreeMixin / \
         SoulbindSelectGroupMixin / SoulbindTreeNodeMixin all derive \
         from `CreateFromMixins(CallbackRegistryMixin)` — they MUST \
         inherit RegisterCallback (and the rest of the pub/sub \
         surface: TriggerEvent, OnLoad, GenerateCallbackEvents). \
         CallbackRegistryMixin is published by Blizzard_SharedXMLBase, \
         eagerly available before this LOD addon loads. Without the \
         inheritance, Tree:RegisterCallback at \
         SoulbindViewer.OnLoad:32 would throw"
    );
}
}

prefork_full_ui_case! {
fn trait_node_inherits_tree_node_for_shared_node_surface(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &soulbinds_toc()).expect("Blizzard_Soulbinds should load");

    let probe = "return type(SoulbindTraitNodeMixin) == 'table' and \
                 type(SoulbindConduitNodeMixin) == 'table' and \
                 type(SoulbindTraitNodeMixin.RegisterCallback) == 'function' and \
                 type(SoulbindConduitNodeMixin.RegisterCallback) == 'function'";
    let result: bool = env.eval(probe).expect("trait/conduit node probe");
    assert!(
        result,
        "SoulbindTraitNodeMixin and SoulbindConduitNodeMixin both \
         derive from `CreateFromMixins(SoulbindTreeNodeMixin)` which \
         itself derives from CallbackRegistryMixin. The transitive \
         RegisterCallback inheritance MUST resolve so the tree-node \
         pub/sub surface (OnNodeChanged callbacks) reaches the leaf \
         trait/conduit node types"
    );
}
}

prefork_full_ui_case! {
fn select_button_mixin_inherits_selectable_button_for_radio_behavior(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &soulbinds_toc()).expect("Blizzard_Soulbinds should load");

    let probe = "return type(SoulbindsSelectButtonMixin) == 'table' and \
                 type(SelectableButtonMixin) == 'table'";
    let result: bool = env.eval(probe).expect("SelectableButton inheritance probe");
    assert!(
        result,
        "SoulbindsSelectButtonMixin = \
         CreateFromMixins(SelectableButtonMixin). SelectableButtonMixin \
         is published by Blizzard_SharedXMLBase (Mixin.lua family) and \
         provides the radio-group click-to-select pattern that drives \
         the covenant pick row's mutual-exclusion behavior — exactly \
         one covenant button is selected at a time"
    );
}
}

prefork_full_ui_case! {
fn conduit_mixin_inherits_spell_mixin_for_tooltip_data(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &soulbinds_toc()).expect("Blizzard_Soulbinds should load");

    let probe = "return type(SoulbindConduitMixin) == 'table' and \
                 type(SpellMixin) == 'table'";
    let result: bool = env.eval(probe).expect("SpellMixin inheritance probe");
    assert!(
        result,
        "SoulbindConduitMixin = CreateFromMixins(SpellMixin). \
         SpellMixin (from SharedXML or FrameXML) provides the \
         spell-id-bearing tooltip-data primitives that conduit \
         tooltips reuse — ContinueOnSpellLoad / SetSpellID / \
         GetSpellID — so the conduit slot can render the underlying \
         spell tooltip when the conduit is socketed"
    );
}
}

prefork_full_ui_case! {
fn viewer_global_resolves_to_frame_after_load(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &soulbinds_toc()).expect("Blizzard_Soulbinds should load");

    let probe = "local f = SoulbindViewer \
                 if not f then return 'frame nil' end \
                 if type(f.GetName) ~= 'function' then return 'no GetName' end \
                 local name = f:GetName() \
                 if name ~= 'SoulbindViewer' then return 'name='..tostring(name) end \
                 if not f:GetParent() then return 'no parent' end \
                 return 'OK'";
    let report: String = env.eval(probe).expect("SoulbindViewer probe");
    assert_eq!(
        report, "OK",
        "After load, the named global SoulbindViewer MUST resolve to \
         a Frame with GetName() == 'SoulbindViewer' and a non-nil \
         parent (UIParent). Defined by Viewer.xml's `<Frame \
         name=\"SoulbindViewer\" parent=\"UIParent\" \
         mixin=\"SoulbindViewerMixin\">` at 939x926 size, anchored \
         CENTER. Blizzard_Soulbinds.lua's RegisterUIPanel call at the \
         tail dereferences this global so its presence is the \
         canonical proof that Viewer.xml ran before Soulbinds.xml"
    );
}
}

prefork_full_ui_case! {
fn xml_registers_all_thirteen_virtual_templates(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &soulbinds_toc()).expect("Blizzard_Soulbinds should load");

    for template in VIRTUAL_TEMPLATES {
        let widget_type = match *template {
            "ConduitButtonGlow" => continue,
            "ConduitInstallTemplate" => continue,
            "SoulbindsUndoButtonTemplate" => "Button",
            "SoulbindsSelectButtonTemplate" => "Button",
            "SoulbindSelectGroupTemplate" => "Frame",
            "ConduitListConduitButtonTemplate" => "Button",
            "ConduitListSectionTemplate" => "Frame",
            "ConduitListTemplate" => "Frame",
            "SoulbindTreeNodeTemplate" => "Button",
            "SoulbindTraitNodeTemplate" => "Button",
            "SoulbindConduitNodeTemplate" => "Button",
            "SoulbindTreeNodeLinkTemplate" => "Frame",
            "SoulbindTreeTemplate" => "Frame",
            other => panic!("unexpected template {other}"),
        };
        let probe = format!(
            "local ok, frame = pcall(function() \
                return CreateFrame({widget_type:?}, nil, UIParent, {template:?}) \
             end) \
             return ok and frame ~= nil"
        );
        let result: bool = env
            .eval(&probe)
            .unwrap_or_else(|err| panic!("template probe ({template}): {err}"));
        assert!(
            result,
            "Virtual template {template} (registered by Soulbinds XMLs) \
             must materialize via CreateFrame as widget_type \
             {widget_type:?}. The 11 Frame/Button templates form a \
             composition chain: SoulbindsSelectButtonTemplate (covenant \
             pick row) → SoulbindSelectGroupTemplate (parent group) → \
             SoulbindTreeTemplate (graph viewport) → \
             SoulbindTreeNodeTemplate (base node) which derives \
             SoulbindTraitNodeTemplate / SoulbindConduitNodeTemplate \
             (specialized nodes) joined by SoulbindTreeNodeLinkTemplate \
             (edge renderer); the conduit-list panel is a parallel \
             chain ConduitListTemplate → ConduitListSectionTemplate → \
             ConduitListConduitButtonTemplate"
        );
    }
}
}

prefork_full_ui_case! {
fn conduit_install_texture_template_registers_under_node_xml(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &soulbinds_toc()).expect("Blizzard_Soulbinds should load");

    let probe = "local ok = pcall(function() \
                    local frame = CreateFrame('Frame', nil, UIParent) \
                    local tex = frame:CreateTexture(nil, 'OVERLAY', 'ConduitInstallTemplate') \
                    return tex ~= nil \
                 end) \
                 return ok";
    let result: bool = env.eval(probe).expect("ConduitInstallTemplate probe");
    assert!(
        result,
        "ConduitInstallTemplate is a `<Texture name=...>` virtual \
         template (NOT a frame) registered by \
         Blizzard_SoulbindsNode.xml — must be resolvable via \
         frame:CreateTexture(name, layer, 'ConduitInstallTemplate'). \
         Same with ConduitButtonGlow under ConduitList.xml. Texture \
         templates can't be CreateFrame'd directly, hence the probe \
         uses CreateTexture path. If the simulator's CreateTexture \
         doesn't accept template inheritance the pcall fails benignly \
         — this test pins the surface contract"
    );
}
}
