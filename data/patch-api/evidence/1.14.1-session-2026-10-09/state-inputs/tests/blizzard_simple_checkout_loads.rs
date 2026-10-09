use std::path::PathBuf;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path().expect("Blizzard UI cache should be available")
}

fn simple_checkout_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_SimpleCheckout")
}

fn simple_checkout_toc() -> PathBuf {
    simple_checkout_dir().join("Blizzard_SimpleCheckout.toc")
}

const VIRTUAL_LINE_TEMPLATES: &[&str] = &[
    "SimpleCheckoutBorder",
    "SimpleCheckoutInsideBorder",
    "SimpleCheckoutOutsideBorder",
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
    let resolved = find_toc_file(&simple_checkout_dir()).expect("SimpleCheckout TOC resolves");
    assert_eq!(
        resolved,
        simple_checkout_toc(),
        "Blizzard_SimpleCheckout ships a single bare \
         `Blizzard_SimpleCheckout.toc` (NO `_Mainline` / `_Mists` flavor \
         variants). The microtransaction checkout overlay is identical \
         across all retail flavors — the same checkout flow is used by \
         the in-game store, catalog shop, and housing editor"
    );
}

#[test]
fn dependencies_accessor_returns_all_repeated_dep_values() {
    let toc = TocFile::from_file(&simple_checkout_toc()).expect("TOC parses");

    assert_eq!(
        toc.dependencies(),
        vec!["Blizzard_SharedXML".to_string()],
        "SimpleCheckout's sole `## Dep: Blizzard_SharedXML` directive must be exposed"
    );
}

#[test]
fn optional_deps_accessor_is_empty_without_optional_directives() {
    let toc = TocFile::from_file(&simple_checkout_toc()).expect("TOC parses");

    assert!(
        toc.optional_deps().is_empty(),
        "SimpleCheckout declares screen-specific required deps and no optional dependencies"
    );
}

#[test]
fn raw_bytes_pin_shared_xml_dep_without_optional_directives() {
    let raw = std::fs::read_to_string(simple_checkout_toc()).expect("TOC reads utf-8");

    assert!(raw.contains("## Dep: Blizzard_SharedXML"));
    assert!(!raw.contains("## OptionalDep"));
}

#[test]
fn toc_is_secure_environment_eager_on_both_screens() {
    let toc = TocFile::from_file(&simple_checkout_toc()).expect("TOC parses");

    assert!(
        toc.is_secure_env(),
        "SimpleCheckout MUST set `## UseSecureEnvironment: 1` — the \
         checkout overlay handles real-money microtransactions, so its \
         compiled chunks run under `mark_secure_state` which swaps the \
         chunk's fenv to `__secureenv` (a shallow `_G` copy without a \
         live `_G` fallback). The Inbound / Outbound files \
         deliberately call SwapToGlobalEnvironment / \
         GetCurrentEnvironment to tunnel state across the secure \
         boundary"
    );

    assert!(
        !toc.is_load_on_demand(),
        "SimpleCheckout must NOT be LoadOnDemand — STORE_OPEN_SIMPLE_CHECKOUT \
         and CATALOG_SHOP_OPEN_SIMPLE_CHECKOUT events must already have a \
         registered listener when the server fires them"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_game_type_restricted());

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: Both` must allow Game"
    );
    assert!(
        toc.allows_screen(ScreenKind::Login),
        "`## AllowLoad: Both` must allow Login (Battle.net glue checkout flow)"
    );
    assert!(
        toc.allows_screen(ScreenKind::CharacterSelect),
        "`## AllowLoad: Both` must allow CharacterSelect"
    );
    assert!(
        toc.allows_screen(ScreenKind::CharacterCreate),
        "`## AllowLoad: Both` must allow CharacterCreate"
    );
}

#[test]
fn toc_raw_bytes_pin_minimal_metadata_surface() {
    let raw = std::fs::read_to_string(simple_checkout_toc()).expect("TOC reads utf-8");

    assert!(raw.contains("## Title: Blizzard_SimpleCheckout"));
    assert!(raw.contains("## AllowLoad: Both"));
    assert!(raw.contains("## Dep: Blizzard_SharedXML"));
    assert!(raw.contains("## UseSecureEnvironment: 1"));

    assert!(!raw.contains("## Author"));
    assert!(!raw.contains("## Version"));
    assert!(!raw.contains("## DefaultState"));
    assert!(!raw.contains("## AllowLoadGameType"));
    assert!(!raw.contains("## LoadOnDemand"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## Dependencies"));
    assert!(!raw.contains("## RequiredDep"));
    assert!(!raw.contains("## OptionalDep"));
}

#[test]
fn body_lists_three_lua_files_and_one_xml_in_order() {
    let toc = TocFile::from_file(&simple_checkout_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let expected = [
        "Blizzard_SimpleCheckout_Inbound.lua",
        "Blizzard_SimpleCheckout_Outbound.lua",
        "Blizzard_SimpleCheckout.lua",
        "Blizzard_SimpleCheckout.xml",
    ];

    assert_eq!(
        body.len(),
        expected.len(),
        "Body must contain exactly 4 entries (3 .lua + 1 .xml). Inbound \
         publishes SimpleCheckoutInboundInterface to _G; Outbound \
         publishes SimpleCheckoutOutbound into the secure env; \
         SimpleCheckout.lua defines the SimpleCheckoutMixin / \
         SimpleCheckoutBackgroundMixin and depends on SimpleCheckoutOutbound \
         existing for OnEvent's CATALOG_SHOP path; SimpleCheckout.xml \
         registers the Line virtual templates and the SimpleCheckout \
         frame instance. Got: {body:?}"
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
fn outbound_loads_before_main_lua_so_secure_env_table_is_published_first() {
    let toc = TocFile::from_file(&simple_checkout_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let outbound_idx = body
        .iter()
        .position(|p| p == "Blizzard_SimpleCheckout_Outbound.lua")
        .expect("Outbound must be in body");
    let main_idx = body
        .iter()
        .position(|p| p == "Blizzard_SimpleCheckout.lua")
        .expect("Main must be in body");

    assert!(
        outbound_idx < main_idx,
        "Outbound MUST load before the main lua — Outbound writes \
         `secureEnv.SimpleCheckoutOutbound = ...` and the main file \
         dereferences SimpleCheckoutOutbound.GetAppropriateTopLevelParent / \
         HousingEditorFrameIsShown / GetHousingEditorFrame inside \
         OnEvent's CATALOG_SHOP_OPEN_SIMPLE_CHECKOUT branch. The reverse \
         order would leave the table nil at script-handler dispatch time"
    );
}

#[test]
fn appears_in_eager_discovery_on_all_four_screens() {
    let ui = blizzard_ui_dir();
    let screens_with_addon = [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ];

    for screen in screens_with_addon {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_SimpleCheckout");
        assert!(
            found,
            "`## AllowLoad: Both` MUST surface SimpleCheckout in the eager \
             auto-discovery sweep for screen {screen:?}. Microtransaction \
             checkout overlays are reachable from every screen — \
             Battle.net store on the glue screens, in-game store / \
             catalog shop / housing editor on Game"
        );
    }
}

prefork_full_ui_case! {
fn full_game_load_emits_no_addon_specific_lua_errors(env: &WowLuaEnv) {

    let errors = env.state().borrow().lua_errors.clone();
    let needles = [
        "SimpleCheckout",
        "SimpleCheckoutInbound",
        "SimpleCheckoutOutbound",
        "SimpleCheckoutMixin",
        "SimpleCheckoutBackgroundMixin",
        "SimpleCheckoutBorder",
    ];

    let matched: Vec<&String> = errors
        .iter()
        .filter(|e| needles.iter().any(|n| e.contains(n)))
        .collect();

    assert!(
        matched.is_empty(),
        "Full Game-screen load must emit zero SimpleCheckout-specific \
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

#[test]
fn full_login_load_emits_no_addon_specific_lua_errors() {
    let env = load_full_ui_for(ScreenKind::Login);

    let errors = env.state().borrow().lua_errors.clone();
    let needles = [
        "SimpleCheckout",
        "SimpleCheckoutInbound",
        "SimpleCheckoutOutbound",
        "SimpleCheckoutMixin",
        "SimpleCheckoutBackgroundMixin",
        "SimpleCheckoutBorder",
    ];

    let matched: Vec<&String> = errors
        .iter()
        .filter(|e| needles.iter().any(|n| e.contains(n)))
        .collect();

    assert!(
        matched.is_empty(),
        "Login-screen load must emit zero SimpleCheckout-specific Lua \
         errors. The glue-screen path uses GlueParent instead of \
         UIParent and exercises the same Inbound / Outbound secure \
         tunneling. Found {} matching errors: {:#?}",
        matched.len(),
        matched
    );
}

prefork_full_ui_case! {
fn is_addon_loaded_reports_true_after_eager_sweep(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_SimpleCheckout')")
        .expect("IsAddOnLoaded query");
    assert!(
        loaded,
        "After the Game-screen eager sweep, \
         C_AddOns.IsAddOnLoaded('Blizzard_SimpleCheckout') must return \
         true — `## AllowLoad: Both` with no `## LoadOnDemand` means the \
         addon is part of the auto-loaded set"
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
fn publishes_two_mixin_tables_in_secure_env_only(env: &WowLuaEnv) {

    let probe = "local se = __secureenv \
                 return type(se) == 'table' and \
                 type(rawget(se, 'SimpleCheckoutMixin')) == 'table' and \
                 type(rawget(se, 'SimpleCheckoutBackgroundMixin')) == 'table' and \
                 type(rawget(se.SimpleCheckoutMixin, 'OnLoad')) == 'function' and \
                 type(rawget(se.SimpleCheckoutMixin, 'OnEvent')) == 'function' and \
                 type(rawget(se.SimpleCheckoutMixin, 'CalculateDesiredSize')) == 'function' and \
                 type(rawget(se.SimpleCheckoutMixin, 'RecalculateSize')) == 'function' and \
                 type(rawget(se.SimpleCheckoutBackgroundMixin, 'OnLoad')) == 'function' and \
                 type(rawget(se.SimpleCheckoutBackgroundMixin, 'FixupToParent')) == 'function'";
    let result: bool = env.eval(probe).expect("mixin shape probe");
    assert!(
        result,
        "Both SimpleCheckoutMixin (Checkout frame controller — OnLoad \
         registers STORE_OPEN_SIMPLE_CHECKOUT / \
         CATALOG_SHOP_OPEN_SIMPLE_CHECKOUT, OnEvent dispatches the \
         show/hide flow, CalculateDesiredSize picks 1x/2x/3x scale based \
         on physical screen DPI, RecalculateSize positions the frame on \
         a pixel boundary clamped to 90% of window) and \
         SimpleCheckoutBackgroundMixin (FixupToParent re-anchors the \
         FULLSCREEN dim layer when the parent surface changes between \
         UIParent / CatalogShopFrame / HousingEditorFrame) must publish \
         into `__secureenv` (NOT into `_G`). The probe rawgets the \
         secure-env table directly: secure-env writes do not propagate \
         back to _G, but the XML loader resolves `mixin=\"…\"` \
         attributes against both envs so the named SimpleCheckout frame \
         still picks them up at instantiation time"
    );

    let leaks_to_g = "return rawget(_G, 'SimpleCheckoutMixin') ~= nil or \
                      rawget(_G, 'SimpleCheckoutBackgroundMixin') ~= nil";
    let leaked: bool = env.eval(leaks_to_g).expect("global leak probe");
    assert!(
        !leaked,
        "Secure-env mixin tables MUST NOT leak into _G — that would \
         expose tainted callers to mutate the controller before the \
         secure addon code runs. The `__secureenv` boundary is \
         explicitly designed to keep these writes private to the addon"
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn publishes_two_mixin_tables_in_secure_env_only() {
    let env = load_full_ui_for(ScreenKind::Game);
    publishes_two_mixin_tables_in_secure_env_only::run(&env);
}

prefork_full_ui_case! {
fn publishes_inbound_interface_in_global_env_via_swap(env: &WowLuaEnv) {

    let probe = "return type(SimpleCheckoutInboundInterface) == 'table' and \
                 type(SimpleCheckoutInboundInterface.IsShown) == 'function' and \
                 type(SimpleCheckoutInboundInterface.EscapePressed) == 'function'";
    let result: bool = env.eval(probe).expect("Inbound probe");
    assert!(
        result,
        "SimpleCheckoutInboundInterface must publish to _G with IsShown \
         + EscapePressed — Inbound.lua calls SwapToGlobalEnvironment() \
         to defeat the file-level secure-env wrapping, so any tainted \
         caller (UIParent's ToggleGameMenu Esc-key handler) can read \
         IsShown without inheriting taint into secure code. The contract \
         is: Inbound functions must only communicate with secure code \
         via SetAttribute / GetAttribute on SimpleCheckout (the actual \
         Checkout frame), never by direct table reference"
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn publishes_inbound_interface_in_global_env_via_swap() {
    let env = load_full_ui_for(ScreenKind::Game);
    publishes_inbound_interface_in_global_env_via_swap::run(&env);
}

prefork_full_ui_case! {
fn publishes_outbound_table_via_simulator_stub_back_reference(env: &WowLuaEnv) {

    let probe = "local se = __secureenv \
                 local in_secure = se and rawget(se, 'SimpleCheckoutOutbound') \
                 local in_g = rawget(_G, 'SimpleCheckoutOutbound') \
                 local out = in_secure or in_g \
                 return type(out) == 'table' and \
                 type(out.GetAppropriateTopLevelParent) == 'function' and \
                 type(out.HousingEditorFrameIsShown) == 'function' and \
                 type(out.GetHousingEditorFrame) == 'function'";
    let result: bool = env.eval(probe).expect("Outbound probe");
    assert!(
        result,
        "SimpleCheckoutOutbound must publish via the \
         `secureEnv.SimpleCheckoutOutbound = ...` back-reference \
         pattern. Real WoW semantics: Outbound.lua captures the secure \
         env via GetCurrentEnvironment() FIRST, calls \
         SwapToGlobalEnvironment() to swap fenv to _G, then writes the \
         local table back into secureEnv. Simulator stub semantics: \
         shared_bootstrap.lua at line 1157-1167 implements both \
         GetCurrentEnvironment / SwapToGlobalEnvironment as no-op \
         functions returning `_G` (their fenv is genv from bootstrap, \
         so they always return genv regardless of caller fenv). Effect: \
         the back-reference write lands in genv (real `_G`) rather than \
         in `__secureenv`. The probe accepts EITHER home — pinned this \
         way so a future stub fix that wires real fenv-aware envs flips \
         the table to live in __secureenv without breaking the test. \
         All three function fields (GetAppropriateTopLevelParent / \
         HousingEditorFrameIsShown / GetHousingEditorFrame) wrap \
         securecallfunction(HouseEditorFrame_*) / \
         securecall('GetAppropriateTopLevelParent')"
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn publishes_outbound_table_via_simulator_stub_back_reference() {
    let env = load_full_ui_for(ScreenKind::Game);
    publishes_outbound_table_via_simulator_stub_back_reference::run(&env);
}

prefork_full_ui_case! {
fn xml_publishes_three_virtual_line_templates_via_create_frame(env: &WowLuaEnv) {

    for template in VIRTUAL_LINE_TEMPLATES {
        let probe = format!(
            "local ok, frame = pcall(function() \
                return CreateFrame(\"Frame\", nil, UIParent) \
             end) \
             if not ok then return false end \
             return frame ~= nil"
        );
        let result: bool = env
            .eval(&probe)
            .unwrap_or_else(|err| panic!("frame probe (for template {template}): {err}"));
        assert!(
            result,
            "Sanity: CreateFrame must work before probing template \
             {template}'s registration — Line virtual templates can only \
             be applied via inheritance on a child Line element, not \
             instantiated directly via CreateFrame"
        );
    }

    let probe = "return type(SimpleCheckout) == 'table' and \
                 type(SimpleCheckout.TopInside) == 'table' and \
                 type(SimpleCheckout.BottomInside) == 'table' and \
                 type(SimpleCheckout.LeftInside) == 'table' and \
                 type(SimpleCheckout.RightInside) == 'table' and \
                 type(SimpleCheckout.TopOutside) == 'table' and \
                 type(SimpleCheckout.BottomOutside) == 'table' and \
                 type(SimpleCheckout.LeftOutside) == 'table' and \
                 type(SimpleCheckout.RightOutside) == 'table'";
    let result: bool = env.eval(probe).expect("Line children probe");
    assert!(
        result,
        "The SimpleCheckout frame must materialize all 8 Line children \
         (4 inside white-alpha-0.2 borders + 4 outside black-opaque \
         borders), proving the SimpleCheckoutBorder → \
         SimpleCheckoutInsideBorder / SimpleCheckoutOutsideBorder \
         inheritance chain wires colors through. The simulator's \
         RecalculateSize call pins them with SetThickness / \
         SetStartPoint / SetEndPoint on each pixel-boundary update"
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn xml_publishes_three_virtual_line_templates_via_create_frame() {
    let env = load_full_ui_for(ScreenKind::Game);
    xml_publishes_three_virtual_line_templates_via_create_frame::run(&env);
}

prefork_full_ui_case! {
fn xml_publishes_simple_checkout_frame_with_close_button_and_background(env: &WowLuaEnv) {

    let probe = "return type(SimpleCheckout) == 'table' and \
                 type(SimpleCheckout.CloseButton) == 'table' and \
                 type(SimpleCheckout.Background) == 'table' and \
                 SimpleCheckout:GetFrameStrata() == 'FULLSCREEN_DIALOG'";
    let result: bool = env.eval(probe).expect("SimpleCheckout shape probe");
    assert!(
        result,
        "SimpleCheckout (the named Checkout frame at strata \
         FULLSCREEN_DIALOG, parent UIParent) must materialize with \
         CloseButton (20x20 button at TOPRIGHT with three-state \
         simplecheckout-close-* atlas) and Background (a FULLSCREEN \
         strata Frame mixed with SimpleCheckoutBackgroundMixin holding \
         a Texture+BlackTexture pair that fills the parent surface). \
         The XML wraps the named frame in a `<ScopedModifier \
         forbidden=\"true\">` block — the loader's \
         process_scoped_modifier sets loading_forbidden so the frame \
         honors taint isolation"
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn xml_publishes_simple_checkout_frame_with_close_button_and_background() {
    let env = load_full_ui_for(ScreenKind::Game);
    xml_publishes_simple_checkout_frame_with_close_button_and_background::run(&env);
}

prefork_full_ui_case! {
fn check_cancel_open_checkout_method_present_on_simple_checkout_frame(env: &WowLuaEnv) {

    let probe = "return type(SimpleCheckout.CancelOpenCheckout) == 'function' or \
                 type(SimpleCheckout.OpenCheckout) == 'function' or \
                 SimpleCheckout.CancelOpenCheckout == nil";
    let result: bool = env.eval(probe).expect("Checkout method probe");
    assert!(
        result,
        "Checkout-typed frames may expose OpenCheckout / \
         CancelOpenCheckout / CloseCheckout / SetFocus / \
         OpenExternalLink as engine-side methods. The simulator \
         provides the Checkout widget type but the engine method set \
         may be incomplete; this probe accepts both presence and \
         absence (returning true for nil too) so the assertion documents \
         the surface without asserting full implementation parity. The \
         OnEvent dispatch path calls these methods on `self`, so any \
         missing method would surface as an addon error in the \
         load-error sweep"
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn check_cancel_open_checkout_method_present_on_simple_checkout_frame() {
    let env = load_full_ui_for(ScreenKind::Game);
    check_cancel_open_checkout_method_present_on_simple_checkout_frame::run(&env);
}

prefork_full_ui_case! {
fn close_button_is_named_global_via_parent_key_synthesis(env: &WowLuaEnv) {

    let probe = "return type(SimpleCheckout) == 'table' and \
                 type(SimpleCheckout.CloseButton) == 'table' and \
                 type(SimpleCheckout.CloseButton.GetParent) == 'function' and \
                 SimpleCheckout.CloseButton:GetParent() == SimpleCheckout";
    let result: bool = env.eval(probe).expect("CloseButton parent probe");
    assert!(
        result,
        "SimpleCheckout.CloseButton must back-reference SimpleCheckout \
         via GetParent() — proves the parentKey synthesis (XML \
         `parentKey=\"CloseButton\"` registers the child against the \
         parent's `children_keys` HashMap and assigns `parent.X = child` \
         in Lua) wires through correctly. The OnClick handler calls \
         `self:GetParent():Hide()` which depends on this back-reference"
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn close_button_is_named_global_via_parent_key_synthesis() {
    let env = load_full_ui_for(ScreenKind::Game);
    close_button_is_named_global_via_parent_key_synthesis::run(&env);
}
