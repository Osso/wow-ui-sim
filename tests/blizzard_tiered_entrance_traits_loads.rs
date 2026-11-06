use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::paths::default_blizzard_ui_addons_path;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

#[path = "common/toc_layout.rs"]
mod toc_layout;

fn blizzard_ui_dir() -> std::path::PathBuf {
    default_blizzard_ui_addons_path().expect("Blizzard UI cache should be synced")
}

fn tiered_entrance_traits_dir() -> std::path::PathBuf {
    blizzard_ui_dir().join("Blizzard_TieredEntranceTraits")
}

fn tiered_entrance_traits_toc() -> std::path::PathBuf {
    tiered_entrance_traits_dir().join("Blizzard_TieredEntranceTraits.toc")
}

const GLUE_SCREENS: &[ScreenKind] = &[
    ScreenKind::Login,
    ScreenKind::CharacterSelect,
    ScreenKind::CharacterCreate,
];

const REQUIRED_DEPS: &[&str] = &["Blizzard_UIFrameManager", "Blizzard_SharedTalentUI"];

const CONTAINER_MIXIN_METHODS: &[&str] = &[
    "OnHide",
    "OnClick",
    "SetTraitTree",
    "SetSpells",
    "Update",
    "SetPressed",
    "UpdateAlignment",
];

const LIST_MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "OrderButtons",
    "GetTemplateForTalentType",
    "CalculateHeight",
    "SetTraitTree",
    "SetSpells",
];

const SPELL_MIXIN_METHODS: &[&str] = &["OnEnter"];

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

// Literal cached inputs absent from Wrath manifest1-382/cache; behavior cases stay enabled.
post_wrath_ui_fixture! {
#[test]
fn find_toc_file_resolves_bare_toc() {
    let resolved =
        find_toc_file(&tiered_entrance_traits_dir()).expect("TieredEntranceTraits TOC resolves");
    assert_eq!(
        resolved,
        tiered_entrance_traits_toc(),
        "Bare TOC — no flavor suffix; mainline-only addon (gated via \
         AllowLoadGameType) but the TOC filename itself has no flavor \
         marker so find_toc_file resolves it via the bare path try"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_is_eager_with_two_required_deps_and_mainline_gametype() {
    let toc = TocFile::from_file(&tiered_entrance_traits_toc()).expect("TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "No `## LoadOnDemand` — TieredEntranceTraits publishes the \
         TieredEntranceTraitsContainer Button template that \
         Blizzard_ScenarioObjectiveTracker.xml inherits at \
         template-registration time, so it must load eagerly before \
         ObjectiveTracker"
    );
    assert_eq!(
        toc.dependencies(),
        REQUIRED_DEPS
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
        "`## RequiredDep: Blizzard_UIFrameManager, Blizzard_SharedTalentUI` \
         — toc.rs:209-217 fallback chain reads RequiredDep (singular) \
         when Dependencies is absent, splits on commas. Two hard deps: \
         UIFrameManager (frame pool helpers used by framePool) + \
         SharedTalentUI (TalentFrameBaseMixin + TalentFrameGridTemplate \
         + TalentFrameDisplayOnlyTemplate that the List inherits). \
         Got: {:?}",
        toc.dependencies()
    );
    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert_eq!(
        toc.is_game_type_restricted(),
        !toc_layout::admits_game_type("mainline"),
        "`## AllowLoadGameType: mainline` is admitted on Retail and PTR (and on Forever as \
         `mainline`), but not on the classic allow-lists"
    );
    assert!(toc.default_enabled());
}
}

post_wrath_ui_fixture! {
#[test]
fn allow_load_absent_restricts_to_game_screen_only() {
    let toc = TocFile::from_file(&tiered_entrance_traits_toc()).expect("TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "AllowLoad absent → toc.rs:305-313 None branch defaults to \
         Game-only — scenario-challenge UI only renders in-world (the \
         scenario tracker block lives on UIParent and shows during \
         scenarios)"
    );
    for screen in GLUE_SCREENS {
        assert!(
            !toc.allows_screen(*screen),
            "Glue screen {screen:?} must be excluded — scenario \
             challenge traits exist only during in-world C_Scenario \
             state"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_raw_bytes_pin_four_metadata_directives() {
    let raw = std::fs::read_to_string(tiered_entrance_traits_toc()).expect("TOC reads utf-8");

    let expected_directives = [
        "## Title: Blizzard Tiered Entrance Traits",
        "## Author: Blizzard Entertainment",
        "## RequiredDep: Blizzard_UIFrameManager, Blizzard_SharedTalentUI",
        "## AllowLoadGameType: mainline",
        "Blizzard_TieredEntranceTraits.lua",
        "Blizzard_TieredEntranceTraits.xml",
    ];

    for directive in expected_directives {
        assert!(
            raw.contains(directive),
            "Raw TOC must pin `{directive}` — 4 metadata directives + \
             2 body files (lua then xml)"
        );
    }

    assert!(!raw.contains("## LoadOnDemand"));
    assert!(!raw.contains("## Dependencies:"));
    assert!(!raw.contains("## OptionalDep"));
    assert!(!raw.contains("## AllowLoad:"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## UseSecureEnvironment"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("## DefaultState"));
}
}

post_wrath_ui_fixture! {
#[test]
fn body_orders_lua_before_xml() {
    let toc = TocFile::from_file(&tiered_entrance_traits_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    assert_eq!(
        body,
        vec![
            "Blizzard_TieredEntranceTraits.lua".to_string(),
            "Blizzard_TieredEntranceTraits.xml".to_string(),
        ],
        "Body must be 2 entries in lua-then-xml order — Lua publishes \
         the 3 mixins, then XML registers the 3 virtual templates that \
         consume them. Got: {body:?}"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn present_only_in_game_screen_eager_discovery() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_TieredEntranceTraits");
    assert!(
        in_game,
        "Blizzard_TieredEntranceTraits must appear in Game eager \
         discovery — not LoadOnDemand, AllowLoad absent → Game-only, \
         AllowLoadGameType=mainline allowed"
    );

    for screen in GLUE_SCREENS {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), *screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_TieredEntranceTraits");
        assert!(
            !found,
            "Blizzard_TieredEntranceTraits must NOT appear in \
             {screen:?} eager discovery — AllowLoad absent → Game-only \
             screen restriction"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn objective_tracker_declares_tiered_entrance_traits_as_required_dep() {
    let toc_path = blizzard_ui_dir()
        .join("Blizzard_ObjectiveTracker")
        .join("Blizzard_ObjectiveTracker.toc");
    let toc = TocFile::from_file(&toc_path).expect("ObjectiveTracker TOC parses");

    assert!(
        toc.dependencies()
            .iter()
            .any(|d| d == "Blizzard_TieredEntranceTraits"),
        "Blizzard_ObjectiveTracker must declare \
         Blizzard_TieredEntranceTraits as a hard RequiredDep — \
         Blizzard_ScenarioObjectiveTracker.xml line 370 inherits the \
         TieredEntranceTraitsContainer template at template-registration \
         time, so the template must be registered before ObjectiveTracker \
         loads. Got deps: {:?}",
        toc.dependencies()
    );
}
}

prefork_full_ui_case! {
fn container_mixin_publishes_with_seven_methods(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(TieredEntranceTraitsContainerMixin)")
        .expect("TieredEntranceTraitsContainerMixin probe");
    assert_eq!(
        kind, "table",
        "TieredEntranceTraitsContainerMixin = table — \
         Blizzard_TieredEntranceTraits.lua line 4 publishes the mixin \
         attached to the virtual TieredEntranceTraitsContainer Button \
         template at xml line 62"
    );

    for method in CONTAINER_MIXIN_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(TieredEntranceTraitsContainerMixin['{method}'])"
            ))
            .unwrap_or_else(|err| {
                panic!("TieredEntranceTraitsContainerMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            kind, "function",
            "TieredEntranceTraitsContainerMixin.{method} must be a \
             function — covers OnHide (hides List+Arrow), OnClick \
             (toggles flyout + lazy-binds traits/spells via needSet \
             flag), SetTraitTree (counts purchased ranks via \
             C_Traits.GetNodeInfo), SetSpells, Update (theme-color \
             overlay via C_ScenarioInfo.GetDisplayInfo), SetPressed \
             (atlas swap), UpdateAlignment (left/right flyout direction \
             based on screen position vs LIST_SAFE_WIDTH=300)"
        );
    }
}
}

prefork_full_ui_case! {
fn list_mixin_publishes_with_six_methods(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(TieredEntranceTraitsListMixin)")
        .expect("TieredEntranceTraitsListMixin probe");
    assert_eq!(
        kind, "table",
        "TieredEntranceTraitsListMixin = table — \
         Blizzard_TieredEntranceTraits.lua line 106 publishes the mixin \
         attached to the virtual TieredEntranceTraitsList Frame \
         template at xml line 4"
    );

    for method in LIST_MIXIN_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(TieredEntranceTraitsListMixin['{method}'])"
            ))
            .unwrap_or_else(|err| {
                panic!("TieredEntranceTraitsListMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            kind, "function",
            "TieredEntranceTraitsListMixin.{method} must be a function \
             — covers OnLoad (creates framePool for spell template, \
             chains TalentFrameBaseMixin.OnLoad), OrderButtons \
             (filters Normal/Maxed visualState + activeRank), \
             GetTemplateForTalentType (always returns \
             TalentButtonScenarioChallengeCircleTemplate), \
             CalculateHeight (rows × LIST_ROW_SPACING=40 + paddings), \
             SetTraitTree (resolves systemID via \
             C_Traits.GetSystemIDByTreeID), SetSpells (icons via \
             C_Spell.GetSpellTexture + AnchorUtil grid layout)"
        );
    }
}
}

prefork_full_ui_case! {
fn spell_mixin_publishes_with_one_method(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(TieredEntranceTraitSpellMixin)")
        .expect("TieredEntranceTraitSpellMixin probe");
    assert_eq!(
        kind, "table",
        "TieredEntranceTraitSpellMixin = table — \
         Blizzard_TieredEntranceTraits.lua line 179 publishes the \
         minimal tooltip-only mixin attached to the virtual \
         TieredEntranceTraitSpellTemplate Frame at xml line 25"
    );

    for method in SPELL_MIXIN_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(TieredEntranceTraitSpellMixin['{method}'])"
            ))
            .unwrap_or_else(|err| {
                panic!("TieredEntranceTraitSpellMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            kind, "function",
            "TieredEntranceTraitSpellMixin.{method} must be a function \
             — OnEnter sets GameTooltip:SetSpellByID(self.spellID); \
             OnLeave is bound directly to GameTooltip_Hide as a \
             function= attribute (no mixin method needed)"
        );
    }
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

    load_addon(&env.loader_env(), &tiered_entrance_traits_toc())
        .expect("Blizzard_TieredEntranceTraits must load via Rust loader");

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let needles = [
        "Blizzard_TieredEntranceTraits",
        "TieredEntranceTraitsContainerMixin",
        "TieredEntranceTraitsListMixin",
        "TieredEntranceTraitSpellMixin",
        "TieredEntranceTraitsContainer",
        "TieredEntranceTraitsList",
        "TieredEntranceTraitSpellTemplate",
    ];

    let matched: Vec<&String> = errors
        .iter()
        .filter(|e| needles.iter().any(|n| e.contains(n)))
        .collect();

    assert!(
        matched.is_empty(),
        "Explicit re-load_addon for TieredEntranceTraits must emit \
         zero addon-specific Lua errors. Found {} matching: {:#?}",
        matched.len(),
        matched
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_reports_true_after_eager_sweep(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_TieredEntranceTraits')")
        .expect("IsAddOnLoaded probe");
    assert!(
        loaded,
        "After full Game eager sweep, IsAddOnLoaded must report true \
         — TieredEntranceTraits is non-LoD, RequiredDeps satisfied \
         (UIFrameManager + SharedTalentUI both present)"
    );
}
}
