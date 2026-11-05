//! Frame-shape surface pins for the `Blizzard_AchievementUI` lane.
//!
//! PLAN.md task: pin that `AchievementFrame` exists, has `frameStrata` of
//! `MEDIUM`, has parent `UIParent`, and is hidden by default. All four
//! facts come from a single XML declaration at
//! `Mainline/Blizzard_AchievementUI.xml:1505`:
//!
//! ```xml
//! <Frame name="AchievementFrame" toplevel="true" parent="UIParent"
//!        frameStrata="MEDIUM" hidden="true" enableMouse="true"
//!        inherits="BackdropTemplate">
//! ```
//!
//! Each fact has its own assertion so a regression touches the smallest
//! possible test surface. The four together pin the panel's identity in
//! the WoW window manager:
//!
//! - **Existence as a global table.** XML `name="AchievementFrame"`
//!   registers the frame in `_G` at XML-load time. Without this the
//!   `TOGGLEACHIEVEMENT` keybind handler at `Blizzard_AchievementUI.lua:195`
//!   (`AchievementFrame_ToggleAchievementFrame` — pinned in
//!   `surface_globals.rs`) would surface a nil-table-method error on
//!   `ShowUIPanel(AchievementFrame)`.
//!
//! - **`frameStrata == "MEDIUM"`.** This is the default UIPanel stratum.
//!   The achievement panel deliberately renders at the same level as
//!   character / spellbook / inventory frames so the standard UIPanel
//!   layout system (`UIPanelWindows["AchievementFrame"]`, registered at
//!   `Blizzard_AchievementUI.lua:151`) can manage its anchor and the
//!   panel-stack push/pop without crossing strata boundaries. A regression
//!   to `LOW` would push it below world chrome; a regression to `HIGH`
//!   would let it cover dialogs.
//!
//! - **`parent == "UIParent"`.** The standard UI root. `parent="UIParent"`
//!   on the XML keeps the frame inside the user-scaled UI (UIParent is
//!   what `SetUIScale` and the resolution-aware reparenting drive against),
//!   not the world frame which renders 3D content. A regression that
//!   reparents this onto `WorldFrame` or some intermediate would break
//!   user-set UI scaling and detach the panel from `UIParent.Hide()`-style
//!   global toggles.
//!
//! - **Hidden by default.** XML `hidden="true"` makes the frame hidden
//!   on creation; `ToggleAchievementFrame` flips it visible via
//!   `ShowUIPanel`. A regression dropping `hidden="true"` would put the
//!   panel on screen at game start, blocking the player's view.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AchievementUI";
const FRAME_NAME: &str = "AchievementFrame";
const XML_SITE: &str = "Mainline/Blizzard_AchievementUI.xml:1505";

/// Direct children of `AchievementFrame` that remain direct in retail 12.1.
/// Filter controls moved below `HeaderDetails.Filters` and are covered by a
/// separate nested-path test.
const PLAN_NAMED_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "Header",
        "Mainline/Blizzard_AchievementUI.xml:1505",
        "parentKey",
    ),
    (
        "HeaderDetails",
        "Mainline/Blizzard_AchievementUI.xml:1675",
        "parentKey",
    ),
    (
        "Categories",
        "Mainline/Blizzard_AchievementUI.xml:1578",
        "parentKey",
    ),
    (
        "Achievements",
        "Mainline/Blizzard_AchievementUI.xml:1518",
        "name-prefix",
    ),
    (
        "Stats",
        "Mainline/Blizzard_AchievementUI.xml:1572",
        "name-prefix",
    ),
    (
        "Summary",
        "Mainline/Blizzard_AchievementUI.xml:1625",
        "name-prefix",
    ),
    (
        "Comparison",
        "Mainline/Blizzard_AchievementUI.xml:2080",
        "name-prefix",
    ),
];

#[cfg(feature = "optional-tests")]
#[test]
fn achievement_frame_publishes_expected_panel_identity() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        assert(AchievementFrame:GetObjectType() == "Frame")
        assert(AchievementFrame:GetFrameStrata() == "MEDIUM")
        assert(AchievementFrame:GetParent() == UIParent)
        assert(not AchievementFrame:IsShown())
        assert(AchievementFrame:GetWidth() == 768 and AchievementFrame:GetHeight() == 500)
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let frame_type: String = env
            .eval(&format!("return type(_G[{FRAME_NAME:?}])"))
            .expect("AchievementFrame global probe must run cleanly");

        assert_eq!(
            frame_type, "table",
            "Expected `_G[{FRAME_NAME:?}]` to be a table after `{ROOT}` loads, got \
             `{frame_type}`. The frame is declared at `{XML_SITE}` with \
             `name=\"AchievementFrame\"` and `parent=\"UIParent\"`, so the named-frame \
             registration runs at XML load time. A nil reading means either the XML did not \
             execute (a regression in the load pipeline) or the frame failed to register its \
             name (a regression in the named-frame routing inside `CreateFrame`). Either way, \
             every downstream consumer that reaches `AchievementFrame.X` would surface a \
             nil-table-index error — including the keybind handler \
             `AchievementFrame_ToggleAchievementFrame` (`Blizzard_AchievementUI.lua:195`) \
             which calls `ShowUIPanel(AchievementFrame)` / `HideUIPanel(AchievementFrame)`."
        );

        let frame_strata: String = env
            .eval(&format!("return _G[{FRAME_NAME:?}]:GetFrameStrata()"))
            .expect("`GetFrameStrata` must run cleanly on AchievementFrame");

        assert_eq!(
            frame_strata, "MEDIUM",
            "Expected `AchievementFrame:GetFrameStrata()` to return `MEDIUM` after `{ROOT}` \
             loads, got `{frame_strata}`. The XML at `{XML_SITE}` declares \
             `frameStrata=\"MEDIUM\"` literally. MEDIUM is the default UIPanel stratum so the \
             achievement panel renders at the same level as character / spellbook / inventory \
             frames — the standard UIPanel layout system manages its anchor and the \
             panel-stack push/pop without crossing strata boundaries. A regression to `LOW` \
             would push it below world chrome; a regression to `HIGH` would let it cover \
             dialogs / tooltip text from other panels."
        );

        let parent_name: String = env
            .eval(&format!("return _G[{FRAME_NAME:?}]:GetParent():GetName()"))
            .expect("`GetParent():GetName()` must run cleanly on AchievementFrame");

        assert_eq!(
            parent_name, "UIParent",
            "Expected `AchievementFrame:GetParent():GetName()` to return `UIParent` after \
             `{ROOT}` loads, got `{parent_name}`. The XML at `{XML_SITE}` declares \
             `parent=\"UIParent\"` literally. `UIParent` is the standard scaled-UI root — \
             `SetUIScale` and the resolution-aware reparenting drive against it, and \
             `UIParent.Hide()`-style global toggles cascade to it. A regression that \
             reparents this onto `WorldFrame` (the 3D world root) or some intermediate would \
             break user-set UI scaling and detach the panel from the global UI toggle."
        );

        let is_shown: bool = env
            .eval(&format!("return _G[{FRAME_NAME:?}]:IsShown()"))
            .expect("`IsShown` must run cleanly on AchievementFrame");

        assert!(
            !is_shown,
            "Expected `AchievementFrame:IsShown()` to return false after `{ROOT}` loads. The \
             XML at `{XML_SITE}` declares `hidden=\"true\"` literally — the frame is hidden on \
             creation, and `ToggleAchievementFrame` flips it visible via `ShowUIPanel` only \
             when the player presses the achievement keybind or clicks the micro menu button. \
             A true reading here means a regression dropped `hidden=\"true\"` from the XML or \
             the loader failed to honour the attribute, putting the panel on screen at game \
             start and blocking the player's view."
        );
    });
}

/// Verify direct parent properties on `AchievementFrame` still resolve.

#[cfg(feature = "optional-tests")]
#[test]
fn achievement_frame_plan_named_children_are_reachable_as_parent_properties() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        for _, name in ipairs({"AchievementFrameHeader", "AchievementFrameCategories", "AchievementFrameAchievements", "AchievementFrameStats", "AchievementFrameSummary", "AchievementFrameComparison"}) do
            local child = assert(_G[name], name)
            assert(child:GetParent() == AchievementFrame, name)
        end
        assert(AchievementFrameBackground:GetParent() == AchievementFrame)
        assert(AchievementFrameHeaderPoints:GetObjectType() == "FontString")
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for (key, xml_site, routing_kind) in PLAN_NAMED_CHILDREN {
            let value_type: String = env
                .eval(&format!(
                    "return type(_G[{FRAME_NAME:?}][{key:?}])",
                    key = key
                ))
                .expect("`type(AchievementFrame[<Key>])` must run cleanly");

            assert_eq!(
                value_type, "table",
                "Expected `AchievementFrame.{key}` to be a table after `{ROOT}` loads, got \
                 `{value_type}`. The XML at `{xml_site}` attaches this child via the \
                 `{routing_kind}` route. A nil reading on a `parentKey` child means the \
                 XML dropped the attribute or `append_parent_key_code` stopped routing it; \
                 a nil reading on a `name-prefix` child means the XML changed the \
                 `name=\"$parent{key}\"` token or `infer_parent_key_from_child_name` \
                 stopped running. Either way the panel's Lua code that reaches \
                 `AchievementFrame.{key}` would surface a nil-table-index error."
            );

            let parent_name: String = env
                .eval(&format!(
                    "return _G[{FRAME_NAME:?}][{key:?}]:GetParent():GetName()"
                ))
                .expect("`GetParent():GetName()` must run cleanly on the child frame");

            assert_eq!(
                parent_name, FRAME_NAME,
                "Expected `AchievementFrame.{key}:GetParent():GetName()` to return \
                 `{FRAME_NAME}`, got `{parent_name}`. The XML at `{xml_site}` nests this \
                 child inside `AchievementFrame`'s `<Frames>` block, so its parent must \
                 be `AchievementFrame` — `AchievementFrame:Hide()` cascading to children \
                 and `SetUIScale` propagating from `UIParent` both depend on the parent \
                 chain landing here."
            );
        }
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn achievement_frame_filter_controls_use_current_nested_paths() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        assert(AchievementFrameFilterDropDown:GetParent() == AchievementFrame)
        assert(AchievementFrameHeaderRightDDLInset:GetObjectType() == "Texture")
        assert(AchievementFrameFilterDropDown:GetWidth() == 110)
        assert(AchievementFrameFilterDropDown:GetHeight() == 32)
        assert(AchievementFrameFilterDropDown:GetScript("OnLoad"))
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let paths: (String, String, String) = env
            .eval(
                r#"
                return type(AchievementFrame.HeaderDetails.Filters.FilterDropdown),
                       type(AchievementFrame.HeaderDetails.Filters.SearchBox),
                       type(AchievementFrame.HeaderDetails.Filters.SearchBox.SearchProgressBar)
                "#,
            )
            .expect("AchievementFrame filter-control path probes must run cleanly");

        assert_eq!(
            paths,
            ("table".into(), "table".into(), "table".into()),
            "Retail XML nests FilterDropdown and SearchBox below HeaderDetails.Filters, with \
             SearchProgressBar below SearchBox."
        );
    });
}

const SEARCH_PROGRESS_BAR_HANDLER: &str = "AchievementFrameSearchProgressBar_OnUpdate";

const COMPARISON_FRAME_NAME: &str = "AchievementFrameComparison";
const COMPARISON_XML_SITE: &str = "Mainline/Blizzard_AchievementUI.xml:2080";

/// PLAN-named comparison-subtree paths. Each tuple is `(lua_path, xml_site,
/// routing_kind)` where `lua_path` is the dotted access from
/// `AchievementFrameComparison`, `xml_site` is the source declaration, and
/// `routing_kind` is `"parentKey"` when the XML uses an explicit
/// `parentKey="<Key>"` attribute and `"name-prefix"` when the XML uses
/// `name="$parent<Key>"` and the simulator's
/// `infer_parent_key_from_child_name` repair (`src/lua_api/globals/template/mod.rs:72`)
/// derives the key by stripping the parent's resolved name.
///
/// `Header.Portrait` is a two-step path: `Header` is a name-prefix child
/// of `AchievementFrameComparison` (XML `name="$parentHeader"`, resolved
/// name `AchievementFrameComparisonHeader`, prefix-strip yields
/// `Header`), and `Portrait` is a Texture inside the Header's
/// `<Layer level="BACKGROUND">` block (XML `name="$parentPortrait"`,
/// resolved name `AchievementFrameComparisonHeaderPortrait`, prefix-strip
/// yields `Portrait`).
const PLAN_NAMED_COMPARISON_PATHS: &[(&str, &str, &str)] = &[
    (
        "Header",
        "Mainline/Blizzard_AchievementUI.xml:2087",
        "name-prefix",
    ),
    (
        "Header.Portrait",
        "Mainline/Blizzard_AchievementUI.xml:2103",
        "name-prefix",
    ),
    (
        "Summary",
        "Mainline/Blizzard_AchievementUI.xml:2149",
        "parentKey",
    ),
    (
        "AchievementContainer",
        "Mainline/Blizzard_AchievementUI.xml:2212",
        "parentKey",
    ),
];

/// Pin `AchievementFrameComparison` and its PLAN-named subtree paths.
///
/// PLAN names three children of the comparison frame: `Header.Portrait`
/// (a two-step path through `Header`), `Summary`, and
/// `AchievementContainer`. The XML at
/// `Mainline/Blizzard_AchievementUI.xml:2080` declares the comparison
/// frame itself as `<Frame name="$parentComparison">`, reachable in the
/// simulator as both `_G.AchievementFrameComparison` and
/// `AchievementFrame.Comparison` — the latter via the same name-prefix
/// inference that handles `AchievementFrame.Achievements` /
/// `AchievementFrame.Stats` / `AchievementFrame.Summary` (pinned by the
/// sibling `achievement_frame_plan_named_children_are_reachable_as_parent_properties`
/// test).
///
/// Subtree routing kinds:
///
/// - `Header` (line 2087) and its `Portrait` texture (line 2103) are both
///   declared via `name="$parent<Key>"` without explicit `parentKey=`
///   attributes. They land on the parent's per-instance table only
///   because the simulator's `infer_parent_key_from_child_name` repair
///   strips the parent's resolved name from each child's resolved name
///   and installs the resulting suffix as the parentKey. A regression
///   that removed the prefix-inference would null both. The Lua source
///   touches `Portrait` via the OnShow handler at line 2144 —
///   `SetPortraitTexture(_G[self:GetName().."Portrait"], "player")` —
///   which uses the global path, but the `Header.Portrait`
///   parent-property path is what every test fixture and downstream
///   consumer expects.
///
/// - `Summary` (line 2149) and `AchievementContainer` (line 2212) are
///   declared with explicit `parentKey="<Key>"` attributes, routed
///   through `append_parent_key_code` in
///   `src/loader/xml_frame_codegen.rs:90`. They participate in the
///   comparison-mode show/hide swap driven by
///   `AchievementFrameComparison_OnEvent` and the panel Lua code at
///   `Blizzard_AchievementUI.xml:2229-2238` (`parent.Summary:Show()` and
///   `parent.Summary:Hide()` from the AchievementContainer OnShow/OnHide
///   scripts) — a nil reading would surface a nil-table-method error in
///   the show/hide cascade.
///
/// The test asserts a uniform contract for all four paths: each is
/// reachable as a chained access from `_G.AchievementFrameComparison`
/// and is itself a table (frame or texture). The first probe
/// additionally confirms the comparison frame itself exists as a global
/// before the chained accesses are attempted, so a missing comparison
/// frame surfaces with a precise message rather than a confusing
/// nil-index error inside the loop.
#[cfg(feature = "optional-tests")]
#[test]
fn achievement_frame_comparison_subtree_publishes_plan_named_paths() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        for _, name in ipairs({"AchievementFrameComparisonHeader", "AchievementFrameComparisonSummaryPlayer", "AchievementFrameComparisonSummaryFriend", "AchievementFrameComparisonContainer", "AchievementFrameComparisonStatsContainer"}) do
            assert(_G[name], name)
        end
        assert(AchievementFrameComparisonHeaderPortrait:GetObjectType() == "Texture")
        assert(AchievementFrameComparisonHeaderPoints:GetObjectType() == "FontString")
        local row = assert(AchievementFrameComparisonContainer.buttons[1])
        assert(row.player and row.friend and row.friend.status)
        assert(row.player:GetParent() == row and row.friend:GetParent() == row)
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let comparison_type: String = env
            .eval(&format!("return type(_G[{COMPARISON_FRAME_NAME:?}])"))
            .expect("AchievementFrameComparison global probe must run cleanly");

        assert_eq!(
            comparison_type, "table",
            "Expected `_G[{COMPARISON_FRAME_NAME:?}]` to be a table after `{ROOT}` \
             loads, got `{comparison_type}`. The XML at `{COMPARISON_XML_SITE}` declares \
             the comparison sub-frame as `<Frame name=\"$parentComparison\">` nested \
             inside `AchievementFrame`'s `<Frames>` block, which resolves the name token \
             to `AchievementFrameComparison` and registers it in `_G`. A nil reading \
             means either the XML changed the name token, the frame was removed, or the \
             file chunk failed before reaching the declaration. Every Lua call-site that \
             drives the comparison panel — `AchievementFrameComparison_OnEvent` at line \
             2814, `AchievementFrameComparison_ForceUpdate` at 2846, the comparison-mode \
             tab swap in `AchievementFrame_SetComparisonTabs` at 332 — would surface a \
             nil-table-method error."
        );

        for (lua_path, xml_site, routing_kind) in PLAN_NAMED_COMPARISON_PATHS {
            let value_type: String = env
                .eval(&format!(
                    "return type(_G[{COMPARISON_FRAME_NAME:?}].{lua_path})"
                ))
                .expect("comparison subtree path probe must run cleanly");

            assert_eq!(
                value_type, "table",
                "Expected `AchievementFrameComparison.{lua_path}` to be a table after \
                 `{ROOT}` loads, got `{value_type}`. The XML at `{xml_site}` attaches \
                 this path via the `{routing_kind}` route. A nil reading on a \
                 `parentKey` path means the XML dropped the attribute or \
                 `append_parent_key_code` stopped routing it; a nil reading on a \
                 `name-prefix` path means the XML changed the `name=\"$parent<Key>\"` \
                 token or `infer_parent_key_from_child_name` stopped running. The \
                 comparison panel's show/hide cascade \
                 (`Blizzard_AchievementUI.xml:2229-2238`, `parent.Summary:Show()` etc.) \
                 and its inline INSPECT_ACHIEVEMENT_READY handler at \
                 `Blizzard_AchievementUI.lua:2815-2819` both walk this subtree."
            );
        }
    });
}

/// Current retail search-progress-bar surface.
const SEARCH_PROGRESS_BAR_PATH: &str =
    "AchievementFrame.HeaderDetails.Filters.SearchBox.SearchProgressBar";

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn achievement_frame_search_progress_bar_uses_nested_search_box_path() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let observations: (String, String, bool, String, String) = env
            .eval(&format!(
                r#"
                local search_box = AchievementFrame.HeaderDetails.Filters.SearchBox
                local bar = search_box.SearchProgressBar
                return type(bar), bar:GetObjectType(), bar:GetParent() == search_box,
                       type(_G[{handler:?}]),
                       type(bar:GetScript("OnUpdate"))
                "#,
                handler = SEARCH_PROGRESS_BAR_HANDLER,
            ))
            .expect("nested SearchProgressBar surface probes must run cleanly");

        assert_eq!(observations.0, "table", "{SEARCH_PROGRESS_BAR_PATH} must exist.");
        assert_eq!(observations.1, "StatusBar", "{SEARCH_PROGRESS_BAR_PATH} must be a StatusBar.");
        assert!(observations.2, "SearchProgressBar must remain parented to SearchBox.");
        assert_eq!(observations.3, "function", "{SEARCH_PROGRESS_BAR_HANDLER} must be global.");
        assert_eq!(observations.4, "nil", "OnUpdate is installed only while search runs.");
    });
}
}
