//! Surface-level globals pinned by the `Blizzard_AchievementUI` lane.
//!
//! PLAN.md task: `AchievementFrame_ToggleAchievementFrame`,
//! `AchievementFrame_DisplayComparison`, `AchievementFrame_ForceUpdate`,
//! `AchievementFrame_SetTabs`, `AchievementFrame_SetComparisonTabs`,
//! `AchievementFrame_UpdateTabs` are functions.
//!
//! All six PLAN-named globals match the actual source verbatim — no
//! spec/source mismatches in this batch. They are declared via the
//! `function Name(args)` syntax (not `Name = function(args)`), which under
//! Lua 5.1 semantics binds the function to the global table:
//!
//! - `AchievementFrame_ToggleAchievementFrame` at line 195: the entry-point
//!   for opening the achievement panel — handles the
//!   `Enum.GameRule.AchievementsPanelDisabled` early-return guard, drives
//!   the tab-state swap (`AchievementFrameTab_OnClick =
//!   AchievementFrameBaseTab_OnClick`), and invokes `ShowUIPanel` /
//!   `HideUIPanel` against the `AchievementFrame` global. This is the
//!   surface bound to the `TOGGLEACHIEVEMENT` keybinding.
//!
//! - `AchievementFrame_DisplayComparison` at line 225: drives the
//!   side-by-side comparison view; takes a `unit` argument and sets
//!   `AchievementFrameComparison.unit = unit` before showing the panel.
//!   Note the source has a stylistic space before the open paren (`function
//!   AchievementFrame_DisplayComparison (unit)`); the Lua parser doesn't
//!   care.
//!
//! - `AchievementFrame_ForceUpdate` at line 317: dispatches to one of
//!   `AchievementFrameAchievements_ForceUpdate` /
//!   `AchievementFrameStats_UpdateDataProvider` /
//!   `AchievementFrameComparison_ForceUpdate` depending on which sub-frame
//!   is currently shown. Zero-arg.
//!
//! - `AchievementFrame_SetTabs` at line 327: configures the tab strip for
//!   the player-only mode (shows tab 2, anchors tab 3 to right of tab 2).
//!   Zero-arg.
//!
//! - `AchievementFrame_SetComparisonTabs` at line 332: configures the tab
//!   strip for comparison mode (hides tab 2, re-anchors tab 3 to right of
//!   tab 1). Zero-arg.
//!
//! - `AchievementFrame_UpdateTabs` at line 337: drives the active-tab
//!   visual state — calls `PanelTemplates_Tab_OnClick`, hides
//!   `AchievementFrame.SearchResults`, and per-tab adjusts the Text
//!   position offset (clicked tab gets y=-5, others get y=-3). Takes the
//!   index of the newly-clicked tab.
//!
//! The test pins each name's `_G` entry as a function under the LoadOnDemand
//! root harness (smoke-shape with `[ROOT]`). A non-function reading would
//! prove either (a) the file chunk failed before reaching the function
//! declaration, OR (b) Blizzard re-shaped the global into a method on a
//! mixin / removed it / renamed it. Each is a meaningful API contract
//! change.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AchievementUI";
const PLAN_NAMED_FRAME_GLOBALS: &[&str] = &[
    "AchievementFrame_ToggleAchievementFrame",
    "AchievementFrame_DisplayComparison",
    "AchievementFrame_ForceUpdate",
    "AchievementFrame_SetTabs",
    "AchievementFrame_SetComparisonTabs",
    "AchievementFrame_UpdateTabs",
];
const PLAN_NAMED_CATEGORIES_GLOBALS: &[&str] = &[
    "AchievementFrameCategories_OnLoad",
    "AchievementFrameCategories_ExpandToCategory",
    "AchievementFrameCategories_SelectDefaultElementData",
    "AchievementFrameCategories_UpdateDataProvider",
];
const PLAN_NAMED_ACHIEVEMENTS_GLOBALS: &[&str] = &[
    "AchievementFrameAchievements_GetSelectedAchievementId",
    "AchievementFrameAchievements_OnAchievementEarned",
    "AchievementFrameAchievements_OnCriteriaUpdate",
    "AchievementFrameAchievements_UpdateDataProvider",
];
/// PLAN-named comparison globals that DO exist verbatim in source.
const PLAN_NAMED_COMPARISON_PRESENT: &[&str] = &["AchievementFrameComparison_UpdateStatusBars"];
/// PLAN-named comparison globals that do NOT exist in source — tripwires.
/// `AchievementFrameComparison_Update` was never declared (closest matches
/// are `_ForceUpdate` at 2846, `_UpdateDataProvider` at 2854,
/// `_UpdateStatsDataProvider` at 2868). `AchievementFrameComparison_OnInspectReady`
/// was never declared either — INSPECT_ACHIEVEMENT_READY is dispatched
/// inline from `AchievementFrameComparison_OnEvent` at line 2814.
const PLAN_NAMED_COMPARISON_ABSENT: &[&str] = &[
    "AchievementFrameComparison_Update",
    "AchievementFrameComparison_OnInspectReady",
];
/// PLAN-named achievement-row globals that DO exist verbatim in source.
const PLAN_NAMED_ACHIEVEMENT_ROW_PRESENT: &[&str] = &["AchievementObjectives_DisplayCriteria"];
/// PLAN-named achievement-row globals that do NOT exist in source — tripwires.
/// `AchievementButton_OnClick` and `AchievementButton_OnEnter` were never
/// declared as standalone globals — the XML `AchievementTemplate` virtual
/// frame at line 733 uses `mixin="AchievementTemplateMixin"` and binds
/// `<OnClick method="OnClick"/>` / `<OnEnter method="OnEnter"/>` (lines
/// 880-881) which dispatches to `AchievementTemplateMixin:OnClick` (1089)
/// and `:OnEnter` (1093) on the frame instance, not to a top-level
/// function. `AchievementButton_OnDoubleClick` does not exist at all —
/// no `OnDoubleClick` script binding appears anywhere in the addon's Lua
/// or XML, so achievement rows have no double-click behaviour to pin.
const PLAN_NAMED_ACHIEVEMENT_ROW_ABSENT: &[&str] = &[
    "AchievementButton_OnClick",
    "AchievementButton_OnEnter",
    "AchievementButton_OnDoubleClick",
];

#[cfg(feature = "optional-tests")]
#[test]
fn achievement_frame_plan_named_globals_are_functions() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        for _, name in ipairs({"AchievementFrame_OnLoad", "AchievementFrame_ToggleAchievementFrame", "AchievementFrame_DisplayComparison", "AchievementFrame_ForceUpdate", "AchievementFrameBaseTab_OnClick", "AchievementFrameComparisonTab_OnClick"}) do
            assert(type(_G[name]) == "function", name)
        end
        AchievementFrame_OnLoad(AchievementFrame)
        assert(AchievementFrame.selectedTab == 1 and AchievementFrame.numTabs == 2)
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for name in PLAN_NAMED_FRAME_GLOBALS {
            let is_function = env
                .eval::<bool>(&format!(r#"return type({name}) == "function""#))
                .expect("file-scope global type probe must run cleanly");

            assert!(
                is_function,
                "Global `{name}` MUST be a function after the smoke-shape harness loads \
                 `Blizzard_AchievementUI`. Each entry in `PLAN_NAMED_FRAME_GLOBALS` is declared \
                 via `function <name>(...)` syntax in `Mainline/Blizzard_AchievementUI.lua` \
                 (lines 195/225/317/327/332/337). A non-function reading here proves either \
                 (a) the file chunk failed before reaching the declaration (load.rs's \
                 `achievement_ui_load_emits_no_lane_specific_lua_errors` would also fail in that \
                 case — the failure mode is multi-test), OR (b) Blizzard re-shaped the global \
                 into a method on a mixin / removed it / renamed it (this lane-specific test \
                 surfaces the API contract drift even when the load smoke remains green). Got \
                 `type({name}) == \"function\"` returned false."
            );
        }
    });
}

/// Pin the `AchievementFrameCategories_*` ScrollBox-driver globals.
///
/// All four PLAN-named globals match the actual source verbatim — no
/// spec/source mismatches in this batch. Each is declared via the
/// `function <name>(...)` syntax in `Mainline/Blizzard_AchievementUI.lua`:
///
/// - `AchievementFrameCategories_OnLoad` at line 584 — wires the
///   ScrollBox/ScrollBar via `ScrollUtil.InitScrollBoxListWithScrollBar`,
///   sets `AchievementCategoryTemplate` as the element initializer (each
///   element's `Init(elementData)` is called per-button), and registers
///   the `ADDON_LOADED` event. Takes the categories frame as `self` —
///   bound from the XML `<OnLoad>` script.
///
/// - `AchievementFrameCategories_ExpandToCategory` at line 603 — drives
///   the cross-tab navigation when a deep-link target lives on the
///   inactive tab. Takes a category id, calls
///   `AchievementFrameCategories_FindCategoryElement` against the active
///   tab's categories first, then falls back to the alternate tab
///   (guild/personal swap via `InGuildView()`) and dispatches
///   `AchievementFrameBaseTab_OnClick(alternateTabIndex)` to switch tabs.
///   When the target is a child category, hides sibling-of-different-
///   parent children to expand only the target's parent block.
///
/// - `AchievementFrameCategories_SelectDefaultElementData` at line 707 —
///   guards against missing data provider (lazy-builds via
///   `AchievementFrameCategories_UpdateDataProvider` when
///   `ScrollBox:HasDataProvider()` is false), then scrolls to index 1
///   with `AlignCenter` and selects the resulting elementData via
///   `AchievementFrameCategories_SelectElementData`. Zero-arg.
///
/// - `AchievementFrameCategories_UpdateDataProvider` at line 718 — builds
///   a fresh `CreateDataProvider()` from `achievementFunctions.categories`
///   filtered by `category.hidden` and the active
///   `AchievementFrame.restrictedCategoryID` (when set, only categories
///   matching the restricted ID or its parent get inserted). Calls
///   `ScrollBox:SetDataProvider(newDataProvider)` at the end. Note the
///   stylistic space `function AchievementFrameCategories_UpdateDataProvider ()`
///   matches the OnLoad declaration's formatting; the parser doesn't care.
#[cfg(feature = "optional-tests")]
#[test]
fn achievement_frame_categories_plan_named_globals_are_functions() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        for _, name in ipairs({"AchievementFrameCategories_OnLoad", "AchievementFrameCategories_GetCategoryList", "AchievementFrameCategories_Update", "AchievementCategoryButton_OnClick"}) do
            assert(type(_G[name]) == "function", name)
        end
        AchievementFrameCategories_GetCategoryList(ACHIEVEMENTUI_CATEGORIES)
        assert(ACHIEVEMENTUI_CATEGORIES[1].id == "summary")
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for name in PLAN_NAMED_CATEGORIES_GLOBALS {
            let is_function = env
                .eval::<bool>(&format!(r#"return type({name}) == "function""#))
                .expect("file-scope global type probe must run cleanly");

            assert!(
                is_function,
                "Global `{name}` MUST be a function after the smoke-shape harness loads \
                 `Blizzard_AchievementUI`. Each entry in `PLAN_NAMED_CATEGORIES_GLOBALS` is \
                 declared via `function <name>(...)` syntax in \
                 `Mainline/Blizzard_AchievementUI.lua` (lines 584/603/707/718). A non-function \
                 reading here proves either (a) the file chunk failed before reaching the \
                 declaration (load.rs's `achievement_ui_load_emits_no_lane_specific_lua_errors` \
                 would also fail in that case — the failure mode is multi-test), OR (b) the \
                 ScrollBox-driver dispatch was re-shaped into a method on \
                 `AchievementFrameCategories` (a frame mixin) / removed / renamed (this \
                 lane-specific test surfaces the API contract drift even when the load smoke \
                 stays green). The four functions form one cohesive surface: OnLoad wires the \
                 ScrollBox view, UpdateDataProvider rebuilds its data provider, \
                 SelectDefaultElementData scrolls to + selects the first entry, and \
                 ExpandToCategory drives cross-tab navigation — a regression on any one of them \
                 would silently break either the panel's first-open population or the deep-link \
                 navigation flow. Got `type({name}) == \"function\"` returned false."
            );
        }
    });
}

/// Pin the `AchievementFrameAchievements_*` selection-and-data-driver globals.
///
/// All four PLAN-named globals match the actual source verbatim — no
/// spec/source mismatches in this batch. Each is declared via the
/// `function <name>(...)` syntax in `Mainline/Blizzard_AchievementUI.lua`:
///
/// - `AchievementFrameAchievements_GetSelectedAchievementId` at line 890 —
///   trampolines through `AchievementFrameAchievements_GetSelectedElementData`
///   (line 886, which itself returns
///   `g_achievementSelectionBehavior:GetFirstSelectedElementData()`) and
///   returns `elementData and elementData.id or 0` — i.e. zero is the
///   sentinel for "nothing currently selected", not a real achievement id.
///   Zero-arg.
///
/// - `AchievementFrameAchievements_OnAchievementEarned` at line 895 — the
///   ACHIEVEMENT_EARNED event handler. Calls `UpdateDataProvider` to rebuild
///   the list, then conditionally invokes
///   `AchievementFrame_SelectAndScrollToAchievementId(AchievementFrameAchievements.ScrollBox, achievementId)`
///   only when the just-earned id matches the currently-selected one (so the
///   list stays anchored on the player's focus), refreshes the category
///   tooltip via `AchievementFrameCategories_UpdateTooltip()`, and
///   re-renders the points header via
///   `AchievementFrame.Header.Points:SetText(BreakUpLargeNumbers(GetTotalAchievementPoints(InGuildView())))`.
///   Takes the freshly-earned achievement id.
///
/// - `AchievementFrameAchievements_OnCriteriaUpdate` at line 907 — the
///   CRITERIA_UPDATE handler. Looks up the currently-selected element data,
///   resolves the rendered button via `ScrollBox:FindFrame(selectedElementData)`,
///   and re-runs `button:Init(selectedElementData)` to refresh the visible
///   progress bars in place (no data provider rebuild — surgical update of
///   the one frame whose criteria changed). Zero-arg.
///
/// - `AchievementFrameAchievements_UpdateDataProvider` at line 960 — the
///   data-provider rebuild driver. Early-returns when the active category
///   is "summary" (which uses a different rendering path), then resolves
///   `numAchievements`/`numCompleted`/`completedOffset` via
///   `ACHIEVEMENTUI_SELECTEDFILTER(category)`, toggles the
///   `AchievementFrameAchievementsFeatOfStrengthText` visibility/text
///   (choosing `GUILD_FEAT_OF_STRENGTH_DESCRIPTION` vs
///   `FEAT_OF_STRENGTH_DESCRIPTION` based on `AchievementFrame.selectedTab == 2`),
///   builds a fresh `CreateDataProvider()` by iterating
///   `GetAchievementInfo(category, filteredIndex)` and inserting
///   `{category, index, id}` records, and dispatches
///   `ScrollBox:SetDataProvider(newDataProvider)` at the end. Zero-arg.
#[cfg(feature = "optional-tests")]
#[test]
fn achievement_frame_achievements_plan_named_globals_are_functions() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        for _, name in ipairs({"AchievementFrameAchievements_OnLoad", "AchievementFrameAchievements_OnEvent", "AchievementFrameAchievements_Update", "AchievementFrameAchievements_ForceUpdate"}) do
            assert(type(_G[name]) == "function", name)
        end
        AchievementFrameAchievements.selection = nil
        AchievementFrameAchievementsObjectives.id = 6
        AchievementFrameAchievements_OnEvent(AchievementFrameAchievements, "CRITERIA_UPDATE")
        assert(AchievementFrameAchievementsObjectives.id == nil)
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for name in PLAN_NAMED_ACHIEVEMENTS_GLOBALS {
            let is_function = env
                .eval::<bool>(&format!(r#"return type({name}) == "function""#))
                .expect("file-scope global type probe must run cleanly");

            assert!(
                is_function,
                "Global `{name}` MUST be a function after the smoke-shape harness loads \
                 `Blizzard_AchievementUI`. Each entry in `PLAN_NAMED_ACHIEVEMENTS_GLOBALS` is \
                 declared via `function <name>(...)` syntax in \
                 `Mainline/Blizzard_AchievementUI.lua` (lines 890/895/907/960). A non-function \
                 reading here proves either (a) the file chunk failed before reaching the \
                 declaration (load.rs's `achievement_ui_load_emits_no_lane_specific_lua_errors` \
                 would also fail in that case — the failure mode is multi-test), OR (b) the \
                 selection/data-driver dispatch was re-shaped into a method on \
                 `AchievementFrameAchievements` (a frame mixin) / removed / renamed (this \
                 lane-specific test surfaces the API contract drift even when the load smoke \
                 stays green). The four functions form one cohesive surface: \
                 GetSelectedAchievementId reads the current selection's id (zero-default), \
                 OnAchievementEarned is the ACHIEVEMENT_EARNED event entry-point that rebuilds \
                 the data provider + conditionally scrolls + refreshes the points header, \
                 OnCriteriaUpdate surgically refreshes the selected button's progress bars on \
                 CRITERIA_UPDATE, and UpdateDataProvider rebuilds the ScrollBox's data \
                 provider for the current category — a regression on any one of them would \
                 silently break either the live progress display or the post-earn list refresh. \
                 Got `type({name}) == \"function\"` returned false."
            );
        }
    });
}

/// Pin the `AchievementFrameComparison_*` globals — split presence/absence.
///
/// **Spec/source mismatch.** The PLAN line names three globals
/// (`AchievementFrameComparison_Update`, `_OnInspectReady`,
/// `_UpdateStatusBars`) but only `_UpdateStatusBars` actually exists in
/// `Mainline/Blizzard_AchievementUI.lua`. The other two were never
/// declared — most likely they are spec-side guesses about how the
/// comparison sub-frame would be structured if Blizzard had factored
/// each event-handler and update-driver into its own top-level function,
/// but the source instead uses a single `_OnEvent` (line 2814) that
/// dispatches inline by event name, plus a `_ForceUpdate` (2846) that
/// branches on the active `achievementFunctions` table.
///
/// We therefore split the test along the spec/source boundary:
///
/// **Presence tripwire** (`PLAN_NAMED_COMPARISON_PRESENT`):
/// - `AchievementFrameComparison_UpdateStatusBars` at line 820 — takes a
///   `category id`, reads `numAchievements`/`numCompleted` via
///   `GetCategoryNumAchievements(id)` and the category name via
///   `GetCategoryInfo(id)` (with `ACHIEVEMENT_SUMMARY_CATEGORY` override
///   when `id == ACHIEVEMENT_COMPARISON_SUMMARY_ID`), then writes the
///   player and friend status-bar pairs:
///   `AchievementFrameComparison.Summary.Player.StatusBar` gets the
///   player's completed count (from `GetCategoryNumAchievements`) and
///   `AchievementFrameComparison.Summary.Friend.StatusBar` gets the
///   compared unit's count (from `GetComparisonCategoryNumAchievements`).
///   Both bars get `SetMinMaxValues(0, numAchievements)` so they share
///   the same denominator. The function is invoked from the inline
///   `INSPECT_ACHIEVEMENT_READY` arm of `AchievementFrameComparison_OnEvent`
///   (line 2818), proving its public-surface role.
///
/// **Absence tripwires** (`PLAN_NAMED_COMPARISON_ABSENT`):
/// - `AchievementFrameComparison_Update` — never declared. The closest
///   real symbols are `_ForceUpdate` (2846, dispatches between
///   `_UpdateDataProvider` 2854 and `_UpdateStatsDataProvider` 2868
///   based on `achievementFunctions == COMPARISON_ACHIEVEMENT_FUNCTIONS`
///   vs `COMPARISON_STAT_FUNCTIONS`). If the spec really meant
///   "the comparison-side update driver", the closest analogue is
///   `_ForceUpdate`. A future Blizzard refactor that introduces a
///   `_Update` symbol would flip this tripwire and force this test to
///   be re-evaluated against the new shape.
/// - `AchievementFrameComparison_OnInspectReady` — never declared.
///   `INSPECT_ACHIEVEMENT_READY` is handled inline from line 2815-2819
///   inside `AchievementFrameComparison_OnEvent`: clear selected
///   categories, fetch current category, call `_UpdateStatusBars`, then
///   set `AchievementFrameComparisonHeader.Points` to
///   `GetComparisonAchievementPoints()`. A future Blizzard refactor
///   that splits this arm into its own `_OnInspectReady` global would
///   flip this tripwire.
///
/// Asserting both `type(present) == "function"` AND `type(absent) ==
/// "nil"` makes the test a two-way contract pin: it fails loudly if
/// `_UpdateStatusBars` is removed/renamed AND if either of the
/// PLAN-anticipated symbols suddenly appears (which would be the
/// signal to re-read the new structure and update the spec).
#[cfg(feature = "optional-tests")]
#[test]
fn achievement_frame_comparison_plan_named_globals_split_presence_absence() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        assert(type(AchievementFrameComparison_Update) == "function")
        assert(type(AchievementFrameComparison_UpdateStatusBars) == "function")
        assert(AchievementFrameComparison_OnInspectReady == nil)
        AchievementFrameComparison_UpdateStatusBars(92)
        local total, completed = GetCategoryNumAchievements(92)
        assert(AchievementFrameComparisonSummaryPlayerStatusBar.text:GetText() == completed.."/"..total)
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for name in PLAN_NAMED_COMPARISON_PRESENT {
            let probe = format!(r#"return type({name})"#);
            let actual_type = env
                .eval::<String>(&probe)
                .expect("file-scope global type probe must run cleanly");

            assert_eq!(
                actual_type, "function",
                "Global `{name}` MUST be a function after the smoke-shape harness loads \
                 `Blizzard_AchievementUI`. This is the only PLAN-named comparison global that \
                 actually exists in `Mainline/Blizzard_AchievementUI.lua` (line 820). A \
                 non-function reading proves either (a) the file chunk failed before reaching \
                 the declaration (load.rs's `achievement_ui_load_emits_no_lane_specific_lua_errors` \
                 would also fail in that case — multi-test failure mode), OR (b) Blizzard \
                 re-shaped `_UpdateStatusBars` into a method on the `AchievementFrameComparison` \
                 sub-frame / removed it / renamed it. Got `type({name}) == \"{actual_type}\"`."
            );
        }

        for name in PLAN_NAMED_COMPARISON_ABSENT {
            let probe = format!(r#"return type({name})"#);
            let actual_type = env
                .eval::<String>(&probe)
                .expect("file-scope global type probe must run cleanly");

            assert_eq!(
                actual_type, "nil",
                "Global `{name}` MUST be nil after the smoke-shape harness loads \
                 `Blizzard_AchievementUI`. The PLAN line names this symbol but it does NOT \
                 exist in `Mainline/Blizzard_AchievementUI.lua` — see the module doc comment for \
                 details on which actual symbols cover the same role (`_OnEvent` line 2814 \
                 inline-dispatches INSPECT_ACHIEVEMENT_READY for the OnInspectReady role; \
                 `_ForceUpdate` line 2846 plus `_UpdateDataProvider` / `_UpdateStatsDataProvider` \
                 cover the Update role). A non-nil reading here is itself meaningful: it would \
                 prove Blizzard has now factored the inline arm into a top-level function, at \
                 which point the spec needs to be updated to point at the new symbol's actual \
                 contract. Got `type({name}) == \"{actual_type}\"`, expected \"nil\"."
            );
        }
    });
}

/// Pin the achievement-row click/enter/criteria-render globals — split
/// presence/absence.
///
/// **Spec/source mismatch.** The PLAN line names four globals
/// (`AchievementButton_OnClick`, `_OnEnter`, `_OnDoubleClick`,
/// `AchievementObjectives_DisplayCriteria`) but only
/// `AchievementObjectives_DisplayCriteria` is declared at the file
/// scope. The three `AchievementButton_*` symbols never existed as
/// top-level globals — Blizzard refactored the per-row interaction
/// surface onto `AchievementTemplateMixin` years ago, and the XML
/// `AchievementTemplate` virtual frame at line 733 uses
/// `mixin="AchievementTemplateMixin"` plus `<OnClick method="OnClick"/>`
/// / `<OnEnter method="OnEnter"/>` (lines 880-881) to dispatch to the
/// mixin instance rather than a global handler. No double-click handler
/// is wired at all — neither the Lua nor the XML mention any
/// `OnDoubleClick` script in the achievement UI lane.
///
/// We therefore split the test along the spec/source boundary:
///
/// **Presence tripwire** (`PLAN_NAMED_ACHIEVEMENT_ROW_PRESENT`):
/// - `AchievementObjectives_DisplayCriteria` at line 1861 — takes
///   `(objectivesFrame, id)` and is the criteria-row layout driver
///   invoked when a row expands. Early-returns on `not id`. Reads
///   `GetAchievementGuildRep(id)` for the rep-requirement banner
///   (writes `objectivesFrame.RepCriteria` text/colour and calls
///   `AddExtraCriteriaRow()` to push subsequent rows down). Continues
///   on to walk the meta-criteria list and the regular criteria list,
///   accumulating `numMetaRows` / `numCriteriaRows` /
///   `numExtraCriteriaRows` for layout sizing. Three local counters
///   plus the `yOffset` accumulator drive the per-row layout — the
///   function is the bridge between the achievement-data API and the
///   ScrollBox button's visible criteria block.
///
/// **Absence tripwires** (`PLAN_NAMED_ACHIEVEMENT_ROW_ABSENT`):
/// - `AchievementButton_OnClick` — never declared. The actual handler
///   is `AchievementTemplateMixin:OnClick(buttonName, down)` at line
///   1089 which delegates to `self:ProcessClick(buttonName, down)`.
///   The XML at 880 binds via `<OnClick method="OnClick"/>` so the
///   dispatch goes through the mixin namespace, not the global table.
/// - `AchievementButton_OnEnter` — never declared. Actual handler is
///   `AchievementTemplateMixin:OnEnter` at line 1093 which shows the
///   `Highlight` child and short-circuits when `self.id` is nil
///   (the empty-row case during ScrollBox button construction). XML
///   binds via `<OnEnter method="OnEnter"/>` at line 881.
/// - `AchievementButton_OnDoubleClick` — never declared **and there
///   is no double-click handler anywhere in the addon**. A grep
///   across both Mainline/Blizzard_AchievementUI.lua AND .xml for
///   `OnDoubleClick` returns zero hits. This is a stronger tripwire
///   than the OnClick/OnEnter ones — those have a clear mixin-method
///   replacement, but a true reading on this name would prove
///   Blizzard added a brand new behaviour to the row, not just
///   refactored an existing one.
///
/// Asserting both `type(present) == "function"` AND `type(absent) ==
/// "nil"` makes the test a two-way contract pin: it fails loudly if
/// `_DisplayCriteria` is removed/renamed AND if any of the three
/// PLAN-anticipated symbols suddenly appears (which would be the
/// signal to re-read the new structure and update the spec).
#[cfg(feature = "optional-tests")]
#[test]
fn achievement_row_plan_named_globals_split_presence_absence() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        for _, name in ipairs({"AchievementButton_OnClick", "AchievementButton_OnLoad", "AchievementButton_Collapse", "AchievementButton_Expand", "AchievementObjectives_DisplayCriteria"}) do
            assert(type(_G[name]) == "function", name)
        end
        assert(AchievementButton_OnDoubleClick == nil)
        local row = CreateFrame("Button", "F08SurfaceRow", AchievementFrameAchievements, "AchievementTemplate")
        assert(row.label and row.icon and row.shield and row.plusMinus)
        assert(row:GetScript("OnClick"))
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for name in PLAN_NAMED_ACHIEVEMENT_ROW_PRESENT {
            let probe = format!(r#"return type({name})"#);
            let actual_type = env
                .eval::<String>(&probe)
                .expect("file-scope global type probe must run cleanly");

            assert_eq!(
                actual_type, "function",
                "Global `{name}` MUST be a function after the smoke-shape harness loads \
                 `Blizzard_AchievementUI`. This is the only PLAN-named achievement-row global \
                 that actually exists at the file scope in `Mainline/Blizzard_AchievementUI.lua` \
                 (line 1861). A non-function reading proves either (a) the file chunk failed \
                 before reaching the declaration (load.rs's \
                 `achievement_ui_load_emits_no_lane_specific_lua_errors` would also fail in that \
                 case — multi-test failure mode), OR (b) Blizzard re-shaped \
                 `_DisplayCriteria` into a method on `AchievementTemplateMixin` (the same \
                 refactor that already absorbed OnClick/OnEnter) / removed it / renamed it. \
                 Got `type({name}) == \"{actual_type}\"`."
            );
        }

        for name in PLAN_NAMED_ACHIEVEMENT_ROW_ABSENT {
            let probe = format!(r#"return type({name})"#);
            let actual_type = env
                .eval::<String>(&probe)
                .expect("file-scope global type probe must run cleanly");

            assert_eq!(
                actual_type, "nil",
                "Global `{name}` MUST be nil after the smoke-shape harness loads \
                 `Blizzard_AchievementUI`. The PLAN line names this symbol but it does NOT exist \
                 in `Mainline/Blizzard_AchievementUI.lua` — see the module doc comment for \
                 details on which actual symbols cover the same role \
                 (`AchievementTemplateMixin:OnClick` line 1089 and `:OnEnter` line 1093 for the \
                 click/enter roles, dispatched from XML via `<OnClick method=\"OnClick\"/>` / \
                 `<OnEnter method=\"OnEnter\"/>` at lines 880-881; NO double-click handler \
                 anywhere in the addon's Lua or XML). A non-nil reading here is itself \
                 meaningful: for OnClick/OnEnter it would prove Blizzard regressed the mixin \
                 refactor back to a top-level global; for OnDoubleClick it would prove Blizzard \
                 added brand-new double-click behaviour to achievement rows that the spec \
                 must now describe. Got `type({name}) == \"{actual_type}\"`, expected \"nil\"."
            );
        }
    });
}
