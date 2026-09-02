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

fn ui_widgets_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_UIWidgets")
}

fn ui_widgets_toc() -> PathBuf {
    ui_widgets_dir().join("Blizzard_UIWidgets.toc")
}

const GLUE_SCREENS: &[ScreenKind] = &[
    ScreenKind::Login,
    ScreenKind::CharacterSelect,
    ScreenKind::CharacterCreate,
];

const TOC_DEPENDENCIES: &[&str] = &[
    "Blizzard_Minimap",
    "Blizzard_Colors",
    "Blizzard_LFGUtil",
    "Blizzard_ManagedFrameSystem",
];

const REPRESENTATIVE_BODY_FILES: &[&str] = &[
    "Blizzard_WidgetsUtil.lua",
    "Blizzard_UIWidgetManager.lua",
    "Blizzard_UIWidgetManager.xml",
    "Blizzard_UIWidgetAnimationTemplates.lua",
    "Blizzard_UIWidgetTemplateBase.lua",
    "Blizzard_UIWidgetTemplateBase.xml",
    "Blizzard_UIWidgetTemplateScenarioHeaderTimer.lua",
    "Blizzard_UIWidgetTemplateFillUpFrames.lua",
    "Blizzard_UIWidgetTopCenterFrame.xml",
    "Blizzard_UIWidgetBelowMinimapFrame.xml",
    "Blizzard_UIWidgetPowerBarFrame.xml",
    "Blizzard_UIWidgetCenterScreenFrame.xml",
];

const MANAGER_AND_CONTAINER_MIXINS: &[&str] = &[
    "UIWidgetManagerMixin",
    "UIWidgetHorizontalWidgetContainerMixin",
    "UIWidgetContainerMixin",
    "UIWidgetContainerResizeMixin",
    "UIWidgetTopCenterContainerMixin",
    "UIWidgetBelowMinimapContainerMixin",
    "UIWidgetPowerBarContainerMixin",
    "UIWidgetCenterScreenContainerMixin",
];

const BASE_AND_HELPER_MIXINS: &[&str] = &[
    "UIWidgetTemplateTooltipFrameMixin",
    "UIWidgetBaseEnabledFrameMixin",
    "UIWidgetBaseTemplateMixin",
    "UIWidgetBaseStatusBarTemplateMixin",
    "UIWidgetBaseStatusBarPartitionTemplateMixin",
    "UIWidgetBaseResourceTemplateMixin",
    "UIWidgetBaseCurrencyTemplateMixin",
    "UIWidgetBaseSpellTemplateMixin",
    "UIWidgetBaseIconTemplateMixin",
    "UIWidgetBaseItemTemplateMixin",
    "UIWidgetBaseStateIconTemplateMixin",
    "UIWidgetBaseTextureAndTextTemplateMixin",
    "UIWidgetBaseControlZoneTemplateMixin",
    "UIWidgetBaseScenarioHeaderTemplateMixin",
    "UIWidgetBaseCircularStatusBarTemplateMixin",
    "UIWidgetBaseTextMixin",
    "UIWidgetBaseButtonTemplateMixin",
    "UIWidgetFillUpFrameTemplateMixin",
];

const TEMPLATE_MIXINS: &[&str] = &[
    "UIWidgetTemplateIconAndTextMixin",
    "UIWidgetTemplateIconTextAndBackgroundMixin",
    "UIWidgetTemplateCaptureBarMixin",
    "UIWidgetTemplateStatusBarMixin",
    "UIWidgetTemplateDoubleStatusBarMixin",
    "UIWidgetTemplateDoubleIconAndTextMixin",
    "UIWidgetTemplateStackedResourceTrackerMixin",
    "UIWidgetTemplateIconTextAndCurrenciesMixin",
    "UIWidgetTemplateTextWithStateMixin",
    "UIWidgetTemplateHorizontalCurrenciesMixin",
    "UIWidgetTemplateBulletTextListMixin",
    "UIWidgetTemplateScenarioHeaderCurrenciesAndBackgroundMixin",
    "UIWidgetTemplateTextureAndTextMixin",
    "UIWidgetTemplateSpellDisplayMixin",
    "UIWidgetTemplateDoubleStateIconRowMixin",
    "UIWidgetTemplateTextureAndTextRowMixin",
    "UIWidgetTemplateZoneControlMixin",
    "UIWidgetTemplateCaptureZoneMixin",
    "UIWidgetTemplateTextureWithAnimationMixin",
    "UIWidgetTemplateDiscreteProgressStepsMixin",
    "UIWidgetTemplateScenarioHeaderTimerMixin",
    "UIWidgetTemplateTextColumnRowMixin",
    "UIWidgetTemplateSpacerMixin",
    "UIWidgetTemplateUnitPowerBarMixin",
    "UIWidgetTemplateFillUpFramesMixin",
    "UIWidgetTemplateTextWithSubtextMixin",
    "UIWidgetTemplateItemDisplayMixin",
    "UIWidgetTemplateTugOfWarMixin",
    "UIWidgetTemplateMapPinAnimationMixin",
    "UIWidgetTemplateScenarioHeaderDelvesMixin",
    "UIWidgetTemplateScenarioHeaderDelvesTierFrameMixin",
    "UIWidgetTemplateButtonHeaderMixin",
    "UIWidgetTemplatePreyHuntProgressMixin",
];

const FLIPBOOK_AND_NESTED_MIXINS: &[&str] = &[
    "DecorFlipbookAnimMixin",
    "FilledFlipbookAnimMixin",
    "BurstFlipbookAnimMixin",
    "TorghastGemsAnimationMixin",
    "UIWidgetTemplateBulletTextListLineMixin",
    "UIWidgetTemplateSpellDisplaySpellMixin",
    "UIWidgetTemplateTextColumnRowColumnMixin",
];

const REPRESENTATIVE_VIRTUAL_TEMPLATES: &[&str] = &[
    "UIWidgetContainerTemplate",
    "UIWidgetContainerNoResizeTemplate",
    "UIWidgetHorizontalWidgetContainerTemplate",
    "UIWidgetBaseTemplate",
    "UIWidgetBaseStatusBarTemplate",
    "UIWidgetBaseIconTemplate",
    "UIWidgetBaseItemTemplate",
    "UIWidgetBaseSpellTemplate",
    "UIWidgetBaseStateIconTemplate",
    "UIWidgetBaseCurrencyTemplate",
    "UIWidgetBaseResourceTemplate",
    "UIWidgetBaseControlZoneTemplate",
    "UIWidgetBaseTextureAndTextTemplate",
    "UIWidgetBaseScenarioHeaderTemplate",
    "UIWidgetBaseCircularStatusBarTemplate",
    "UIWidgetBaseStatusBarPartitionTemplate",
    "UIWidgetBaseButtonTemplate",
    "UIWidgetFillUpFrameTemplate",
    "UIWidgetTemplateIconAndText",
    "UIWidgetTemplateStatusBar",
    "UIWidgetTemplateDoubleStatusBar",
    "UIWidgetTemplateCaptureBar",
    "UIWidgetTemplateCaptureZone",
    "UIWidgetTemplateZoneControl",
    "UIWidgetTemplateScenarioHeaderTimer",
    "UIWidgetTemplateScenarioHeaderDelves",
    "UIWidgetTemplateMapPinAnimation",
    "UIWidgetTemplateTugOfWar",
    "UIWidgetTemplateItemDisplay",
    "UIWidgetTemplateSpellDisplay",
    "UIWidgetTemplateFillUpFrames",
    "UIWidgetTemplateBulletTextList",
    "UIWidgetTemplateBulletTextListLine",
    "UIWidgetTemplateButtonHeader",
    "UIWidgetTemplatePreyHuntProgress",
    "UIWidgetTemplateTextColumnRow",
    "UIWidgetTemplateTextColumnRowColumnTemplate",
    "UIWidgetTemplateSpacer",
    "TorghastGemsAnimationTemplate",
];

const NAMED_TOP_LEVEL_FRAMES: &[&str] = &[
    "UIWidgetManager",
    "UIWidgetTopCenterContainerFrame",
    "UIWidgetBelowMinimapContainerFrame",
    "UIWidgetCenterScreenContainerFrame",
    "UIWidgetPowerBarContainerFrame",
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
    let resolved = find_toc_file(&ui_widgets_dir()).expect("UIWidgets TOC resolves");
    assert_eq!(
        resolved,
        ui_widgets_toc(),
        "Blizzard_UIWidgets ships the active retail contract in one bare TOC"
    );
}

#[test]
fn toc_is_eager_with_four_dependencies() {
    let toc = TocFile::from_file(&ui_widgets_toc()).expect("TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "No `## LoadOnDemand` directive → eagerly loaded. The widget \
         system publishes the global UIWidgetManager singleton frame \
         that registers UPDATE_UI_WIDGET / UPDATE_ALL_UI_WIDGETS event \
         handlers — it must be alive at PLAYER_ENTERING_WORLD so server \
         widget updates have somewhere to land"
    );

    let deps = toc.dependencies();
    assert_eq!(
        deps, TOC_DEPENDENCIES,
        "TOC must declare the current Minimap, Colors, LFGUtil, and ManagedFrameSystem dependencies. Got: {deps:?}"
    );

    assert!(toc.optional_deps().is_empty());
    assert!(toc.load_with().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.saved_variables_per_character().is_empty());
    assert!(!toc.is_load_first());
    assert!(toc.default_enabled());
}

#[test]
fn allow_load_game_restricts_to_in_world() {
    let toc = TocFile::from_file(&ui_widgets_toc()).expect("TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: game` (lowercase) hits toc.rs:308 → Game-only. \
         Server-driven widgets are world-state, no equivalent at glue"
    );
    for screen in GLUE_SCREENS {
        assert!(
            !toc.allows_screen(*screen),
            "Glue screen {screen:?} must be excluded — `AllowLoad: game` \
             matches only the Game variant via toc.rs:308"
        );
    }
}

#[test]
fn toc_omits_game_type_restriction() {
    let toc = TocFile::from_file(&ui_widgets_toc()).expect("TOC parses");

    assert!(
        !toc.is_game_type_restricted(),
        "The current bare TOC has no `## AllowLoadGameType` directive, so it must remain unrestricted"
    );
}

#[test]
fn toc_raw_bytes_pin_directives_and_representative_body_files() {
    let raw = std::fs::read_to_string(ui_widgets_toc()).expect("TOC reads utf-8");

    let expected_directives = [
        "## Title: Blizzard_UIWidgets",
        "## Dependencies: Blizzard_Minimap, Blizzard_Colors, Blizzard_LFGUtil, Blizzard_ManagedFrameSystem",
        "## AllowLoad: game",
    ];

    for line in expected_directives {
        assert!(raw.contains(line), "Raw TOC must pin directive `{line}`");
    }

    for path in REPRESENTATIVE_BODY_FILES {
        assert!(
            raw.contains(path),
            "Raw TOC must pin body path `{path}`. The TOC ships ~79 \
             body files arranged as a Util / Manager / AnimationTemplates \
             / Base prelude followed by ~32 Template{{Lua,Xml}} pairs and \
             4 named container frame pairs — order matters because \
             template Mixin globals must be declared before the matching \
             XML <Frame mixin=...> processes them"
        );
    }

    assert!(!raw.contains("## LoadOnDemand"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("## LoadWith"));
    assert!(!raw.contains("## OptionalDeps"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## RequiredDep"));
}

#[test]
fn appears_in_game_eager_discovery() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let found = addons.iter().any(|(name, _)| name == "Blizzard_UIWidgets");
    assert!(
        found,
        "Blizzard_UIWidgets must appear in Game eager discovery — \
         every server-driven widget in the world (capture bars, \
         scenario timers, status bars, zone control widgets, delve \
         tier frames) routes through this addon's container frames"
    );
}

#[test]
fn absent_from_glue_screens_eager_discovery() {
    for screen in GLUE_SCREENS {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), *screen);
        let found = addons.iter().any(|(name, _)| name == "Blizzard_UIWidgets");
        assert!(
            !found,
            "Blizzard_UIWidgets must NOT appear on {screen:?} — \
             AllowLoad:game restricts to in-world via toc.rs:308, \
             checked at loader/mod.rs:527 BEFORE pool partitioning"
        );
    }
}

#[test]
fn dep_directories_exist_on_disk() {
    for dep in TOC_DEPENDENCIES {
        let dir = blizzard_ui_dir().join(dep);
        assert!(
            dir.is_dir(),
            "Hard-dep directory `{dep}` must exist on disk"
        );
        assert!(
            find_toc_file(&dir).is_some(),
            "{dep} must have a discoverable TOC"
        );
    }
}

prefork_full_ui_case! {
fn full_game_load_publishes_manager_and_container_mixins(env: &WowLuaEnv) {

    for mixin in MANAGER_AND_CONTAINER_MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must be a global table after load. \
             UIWidgetManagerMixin lives on the UIWidgetManager singleton \
             frame and dispatches UPDATE_UI_WIDGET to registered \
             container frames; UIWidgetContainerMixin / \
             UIWidgetContainerResizeMixin / \
             UIWidgetHorizontalWidgetContainerMixin are the layout \
             chassis the 4 named container frames inherit; the four \
             *ContainerMixin entries (TopCenter / BelowMinimap / \
             PowerBar / CenterScreen) carry per-container layout \
             tweaks (anchor logic, max-width clamp, hidden-by-default \
             behaviour)"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_publishes_base_and_helper_mixins(env: &WowLuaEnv) {

    for mixin in BASE_AND_HELPER_MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must be a global table after load. \
             UIWidgetBaseTemplateMixin extends UIWidgetTemplateTooltipFrameMixin \
             and is the parent mixin every Template* widget mixin \
             extends via CreateFromMixins; \
             UIWidgetBaseEnabledFrameMixin handles the enable/disable \
             desaturation; UIWidgetFillUpFrameTemplateMixin is the \
             only secondary mixin chain that extends \
             UIWidgetTemplateTooltipFrameMixin instead of going through \
             UIWidgetBaseTemplateMixin"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_publishes_template_mixins(env: &WowLuaEnv) {

    for mixin in TEMPLATE_MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must be a global table after load. The widget \
             template family covers ~32 distinct widget-type mixins, \
             each backing one Enum.UIWidgetVisualizationType variant \
             that the server can publish: status bars (single / double \
             / circular / partition), capture bars, capture zones, \
             zone control, double-icon-and-text (used by 2v2 score \
             widgets), stacked resource tracker (the unit power bar \
             pip stacks), bullet text lists (Torghast power list), \
             scenario-header variants (currencies+background, timer, \
             delves with its multi-mixin TierFrame), spell display, \
             item display, tug-of-war, map pin animation, button \
             header, prey-hunt progress (TWW Storm Worm). The \
             ScenarioHeaderTimer mixin is special: it extends BOTH \
             UIWidgetBaseTemplateMixin AND \
             UIWidgetBaseScenarioHeaderTemplateMixin via \
             CreateFromMixins(a, b) — the ONLY multi-mixin combination \
             in the addon"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_publishes_flipbook_and_nested_mixins(env: &WowLuaEnv) {

    for mixin in FLIPBOOK_AND_NESTED_MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must be a global table after load. The 3 flipbook \
             anim mixins (Decor / Filled / Burst) live in \
             Blizzard_UIWidgetAnimationTemplates.lua and back the \
             frame-by-frame animations on torghast / capture-zone art; \
             TorghastGemsAnimationMixin powers the gem-fill-up burst \
             effect. The nested per-line mixins \
             (BulletTextListLineMixin, SpellDisplaySpellMixin, \
             TextColumnRowColumnMixin) are the row-level mixins inside \
             each pooled child frame — they're declared as plain `{{}}` \
             tables (not CreateFromMixins) because they don't inherit \
             from the widget base"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_publishes_widget_util(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(WidgetUtil)")
        .expect("WidgetUtil probe failed");
    assert_eq!(
        kind, "table",
        "Global WidgetUtil table from Blizzard_WidgetsUtil.lua:4 must \
         be present. Hosts WidgetUtil.FormatTextByType (text-format \
         dispatch) and WidgetUtil.UpdateTextWithAnimation (text-change \
         pulse / fade animations) consumed by Template* mixins"
    );

    for method in ["FormatTextByType", "UpdateTextWithAnimation"] {
        let method_kind: String = env
            .eval(&format!("return type(WidgetUtil.{method})"))
            .unwrap_or_else(|err| panic!("WidgetUtil.{method} probe failed: {err}"));
        assert_eq!(
            method_kind, "function",
            "WidgetUtil.{method} must be a function on the WidgetUtil \
             namespace"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_registers_representative_virtual_templates(env: &WowLuaEnv) {
    let _ = env;

    for template in REPRESENTATIVE_VIRTUAL_TEMPLATES {
        let resolved = wow_ui_sim::xml::get_template(template);
        assert!(
            resolved.is_some(),
            "Virtual template `{template}` must be registered in the \
             template registry. Blizzard_UIWidgets ships ~60 virtual \
             templates total: 5 container chassis (UIWidgetContainerTemplate, \
             UIWidgetContainerNoResizeTemplate, \
             UIWidgetHorizontalWidgetContainerTemplate, \
             TorghastGemsAnimationTemplate plus the FillUp \
             tooltip-extended secondary chain) + ~15 base widget pieces \
             (Base / BaseStatusBar / BaseIcon / BaseSpell / BaseCurrency / \
             BaseResource / BaseControlZone / BaseTextureAndText / \
             BaseScenarioHeader / BaseCircularStatusBar / \
             BaseStatusBarPartition / BaseButton / BaseStateIcon) + \
             ~32 widget-type templates (IconAndText, StatusBar, \
             DoubleStatusBar, CaptureBar, ZoneControl, ScenarioHeaderTimer, \
             MapPinAnimation, TugOfWar, ItemDisplay, SpellDisplay, \
             FillUpFrames, BulletTextList, ScenarioHeaderDelves, \
             ScenarioHeaderDelvesTierFrame, etc.) + nested templates \
             like UIWidgetTemplateBulletTextListLine and \
             UIWidgetTemplateTextColumnRowColumnTemplate. \
             Container/Base/Template registration drives \
             CreateFrame(\"Frame\", name, parent, \"UIWidgetTemplate*\") \
             at server-widget-show time"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_publishes_named_top_level_frames(env: &WowLuaEnv) {

    for name in NAMED_TOP_LEVEL_FRAMES {
        let frame_kind: String = env
            .eval(&format!("return type(_G[{name:?}])"))
            .unwrap_or_else(|err| panic!("{name} probe failed: {err}"));
        assert!(
            frame_kind == "table" || frame_kind == "userdata",
            "Named top-level frame `{name}` must exist as a global \
             after Blizzard_UIWidgets loads (got type={frame_kind}). \
             UIWidgetManager is the toplevel singleton owning the \
             event registration and per-set dispatch; \
             UIWidgetTopCenterContainerFrame anchors top-center under \
             ObjectiveTrackerFrame; UIWidgetBelowMinimapContainerFrame \
             anchors right-side under the Minimap and inherits \
             UIParentRightManagedFrameTemplate (it's a participant in \
             the UIParent right-managed frame layout chain published \
             by Blizzard_UIParent); UIWidgetCenterScreenContainerFrame \
             is hidden=true by default and shown only for full-screen \
             scenario events; UIWidgetPowerBarContainerFrame parents \
             to EncounterBar (NOT UIParent) and is hidden=true by \
             default — visible only when the encounter publishes a \
             power-bar widget set"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_emits_no_addon_specific_errors(env: &WowLuaEnv) {

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let addon_specific: Vec<&String> = errors
        .iter()
        .filter(|e| e.contains("Blizzard_UIWidgets/"))
        .collect();

    assert!(
        addon_specific.is_empty(),
        "Full game-screen load must emit zero Blizzard_UIWidgets \
         body errors. The 79 body files (~8361 lua+xml lines, dominated \
         by the 1969-line Blizzard_UIWidgetTemplateBase.lua that \
         publishes the 13 UIWidgetBase* tooltip / icon / item / spell / \
         currency / status-bar foundation mixins) load eagerly after \
         Blizzard_Minimap and Blizzard_Colors; any failure cascades \
         into every widget-type Template extending \
         UIWidgetBaseTemplateMixin. Found: {addon_specific:?}"
    );
}
}
