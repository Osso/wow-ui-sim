//! Mixin-method surface pin for `AccountStoreCategoryListMixin` (left-side category column).
//!
//! Spec/source mismatch finding (PLAN.md task for
//! `AccountStoreCategoryListMixin`): the plan names two methods —
//! `Refresh`, `SelectCategoryByID` — but
//! `Blizzard_AccountStoreCategoryList.lua:18-78` declares six
//! methods on the list mixin. Both PLAN names are absent. Verified
//! by `grep -rn "function AccountStoreCategoryListMixin:Refresh"
//! Interface/BlizzardUI/Blizzard_AccountStore/` returning zero
//! matches and the same for `SelectCategoryByID`. The closest
//! functional analogues live on the same mixin under different
//! names: the list's "rebuild from category data" path is
//! `SetCategories` (line 67, called by `OnStoreFrontSet`); the
//! list's "highlight a specific category by id" path is
//! `OnCategorySelected` (line 57, registered via
//! `AddStaticEventMethod` for the `AccountStore.CategorySelected`
//! EventRegistry callback). The list owns no public `Refresh()` —
//! ScrollBox-level data refresh is implicit in `SetCategories` via
//! `SetDataProvider` with `RetainScrollPosition`. All six methods
//! the mixin actually owns were PLAN-omitted:
//!
//! | PLAN name             | Status                                                     |
//! |-----------------------|------------------------------------------------------------|
//! | `Refresh`             | absent — no method by this name on `AccountStoreCategoryListMixin`. The closest functional analogue is `SetCategories` (line 67), which rebuilds the ScrollBox data provider from a categories list and triggers the `AccountStore.CategorySelected` EventRegistry callback for `categories[1]` (the first-category-auto-select behavior). PLAN's `Refresh` is also a near-miss for `OnStoreFrontSet` (line 53), which calls `SetCategories(C_AccountStore.GetCategories(storeFrontID))` as the public entry point for "fetch categories for the new store and re-render". |
//! | `SelectCategoryByID`  | absent — no method by this name on `AccountStoreCategoryListMixin`. The closest functional analogue is `OnCategorySelected` (line 57), which finds the row by `elementData.categoryID == categoryID` predicate and calls `SetRowSelectedState(button)`. The selection-by-id path is event-driven via the `AccountStore.CategorySelected` EventRegistry trigger (fired by `AccountStoreCategoryMixin:OnClick` and by `SetCategories` for the auto-select first-category case), not exposed as a public `SelectCategoryByID` method. |
//!
//! Architecturally-critical PLAN-omitted-but-present methods (the
//! six the surface test pins as guards against silent removal):
//!
//! | Method                | Source site                                                  | Role |
//! |-----------------------|--------------------------------------------------------------|------|
//! | `OnLoad`              | `Blizzard_AccountStoreCategoryList.lua:20`                   | Caches `self.SelectionHighlight = self.ScrollBox.SelectionHighlight`, calls `InitScrollBox` to wire the ScrollBox + ScrollBar + selection behavior, and registers two EventRegistry static callbacks via `AddStaticEventMethod` (`AccountStore.StoreFrontSet` → `OnStoreFrontSet`, `AccountStore.CategorySelected` → `OnCategorySelected`). |
//! | `InitScrollBox`       | `Blizzard_AccountStoreCategoryList.lua:29`                   | Creates a `CreateScrollBoxListLinearView` with element initializer that calls `button:SetCategory(elementData.categoryID)`, sets 16-px top padding, and binds the view to ScrollBox + ScrollBar. Adds an `Intrusive` selection behavior whose callback hides the SelectionHighlight on deselect and calls `SetRowSelectedState` on select. |
//! | `OnStoreFrontSet`     | `Blizzard_AccountStoreCategoryList.lua:53`                   | Handler for the `AccountStore.StoreFrontSet` EventRegistry callback — fetches categories via `C_AccountStore.GetCategories(storeFrontID)` and calls `SetCategories`. The public entry point for "store changed, re-render the category column". |
//! | `OnCategorySelected`  | `Blizzard_AccountStoreCategoryList.lua:57`                   | Handler for the `AccountStore.CategorySelected` EventRegistry callback — finds the row by `elementData.categoryID == categoryID` predicate via `FindFrameByPredicate` and calls `SetRowSelectedState(button)`. The closest-name analogue to PLAN's `SelectCategoryByID`. |
//! | `SetCategories`       | `Blizzard_AccountStoreCategoryList.lua:67`                   | Wraps the categories list in `CreateDataProviderWithAssignedKey(categories, "categoryID")` and feeds it to `ScrollBox:SetDataProvider` with `RetainScrollPosition`. Then triggers `AccountStore.CategorySelected` for `categories[1]` (the auto-select-first-category behavior). The closest-name analogue to PLAN's `Refresh`. |
//! | `SetRowSelectedState` | `Blizzard_AccountStoreCategoryList.lua:75`                   | Positions the `SelectionHighlight` overlay anchored CENTER to the row button (with y-offset +2) and shows it. Called from the `OnCategorySelected` event handler and from the ScrollBox selection-behavior callback in `InitScrollBox`. |

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AccountStore";
const CATEGORY_LIST_MIXIN_NAME: &str = "AccountStoreCategoryListMixin";

const ACTUAL_CATEGORY_LIST_METHODS: &[(&str, &str)] = &[
    (
        "OnLoad",
        "Blizzard_AccountStoreCategoryList.lua:20 — PLAN-omitted-but-present. Caches \
         self.SelectionHighlight = self.ScrollBox.SelectionHighlight, calls InitScrollBox, \
         and registers two EventRegistry static callbacks via AddStaticEventMethod \
         (AccountStore.StoreFrontSet → OnStoreFrontSet, AccountStore.CategorySelected → \
         OnCategorySelected)",
    ),
    (
        "InitScrollBox",
        "Blizzard_AccountStoreCategoryList.lua:29 — PLAN-omitted-but-present. Creates a \
         CreateScrollBoxListLinearView with element initializer that calls \
         button:SetCategory(elementData.categoryID), sets 16-px top padding, and binds the \
         view to ScrollBox + ScrollBar via ScrollUtil.InitScrollBoxListWithScrollBar. Adds \
         an Intrusive selection behavior whose callback hides SelectionHighlight on \
         deselect and calls SetRowSelectedState on select",
    ),
    (
        "OnStoreFrontSet",
        "Blizzard_AccountStoreCategoryList.lua:53 — PLAN-omitted-but-present. Handler for \
         the AccountStore.StoreFrontSet EventRegistry callback — fetches categories via \
         C_AccountStore.GetCategories(storeFrontID) and calls SetCategories. The public \
         entry point for `store changed, re-render the category column`",
    ),
    (
        "OnCategorySelected",
        "Blizzard_AccountStoreCategoryList.lua:57 — PLAN-omitted-but-present. Handler for \
         the AccountStore.CategorySelected EventRegistry callback — finds the row by \
         elementData.categoryID == categoryID predicate via \
         ScrollBox:FindFrameByPredicate and calls SetRowSelectedState(button). The \
         closest-name analogue to PLAN's `SelectCategoryByID`",
    ),
    (
        "SetCategories",
        "Blizzard_AccountStoreCategoryList.lua:67 — PLAN-omitted-but-present. Wraps the \
         categories list in CreateDataProviderWithAssignedKey(categories, \"categoryID\") \
         and feeds it to ScrollBox:SetDataProvider with RetainScrollPosition. Then \
         triggers AccountStore.CategorySelected for categories[1] (the \
         auto-select-first-category behavior). The closest-name analogue to PLAN's \
         `Refresh`",
    ),
    (
        "SetRowSelectedState",
        "Blizzard_AccountStoreCategoryList.lua:75 — PLAN-omitted-but-present. Positions \
         the SelectionHighlight overlay anchored CENTER to the row button (with y-offset \
         +2) and shows it. Called from the OnCategorySelected event handler and from the \
         ScrollBox selection-behavior callback wired in InitScrollBox",
    ),
];

const PLAN_NAMED_CATEGORY_LIST_METHODS_ABSENT: &[(&str, &str)] = &[
    (
        "Refresh",
        "no method by this name on AccountStoreCategoryListMixin. Verified by \
         `grep -rn \"function AccountStoreCategoryListMixin:Refresh\" \
         Interface/BlizzardUI/Blizzard_AccountStore/` returning zero matches. The closest \
         functional analogue is SetCategories \
         (Blizzard_AccountStoreCategoryList.lua:67), which rebuilds the ScrollBox data \
         provider from a categories list. PLAN's `Refresh` is also a near-miss for \
         OnStoreFrontSet (line 53), the `store changed, re-render` public entry point. \
         The list owns no public Refresh() — ScrollBox-level data refresh is implicit in \
         SetCategories via SetDataProvider with RetainScrollPosition",
    ),
    (
        "SelectCategoryByID",
        "no method by this name on AccountStoreCategoryListMixin. Verified by \
         `grep -rn SelectCategoryByID Interface/BlizzardUI/Blizzard_AccountStore/` \
         returning zero matches. The closest functional analogue is OnCategorySelected \
         (line 57), which finds the row by elementData.categoryID == categoryID predicate \
         and calls SetRowSelectedState(button). Selection-by-id is event-driven via the \
         AccountStore.CategorySelected EventRegistry trigger (fired by \
         AccountStoreCategoryMixin:OnClick at line 6 and by SetCategories for the \
         auto-select first-category case at line 72), not exposed as a public \
         SelectCategoryByID method",
    ),
];

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn account_store_category_list_mixin_methods_match_actual_source() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let mixin_type: String = env
            .eval(&format!("return type(_G[{CATEGORY_LIST_MIXIN_NAME:?}])"))
            .expect("AccountStoreCategoryListMixin global probe must run cleanly");

        assert_eq!(
            mixin_type, "table",
            "Expected `_G[{CATEGORY_LIST_MIXIN_NAME:?}]` to be a table after `{ROOT}` \
             loads, got `{mixin_type}`. The mixin is declared at \
             `Blizzard_AccountStoreCategoryList.lua:18` as \
             `AccountStoreCategoryListMixin = {{}}` and gets six methods attached. The \
             list XML template references the mixin via \
             `mixin=\"AccountStoreCategoryListMixin\"` on the virtual template, which \
             `AccountStoreFrame.CategoryList` (parentKey on the panel) inherits — so the \
             XML loader's mixin-resolution path must find the table by name in `_G` and \
             copy its methods onto the CategoryList frame instance. A nil reading means \
             either the Lua file did not execute (a regression in the addon load pipeline) \
             or the global was shadowed (a regression in the addon environment isolation)."
        );

        for (method_name, source_site) in ACTUAL_CATEGORY_LIST_METHODS {
            let method_type: String = env
                .eval(&format!(
                    "return type(_G[{CATEGORY_LIST_MIXIN_NAME:?}][{method_name:?}])"
                ))
                .unwrap_or_else(|error| {
                    panic!("failed to probe `AccountStoreCategoryListMixin.{method_name}`: {error}")
                });

            assert_eq!(
                method_type, "function",
                "Expected `AccountStoreCategoryListMixin.{method_name}` to be a function \
                 after `{ROOT}` loads ({source_site}), got `{method_type}`. The list \
                 mixin's six methods together form the entire category-column contract: \
                 OnLoad wires the ScrollBox + EventRegistry callbacks, InitScrollBox \
                 builds the linear view + selection behavior, OnStoreFrontSet is the \
                 store-change re-render entry point, OnCategorySelected is the \
                 selection-changed handler, SetCategories rebuilds the data provider + \
                 auto-selects the first row, and SetRowSelectedState positions the \
                 selection-highlight overlay. Losing any one would break a downstream \
                 store-front interaction — for example, dropping `SetCategories` would \
                 leave OnStoreFrontSet calling a nil method, so changing the active \
                 storeFrontID would silently fail to populate the category list."
            );
        }
    });
}
}

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn account_store_category_list_mixin_does_not_define_plan_named_methods_that_are_actually_absent() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for (absent_method, mismatch_reason) in PLAN_NAMED_CATEGORY_LIST_METHODS_ABSENT {
            let method_type: String = env
                .eval(&format!(
                    "return type(_G[{CATEGORY_LIST_MIXIN_NAME:?}][{absent_method:?}])"
                ))
                .unwrap_or_else(|error| {
                    panic!(
                        "failed to probe absence of \
                         `AccountStoreCategoryListMixin.{absent_method}`: {error}"
                    )
                });

            assert_eq!(
                method_type, "nil",
                "Expected `AccountStoreCategoryListMixin.{absent_method}` to be nil after \
                 `{ROOT}` loads (PLAN.md spec/source mismatch tripwire — \
                 {mismatch_reason}), got `{method_type}`. The PLAN.md task names \
                 `{absent_method}` as a method on `AccountStoreCategoryListMixin`, but \
                 `Blizzard_AccountStoreCategoryList.lua:18-78` declares the mixin as \
                 `AccountStoreCategoryListMixin = {{}}` and attaches exactly six methods \
                 (`OnLoad`, `InitScrollBox`, `OnStoreFrontSet`, `OnCategorySelected`, \
                 `SetCategories`, `SetRowSelectedState`) — none named `{absent_method}`. \
                 A non-nil reading here means either (a) Blizzard added the PLAN-named \
                 method (forcing a re-pin against the new contract — and likely renaming \
                 `SetCategories` → `Refresh` and `OnCategorySelected` → \
                 `SelectCategoryByID`), (b) some other addon monkey-patched the mixin \
                 (a layering violation worth investigating), or (c) the mixin's metatable \
                 started inheriting from a parent table that defines this method (a \
                 regression in the mixin's isolation)."
            );
        }
    });
}
}
