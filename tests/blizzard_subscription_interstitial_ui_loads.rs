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

fn subscription_interstitial_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_SubscriptionInterstitialUI")
}

fn subscription_interstitial_toc() -> PathBuf {
    subscription_interstitial_dir().join("Blizzard_SubscriptionInterstitialUI.toc")
}

const GLUE_SCREENS: &[ScreenKind] = &[
    ScreenKind::Login,
    ScreenKind::CharacterSelect,
    ScreenKind::CharacterCreate,
];

const FRAME_MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "OnShow",
    "OnHide",
    "OnEvent",
    "OnCinematicStarting",
    "OnCinematicStopped",
    "SetInterstitialType",
];

const SUBSCRIBE_BUTTON_BASE_MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "OnShow",
    "OnClick",
    "WasClicked",
    "ClearClickState",
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
    let resolved =
        find_toc_file(&subscription_interstitial_dir()).expect("SubscriptionInterstitialUI TOC");
    assert_eq!(
        resolved,
        subscription_interstitial_toc(),
        "Bare TOC — no flavor suffix; addon ships only the mainline body"
    );
}

#[test]
fn toc_is_load_on_demand_with_no_dependencies() {
    let toc = TocFile::from_file(&subscription_interstitial_toc()).expect("TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "`## LoadOnDemand: 1` — interstitial only loads when the engine \
         fires SHOW_SUBSCRIPTION_INTERSTITIAL (post-trial expiry, \
         max-level subscribe nag, etc.)"
    );
    assert!(
        toc.dependencies().is_empty(),
        "No `## RequiredDep` / `## Dependencies` — leaf addon. Got: {:?}",
        toc.dependencies()
    );
    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
}

#[test]
fn toc_has_no_allow_load_or_game_type_restriction_or_secure_env() {
    let toc = TocFile::from_file(&subscription_interstitial_toc()).expect("TOC parses");

    assert!(
        !toc.is_game_type_restricted(),
        "No `## AllowLoadGameType` — interstitial works on every flavor"
    );
    assert!(!toc.is_secure_env());
    assert!(!toc.is_load_first());
    assert!(toc.default_enabled());
}

#[test]
fn allow_load_absent_restricts_to_game_screen_only() {
    let toc = TocFile::from_file(&subscription_interstitial_toc()).expect("TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "AllowLoad absent → toc.rs:305-313 None branch → Game-only"
    );
    for screen in GLUE_SCREENS {
        assert!(
            !toc.allows_screen(*screen),
            "Glue screen {screen:?} excluded — sub nag only fires in-world"
        );
    }
}

#[test]
fn toc_raw_bytes_pin_metadata_and_bootstrap_xml_body() {
    let raw = std::fs::read_to_string(subscription_interstitial_toc()).expect("TOC reads utf-8");

    let expected_directives = [
        "## Title: Blizzard Subscription Interstitial UI",
        "## LoadOnDemand: 1",
        "Blizzard_SubscriptionInterstitialUI_Bootstrap.lua [Bootstrap]",
        "Blizzard_SubscriptionInterstitialUI.xml",
    ];

    for directive in expected_directives {
        assert!(
            raw.contains(directive),
            "Raw TOC must pin current SubscriptionInterstitial metadata and body entry `{directive}`"
        );
    }

    assert!(!raw.contains("## Dependencies"));
    assert!(!raw.contains("## RequiredDep"));
    assert!(!raw.contains("## OptionalDep"));
    assert!(!raw.contains("## AllowLoad"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## UseSecureEnvironment"));
    assert!(!raw.contains("## LoadFirst"));
}

#[test]
fn body_resolves_bootstrap_then_xml_entry() {
    let toc = TocFile::from_file(&subscription_interstitial_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    assert_eq!(
        body,
        vec![
            "Blizzard_SubscriptionInterstitialUI_Bootstrap.lua".to_string(),
            "Blizzard_SubscriptionInterstitialUI.xml".to_string(),
        ],
        "Bootstrap loads before the XML, which chains the matching .lua via <Script file=...>. \
         Got: {body:?}"
    );
}

#[test]
fn xml_chains_lua_via_script_file_directive() {
    let xml_path = subscription_interstitial_dir().join("Blizzard_SubscriptionInterstitialUI.xml");
    let raw = std::fs::read_to_string(&xml_path).expect("XML reads");

    assert!(
        raw.contains("<Script file=\"Blizzard_SubscriptionInterstitialUI.lua\""),
        "XML must `<Script file=\"Blizzard_SubscriptionInterstitialUI.lua\"/>` \
         — drives the 5-mixin publication chain since the .lua is NOT \
         listed in the TOC body directly"
    );
}

#[test]
fn absent_from_game_eager_discovery() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let found = addons
        .iter()
        .any(|(name, _)| name == "Blizzard_SubscriptionInterstitialUI");
    assert!(
        !found,
        "`## LoadOnDemand: 1` excludes from Game eager sweep — no other \
         addon declares it as a dep, so pull_required_lod_addons does NOT \
         promote it"
    );
}

#[test]
fn absent_from_every_glue_screen_discovery() {
    for screen in GLUE_SCREENS {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), *screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_SubscriptionInterstitialUI");
        assert!(
            !found,
            "Must not appear on glue screen {screen:?} — AllowLoad absent \
             defaults to Game-only AND LoadOnDemand both exclude"
        );
    }
}

prefork_full_ui_case! {
fn explicit_load_addon_emits_no_addon_specific_errors(env: &WowLuaEnv) {

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_addon(&env.loader_env(), &subscription_interstitial_toc())
        .expect("SubscriptionInterstitialUI must load via Rust loader");

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let needles = [
        "Blizzard_SubscriptionInterstitialUI.lua",
        "Blizzard_SubscriptionInterstitialUI.xml",
        "SubscriptionInterstitialFrameMixin",
        "SubscriptionInterstitialSubscribeButton",
        "SubscriptionInterstitialUpgradeButton",
        "SubscriptionInterstitialCloseButton",
        "Blizzard_SubscriptionInterstitialUI",
    ];

    let matched: Vec<&String> = errors
        .iter()
        .filter(|e| needles.iter().any(|n| e.contains(n)))
        .collect();

    assert!(
        matched.is_empty(),
        "Explicit load_addon must emit zero addon-specific Lua errors. \
         Found {} matching: {:#?}",
        matched.len(),
        matched
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_reports_true_after_explicit_load(env: &WowLuaEnv) {

    let before: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_SubscriptionInterstitialUI')")
        .expect("IsAddOnLoaded probe");
    assert!(
        !before,
        "Before explicit load, IsAddOnLoaded must report false"
    );

    load_addon(&env.loader_env(), &subscription_interstitial_toc())
        .expect("SubscriptionInterstitialUI must load via Rust loader");

    let after: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_SubscriptionInterstitialUI')")
        .expect("IsAddOnLoaded probe");
    assert!(
        after,
        "After explicit load_addon, IsAddOnLoaded must report true"
    );
}
}

prefork_full_ui_case! {
fn frame_mixin_publishes_with_seven_lifecycle_methods(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &subscription_interstitial_toc())
        .expect("SubscriptionInterstitialUI must load via Rust loader");

    let kind: String = env
        .eval("return type(SubscriptionInterstitialFrameMixin)")
        .expect("FrameMixin probe");
    assert_eq!(
        kind, "table",
        "SubscriptionInterstitialFrameMixin = table — drives the named \
         SubscriptionInterstitialFrame Frame, registers \
         SHOW_SUBSCRIPTION_INTERSTITIAL event, manages cinematic-aware \
         hide-then-reshow flow via EventRegistry callbacks"
    );

    for method in FRAME_MIXIN_METHODS {
        let probe = format!("return type(SubscriptionInterstitialFrameMixin['{method}'])");
        let kind: String = env
            .eval(&probe)
            .unwrap_or_else(|err| panic!("FrameMixin.{method} probe: {err}"));
        assert_eq!(
            kind, "function",
            "FrameMixin.{method} = function — OnLoad registers event, \
             OnShow/OnHide manage cinematic-paused state via \
             EventRegistry, OnEvent branches on SHOW_SUBSCRIPTION_INTERSTITIAL \
             firing SetInterstitialType + ShowUIPanel, \
             OnCinematicStarting/Stopped re-show after cinematic break, \
             SetInterstitialType toggles SubscribeButton vs UpgradeButton \
             based on Enum.SubscriptionInterstitialType.MaxLevel"
        );
    }
}
}

prefork_full_ui_case! {
fn subscribe_button_base_mixin_publishes_with_five_methods(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &subscription_interstitial_toc())
        .expect("SubscriptionInterstitialUI must load via Rust loader");

    let kind: String = env
        .eval("return type(SubscriptionInterstitialSubscribeButtonBaseMixin)")
        .expect("SubscribeButtonBaseMixin probe");
    assert_eq!(
        kind, "table",
        "SubscribeButtonBaseMixin = table — base for both Subscribe and \
         Upgrade button mixins (DRY pattern: parent/child mixin chain via \
         direct `BaseMixin.OnLoad(self)` rather than CreateFromMixins)"
    );

    for method in SUBSCRIBE_BUTTON_BASE_MIXIN_METHODS {
        let probe =
            format!("return type(SubscriptionInterstitialSubscribeButtonBaseMixin['{method}'])");
        let kind: String = env
            .eval(&probe)
            .unwrap_or_else(|err| panic!("BaseMixin.{method} probe: {err}"));
        assert_eq!(
            kind, "function",
            "BaseMixin.{method} = function — OnLoad sets background atlas \
             + scales button text, OnShow clears click state, OnClick \
             plays sound + sends Clicked/WebRedirect response via \
             SendSubscriptionInterstitialResponse, WasClicked/\
             ClearClickState manage the wasClicked flag consumed by \
             frame's OnHide to send a Closed response when neither button \
             was hit"
        );
    }
}
}

prefork_full_ui_case! {
fn subscribe_and_upgrade_button_mixins_publish_with_their_specializations(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &subscription_interstitial_toc())
        .expect("SubscriptionInterstitialUI must load via Rust loader");

    for (mixin, expected_methods) in [
        (
            "SubscriptionInterstitialSubscribeButtonMixin",
            &["OnLoad"][..],
        ),
        (
            "SubscriptionInterstitialUpgradeButtonMixin",
            &["OnLoad"][..],
        ),
        ("SubscriptionInterstitialCloseButtonMixin", &["OnClick"][..]),
    ] {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} probe: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} = table — Subscribe specializes Base.OnLoad to scale \
             FirstLine/SecondLine/ThirdLine, Upgrade specializes Base.OnLoad \
             to lay out a 10-bullet GridLayoutFactoryByCount via \
             AnchorUtil + bulletPointPool of \
             SubscriptionInterstitialBulletPointTemplate, Close clicks \
             call HideUIPanel"
        );
        for method in expected_methods {
            let kind: String = env
                .eval(&format!("return type({mixin}['{method}'])"))
                .unwrap_or_else(|err| panic!("{mixin}.{method} probe: {err}"));
            assert_eq!(kind, "function", "{mixin}.{method} = function");
        }
    }
}
}

prefork_full_ui_case! {
fn ui_panel_window_entry_seeds_for_subscription_interstitial_frame(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &subscription_interstitial_toc())
        .expect("SubscriptionInterstitialUI must load via Rust loader");

    let probe = "local entry = UIPanelWindows and UIPanelWindows['SubscriptionInterstitialFrame'] \
                  return type(entry) == 'table' \
                     and entry.area == 'center' \
                     and entry.pushable == 0 \
                     and entry.whileDead == 1";

    let entry_present: bool = env.eval(probe).expect("UIPanelWindows entry probe");
    assert!(
        entry_present,
        "Blizzard_SubscriptionInterstitialUI.lua line 5 sets \
         `UIPanelWindows['SubscriptionInterstitialFrame'] = {{ area = 'center', \
         pushable = 0, whileDead = 1 }}` so the panel participates in the \
         central UIParent panel-stacking system as a non-pushable \
         center-area panel that can show while dead"
    );
}
}

prefork_full_ui_case! {
fn subscription_interstitial_frame_materializes_as_named_frame(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &subscription_interstitial_toc())
        .expect("SubscriptionInterstitialUI must load via Rust loader");

    let probe = "local f = _G.SubscriptionInterstitialFrame \
                  return type(f) == 'table' and f:GetParent() == UIParent";

    let frame_present: bool = env.eval(probe).expect("named frame probe");
    assert!(
        frame_present,
        "SubscriptionInterstitialFrame must materialize as a named Frame \
         parented to UIParent — XML declares mixin=\
         SubscriptionInterstitialFrameMixin inherits=DefaultPanelTemplate \
         toplevel=true hidden=true with SubscribeButton + UpgradeButton + \
         CloseButton + Inset + ShadowOverlay + Title fontstring children"
    );
}
}

prefork_full_ui_case! {
fn subscription_interstitial_button_templates_materialize_via_create_frame(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &subscription_interstitial_toc())
        .expect("SubscriptionInterstitialUI must load via Rust loader");

    let bullet_probe = "local ok, frame = pcall(function() \
                          return CreateFrame('Frame', nil, UIParent, \
                                              'SubscriptionInterstitialBulletPointTemplate') \
                        end) \
                        return ok and frame ~= nil";
    let bullet_ok: bool = env.eval(bullet_probe).expect("bullet template probe");
    assert!(
        bullet_ok,
        "SubscriptionInterstitialBulletPointTemplate (virtual=true Frame \
         320x16 with interstitial-newplayerexperience-upgrade-bullet \
         atlas + Text FontString) must materialize via CreateFrame — \
         consumed by UpgradeButton's bulletPointPool"
    );

    let inheritor_probe = "local sub = SubscriptionInterstitialFrame and \
                                     SubscriptionInterstitialFrame.SubscribeButton \
                            local up = SubscriptionInterstitialFrame and \
                                     SubscriptionInterstitialFrame.UpgradeButton \
                            return sub ~= nil and up ~= nil";
    let inheritors_present: bool = env
        .eval(inheritor_probe)
        .expect("subscribe/upgrade button child probe");
    assert!(
        inheritors_present,
        "SubscriptionInterstitialSubscribeButtonTemplate is an abstract \
         template — its OnLoad reads self.backgroundAtlas KeyValue and \
         self.ButtonText FontString that only the concrete inheritors \
         SubscribeButton + UpgradeButton supply, so direct CreateFrame \
         would crash. Verify both concrete inheritors exist as children \
         of SubscriptionInterstitialFrame instead"
    );
}
}
