//! Mixin-method surface pin for `AccountStoreCategoryMixin` (per-row category button).
//!
//! Spec/source mismatch finding (PLAN.md task for
//! `AccountStoreCategoryMixin`): the plan names four methods —
//! `SetCategory`, `OnClick`, `OnEnter`, `OnLeave` — but
//! `Blizzard_AccountStoreCategoryList.lua:2-15` declares exactly
//! two methods on the mixin. Two of the four PLAN names match the
//! source verbatim; two (`OnEnter`, `OnLeave`) are genuinely absent
//! and have no closest-name analogue anywhere on the mixin: the
//! category button does NOT model hover state. Mouse-over highlight
//! comes from the parent `AccountStoreCategoryListMixin` selection
//! behavior (`SelectionHighlight` overlay positioned in
//! `SetRowSelectedState`, `Blizzard_AccountStoreCategoryList.lua:75`)
//! rather than from per-row OnEnter/OnLeave handlers; click is the
//! only direct user input the row owns. The two-line mixin body is
//! the entire row-button contract:
//!
//! | PLAN name        | Status                                                     |
//! |------------------|------------------------------------------------------------|
//! | `SetCategory`    | present (`Blizzard_AccountStoreCategoryList.lua:9`)        |
//! | `OnClick`        | present (`Blizzard_AccountStoreCategoryList.lua:4`)        |
//! | `OnEnter`        | absent — no method by this name on `AccountStoreCategoryMixin`. The mixin owns no per-row hover state. The only highlight visible is `AccountStoreCategoryListMixin.SelectionHighlight` (line 21, sourced from `self.ScrollBox.SelectionHighlight`), which is positioned in `SetRowSelectedState` (line 75) — driven by the parent list's selection-changed callback (line 50), not by per-row mouseover. |
//! | `OnLeave`        | absent — same reasoning as `OnEnter`: no per-row hover behavior on the mixin, the parent list owns the highlight. |
//!
//! No PLAN-omitted-but-present methods — the mixin's complete
//! source surface is just the two methods (`OnClick`, `SetCategory`).

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AccountStore";
const CATEGORY_MIXIN_NAME: &str = "AccountStoreCategoryMixin";

const ACTUAL_CATEGORY_METHODS: &[(&str, &str)] = &[
    (
        "OnClick",
        "Blizzard_AccountStoreCategoryList.lua:4 — plays \
         ACCOUNT_STORE_CATEGORY_SELECT sound and triggers EventRegistry callback \
         `AccountStore.CategorySelected` with self.categoryID. The trigger is consumed by \
         AccountStoreCategoryListMixin:OnCategorySelected (line 57, finds the row and calls \
         SetRowSelectedState) and AccountStoreItemDisplayMixin:OnCategorySelected (line 114, \
         switches currentItemRack and drives SetPage)",
    ),
    (
        "SetCategory",
        "Blizzard_AccountStoreCategoryList.lua:9 — stores categoryID on self, then fetches \
         categoryInfo via C_AccountStore.GetCategoryInfo and writes self.Text:SetText(name) + \
         self.Icon:SetTexture(icon). The single entry point invoked by the ScrollBox view \
         element-initializer (line 32: `button:SetCategory(elementData.categoryID)`)",
    ),
];

const PLAN_NAMED_CATEGORY_METHODS_ABSENT: &[(&str, &str)] = &[
    (
        "OnEnter",
        "no method by this name on AccountStoreCategoryMixin. The category row owns no \
         per-row hover state — the only highlight visible is \
         AccountStoreCategoryListMixin.SelectionHighlight (line 21, sourced from \
         self.ScrollBox.SelectionHighlight), positioned in SetRowSelectedState (line 75), \
         driven by the parent list's selection-changed callback (line 50). PLAN's `OnEnter` \
         has no closest-name analogue on the mixin",
    ),
    (
        "OnLeave",
        "no method by this name on AccountStoreCategoryMixin. Same reasoning as OnEnter: \
         no per-row hover behavior on the mixin, the parent list owns the highlight via the \
         SelectionHighlight overlay. PLAN's `OnLeave` has no closest-name analogue on the \
         mixin",
    ),
];

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn account_store_category_mixin_methods_match_actual_source() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let mixin_type: String = env
            .eval(&format!("return type(_G[{CATEGORY_MIXIN_NAME:?}])"))
            .expect("AccountStoreCategoryMixin global probe must run cleanly");

        assert_eq!(
            mixin_type, "table",
            "Expected `_G[{CATEGORY_MIXIN_NAME:?}]` to be a table after `{ROOT}` loads, got \
             `{mixin_type}`. The mixin is declared at \
             `Blizzard_AccountStoreCategoryList.lua:2` as `AccountStoreCategoryMixin = {{}}` \
             and immediately gets two methods attached (`OnClick` at line 4, `SetCategory` \
             at line 9). The XML category-row template references the mixin via \
             `mixin=\"AccountStoreCategoryMixin\"` so the ScrollBox view's element-pool \
             instantiation copies these methods onto every row-button frame. A nil reading \
             means either the Lua file did not execute (a regression in the addon load \
             pipeline) or the global was shadowed (a regression in the addon environment \
             isolation)."
        );

        for (method_name, source_site) in ACTUAL_CATEGORY_METHODS {
            let method_type: String = env
                .eval(&format!(
                    "return type(_G[{CATEGORY_MIXIN_NAME:?}][{method_name:?}])"
                ))
                .unwrap_or_else(|error| {
                    panic!("failed to probe `AccountStoreCategoryMixin.{method_name}`: {error}")
                });

            assert_eq!(
                method_type, "function",
                "Expected `AccountStoreCategoryMixin.{method_name}` to be a function after \
                 `{ROOT}` loads ({source_site}), got `{method_type}`. The category-row's two \
                 methods are the entire row-button contract: `SetCategory` is the \
                 element-initializer entry point that stamps the row with categoryID + \
                 displayed name + icon, `OnClick` is the only direct user input the row \
                 emits (firing the `AccountStore.CategorySelected` EventRegistry trigger). \
                 Losing `SetCategory` would surface a nil-method error in the ScrollBox \
                 view initializer (line 32) so every row would render blank. Losing \
                 `OnClick` would leave click events with no handler so category selection \
                 would silently break."
            );
        }
    });
}
}

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn account_store_category_mixin_does_not_define_plan_named_methods_that_are_actually_absent() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for (absent_method, mismatch_reason) in PLAN_NAMED_CATEGORY_METHODS_ABSENT {
            let method_type: String = env
                .eval(&format!(
                    "return type(_G[{CATEGORY_MIXIN_NAME:?}][{absent_method:?}])"
                ))
                .unwrap_or_else(|error| {
                    panic!(
                        "failed to probe absence of \
                         `AccountStoreCategoryMixin.{absent_method}`: {error}"
                    )
                });

            assert_eq!(
                method_type, "nil",
                "Expected `AccountStoreCategoryMixin.{absent_method}` to be nil after \
                 `{ROOT}` loads (PLAN.md spec/source mismatch tripwire — {mismatch_reason}), \
                 got `{method_type}`. The PLAN.md task names `{absent_method}` as a method \
                 on `AccountStoreCategoryMixin`, but \
                 `Blizzard_AccountStoreCategoryList.lua:2-15` declares the mixin as \
                 `AccountStoreCategoryMixin = {{}}` and attaches exactly two methods \
                 (`OnClick`, `SetCategory`) — none named `{absent_method}`. A non-nil \
                 reading here means either (a) Blizzard added per-row hover handlers \
                 (forcing a re-pin against the new hover-highlight contract — and likely \
                 retiring the parent's SelectionHighlight overlay), (b) some other addon \
                 monkey-patched the mixin (a layering violation worth investigating), or \
                 (c) the mixin's metatable started inheriting from a parent table that \
                 defines this method (a regression in the mixin's isolation)."
            );
        }
    });
}
}
