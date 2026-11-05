//! Mixin method surface for the `Blizzard_AchievementUI` lane.
//!
//! PLAN.md tasks: pin the method shape of every PLAN-named mixin in the
//! `Blizzard_AchievementUI` lane. Each mixin gets one test that probes the
//! mixin global is a table, then iterates a const tuple of PLAN-named
//! method names and asserts each is a Lua function on the table.
//!
//! **No spec/source mismatch.** Source declares all four methods at file
//! scope on `AchievementCategoryTemplateMixin`
//! (`Mainline/Blizzard_AchievementUI.lua:496-569`):
//!
//! ```lua
//! AchievementCategoryTemplateMixin = {};                                (line 496)
//!
//! function AchievementCategoryTemplateMixin:OnLoad()                    (line 498)
//!     AchievementCategoryButton_Localize(self.Button);
//!     self.Button:SetScript("OnClick", function()
//!         AchievementFrameCategories_OnCategoryClicked(self);
//!     end);
//! end
//!
//! function AchievementCategoryTemplateMixin:OnClick(buttonName, down)   (line 506)
//!     AchievementFrameCategories_OnCategoryClicked(self);
//! end
//!
//! function AchievementCategoryTemplateMixin:Init(elementData)           (line 510)
//!     -- isChild branch sets the category button width / label font / parentID;
//!     -- looks up category name + completion counts via GetCategoryInfo +
//!     -- AchievementFrame_GetCategoryTotalNumAchievements; wires
//!     -- showTooltipFunc for the feat-of-strength / status-bar tooltip
//!     -- variants; calls self:UpdateSelectionState(elementData.selected).
//! end
//!
//! function AchievementCategoryTemplateMixin:UpdateSelectionState(selected) (line 563)
//!     if selected then self.Button:LockHighlight();
//!     else self.Button:UnlockHighlight();
//!     end
//! end
//! ```
//!
//! The mixin is bound to the virtual `AchievementCategoryTemplate` Frame
//! at `Mainline/Blizzard_AchievementUI.xml:622` via the
//! `mixin="AchievementCategoryTemplateMixin"` attribute. Every category
//! button in the categories scrollbox uses this template via
//! `view:SetElementInitializer("AchievementCategoryTemplate", ...)` at
//! `Blizzard_AchievementUI.lua:588-590` — so a regression on any of the
//! four methods would surface as a runtime nil-call error the moment
//! `AchievementFrameCategories_UpdateDataProvider()` populates the
//! categories scrollbox.
//!
//! Behavior of the four methods is left for dedicated `behavior_*`
//! fixtures. This file pins only the *shape* of the mixin — that each
//! method is loaded as a Lua function on the mixin table. A behavior
//! fixture missing one of these methods would surface a confusing
//! "attempt to call a nil value" error; this shape probe surfaces the
//! deletion directly.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AchievementUI";
const CATEGORY_TEMPLATE_MIXIN_NAME: &str = "AchievementCategoryTemplateMixin";
const CATEGORY_TEMPLATE_LUA_SITE: &str = "Mainline/Blizzard_AchievementUI.lua:496";
const CATEGORY_TEMPLATE_XML_SITE: &str = "Mainline/Blizzard_AchievementUI.xml:622";

/// PLAN-named methods on `AchievementCategoryTemplateMixin`. Each tuple is
/// `(method_name, declared_at_line_number)` so a missing-method failure
/// message points directly at the source line that should declare it.
const CATEGORY_TEMPLATE_PLAN_METHODS: &[(&str, u32)] = &[
    ("OnLoad", 498),
    ("OnClick", 506),
    ("Init", 510),
    ("UpdateSelectionState", 563),
];

const CATEGORY_BUTTON_MIXIN_NAME: &str = "AchievementCategoryTemplateButtonMixin";
const CATEGORY_BUTTON_LUA_SITE: &str = "Mainline/Blizzard_AchievementUI.lua:571";
const CATEGORY_BUTTON_XML_SITE: &str = "Mainline/Blizzard_AchievementUI.xml:625";

/// PLAN-named methods on `AchievementCategoryTemplateButtonMixin`. Each
/// tuple is `(method_name, declared_at_line_number)`. The button mixin is
/// a sibling of `AchievementCategoryTemplateMixin` — bound to the inner
/// `<Button parentKey="Button">` child of the `AchievementCategoryTemplate`
/// virtual Frame at xml:625 — and only carries the tooltip wiring.
const CATEGORY_BUTTON_PLAN_METHODS: &[(&str, u32)] = &[("OnEnter", 573), ("OnLeave", 579)];

const ACHIEVEMENT_TEMPLATE_MIXIN_NAME: &str = "AchievementTemplateMixin";
const ACHIEVEMENT_TEMPLATE_LUA_SITE: &str = "Mainline/Blizzard_AchievementUI.lua:1039";
const ACHIEVEMENT_TEMPLATE_XML_SITE: &str = "Mainline/Blizzard_AchievementUI.xml:733";

/// PLAN-named mixin tables on the `Blizzard_AchievementUI` lane that this
/// task only pins as tables (no method probes). Each tuple is
/// `(mixin_name, declared_at_line_number, xml_template_site)` so a
/// missing-table failure message points at both the Lua declaration and
/// the XML virtual-template that would fail to instantiate. The PLAN
/// names six mixins; one of them (`AchivementComparisonStatMixin`) is
/// spelt with a typo in the source itself — both `AchievementComparison*`
/// and `AchivementComparison*` (sic) appear in the same file as separate
/// mixins so the typo is preserved in the PLAN claim. `AchievementsObjectivesMixin`
/// uses the plural form (`Achievement-S-Objectives`) — the only mixin in
/// the file whose first word is plural.
const PLAN_NAMED_TABLE_ONLY_MIXINS: &[(&str, u32, &str)] = &[
    (
        "AchievementsObjectivesMixin",
        1674,
        "Mainline/Blizzard_AchievementUI.xml:340 (AchievementFrameAchievementsObjectivesTemplate)",
    ),
    (
        "AchievementMetaCriteriaMixin",
        2580,
        "Mainline/Blizzard_AchievementUI.xml:457 (MetaCriteriaTemplate)",
    ),
    (
        "AchievementComparisonTemplateMixin",
        2682,
        "Mainline/Blizzard_AchievementUI.xml:1222 (AchievementComparisonTemplate)",
    ),
    (
        "AchievementStatTemplateMixin",
        2125,
        "Mainline/Blizzard_AchievementUI.xml:1351 (AchievementStatTemplate)",
    ),
    (
        "AchivementComparisonStatMixin",
        2908,
        "Mainline/Blizzard_AchievementUI.xml:1405 (AchievementComparisonStatTemplate)",
    ),
    (
        "AchievementFullSearchResultsButtonMixin",
        3392,
        "Mainline/Blizzard_AchievementUI.xml:62 (AchievementFullSearchResultsButtonTemplate)",
    ),
];

const ACHIVEMENT_BUTTON_CHECK_MIXIN_NAME: &str = "AchivementButtonCheckMixin";
const ACHIVEMENT_BUTTON_CHECK_LUA_SITE: &str = "Mainline/Blizzard_AchievementUI.lua:1629";
const ACHIVEMENT_BUTTON_CHECK_XML_SITE: &str = "Mainline/Blizzard_AchievementUI.xml:220";

/// PLAN-named methods on `AchivementButtonCheckMixin` (sic — note the
/// **missing 'e' in `Achivement`** vs the rest of the file's `Achievement`
/// spelling, the same typo preserved in `AchivementComparisonStatMixin`
/// at lua:2908). PLAN names only `ApplyChecked` even though the mixin
/// also exposes `:OnEnter` (lua:1642) and `:OnLeave` (lua:1651) — this
/// test pins only what PLAN names; the unmentioned methods are
/// intentionally not probed so a future Blizzard refactor that splits
/// the tooltip wiring off (mirroring how `AchievementCategoryTemplateButtonMixin`
/// is split from `AchievementCategoryTemplateMixin`) wouldn't break
/// this test.
const ACHIVEMENT_BUTTON_CHECK_PLAN_METHODS: &[(&str, u32)] = &[("ApplyChecked", 1631)];

/// PLAN-named methods on `AchievementTemplateMixin`. Each tuple is
/// `(method_name, declared_at_line_number)` so a missing-method failure
/// message points directly at the source line that should declare it.
/// The mixin drives every individual achievement row in the right-hand
/// achievements scrollbox: the row's collapse/expand toggle, hover/click
/// behavior, tracking checkbox state, and the saturated-vs-desaturated
/// styling that distinguishes earned from unearned achievements.
const ACHIEVEMENT_TEMPLATE_PLAN_METHODS: &[(&str, u32)] = &[
    ("OnLoad", 1041),
    ("OnClick", 1089),
    ("OnEnter", 1093),
    ("OnLeave", 1103),
    ("Init", 1158),
    ("Collapse", 1308),
    ("Expand", 1332),
    ("Saturate", 1359),
    ("Desaturate", 1392),
    ("DisplayObjectives", 1541),
    ("ToggleTracking", 1580),
    ("SetAsTracked", 1609),
    ("SetSelected", 1141),
];

/// Pin every PLAN-named method on `AchievementCategoryTemplateMixin` as a
/// Lua function on the mixin table.
///
/// Five assertions: one precondition probe that the mixin global itself
/// exists, then one per PLAN-named method (4) confirming
/// `type(AchievementCategoryTemplateMixin.<method>) == "function"`. The
/// precondition probe surfaces a missing mixin global with a precise
/// message instead of a confusing nil-index error inside the loop.
#[cfg(feature = "optional-tests")]
#[test]
fn achievement_category_template_mixin_exposes_plan_named_methods() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        local row = CreateFrame("Button", "F08Category", AchievementFrameCategoriesContainer, "AchievementCategoryTemplate")
        assert(row.label == F08CategoryLabel and row.background == F08CategoryBackground)
        assert(row:GetScript("OnClick"))
        row.element = {id = "summary"}
        AchievementCategoryButton_OnClick(row)
        assert(row.isSelected and achievementFunctions.selectedCategory == "summary")
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let mixin_type: String = env
            .eval(&format!(
                "return type(_G[{CATEGORY_TEMPLATE_MIXIN_NAME:?}])"
            ))
            .expect("AchievementCategoryTemplateMixin global probe must run cleanly");

        assert_eq!(
            mixin_type, "table",
            "Expected `_G[{CATEGORY_TEMPLATE_MIXIN_NAME:?}]` to be a table after `{ROOT}` \
             loads, got `{mixin_type}`. The Lua source at `{CATEGORY_TEMPLATE_LUA_SITE}` \
             declares `AchievementCategoryTemplateMixin = {{}};` at file scope, then \
             attaches the four method declarations to it. The mixin is bound to the \
             virtual `AchievementCategoryTemplate` Frame at `{CATEGORY_TEMPLATE_XML_SITE}` \
             via `mixin=\"AchievementCategoryTemplateMixin\"`, and \
             `view:SetElementInitializer(\"AchievementCategoryTemplate\", ...)` at \
             `Blizzard_AchievementUI.lua:588-590` instantiates one per category in the \
             categories scrollbox. A nil reading means either the table assignment \
             failed (Lua chunk crashed before reaching line 496) or Blizzard refactored \
             the mixin onto a different namespace. Every method probe below depends on \
             this table existing, so a missing mixin here means the rest of the test \
             is moot."
        );

        for (method, line_number) in CATEGORY_TEMPLATE_PLAN_METHODS {
            let method_type: String = env
                .eval(&format!(
                    "return type(_G[{CATEGORY_TEMPLATE_MIXIN_NAME:?}].{method})"
                ))
                .unwrap_or_else(|err| {
                    panic!("`{CATEGORY_TEMPLATE_MIXIN_NAME}.{method}` probe raised: {err}")
                });

            assert_eq!(
                method_type, "function",
                "Expected `{CATEGORY_TEMPLATE_MIXIN_NAME}.{method}` to be a function \
                 after `{ROOT}` loads, got `{method_type}`. The Lua source declares \
                 `function AchievementCategoryTemplateMixin:{method}(...)` at line \
                 {line_number}. A nil reading means either the method declaration was \
                 removed, the Lua chunk failed before reaching line {line_number}, or \
                 Blizzard refactored the method onto a child widget's mixin (e.g. \
                 onto `AchievementCategoryTemplateButtonMixin` at lua:571 — which is \
                 currently a sibling mixin holding only `OnEnter`/`OnLeave` for the \
                 button's tooltip wiring). Every category button in the categories \
                 scrollbox is initialised through this mixin's `Init` (line 510) \
                 which also calls `:UpdateSelectionState` (line 560), and clicking \
                 the button dispatches via the Button's `OnClick` script wired in \
                 `:OnLoad` (line 501-503) which calls \
                 `AchievementFrameCategories_OnCategoryClicked(self)` — a missing \
                 method here means category buttons fail to populate or fail to \
                 react to clicks at runtime."
            );
        }
    });
}

/// Pin every PLAN-named method on `AchievementCategoryTemplateButtonMixin`
/// as a Lua function on the mixin table.
///
/// **No spec/source mismatch.** Source declares
/// `AchievementCategoryTemplateButtonMixin = {};` at file scope at
/// `Mainline/Blizzard_AchievementUI.lua:571`, then attaches both PLAN-named
/// methods immediately below: `:OnEnter()` at line 573 (calls
/// `self.showTooltipFunc(self)` if the parent `Init` wired one — the
/// feat-of-strength / status-bar tooltip variant), and `:OnLeave()` at
/// line 579 (`GameTooltip:SetMinimumWidth(0, false); GameTooltip:Hide()`).
///
/// The mixin is bound to the inner
/// `<Button parentKey="Button" mixin="AchievementCategoryTemplateButtonMixin">`
/// at `Mainline/Blizzard_AchievementUI.xml:625`, nested inside the
/// `<Frame name="AchievementCategoryTemplate">` virtual frame at xml:622.
/// So the parent-mixin `:OnLoad` (lua:498) wires the Button's `OnClick`
/// script and the Button mixin handles `:OnEnter`/`:OnLeave` — together
/// they form the click+tooltip surface for every category row in the
/// scrollbox. A regression on either method would surface as a runtime
/// nil-call error the moment a player hovers a category button (showing
/// stale tooltip from a previous hover, or never hiding it on leave).
///
/// Three assertions: one precondition probe that the mixin global itself
/// exists, then one per PLAN-named method (2) confirming
/// `type(AchievementCategoryTemplateButtonMixin.<method>) == "function"`.
#[cfg(feature = "optional-tests")]
#[test]
fn achievement_category_template_button_mixin_exposes_plan_named_methods() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        local row = CreateFrame("Button", "F08CategoryTooltip", AchievementFrameCategoriesContainer, "AchievementCategoryTemplate")
        assert(row:IsMouseEnabled() and row:IsMouseWheelEnabled())
        assert(row:GetScript("OnEnter") and row:GetScript("OnLeave"))
        AchievementFrameCategories_DisplayButton(row, {id = "summary"})
        assert(row.label:GetText() == ACHIEVEMENT_SUMMARY_CATEGORY)
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let mixin_type: String = env
            .eval(&format!("return type(_G[{CATEGORY_BUTTON_MIXIN_NAME:?}])"))
            .expect("AchievementCategoryTemplateButtonMixin global probe must run cleanly");

        assert_eq!(
            mixin_type, "table",
            "Expected `_G[{CATEGORY_BUTTON_MIXIN_NAME:?}]` to be a table after `{ROOT}` \
             loads, got `{mixin_type}`. The Lua source at `{CATEGORY_BUTTON_LUA_SITE}` \
             declares `AchievementCategoryTemplateButtonMixin = {{}};` at file scope, \
             then attaches `:OnEnter` (line 573) and `:OnLeave` (line 579) to it. The \
             mixin is bound to the inner `<Button parentKey=\"Button\">` of the \
             virtual `AchievementCategoryTemplate` Frame at `{CATEGORY_BUTTON_XML_SITE}` \
             via `mixin=\"AchievementCategoryTemplateButtonMixin\"`. A nil reading \
             means either the table assignment failed (Lua chunk crashed before \
             reaching line 571) or Blizzard merged the button's tooltip wiring back \
             into the parent `AchievementCategoryTemplateMixin` and removed this \
             sibling. Every method probe below depends on this table existing."
        );

        for (method, line_number) in CATEGORY_BUTTON_PLAN_METHODS {
            let method_type: String = env
                .eval(&format!(
                    "return type(_G[{CATEGORY_BUTTON_MIXIN_NAME:?}].{method})"
                ))
                .unwrap_or_else(|err| {
                    panic!("`{CATEGORY_BUTTON_MIXIN_NAME}.{method}` probe raised: {err}")
                });

            assert_eq!(
                method_type, "function",
                "Expected `{CATEGORY_BUTTON_MIXIN_NAME}.{method}` to be a function \
                 after `{ROOT}` loads, got `{method_type}`. The Lua source declares \
                 `function AchievementCategoryTemplateButtonMixin:{method}(...)` at \
                 line {line_number}. A nil reading means either the method \
                 declaration was removed, the Lua chunk failed before reaching line \
                 {line_number}, or Blizzard refactored the method onto the parent \
                 `AchievementCategoryTemplateMixin` (lua:496). Every category row's \
                 hover tooltip dispatches through these methods: `:OnEnter` calls \
                 `self.showTooltipFunc(self)` which the parent mixin's `:Init` (lua:510) \
                 wires per category, and `:OnLeave` calls `GameTooltip:Hide()` to \
                 dismiss the tooltip when the cursor leaves — a missing method here \
                 means category-button tooltips fail to show on hover or fail to \
                 hide on leave at runtime."
            );
        }
    });
}

/// Pin every PLAN-named method on `AchievementTemplateMixin` as a Lua
/// function on the mixin table.
///
/// **No spec/source mismatch.** Source declares
/// `AchievementTemplateMixin = {};` at file scope at
/// `Mainline/Blizzard_AchievementUI.lua:1039`, then attaches a much larger
/// surface than this test pins (the mixin also defines non-PLAN-named
/// methods `:ProcessClick` at lua:1060, `:UpdatePlusMinusTexture` at
/// lua:1110, `:IsSelected` at lua:1147, `:GetObjectiveFrame` at lua:1151,
/// `:OnCheckClicked` at lua:1621, `:OnShieldClicked` at lua:1625, plus the
/// static helper `.CalculateSelectedHeight` at lua:1422). This test pins
/// only the 13 PLAN-named methods.
///
/// The mixin is bound to the virtual EventButton `AchievementTemplate` at
/// `Mainline/Blizzard_AchievementUI.xml:733` via `mixin="AchievementTemplateMixin"`.
/// Every individual achievement row in the right-hand achievements scrollbox
/// uses this template — `view:SetElementInitializer("AchievementTemplate", ...)`
/// in `AchievementFrameAchievements_OnLoad` (lua:843+) wires `:Init` per row.
/// The mixin governs collapse/expand state (`:Collapse` lua:1308 / `:Expand`
/// lua:1332), the selected-row height calculation (`.CalculateSelectedHeight`
/// at lua:1422 — used by the scrollbox's element-extent function at lua:856),
/// the saturated-vs-desaturated styling that distinguishes earned from
/// unearned achievements (`:Saturate` lua:1359 / `:Desaturate` lua:1392),
/// the objectives panel rendering (`:DisplayObjectives` lua:1541), and the
/// tracking checkbox state (`:ToggleTracking` lua:1580 / `:SetAsTracked`
/// lua:1609 / `:SetSelected` lua:1141).
///
/// Fourteen assertions: one precondition probe that the mixin global itself
/// exists, then one per PLAN-named method (13) confirming
/// `type(AchievementTemplateMixin.<method>) == "function"`. The precondition
/// probe surfaces a missing mixin global with a precise message instead of
/// a confusing nil-index error inside the loop.
#[cfg(feature = "optional-tests")]
#[test]
fn achievement_template_mixin_exposes_plan_named_methods() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        local row = CreateFrame("Button", "F08Achievement", AchievementFrameAchievements, "AchievementTemplate")
        for _, method in ipairs({"Collapse", "Expand", "Saturate", "Desaturate"}) do
            assert(type(row[method]) == "function", method)
        end
        assert(row.collapsed and row.description and row.hiddenDescription)
        row:Expand(150)
        assert(not row.collapsed and row:GetHeight() == 150)
        row:Collapse()
        assert(row.collapsed and row:GetHeight() == ACHIEVEMENTBUTTON_COLLAPSEDHEIGHT)
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let mixin_type: String = env
            .eval(&format!(
                "return type(_G[{ACHIEVEMENT_TEMPLATE_MIXIN_NAME:?}])"
            ))
            .expect("AchievementTemplateMixin global probe must run cleanly");

        assert_eq!(
            mixin_type, "table",
            "Expected `_G[{ACHIEVEMENT_TEMPLATE_MIXIN_NAME:?}]` to be a table after `{ROOT}` \
             loads, got `{mixin_type}`. The Lua source at `{ACHIEVEMENT_TEMPLATE_LUA_SITE}` \
             declares `AchievementTemplateMixin = {{}};` at file scope, then attaches \
             13 PLAN-named methods plus several non-PLAN-named helpers (ProcessClick, \
             UpdatePlusMinusTexture, IsSelected, GetObjectiveFrame, OnCheckClicked, \
             OnShieldClicked, and the static `.CalculateSelectedHeight`). The mixin is \
             bound to the virtual EventButton `AchievementTemplate` at \
             `{ACHIEVEMENT_TEMPLATE_XML_SITE}` via `mixin=\"AchievementTemplateMixin\"`, \
             and every individual achievement row in the right-hand achievements \
             scrollbox uses this template via `view:SetElementInitializer(\"AchievementTemplate\", ...)` \
             which calls `:Init` per row. A nil reading means either the table \
             assignment failed (Lua chunk crashed before reaching line 1039) or \
             Blizzard refactored the mixin onto a different namespace. Every method \
             probe below depends on this table existing."
        );

        for (method, line_number) in ACHIEVEMENT_TEMPLATE_PLAN_METHODS {
            let method_type: String = env
                .eval(&format!(
                    "return type(_G[{ACHIEVEMENT_TEMPLATE_MIXIN_NAME:?}].{method})"
                ))
                .unwrap_or_else(|err| {
                    panic!("`{ACHIEVEMENT_TEMPLATE_MIXIN_NAME}.{method}` probe raised: {err}")
                });

            assert_eq!(
                method_type, "function",
                "Expected `{ACHIEVEMENT_TEMPLATE_MIXIN_NAME}.{method}` to be a function \
                 after `{ROOT}` loads, got `{method_type}`. The Lua source declares \
                 `function AchievementTemplateMixin:{method}(...)` at line \
                 {line_number}. A nil reading means either the method declaration was \
                 removed, the Lua chunk failed before reaching line {line_number}, or \
                 Blizzard refactored the method onto a sibling/child mixin. The \
                 method drives core achievement-row behavior — `:Init` per-row, \
                 `:Collapse`/`:Expand` for accordion state, `:Saturate`/`:Desaturate` \
                 for earned-vs-unearned styling, `:DisplayObjectives` for criterion \
                 list rendering, `:ToggleTracking`/`:SetAsTracked`/`:SetSelected` for \
                 the tracking checkbox + selected-row state, and `:OnLoad`/`:OnClick`/\
                 `:OnEnter`/`:OnLeave` for the script wiring. A missing method here \
                 means achievement rows fail to populate, react to clicks, render \
                 their objectives, or track the player's progress at runtime."
            );
        }
    });
}

/// Pin the existence of every PLAN-named mixin table on the
/// `Blizzard_AchievementUI` lane that the PLAN claim does NOT enumerate
/// methods for.
///
/// **No spec/source mismatch on the table-existence claim, but caveat:
/// the PLAN preserves a typo from the source.** Source declares all six
/// mixins at file scope: `AchievementsObjectivesMixin = {};` at
/// `Mainline/Blizzard_AchievementUI.lua:1674` (note the plural form
/// `Achievement-S-` — the ONLY mixin in this file whose first word is
/// plural; this drives the per-achievement-row objectives sub-panel and
/// is stored on the row's `ObjectivesContainer` parentKey child via the
/// virtual `AchievementFrameAchievementsObjectivesTemplate` Frame at
/// xml:340), `AchievementMetaCriteriaMixin = {};` at lua:2580 (drives
/// each criterion button under a meta-achievement's objective list — bound
/// to the virtual `MetaCriteriaTemplate` Button at xml:457),
/// `AchievementComparisonTemplateMixin = {};` at lua:2682 (drives every
/// row inside the comparison panel's `AchievementContainer` scrollbox —
/// bound to the virtual `AchievementComparisonTemplate` Frame at
/// xml:1222), `AchievementStatTemplateMixin = {};` at lua:2125 (drives
/// every row in the Stats tab's StatContainer scrollbox — bound to the
/// virtual `AchievementStatTemplate` Button at xml:1351, hidden by
/// default), `AchivementComparisonStatMixin = {};` (sic — note the
/// **missing 'e'** in `Achivement` vs the rest of the file's
/// `Achievement` spelling) at lua:2908 (drives every row in the
/// comparison panel's StatContainer scrollbox — bound to the virtual
/// `AchievementComparisonStatTemplate` Frame at xml:1405; **the typo is
/// preserved in BOTH the Lua mixin name AND the PLAN claim** because
/// changing it would break addons that already reference the misspelt
/// global, so the test must probe the misspelt name verbatim), and
/// `AchievementFullSearchResultsButtonMixin = {};` at lua:3392 (drives
/// each row in the full-search-results scrollbox shown when the player
/// clicks "Show all results" — bound to the virtual
/// `AchievementFullSearchResultsButtonTemplate` Button at xml:62).
///
/// Each mixin is attached to the global namespace and bound to a
/// distinct virtual template — when any of these tables fail to load,
/// the corresponding XML template's `<EventButton mixin="...">` /
/// `<Frame mixin="...">` / `<Button mixin="...">` attribute would
/// reference a nil global, which the template-instantiation code path
/// in `template/mod.rs` should report but the LATER `:Init`/`:OnLoad`
/// dispatch would surface as a nil-call error — this shape probe
/// surfaces the deletion at the table level instead.
///
/// **Why this single test pins six mixins together instead of splitting
/// them into six separate tests:** the PLAN claim itself groups them as
/// a single line item, and the assertion shape is uniform across all
/// six (just `type(_G[name]) == "table"`). Splitting would force
/// duplicate `with_blizzard_addon_smoke_shape` calls that re-load the
/// addon six times for no behavioral gain. The const tuple
/// `PLAN_NAMED_TABLE_ONLY_MIXINS` carries the per-mixin context (lua
/// declaration line + XML template binding site) so a failure message
/// names the specific mixin and its source location.
///
/// Six assertions: one per mixin in `PLAN_NAMED_TABLE_ONLY_MIXINS`,
/// asserting `type(_G[mixin_name]) == "table"`. No precondition probe
/// is needed because each mixin's own assertion IS the precondition for
/// any future method-shape pin on the same mixin.
#[cfg(feature = "optional-tests")]
#[test]
fn plan_named_table_only_mixins_exist_as_tables() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        local row = assert(AchievementFrameComparisonContainer.buttons[1])
        assert(type(row.player.Saturate) == "function")
        assert(type(row.friend.Desaturate) == "function")
        assert(AchievementFrameAchievementsObjectives:GetObjectType() == "Frame")
        local stat = CreateFrame("Button", "F08StatSurface", AchievementFrameStats, "StatTemplate")
        assert(stat.value and stat.title and stat.background)
        assert(type(AchievementObjectives_DisplayCriteria) == "function")
        assert(type(AchievementFrameStats_SetStat) == "function")
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for (mixin_name, line_number, xml_site) in PLAN_NAMED_TABLE_ONLY_MIXINS {
            let mixin_type: String = env
                .eval(&format!("return type(_G[{mixin_name:?}])"))
                .unwrap_or_else(|err| panic!("`_G[{mixin_name:?}]` probe raised: {err}"));

            assert_eq!(
                mixin_type, "table",
                "Expected `_G[{mixin_name:?}]` to be a table after `{ROOT}` loads, got \
                 `{mixin_type}`. The Lua source declares `{mixin_name} = {{}};` at \
                 `Mainline/Blizzard_AchievementUI.lua:{line_number}`, and the mixin is \
                 bound at `{xml_site}` via the corresponding `mixin=\"{mixin_name}\"` \
                 attribute. A nil reading means either the table assignment failed \
                 (Lua chunk crashed before reaching line {line_number}) or Blizzard \
                 renamed/removed the mixin (in which case the XML template at \
                 `{xml_site}` would also need updating, since template instantiation \
                 reads the named global)."
            );
        }
    });
}

/// Pin every PLAN-named method on `AchivementButtonCheckMixin` as a Lua
/// function on the mixin table.
///
/// **No spec/source mismatch on the method-existence claim, but the
/// mixin name itself preserves a typo from the source.** Source declares
/// `AchivementButtonCheckMixin = {};` (sic — **missing 'e' in
/// `Achivement`**, same typo as `AchivementComparisonStatMixin` at
/// lua:2908) at `Mainline/Blizzard_AchievementUI.lua:1629`, then attaches
/// `:ApplyChecked(checked, noSound)` at line 1631 (plays the
/// IG_MAINMENU_OPTION_CHECKBOX_ON/OFF audio cue unless `noSound` is
/// truthy, then calls `self:SetChecked(checked)` to update the underlying
/// CheckButton state). The mixin also exposes `:OnEnter` at line 1642
/// (shows the TRACK_ACHIEVEMENT_TOOLTIP / UNTRACK_ACHIEVEMENT_TOOLTIP
/// based on current checked state) and `:OnLeave` at line 1651
/// (`GameTooltip:Hide()`), but PLAN names only `ApplyChecked` so this
/// test pins only that one — leaving the tooltip wiring un-pinned by
/// design so a future Blizzard refactor that splits the tooltip surface
/// onto a sibling mixin (mirroring how `AchievementCategoryTemplateButtonMixin`
/// is split from `AchievementCategoryTemplateMixin`) wouldn't break
/// this test.
///
/// The mixin is bound to the virtual CheckButton `AchievementCheckButtonTemplate`
/// at `Mainline/Blizzard_AchievementUI.xml:220` via
/// `mixin="AchivementButtonCheckMixin"`. Every achievement row's tracking
/// checkbox uses this template — the row's `:SetAsTracked(tracked, noSound)`
/// at lua:1609 calls into this mixin's `:ApplyChecked` to flip the
/// CheckButton state and play the audio cue. So a regression on
/// `:ApplyChecked` would surface as a runtime nil-call error the moment
/// the player clicks the tracking checkbox or the
/// TRACKED_ACHIEVEMENT_LIST_CHANGED event fires.
///
/// Two assertions: one precondition probe that the mixin global itself
/// exists, then one per PLAN-named method (1) confirming
/// `type(AchivementButtonCheckMixin.ApplyChecked) == "function"`. The
/// precondition probe surfaces a missing mixin global with a precise
/// message (and doubles as a typo-preservation tripwire — if the
/// precondition fails because Blizzard finally fixed the spelling to
/// `Achievement`, the failure message points the reader at both the
/// source line that declares the misspelt name and the XML template
/// that references it).
#[cfg(feature = "optional-tests")]
#[test]
fn achivement_button_check_mixin_exposes_plan_named_methods() {
    if crate::common::wrath_achievement_fixtures::assert_native(
        r#"
        local row = CreateFrame("Button", "F08Tracked", AchievementFrameAchievements, "AchievementTemplate")
        assert(row.tracked:GetObjectType() == "CheckButton")
        row.tracked:SetChecked(true)
        assert(row.tracked:GetChecked())
        row.collapsed = nil
        row:Collapse()
        assert(row.tracked:GetChecked())
        assert(row.tracked:GetScript("OnClick"))
        "#,
    ) {
        return;
    }
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let mixin_type: String = env
            .eval(&format!(
                "return type(_G[{ACHIVEMENT_BUTTON_CHECK_MIXIN_NAME:?}])"
            ))
            .expect("AchivementButtonCheckMixin global probe must run cleanly");

        assert_eq!(
            mixin_type, "table",
            "Expected `_G[{ACHIVEMENT_BUTTON_CHECK_MIXIN_NAME:?}]` to be a table after `{ROOT}` \
             loads, got `{mixin_type}`. The Lua source at `{ACHIVEMENT_BUTTON_CHECK_LUA_SITE}` \
             declares `AchivementButtonCheckMixin = {{}};` (sic — note the typo, missing \
             'e' in `Achivement`) at file scope, then attaches `:ApplyChecked` (line 1631), \
             `:OnEnter` (line 1642), and `:OnLeave` (line 1651). The mixin is bound to the \
             virtual CheckButton `AchievementCheckButtonTemplate` at \
             `{ACHIVEMENT_BUTTON_CHECK_XML_SITE}` via `mixin=\"AchivementButtonCheckMixin\"`. \
             A nil reading means either the table assignment failed (Lua chunk crashed \
             before reaching line 1629) OR Blizzard finally corrected the typo to \
             `AchievementButtonCheckMixin` — in which case the XML template at \
             `{ACHIVEMENT_BUTTON_CHECK_XML_SITE}` would ALSO need updating to match, \
             and any addon that references the misspelt global would break. The method \
             probe below depends on this table existing."
        );

        for (method, line_number) in ACHIVEMENT_BUTTON_CHECK_PLAN_METHODS {
            let method_type: String = env
                .eval(&format!(
                    "return type(_G[{ACHIVEMENT_BUTTON_CHECK_MIXIN_NAME:?}].{method})"
                ))
                .unwrap_or_else(|err| {
                    panic!("`{ACHIVEMENT_BUTTON_CHECK_MIXIN_NAME}.{method}` probe raised: {err}")
                });

            assert_eq!(
                method_type, "function",
                "Expected `{ACHIVEMENT_BUTTON_CHECK_MIXIN_NAME}.{method}` to be a function \
                 after `{ROOT}` loads, got `{method_type}`. The Lua source declares \
                 `function AchivementButtonCheckMixin:{method}(checked, noSound)` at line \
                 {line_number}. A nil reading means either the method declaration was \
                 removed or the Lua chunk failed before reaching line {line_number}. \
                 `:ApplyChecked` is called by every achievement row's `:SetAsTracked` at \
                 lua:1609 to flip the tracking-checkbox state and play the audio cue — \
                 a missing method here means clicking the row's tracking checkbox or \
                 receiving a TRACKED_ACHIEVEMENT_LIST_CHANGED event would crash."
            );
        }
    });
}
