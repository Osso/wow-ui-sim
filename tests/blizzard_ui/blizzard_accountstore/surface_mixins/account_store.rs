//! Mixin-method surface pin for `AccountStoreMixin` (panel root).
//!
//! Spec/source mismatch finding (PLAN.md task for `AccountStoreMixin`):
//! the plan names eight methods —
//! `OnLoad`, `OnShow`, `OnHide`, `OnEvent`,
//! `SetStoreFrontID`, `OnStoreFrontSet`,
//! `CategorySelected`, `RefreshCategories` — but
//! `Blizzard_AccountStore.lua` defines exactly five methods on
//! `AccountStoreMixin`: `OnLoad` (line 18), `OnShow` (line 26),
//! `OnHide` (line 31), `SetStoreFrontID` (line 42), and
//! `SetFullscreenMode` (line 50). Four PLAN-named methods are
//! genuinely absent from this mixin, and one method actually present
//! (`SetFullscreenMode`) was omitted from the PLAN list:
//!
//! | PLAN name           | Status                                               |
//! |---------------------|------------------------------------------------------|
//! | `OnLoad`            | present (`Blizzard_AccountStore.lua:18`)             |
//! | `OnShow`            | present (`Blizzard_AccountStore.lua:26`)             |
//! | `OnHide`            | present (`Blizzard_AccountStore.lua:31`)             |
//! | `OnEvent`           | absent — `OnLoad` never calls `RegisterEvent`, so no script-handler `OnEvent` body is needed. |
//! | `SetStoreFrontID`   | present (`Blizzard_AccountStore.lua:42`)             |
//! | `OnStoreFrontSet`   | absent on `AccountStoreMixin` — lives on `AccountStoreCategoryListMixin` (`Blizzard_AccountStoreCategoryList.lua:53`) and `AccountStoreItemDisplayMixin` (`Blizzard_AccountStoreItemDisplay.lua:104`) as the `EventRegistry`-callback handler for `"AccountStore.StoreFrontSet"`. The `AccountStoreMixin:SetStoreFrontID` body fires that event (line 47), but the handler that consumes it lives on the two child-frame mixins, not on `AccountStoreMixin`. |
//! | `CategorySelected`  | absent — no method by this name on any mixin. There IS a custom `EventRegistry` event named `"AccountStore.CategorySelected"` (triggered at `Blizzard_AccountStoreCategoryList.lua:6`, `:72`), and the handler method on the consumer mixins is `OnCategorySelected` (note the `On` prefix), defined on `AccountStoreCategoryListMixin` (`:57`) and `AccountStoreItemDisplayMixin` (`:114`). The closest-name match — `OnCategorySelected` — is also absent on `AccountStoreMixin`. |
//! | `RefreshCategories` | absent — not defined anywhere in the `Blizzard_AccountStore` lane. The closest functional analogue is `AccountStoreCategoryListMixin:SetCategories` (`Blizzard_AccountStoreCategoryList.lua:67`), which rebuilds the ScrollBox data provider from a categories list. That lives on the child category-list mixin, not on `AccountStoreMixin`. |
//!
//! Only PLAN-omitted method actually present:
//!
//! | Method               | Source site                              | Role |
//! |----------------------|------------------------------------------|------|
//! | `SetFullscreenMode`  | `Blizzard_AccountStore.lua:50`           | Reparents `AccountStoreFrame` between `UIParent` and `FullscreenAccountStoreContainer` for WoW Labs / Plunderstorm fullscreen mode; toggles the container visibility and re-anchors the panel (CENTER when fullscreen, LEFT-50 otherwise). |

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AccountStore";
const MIXIN_NAME: &str = "AccountStoreMixin";

const ACTUAL_MIXIN_METHODS: &[(&str, &str)] = &[
    (
        "OnLoad",
        "Blizzard_AccountStore.lua:18 — sets portrait icon and registers `AccountStoreFrame` in \
         UIPanelWindows for left-area pushable behavior",
    ),
    (
        "OnShow",
        "Blizzard_AccountStore.lua:26 — plays ACCOUNT_STORE_OPEN sound and triggers \
         EventRegistry callback `AccountStore.ShownState` with true",
    ),
    (
        "OnHide",
        "Blizzard_AccountStore.lua:31 — plays ACCOUNT_STORE_CLOSE sound, triggers \
         EventRegistry callback `AccountStore.ShownState` with false, closes static popups, \
         and exits fullscreen mode if active",
    ),
    (
        "SetStoreFrontID",
        "Blizzard_AccountStore.lua:42 — stores the storeFrontID, sets the panel title from \
         the STORE_FRONT_TO_TITLE table (WOWHACK_ACCOUNT_STORE_TITLE / \
         PLUNDERSTORM_PLUNDER_STORE_TITLE), and triggers EventRegistry callback \
         `AccountStore.StoreFrontSet` with the id",
    ),
    (
        "SetFullscreenMode",
        "Blizzard_AccountStore.lua:50 — PLAN-omitted-but-present. Reparents AccountStoreFrame \
         between UIParent and FullscreenAccountStoreContainer for WoW Labs / Plunderstorm \
         fullscreen mode; toggles the container visibility and re-anchors the panel (CENTER \
         when fullscreen, LEFT x=50 otherwise)",
    ),
];

const PLAN_NAMED_METHODS_ABSENT: &[(&str, &str)] = &[
    (
        "OnEvent",
        "no method by this name on AccountStoreMixin. AccountStoreMixin:OnLoad never calls \
         RegisterEvent (Blizzard_AccountStore.lua:18-24 only does SetPortraitToAsset and \
         UIPanelWindows write), so the panel does not need an OnEvent script-handler body",
    ),
    (
        "OnStoreFrontSet",
        "lives on AccountStoreCategoryListMixin (Blizzard_AccountStoreCategoryList.lua:53) and \
         AccountStoreItemDisplayMixin (Blizzard_AccountStoreItemDisplay.lua:104) as the \
         EventRegistry-callback handler for the `AccountStore.StoreFrontSet` event that \
         AccountStoreMixin:SetStoreFrontID fires (line 47). The trigger lives on \
         AccountStoreMixin; the handler does not",
    ),
    (
        "CategorySelected",
        "no method by this name on any mixin. The custom EventRegistry event named \
         `AccountStore.CategorySelected` is triggered by AccountStoreCategoryMixin:OnClick \
         (Blizzard_AccountStoreCategoryList.lua:6) and SetCategories (line 72); the handler \
         method on consumer mixins is `OnCategorySelected` (with `On` prefix) defined on \
         AccountStoreCategoryListMixin (line 57) and AccountStoreItemDisplayMixin (line 114). \
         The closest-name match is also absent on AccountStoreMixin",
    ),
    (
        "RefreshCategories",
        "not defined anywhere in the Blizzard_AccountStore lane. The closest functional \
         analogue is AccountStoreCategoryListMixin:SetCategories \
         (Blizzard_AccountStoreCategoryList.lua:67), which rebuilds the ScrollBox data \
         provider from a categories list — but it lives on the child category-list mixin, \
         not on AccountStoreMixin",
    ),
];

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn account_store_mixin_methods_match_actual_source() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let mixin_type: String = env
            .eval(&format!("return type(_G[{MIXIN_NAME:?}])"))
            .expect("AccountStoreMixin global probe must run cleanly");

        assert_eq!(
            mixin_type, "table",
            "Expected `_G[{MIXIN_NAME:?}]` to be a table after `{ROOT}` loads, got \
             `{mixin_type}`. The mixin is declared at `Blizzard_AccountStore.lua:16` as \
             `AccountStoreMixin = {{}}` and immediately gets five methods attached \
             (`OnLoad`, `OnShow`, `OnHide`, `SetStoreFrontID`, `SetFullscreenMode`). The \
             panel XML at `Blizzard_AccountStore.xml:78` references the mixin via \
             `mixin=\"AccountStoreMixin\"`, so the XML loader's mixin-resolution path must \
             find the table by name in `_G` and copy its methods onto the frame instance. \
             A nil reading means either the Lua file did not execute (a regression in the \
             addon load pipeline) or the global was shadowed (a regression in the addon \
             environment isolation)."
        );

        for (method_name, source_site) in ACTUAL_MIXIN_METHODS {
            let method_type: String = env
                .eval(&format!("return type(_G[{MIXIN_NAME:?}][{method_name:?}])"))
                .unwrap_or_else(|error| {
                    panic!("failed to probe `AccountStoreMixin.{method_name}`: {error}")
                });

            assert_eq!(
                method_type, "function",
                "Expected `AccountStoreMixin.{method_name}` to be a function after `{ROOT}` \
                 loads ({source_site}), got `{method_type}`. A nil reading means either the \
                 method definition was dropped from the Lua source or never reached because \
                 of an earlier syntax error. A non-function reading means a different value \
                 type was assigned to the same key (likely a regression from an inline table \
                 literal). Either way, every consumer that calls \
                 `AccountStoreMixin:{method_name}` or invokes it via the panel's mixin chain \
                 (`AccountStoreFrame:{method_name}`) would surface a missing-method error."
            );
        }
    });
}
}

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn account_store_mixin_does_not_define_plan_named_methods_that_are_actually_absent() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for (absent_method, mismatch_reason) in PLAN_NAMED_METHODS_ABSENT {
            let method_type: String = env
                .eval(&format!(
                    "return type(_G[{MIXIN_NAME:?}][{absent_method:?}])"
                ))
                .unwrap_or_else(|error| {
                    panic!(
                        "failed to probe absence of `AccountStoreMixin.{absent_method}`: {error}"
                    )
                });

            assert_eq!(
                method_type, "nil",
                "Expected `AccountStoreMixin.{absent_method}` to be nil after `{ROOT}` loads \
                 (PLAN.md spec/source mismatch tripwire — {mismatch_reason}), got \
                 `{method_type}`. The PLAN.md task names `{absent_method}` as a method on \
                 `AccountStoreMixin`, but `Blizzard_AccountStore.lua:16-64` declares the \
                 mixin as `AccountStoreMixin = {{}}` and attaches exactly five methods: \
                 `OnLoad`, `OnShow`, `OnHide`, `SetStoreFrontID`, `SetFullscreenMode`. A \
                 non-nil reading here means either (a) Blizzard added the PLAN-named method \
                 to `AccountStoreMixin` (forcing a re-pin against the new contract), (b) \
                 some other addon monkey-patched the mixin (which would be a layering \
                 violation worth investigating), or (c) the mixin's metatable started \
                 inheriting from a parent table that defines this method (a regression in \
                 the mixin's isolation). The closest-name substitutes for the four absent \
                 PLAN names live on the child-frame mixins \
                 (`AccountStoreCategoryListMixin` and `AccountStoreItemDisplayMixin`), not \
                 on `AccountStoreMixin` itself."
            );
        }
    });
}
}
