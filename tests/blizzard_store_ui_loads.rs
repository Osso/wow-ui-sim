use std::path::PathBuf;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn store_ui_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_StoreUI")
}

fn store_ui_toc() -> PathBuf {
    store_ui_dir().join("Blizzard_StoreUI.toc")
}

const ALL_FOUR_SCREENS: &[ScreenKind] = &[
    ScreenKind::Game,
    ScreenKind::Login,
    ScreenKind::CharacterSelect,
    ScreenKind::CharacterCreate,
];

const STORE_BUTTON_MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "OnShow",
    "OnDisable",
    "OnEnable",
    "OnMouseDown",
    "OnMouseUp",
];

const BODY_FILES: &[&str] = &[
    "Blizzard_Shared_StoreButtonMixin.lua",
    "Blizzard_Shared_StoreUITemplates.xml",
    "Blizzard_Shared_ProductCardTemplates.xml",
    "Blizzard_Shared_StoreUIToggle.lua",
    "Blizzard_Shared_StoreUI.xml",
    "Blizzard_Shared_StoreUIInsecure.lua",
    "Shared_Localization.lua",
];

fn fresh_game_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    env
}

fn load_full_game_ui() -> WowLuaEnv {
    let env = fresh_game_env();

    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn find_toc_file_resolves_bare_toc() {
    let resolved = find_toc_file(&store_ui_dir()).expect("StoreUI TOC resolves");
    assert_eq!(
        resolved,
        store_ui_toc(),
        "Current retail cache ships one bare Blizzard_StoreUI.toc with \
         per-file game-type annotations"
    );
}

#[test]
fn toc_dependencies_include_shared_and_screen_scoped_addons() {
    let toc = TocFile::from_file(&store_ui_toc()).expect("TOC parses");

    let deps = toc.dependencies();
    assert_eq!(
        deps,
        vec![
            "Blizzard_SharedXML".to_string(),
            "Blizzard_GameMenuEsc".to_string(),
            "Blizzard_SimpleCheckout".to_string(),
            "Blizzard_CatalogShop".to_string(),
            "Blizzard_GlueParent".to_string(),
            "Blizzard_FrameXMLBase".to_string(),
        ],
        "Repeated `## Dep:` directives include shared dependencies plus \
         screen-scoped glue and game dependencies. Got: {deps:?}"
    );
}

#[test]
fn toc_declares_plunderstorm_as_optional_dependency() {
    let toc = TocFile::from_file(&store_ui_toc()).expect("TOC parses");

    assert_eq!(
        toc.optional_deps(),
        vec!["Blizzard_Plunderstorm".to_string()],
        "Retail 12.1.0.69497 declares Blizzard_Plunderstorm as StoreUI's optional dependency"
    );
}

#[test]
fn toc_uses_secure_environment() {
    let toc = TocFile::from_file(&store_ui_toc()).expect("TOC parses");

    assert!(
        toc.is_secure_env(),
        "`## UseSecureEnvironment: 1` — the StoreUI body files load into \
         __secureenv. Secure-env mixin lookups must fall back through \
         `_G[name] or rawget(__secureenv, name)` to find StoreButtonMixin"
    );
}

#[test]
fn toc_allow_load_both_resolves_to_all_four_screens() {
    let toc = TocFile::from_file(&store_ui_toc()).expect("TOC parses");

    for screen in ALL_FOUR_SCREENS {
        assert!(
            toc.allows_screen(*screen),
            "`## AllowLoad: Both` must surface StoreUI on {screen:?} — store \
             can be opened from the in-game escape menu and from glue screens"
        );
    }
}

#[test]
fn toc_has_no_addon_level_game_type_restriction() {
    let toc = TocFile::from_file(&store_ui_toc()).expect("TOC parses");

    assert!(
        !toc.is_game_type_restricted(),
        "The bare TOC has no addon-level `## AllowLoadGameType`; flavor \
         selection happens on individual body entries"
    );
}

#[test]
fn toc_is_eager_no_load_on_demand_or_load_first() {
    let toc = TocFile::from_file(&store_ui_toc()).expect("TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "Eager — store may need to react to entitlement-changed events at \
         file scope before the player ever opens the panel"
    );
    assert!(!toc.is_load_first());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.default_enabled());
}

#[test]
fn toc_raw_bytes_pin_current_metadata_directives() {
    let raw = std::fs::read_to_string(store_ui_toc()).expect("TOC reads utf-8");

    let expected_directives = [
        "## Title: Blizzard_StoreUI",
        "## AllowLoad: Both",
        "## Dep: Blizzard_SharedXML",
        "## Dep: Blizzard_GameMenuEsc",
        "## Dep: Blizzard_SimpleCheckout",
        "## Dep: Blizzard_CatalogShop",
        "## Dep: Blizzard_GlueParent [AllowLoad glue]",
        "## Dep: Blizzard_FrameXMLBase [AllowLoad game]",
        "## OptionalDeps: Blizzard_Plunderstorm",
        "## UseSecureEnvironment: 1",
    ];

    for directive in expected_directives {
        assert!(
            raw.contains(directive),
            "Raw bare TOC must contain current directive `{directive}`"
        );
    }

    assert!(!raw.contains("## AllowLoadGameType:"));
    assert!(!raw.contains("## DefaultState"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## LoadOnDemand"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("## Dependencies:"));
}

#[test]
fn retail_body_orders_seven_selected_entries_terminating_in_localization() {
    let toc = TocFile::from_file(&store_ui_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    assert_eq!(
        body.len(),
        BODY_FILES.len(),
        "Retail parsing selects seven mainline-gated entries, including \
         Blizzard_Shared_StoreUIToggle.lua. Got: {body:?}"
    );

    for (i, want) in BODY_FILES.iter().enumerate() {
        assert_eq!(&body[i], want, "Body entry {i}: expected {want}");
    }
}

#[test]
fn appears_in_eager_discovery_on_all_four_screens() {
    let ui = blizzard_ui_dir();

    for screen in ALL_FOUR_SCREENS {
        let addons = discover_blizzard_addons_for_screen(&ui, *screen);
        let found = addons.iter().any(|(name, _)| name == "Blizzard_StoreUI");
        assert!(
            found,
            "`## AllowLoad: Both` must surface StoreUI on {screen:?} eager \
             discovery — store panel needs to be ready on every screen"
        );
    }
}

prefork_full_ui_case! {
fn full_game_load_emits_no_addon_specific_lua_errors(env: &WowLuaEnv) {

    let errors = env.state().borrow().lua_errors.clone();
    let needles = [
        "Blizzard_Shared_StoreButtonMixin.lua",
        "Blizzard_Shared_StoreUI.xml",
        "Blizzard_Shared_StoreUITemplates.xml",
        "Blizzard_Shared_ProductCardTemplates.xml",
        "Blizzard_Shared_StoreUIInsecure.lua",
        "Blizzard_StoreUI",
        "StoreButtonMixin",
    ];

    let matched: Vec<&String> = errors
        .iter()
        .filter(|e| needles.iter().any(|n| e.contains(n)))
        .collect();

    assert!(
        matched.is_empty(),
        "Full Game-screen load must emit zero StoreUI-specific Lua errors. \
         Found {} matching: {:#?}",
        matched.len(),
        matched
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_reports_true_after_eager_load(env: &WowLuaEnv) {
    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_StoreUI')")
        .expect("IsAddOnLoaded probe");
    assert!(
        loaded,
        "After eager Game-screen sweep, IsAddOnLoaded must report true \
         (AllowLoad: Both + non-LoD = always loaded eagerly)"
    );
}
}

prefork_full_ui_case! {
fn store_button_mixin_publishes_with_six_atlas_swap_methods(env: &WowLuaEnv) {

    let probe = "local function lookup(name) \
                    return _G[name] or (__secureenv and rawget(__secureenv, name)) \
                  end \
                  local m = lookup('StoreButtonMixin') \
                  return type(m) == 'table'";

    let table_present: bool = env.eval(probe).expect("StoreButtonMixin probe");
    assert!(
        table_present,
        "StoreButtonMixin = table — Blizzard_Shared_StoreButtonMixin.lua \
         line 4 publishes it. Lookup falls back through __secureenv since \
         UseSecureEnvironment=1 routes the body into the secure env"
    );

    for method in STORE_BUTTON_MIXIN_METHODS {
        let probe = format!(
            "local function lookup(name) \
                return _G[name] or (__secureenv and rawget(__secureenv, name)) \
             end \
             local m = lookup('StoreButtonMixin') \
             return type(m['{method}']) == 'function'"
        );
        let present: bool = env
            .eval(&probe)
            .unwrap_or_else(|err| panic!("StoreButtonMixin.{method} probe: {err}"));
        assert!(
            present,
            "StoreButtonMixin.{method} = function — atlas-swap helper for \
             shop-button-large-{{up,down,disabled}}-{{left,middle,right}} \
             three-slice button states (6 methods total)"
        );
    }
}
}

#[test]
fn store_ui_xml_script_file_includes_chain_through_seven_lua_files() {
    let templates_xml = store_ui_dir().join("Blizzard_Shared_StoreUITemplates.xml");
    let product_xml = store_ui_dir().join("Blizzard_Shared_ProductCardTemplates.xml");
    let main_xml = store_ui_dir().join("Blizzard_Shared_StoreUI.xml");

    let templates = std::fs::read_to_string(&templates_xml).expect("templates XML reads");
    let product = std::fs::read_to_string(&product_xml).expect("product XML reads");
    let main = std::fs::read_to_string(&main_xml).expect("main XML reads");

    assert!(
        templates.contains("<Script file=\"Blizzard_Shared_StoreUITemplates.lua\""),
        "Templates XML must <Script file=...> the matching .lua so its 3 \
         mixins (StoreTooltipBackdrop / StoreBulletPoint / \
         CategoryTreeScrollContainer) load alongside the templates"
    );

    let product_includes = [
        "Blizzard_Shared_ProductCardBuyButtonMixin.lua",
        "Blizzard_Shared_ProductCardTemplates.lua",
        "Blizzard_Shared_SmallProductCardTemplates.lua",
        "Blizzard_Shared_LargeProductCardTemplates.lua",
        "Blizzard_Shared_FullProductCardTemplates.lua",
        "Blizzard_Shared_ProductCardMagnifierTemplates.lua",
    ];
    for include in product_includes {
        assert!(
            product.contains(include),
            "ProductCardTemplates XML must <Script file=\"{include}\"> — 6 \
             card-style variants (small/large/full + buy button + magnifier \
             zoom + base) load through the XML script-include chain"
        );
    }

    let main_includes = [
        "Blizzard_Shared_StoreUIInbound.lua",
        "Blizzard_Shared_StoreUIOutbound.lua",
        "Blizzard_Shared_StoreUISecure.lua",
    ];
    for include in main_includes {
        assert!(
            main.contains(include),
            "Main StoreUI XML must <Script file=\"{include}\"> — Inbound + \
             Outbound bridge tables flank the giant Secure.lua (4324 lines)"
        );
    }
}

#[test]
fn store_ui_xml_wraps_body_in_scoped_modifier_forbidden() {
    let main_xml = store_ui_dir().join("Blizzard_Shared_StoreUI.xml");
    let raw = std::fs::read_to_string(&main_xml).expect("main XML reads");

    assert!(
        raw.contains("<ScopedModifier forbidden=\"true\">"),
        "Main StoreUI XML must wrap its body in `<ScopedModifier \
         forbidden=\"true\">` — every named frame (StoreFrame, StoreDialog, \
         StoreStateDriverFrame, StoreConfirmationFrame, \
         StoreVASValidationFrame, StoreTooltip, ServicesLogoutPopup) is \
         created as a forbidden frame, paired with UseSecureEnvironment=1"
    );
}

#[test]
fn store_ui_inbound_outbound_secure_bridge_files_exist() {
    let inbound = store_ui_dir().join("Blizzard_Shared_StoreUIInbound.lua");
    let outbound = store_ui_dir().join("Blizzard_Shared_StoreUIOutbound.lua");
    let secure = store_ui_dir().join("Blizzard_Shared_StoreUISecure.lua");
    let insecure = store_ui_dir().join("Blizzard_Shared_StoreUIInsecure.lua");

    for path in [&inbound, &outbound, &insecure] {
        let raw = std::fs::read_to_string(path).expect("bridge file reads");
        assert!(
            raw.contains("SwapToGlobalEnvironment()"),
            "{} must call SwapToGlobalEnvironment() — bridge files \
             intentionally re-enter the global env to break out of \
             __secureenv for the cross-boundary publication points",
            path.display()
        );
    }

    let secure_raw = std::fs::read_to_string(&secure).expect("secure file reads");
    assert!(
        !secure_raw.contains("SwapToGlobalEnvironment()"),
        "Secure.lua must NOT call SwapToGlobalEnvironment — it stays in \
         __secureenv for the entire 4324-line body, hosting the \
         secure-only logic (purchase flow, payment handshake, etc.)"
    );
}

#[test]
fn bare_toc_keeps_flavor_selection_on_body_entries() {
    let raw = std::fs::read_to_string(store_ui_toc()).expect("TOC reads utf-8");

    assert!(
        raw.contains("Blizzard_Shared_StoreButtonMixin.lua [AllowLoadGameType mainline]"),
        "Current retail cache must expose the mainline StoreButtonMixin entry"
    );
    assert!(
        raw.contains("[Family]\\Blizzard_StoreUIInsecure.lua [AllowLoadGameType classic]"),
        "Current retail cache's bare TOC must retain its inline classic entry; \
         this assertion observes the cached file without claiming another \
         profile's parsed behavior"
    );
}
