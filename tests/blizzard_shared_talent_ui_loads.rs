use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

#[path = "common/toc_layout.rs"]
mod toc_layout;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn talent_ui_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_SharedTalentUI")
}

fn talent_ui_toc() -> PathBuf {
    talent_ui_dir().join("Blizzard_SharedTalentUI.toc")
}

const PUBLIC_MIXINS: &[&str] = &[
    "TalentArmorSetMixin",
    "TalentButtonArtMixin",
    "TalentButtonBaseMixin",
    "TalentButtonCapstonePipMixin",
    "TalentButtonCapstoneTooltipMixin",
    "TalentButtonCapstoneWithTrackMixin",
    "TalentButtonSearchIconMixin",
    "TalentButtonSelectExpandedButtonMixin",
    "TalentButtonSelectExpandedDisplayMixin",
    "TalentButtonSelectMixin",
    "TalentButtonSpendMixin",
    "TalentButtonSplitIconMixin",
    "TalentButtonSplitSelectMixin",
    "TalentCardMixin",
    "TalentDescriptionCardMixin",
    "TalentDisplayAnimationMixin",
    "TalentDisplayAnimationStateControllerMixin",
    "TalentDisplayMixin",
    "TalentEdgeArrowMixin",
    "TalentEdgeBaseMixin",
    "TalentEdgeStraightMixin",
    "TalentFrameBaseButtonsParentMixin",
    "TalentFrameBaseMixin",
    "TalentFrameCurrencyDisplayMixin",
    "TalentFrameDisplayOnlyMixin",
    "TalentFrameFixedPositionsMixin",
    "TalentFrameGateMixin",
    "TalentFrameGridMixin",
    "TalentFrameHeaderMixin",
    "TalentFrameListMixin",
    "TalentFrameStarGridMixin",
    "TalentFrameTreeSelectorHorizontalMixin",
    "TalentFrameTreeSelectorMixin",
    "TalentNameCardMixin",
    "TalentRedButtonMixin",
    "TalentSelectionChoiceArtMixin",
    "TalentSelectionChoiceFrameMixin",
    "TalentSelectionChoiceMixin",
    "TalentSubTreeHeaderMixin",
    "TalentTreeSelectableButtonMixin",
    "TraitsCommitControlsContainerMixin",
];

const PUBLIC_UTIL_TABLES: &[&str] = &[
    "TalentUtil",
    "TalentFrameUtil",
    "TalentButtonUtil",
    "TalentButtonAnimUtil",
];

// Literal cached-tree facts only: Wrath manifest1-382/cache has no
// Blizzard_SharedTalentUI input tree. Prefork behavior below remains enabled.
post_wrath_ui_fixture! {
#[test]
fn find_toc_file_resolves_bare_toc() {
    let resolved = find_toc_file(&talent_ui_dir()).expect("Blizzard_SharedTalentUI TOC resolves");
    assert_eq!(
        resolved,
        talent_ui_toc(),
        "Blizzard_SharedTalentUI ships exactly one bare \
         `Blizzard_SharedTalentUI.toc` — no flavor variants. The shared talent \
         UI is a cross-flavor library: PlayerSpells, Professions, \
         GenericTraitUI, DelvesCompanionConfiguration, RemixArtifactUI, \
         TieredEntranceTraits all consume it as a hard dependency, and they \
         exist in both Mainline and (subsets of) Classic flavors"
    );
}

#[test]
fn toc_declares_explicit_load_on_demand_zero_with_single_dep() {
    let toc = TocFile::from_file(&talent_ui_toc()).expect("Blizzard_SharedTalentUI TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "TOC must declare `## LoadOnDemand: 0` (explicit zero, semantically \
         equivalent to absent). This is an eager-load addon — every dependent \
         addon (PlayerSpells, Professions, etc.) declares it as a hard \
         Dependency, which would force eager load anyway"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(!toc.is_glue_only());

    let deps = toc.dependencies();
    let expected_deps: Vec<String> = if toc_layout::unified_toc_tree() {
        vec![
            "Blizzard_SpellSearch".to_string(),
            "Blizzard_SpellTypeIconOverlay".to_string(),
        ]
    } else {
        vec!["Blizzard_SpellSearch".to_string()]
    };
    assert_eq!(
        deps, expected_deps,
        "TOC must declare Blizzard_SpellSearch as a hard Dependency; the unified PTR TOC also \
         depends on Blizzard_SpellTypeIconOverlay. The talent button mixins reference SpellSearch \
         globals (SpellSearchUtil, the search-icon mixin's tag registration) so the dep must \
         resolve before this addon's Lua runs. Got: {deps:?}"
    );
    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.saved_variables_per_character().is_empty());
}

#[test]
fn toc_lacks_allow_load_so_falls_through_to_game_only() {
    let toc = TocFile::from_file(&talent_ui_toc()).expect("Blizzard_SharedTalentUI TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Without `## AllowLoad`, src/toc.rs None arm restricts the addon to \
         the Game screen. Talent UI is in-game only — talents have no glue \
         representation"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Glue screen {screen:?} must NOT be allowed — talent mixins call \
             C_Traits.* APIs that have no meaningful state outside an active \
             game session"
        );
    }
}

#[test]
fn toc_raw_bytes_pin_three_metadata_lines_and_no_extra_directives() {
    let raw = std::fs::read_to_string(talent_ui_toc()).expect("TOC reads utf-8");

    assert!(raw.contains("## Title: Blizzard Shared Talent UI"));
    assert!(raw.contains("## LoadOnDemand: 0"));
    assert!(raw.contains("## Dependencies: Blizzard_SpellSearch"));

    assert!(
        !raw.contains("## AllowLoad"),
        "TOC must NOT declare AllowLoad — Game-only fallthrough is the \
         intended behavior, not an explicit grant"
    );
    assert!(!raw.contains("## AllowLoadGameType"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## SavedVariablesPerCharacter"));
    assert!(!raw.contains("## RequiredDep"));
    assert!(!raw.contains("## OptionalDep"));
    assert!(!raw.contains("## DefaultState"));
    assert!(
        !raw.contains("## Author"),
        "TOC must NOT declare Author — this addon predates the Author field \
         conventions used by newer Blizzard_* addons"
    );
}

#[test]
fn toc_body_starts_with_util_then_edge_templates_then_button_templates() {
    let raw = std::fs::read_to_string(talent_ui_toc()).expect("TOC reads utf-8");

    let body_lines: Vec<&str> = raw
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        })
        .collect();

    let first_three = &body_lines[..3];
    assert_eq!(
        first_three,
        [
            "Blizzard_SharedTalentUtil.lua",
            "Blizzard_SharedTalentEdgeTemplates.xml",
            "Blizzard_SharedTalentButtonTemplates.lua",
        ],
        "TOC body MUST start with these 3 entries in order: (1) \
         Blizzard_SharedTalentUtil.lua publishes the TalentUtil / \
         TalentFrameUtil / TalentButtonUtil tables and the BaseVisualState \
         enum that downstream files key off of. (2) \
         Blizzard_SharedTalentEdgeTemplates.xml registers edge virtual \
         templates (TalentArrowEdgeTemplate, TalentStraightEdgeTemplate) and \
         pulls in its companion .lua via `<Script file=...>`. (3) \
         Blizzard_SharedTalentButtonTemplates.lua publishes shared button \
         template helpers used by all subsequent .xml templates. Reordering \
         would break: every later .lua's `CreateFromMixins(<X>)` call needs \
         the base mixin tables to already exist. Got: {first_three:?}"
    );
}

#[test]
fn toc_body_ends_with_frame_grid_then_auto_commit_frame() {
    let expected_tail = [
        "Blizzard_TalentFrameGrid.lua",
        "Blizzard_TalentFrameGrid.xml",
        "Blizzard_AutoCommitTraitFrame.lua",
        "Blizzard_AutoCommitTraitFrame.xml",
    ];

    if toc_layout::unified_toc_tree() {
        // The unified PTR TOC appends a camelot-gated override after the pair
        // and lists Blizzard_ClassTalentSearch between the FrameGrid and
        // AutoCommitTraitFrame entries.
        let toc = TocFile::from_file(&talent_ui_toc()).expect("TOC parses");
        let tail: Vec<String> = toc
            .files
            .iter()
            .rev()
            .take(5)
            .rev()
            .map(|path| path.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            tail,
            [
                "Blizzard_TalentFrameGrid.lua",
                "Blizzard_TalentFrameGrid.xml",
                "Blizzard_ClassTalentSearch.lua",
                "Blizzard_AutoCommitTraitFrame.lua",
                "Blizzard_AutoCommitTraitFrame.xml",
            ],
            "Resolved TOC body MUST end with the FrameGrid pair before the \
             AutoCommitTraitFrame pair. The unified PTR TOC lists \
             Blizzard_ClassTalentSearch.lua between them. Got: {tail:?}"
        );
        return;
    }

    let raw = std::fs::read_to_string(talent_ui_toc()).expect("TOC reads utf-8");

    let body_lines: Vec<&str> = raw
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        })
        .collect();

    let last_four = &body_lines[body_lines.len() - 4..];
    assert_eq!(
        last_four,
        expected_tail,
        "TOC body MUST end with the FrameGrid pair followed by the \
         AutoCommitTraitFrame pair. FrameGrid completes the shared layout \
         chain before AutoCommitTraitFrame consumes the published talent \
         frame and button mixins. Got: {last_four:?}"
    );
}

#[test]
fn toc_body_count_breakdown_matches_filesystem_layout() {
    let toc = TocFile::from_file(&talent_ui_toc()).expect("TOC parses");

    let lua_count = toc
        .files
        .iter()
        .filter(|f| f.extension().is_some_and(|ext| ext == "lua"))
        .count();
    let xml_count = toc
        .files
        .iter()
        .filter(|f| f.extension().is_some_and(|ext| ext == "xml"))
        .count();

    let expected_total = if toc_layout::unified_toc_tree() { 35 } else { 34 };
    let expected_lua = if toc_layout::unified_toc_tree() { 18 } else { 17 };
    assert_eq!(
        lua_count + xml_count,
        expected_total,
        "TOC body lists {expected_total} file entries from the active Blizzard UI cache. Got lua={lua_count} xml={xml_count}"
    );
    assert_eq!(
        lua_count, expected_lua,
        "TOC body must list {expected_lua} .lua entries (Util + AnimUtil + \
         per-component files, including AutoCommitTraitFrame). Additional \
         .lua siblings (EdgeTemplates, SelectionTemplates, FrameTemplates, \
         Frame) are loaded indirectly via `<Script file=...>` from their \
         companion .xml and so are NOT listed in the TOC body. Got: \
         {lua_count}"
    );
    assert_eq!(
        xml_count, 17,
        "TOC body must list exactly 17 .xml entries — one per component that \
         registers virtual templates or instantiates frames, including \
         AutoCommitTraitFrame. Got: {xml_count}"
    );
}

#[test]
fn xml_companion_scripts_pull_in_four_indirect_lua_files() {
    let companions = [
        (
            "Blizzard_SharedTalentEdgeTemplates.xml",
            "Blizzard_SharedTalentEdgeTemplates.lua",
        ),
        (
            "Blizzard_SharedTalentSelectionTemplates.xml",
            "Blizzard_SharedTalentSelectionTemplates.lua",
        ),
        (
            "Blizzard_SharedTalentFrameTemplates.xml",
            "Blizzard_SharedTalentFrameTemplates.lua",
        ),
        (
            "Blizzard_SharedTalentFrame.xml",
            "Blizzard_SharedTalentFrame.lua",
        ),
    ];

    for (xml_name, lua_name) in &companions {
        let xml_raw = std::fs::read_to_string(talent_ui_dir().join(xml_name))
            .unwrap_or_else(|_| panic!("{xml_name} reads utf-8"));
        let directive = format!("<Script file=\"{lua_name}\"/>");
        assert!(
            xml_raw.contains(&directive),
            "{xml_name} must contain `{directive}` — these 4 .lua siblings \
             are NOT in the TOC body and rely on XML chaining to load. \
             Removing the directive would silently drop the .lua and the \
             mixins/templates declared inside would never publish to _G"
        );
        assert!(
            talent_ui_dir().join(lua_name).is_file(),
            "{lua_name} must exist on disk — referenced by {xml_name}'s \
             Script file directive"
        );
    }
}

#[test]
fn eager_discovery_includes_addon_on_game_screen_only() {
    let ui = blizzard_ui_dir();

    let game_addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let game_found = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_SharedTalentUI");
    assert!(
        game_found,
        "Blizzard_SharedTalentUI MUST appear in Game-screen eager discovery \
         — `## LoadOnDemand: 0` keeps it in the eager pool, and 6 downstream \
         addons (Blizzard_PlayerSpells, Blizzard_Professions, \
         Blizzard_GenericTraitUI, Blizzard_DelvesCompanionConfiguration, \
         Blizzard_RemixArtifactUI, Blizzard_TieredEntranceTraits) declare it \
         as a hard Dependency"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_SharedTalentUI");
        assert!(
            !found,
            "Blizzard_SharedTalentUI must be excluded from eager discovery \
             on {screen:?} — Game-only fallthrough applies, and no glue-screen \
             addon depends on it"
        );
    }
}

}

prefork_full_ui_case! {
    fn full_game_load_emits_no_addon_specific_lua_errors(env: &WowLuaEnv) {

        let errors: Vec<String> = env.state().borrow().lua_errors.clone();
        let relevant: Vec<&String> = errors
            .iter()
            .filter(|e| {
                e.contains("Blizzard_SharedTalentUI")
                    || e.contains("SharedTalent")
                    || e.contains("TalentButton")
                    || e.contains("TalentDisplay")
                    || e.contains("TalentFrame")
                    || e.contains("TalentEdge")
            })
            .collect();
        assert!(
            relevant.is_empty(),
            "Eager load via full Game UI discovery must emit zero addon-specific \
             Lua errors. Got:\n  {}",
            relevant
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join("\n  ")
        );
    }
}

prefork_full_ui_case! {
    fn is_addon_loaded_returns_true_after_full_game_discovery(env: &WowLuaEnv) {

        let loaded: bool = env
            .eval("return C_AddOns.IsAddOnLoaded('Blizzard_SharedTalentUI')")
            .expect("IsAddOnLoaded probe");
        assert!(
            loaded,
            "C_AddOns.IsAddOnLoaded('Blizzard_SharedTalentUI') must be true \
             after eager discovery loads it on Game screen"
        );

        let dep_loaded: bool = env
            .eval("return C_AddOns.IsAddOnLoaded('Blizzard_SpellSearch')")
            .expect("Blizzard_SpellSearch IsAddOnLoaded probe");
        assert!(
            dep_loaded,
            "Blizzard_SpellSearch must also be loaded — it's the single hard \
             Dependency of Blizzard_SharedTalentUI, dragged in via the dep graph"
        );
    }
}

prefork_full_ui_case! {
    fn publishes_41_mixin_tables_for_talent_ui_widget_families(env: &WowLuaEnv) {

        for mixin in PUBLIC_MIXINS {
            let kind: String = env
                .eval(&format!("return type({mixin})"))
                .unwrap_or_else(|_| panic!("{mixin} probe failed"));
            assert_eq!(
                kind, "table",
                "_G.{mixin} must be a table — every consumer talent UI calls \
                 CreateFromMixins(<Mixin>) on these. Missing any one means the \
                 corresponding talent component fails to instantiate"
            );
        }
    }
}

prefork_full_ui_case! {
    fn publishes_four_util_namespace_tables(env: &WowLuaEnv) {

        for util in PUBLIC_UTIL_TABLES {
            let kind: String = env
                .eval(&format!("return type({util})"))
                .unwrap_or_else(|_| panic!("{util} probe failed"));
            assert_eq!(
                kind, "table",
                "_G.{util} must be a table — TalentUtil holds spell/talent name \
                 helpers, TalentFrameUtil holds currency/subtree query helpers, \
                 TalentButtonUtil holds geometry constants and the BaseVisualState \
                 enum, TalentButtonAnimUtil holds the animation reset hook"
            );
        }
    }
}

prefork_full_ui_case! {
    fn talent_button_util_publishes_base_visual_state_enum_with_nine_states(env: &WowLuaEnv) {

        let kind: String = env
            .eval("return type(TalentButtonUtil.BaseVisualState)")
            .expect("BaseVisualState probe");
        assert_eq!(
            kind, "table",
            "TalentButtonUtil.BaseVisualState must be a table — declared in \
             Blizzard_SharedTalentUtil.lua line 249. The enum drives the alpha / \
             border / overlay rendering choice for every talent button"
        );

        let expected_states = [
            ("Normal", 1),
            ("Gated", 2),
            ("Disabled", 3),
            ("Locked", 4),
            ("Selectable", 5),
            ("Maxed", 6),
            ("Invisible", 7),
            ("RefundInvalid", 8),
            ("DisplayError", 9),
        ];

        for (state_name, expected_id) in expected_states {
            let id: i64 = env
                .eval(&format!(
                    "return TalentButtonUtil.BaseVisualState.{state_name}"
                ))
                .unwrap_or_else(|_| panic!("BaseVisualState.{state_name} probe failed"));
            assert_eq!(
                id, expected_id,
                "TalentButtonUtil.BaseVisualState.{state_name} must equal \
                 {expected_id} — the enum is hand-numbered (not EnumUtil.MakeEnum) \
                 so the values are pinned at the source. Reordering would silently \
                 reassign meanings to lookup tables keyed off these ids"
            );
        }
    }
}

prefork_full_ui_case! {
    fn talent_button_anim_util_exposes_animation_reset_hook(env: &WowLuaEnv) {

        let kind: String = env
            .eval("return type(TalentButtonAnimUtil.TalentButtonAnimationReset)")
            .expect("TalentButtonAnimationReset probe");
        assert_eq!(
            kind, "function",
            "TalentButtonAnimUtil.TalentButtonAnimationReset must be a function — \
             passed as the resetterFunc to CreateFramePool by every talent frame \
             that pools button animations. Missing this would leak animation \
             state between pooled button reuses"
        );

        let state_kind: String = env
            .eval("return type(TalentButtonAnimUtil.TalentButtonAnimState)")
            .expect("TalentButtonAnimState probe");
        assert_eq!(
            state_kind, "table",
            "TalentButtonAnimUtil.TalentButtonAnimState must be a table — \
             enumerates the per-button animation states (Idle, Pulsing, etc.) \
             that the reset hook clears"
        );
    }
}

prefork_full_ui_case! {
    fn talent_button_select_mixin_inherits_from_talent_button_base_mixin(env: &WowLuaEnv) {

        let inherits: bool = env
            .eval(
                "return type(TalentButtonSelectMixin) == 'table' and \
                 type(TalentButtonSelectMixin.OnLoad) == 'function'",
            )
            .expect("TalentButtonSelectMixin inheritance probe");
        assert!(
            inherits,
            "TalentButtonSelectMixin must inherit from TalentButtonBaseMixin via \
             CreateFromMixins (declared at the top of \
             Blizzard_TalentButtonSelect.lua). OnLoad lives on the base mixin and \
             must be reachable through the inheritance chain — without it, \
             CHOICE-type talent buttons fail their template OnLoad dispatch"
        );
    }
}

prefork_full_ui_case! {
    fn talent_frame_base_mixin_inherits_callback_registry_capability(env: &WowLuaEnv) {

        let inherits: bool = env
            .eval(
                "return type(TalentFrameBaseMixin) == 'table' and \
                 type(TalentFrameBaseMixin.OnLoad) == 'function' and \
                 type(TalentFrameBaseMixin.RegisterCallback) == 'function'",
            )
            .expect("TalentFrameBaseMixin probe");
        assert!(
            inherits,
            "TalentFrameBaseMixin must inherit from CallbackRegistryMixin — \
             consumer code (e.g., PlayerSpells, Professions) keys off \
             TalentFrameBaseMixin's OnConfigIDSet / OnNodeChanged callbacks to \
             re-render their views when the underlying C_Traits config mutates. \
             Without RegisterCallback the event router would have no subscribers"
        );
    }
}

prefork_full_ui_case! {
    fn talent_util_get_talent_name_helper_is_callable_function(env: &WowLuaEnv) {

        let kind: String = env
            .eval("return type(TalentUtil.GetTalentName)")
            .expect("TalentUtil.GetTalentName probe");
        assert_eq!(
            kind, "function",
            "TalentUtil.GetTalentName must be a function — given an \
             (overrideName, spellID) pair, returns the user-facing talent name \
             with override priority. Used by every talent button tooltip and \
             button label render path"
        );

        let result: String = env
            .eval("return TalentUtil.GetTalentName('CustomName', 12345) or 'nil'")
            .expect("GetTalentName invocation probe");
        assert_eq!(
            result, "CustomName",
            "TalentUtil.GetTalentName('CustomName', 12345) must return the \
             override name unchanged — override has priority over spellID lookup. \
             The simulator's GetSpellInfo stub may return nil for unknown IDs but \
             the override path bypasses that lookup entirely"
        );
    }
}

prefork_full_ui_case! {
    fn talent_button_util_geometry_constants_are_floats_in_expected_range(env: &WowLuaEnv) {

        let circle_offset: f64 = env
            .eval("return TalentButtonUtil.CircleEdgeDiameterOffset")
            .expect("CircleEdgeDiameterOffset probe");
        assert!(
            (circle_offset - 1.2).abs() < 1e-6,
            "TalentButtonUtil.CircleEdgeDiameterOffset must equal 1.2 — fixed \
             constant used by edge-render code to scale circle button outlines \
             past the icon diameter so edges visibly attach to the button \
             perimeter. Got: {circle_offset}"
        );

        let square_min: f64 = env
            .eval("return TalentButtonUtil.SquareEdgeMinDiameterOffset")
            .expect("SquareEdgeMinDiameterOffset probe");
        let square_max: f64 = env
            .eval("return TalentButtonUtil.SquareEdgeMaxDiameterOffset")
            .expect("SquareEdgeMaxDiameterOffset probe");
        assert!(
            square_min < square_max,
            "Square edge min={square_min} must be < max={square_max} — the \
             min/max pair gives the talent renderer a range to interpolate edge \
             attachment offset based on talent button size"
        );
    }
}

prefork_full_ui_case! {
    fn shared_talent_virtual_templates_are_not_global_frames(env: &WowLuaEnv) {

        for template_name in [
            "TalentArrowEdgeTemplate",
            "TalentStraightEdgeTemplate",
            "TalentButtonTemplate",
            "TalentDisplayTemplate",
        ] {
            let kind: String = env
                .eval(&format!("return type({template_name})"))
                .unwrap_or_else(|_| panic!("{template_name} probe failed"));
            assert_eq!(
                kind, "nil",
                "_G.{template_name} must be nil — virtual XML templates live in \
                 the template registry only, never in `_G`. Talent component \
                 XMLs reference them via `inherits=\"...\"` which the XML loader \
                 resolves at parse time against the registry. A non-nil result \
                 would mean the loader is leaking virtual templates into the \
                 global namespace"
            );
        }
    }
}

post_wrath_ui_fixture! {
#[test]
fn raw_pngs_in_addon_dir_are_not_listed_in_toc_body() {
    let raw = std::fs::read_to_string(talent_ui_toc()).expect("TOC reads utf-8");

    for png_name in [
        "talents-diamond-mask.png",
        "talents-hexagon-mask.png",
        "talents-octagon-mask-half.png",
        "talents-octagon-mask.png",
    ] {
        assert!(
            !raw.contains(png_name),
            "TOC body must NOT list raw .png assets — texture files are \
             referenced via XML `<Texture file=\"...\"/>` paths and resolved \
             through the asset pipeline at frame-build time. Listing a .png \
             in the TOC body would attempt to parse it as XML or Lua and \
             fail. Got unexpected reference: {png_name}"
        );
        assert!(
            talent_ui_dir().join(png_name).is_file(),
            "{png_name} must exist on disk — used as a texture mask for \
             talent button shape rendering (diamond/hexagon/octagon button \
             silhouettes)"
        );
    }
}
}
