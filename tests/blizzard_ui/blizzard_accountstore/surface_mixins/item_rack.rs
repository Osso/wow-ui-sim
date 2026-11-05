//! Mixin-method surface pin for `AccountStoreItemRackMixin` (per-category card pool).
//!
//! Spec/source mismatch finding (PLAN.md task for
//! `AccountStoreItemRackMixin`): the plan names three methods —
//! `Refresh`, `SelectCardByItemID`, `GetSelectedItemID` — but
//! `Blizzard_AccountStoreItemRack.lua:18-56` declares four methods on
//! the rack mixin. One of the three PLAN names matches the source
//! verbatim (`Refresh`); the other two
//! (`SelectCardByItemID`, `GetSelectedItemID`) are absent — the rack
//! mixin does NOT model per-card selection state at all. Both
//! "selection" PLAN names returned zero matches under
//! `grep -rn SelectCardByItemID Interface/BlizzardUI/Blizzard_AccountStore/`
//! and the same for `GetSelectedItemID` and `selectedItemID`. The
//! rack just owns a card-frame pool that gets released and re-acquired
//! on every `Refresh()`. Per-card selection lives entirely outside
//! this mixin (the page-level state tracking is on the parent
//! `AccountStoreItemDisplayMixin` via `categoryLastPage` / `categoryID`
//! / `currentPage`; per-card buy / refund click handling lives on
//! `AccountStoreBaseCardMixin:SelectCard` via the BuyButton OnClick
//! handler — neither stores a "selected card by item id" pointer).
//! Two of the rack's four methods (`SetCategoryType`, `SetItems`) and
//! one helper (`GetMaxCards`) were PLAN-omitted:
//!
//! | PLAN name             | Status                                                     |
//! |-----------------------|------------------------------------------------------------|
//! | `Refresh`             | present (`Blizzard_AccountStoreItemRack.lua:31`)           |
//! | `SelectCardByItemID`  | absent — no method by this name on `AccountStoreItemRackMixin` and zero matches anywhere in the `Blizzard_AccountStore` lane. The rack owns no selection state — it's a stateless pooled grid layout that re-renders the entire card list on every `Refresh`. |
//! | `GetSelectedItemID`   | absent — no method by this name on `AccountStoreItemRackMixin` and zero matches anywhere in the lane. Same reasoning as `SelectCardByItemID`: the rack owns no per-card selection state. |
//!
//! Architecturally-critical PLAN-omitted-but-present methods (the
//! three the surface test pins as guards against silent removal):
//!
//! | Method                | Source site                                                  | Role |
//! |-----------------------|--------------------------------------------------------------|------|
//! | `SetCategoryType`     | `Blizzard_AccountStoreItemRack.lua:20`                       | Looks up the category-type → info table (`AccountStoreCategoryToInfo` at lines 10-15: Creature/4, TransmogSet/2, Mount/1, Icon/4) and creates a `BUTTON` framepool from `categoryInfo.cardTemplate` (one of the four `AccountStoreCreatureCardTemplate` / `AccountStoreTransmogSetCardTemplate` / `AccountStoreMountCardTemplate` / `AccountStoreIconCardTemplate` virtual templates). Stores `self.maxCards` from the same info entry. Called via `AccountStoreItemDisplayMixin:CreateItemRack` (`Blizzard_AccountStoreItemDisplay.lua:139`). |
//! | `SetItems`            | `Blizzard_AccountStoreItemRack.lua:26`                       | Stores the items list on `self.items` and immediately calls `self:Refresh()`. The single entry point `AccountStoreItemDisplayMixin:SetPage` calls when paging through cards (`Blizzard_AccountStoreItemDisplay.lua:170`). |
//! | `GetMaxCards`         | `Blizzard_AccountStoreItemRack.lua:54`                       | Returns `self.maxCards` (the per-category-type cap from `AccountStoreCategoryToInfo`). Used by `AccountStoreItemDisplayMixin:GetMaxPage` for ceiling math (`Blizzard_AccountStoreItemDisplay.lua:147`) and by `AccountStoreItemDisplayMixin:SetPage` for items-per-page slicing (line 162). |

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AccountStore";
const ITEM_RACK_MIXIN_NAME: &str = "AccountStoreItemRackMixin";

const ACTUAL_ITEM_RACK_METHODS: &[(&str, &str)] = &[
    (
        "Refresh",
        "Blizzard_AccountStoreItemRack.lua:31 — releases all card-pool entries, then walks \
         the items list and acquires fresh card frames from cardPool, calling SetItemID on \
         each. Lays them out via AnchorUtil.GridLayoutFactoryByCount with stride=2 + \
         padding=5. The TOP vs TOPLEFT initial anchor flips when maxCards == 1 (Mount \
         category) so the single mount card centers horizontally",
    ),
    (
        "SetCategoryType",
        "Blizzard_AccountStoreItemRack.lua:20 — PLAN-omitted-but-present. Looks up the \
         category-type → info table (AccountStoreCategoryToInfo at lines 10-15: Creature/4, \
         TransmogSet/2, Mount/1, Icon/4) and creates a BUTTON framepool from \
         categoryInfo.cardTemplate (one of the four virtual card templates). Stores \
         self.maxCards. Called via AccountStoreItemDisplayMixin:CreateItemRack",
    ),
    (
        "SetItems",
        "Blizzard_AccountStoreItemRack.lua:26 — PLAN-omitted-but-present. Stores the items \
         list on self.items and immediately calls self:Refresh(). The single entry point \
         AccountStoreItemDisplayMixin:SetPage calls when paging through cards \
         (Blizzard_AccountStoreItemDisplay.lua:170)",
    ),
    (
        "GetMaxCards",
        "Blizzard_AccountStoreItemRack.lua:54 — PLAN-omitted-but-present. Returns \
         self.maxCards (the per-category-type cap from AccountStoreCategoryToInfo). Used by \
         AccountStoreItemDisplayMixin:GetMaxPage for ceiling math \
         (Blizzard_AccountStoreItemDisplay.lua:147) and by SetPage for items-per-page \
         slicing (line 162)",
    ),
];

const PLAN_NAMED_ITEM_RACK_METHODS_ABSENT: &[(&str, &str)] = &[
    (
        "SelectCardByItemID",
        "no method by this name on AccountStoreItemRackMixin and zero matches under \
         `grep -rn SelectCardByItemID Interface/BlizzardUI/Blizzard_AccountStore/`. The rack \
         owns no per-card selection state — it's a stateless pooled grid layout that \
         re-renders the entire card list on every Refresh. Per-card buy / refund click \
         handling lives on AccountStoreBaseCardMixin:SelectCard via BuyButton OnClick, not \
         on the rack",
    ),
    (
        "GetSelectedItemID",
        "no method by this name on AccountStoreItemRackMixin and zero matches under \
         `grep -rn GetSelectedItemID Interface/BlizzardUI/Blizzard_AccountStore/`. Same \
         reasoning as SelectCardByItemID: the rack owns no per-card selection state. The \
         page-level state tracking lives on AccountStoreItemDisplayMixin via \
         categoryLastPage / categoryID / currentPage — not as a `selectedItemID` pointer",
    ),
];

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn account_store_item_rack_mixin_methods_match_actual_source() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let mixin_type: String = env
            .eval(&format!("return type(_G[{ITEM_RACK_MIXIN_NAME:?}])"))
            .expect("AccountStoreItemRackMixin global probe must run cleanly");

        assert_eq!(
            mixin_type, "table",
            "Expected `_G[{ITEM_RACK_MIXIN_NAME:?}]` to be a table after `{ROOT}` loads, got \
             `{mixin_type}`. The mixin is declared at `Blizzard_AccountStoreItemRack.lua:18` \
             as `AccountStoreItemRackMixin = {{}}` and gets four methods attached. The \
             item-rack XML template at `Blizzard_AccountStoreItemRack.xml` references the \
             mixin via `mixin=\"AccountStoreItemRackMixin\"` on the virtual template, which \
             `AccountStoreItemDisplayMixin:CreateItemRack` instantiates per category type \
             (`Blizzard_AccountStoreItemDisplay.lua:138`: \
             `CreateFrame(\"Frame\", nil, self, \"AccountStoreItemRackTemplate\")`). A nil \
             reading means either the Lua file did not execute (a regression in the addon \
             load pipeline) or the global was shadowed (a regression in the addon \
             environment isolation)."
        );

        for (method_name, source_site) in ACTUAL_ITEM_RACK_METHODS {
            let method_type: String = env
                .eval(&format!(
                    "return type(_G[{ITEM_RACK_MIXIN_NAME:?}][{method_name:?}])"
                ))
                .unwrap_or_else(|error| {
                    panic!("failed to probe `AccountStoreItemRackMixin.{method_name}`: {error}")
                });

            assert_eq!(
                method_type, "function",
                "Expected `AccountStoreItemRackMixin.{method_name}` to be a function after \
                 `{ROOT}` loads ({source_site}), got `{method_type}`. The rack's four \
                 methods together form the entire stateless-pool contract: \
                 `SetCategoryType` creates the per-category framepool, `SetItems` updates \
                 the items list and triggers re-render, `Refresh` releases + re-acquires \
                 the visible cards via grid layout, `GetMaxCards` exposes the maxCards cap \
                 for the parent's paging math. Losing any one would break a downstream \
                 paging interaction — for example, dropping `GetMaxCards` would make \
                 `AccountStoreItemDisplayMixin:GetMaxPage` surface a nil-method error, so \
                 SetPage's clamp call would fail and paging would silently break."
            );
        }
    });
}
}

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn account_store_item_rack_mixin_does_not_define_plan_named_methods_that_are_actually_absent() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for (absent_method, mismatch_reason) in PLAN_NAMED_ITEM_RACK_METHODS_ABSENT {
            let method_type: String = env
                .eval(&format!(
                    "return type(_G[{ITEM_RACK_MIXIN_NAME:?}][{absent_method:?}])"
                ))
                .unwrap_or_else(|error| {
                    panic!(
                        "failed to probe absence of \
                         `AccountStoreItemRackMixin.{absent_method}`: {error}"
                    )
                });

            assert_eq!(
                method_type, "nil",
                "Expected `AccountStoreItemRackMixin.{absent_method}` to be nil after \
                 `{ROOT}` loads (PLAN.md spec/source mismatch tripwire — {mismatch_reason}), \
                 got `{method_type}`. The PLAN.md task names `{absent_method}` as a method \
                 on `AccountStoreItemRackMixin`, but \
                 `Blizzard_AccountStoreItemRack.lua:18-56` declares the mixin as \
                 `AccountStoreItemRackMixin = {{}}` and attaches exactly four methods \
                 (SetCategoryType, SetItems, Refresh, GetMaxCards) — none named \
                 `{absent_method}`. The PLAN spec implies the rack tracks per-card \
                 selection state, but the rack source models no selection at all. A \
                 non-nil reading means either (a) Blizzard added rack-level selection \
                 state (forcing a re-pin against the new selection-tracking contract), \
                 (b) some other addon monkey-patched the mixin (a layering violation \
                 worth investigating), or (c) the mixin's metatable started inheriting \
                 from a parent table that defines this method (a regression in the \
                 mixin's isolation)."
            );
        }
    });
}
}
