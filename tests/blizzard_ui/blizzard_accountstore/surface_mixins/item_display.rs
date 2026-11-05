//! Mixin-method surface pin for `AccountStoreItemDisplayMixin` (StoreDisplay panel).
//!
//! Spec/source mismatch finding (PLAN.md task for
//! `AccountStoreItemDisplayMixin`): the plan names six methods —
//! `OnLoad`, `OnShow`, `OnHide`, `OnCategorySelected`,
//! `RefreshSelectedCard`, `SetPage` — but
//! `Blizzard_AccountStoreItemDisplay.lua:2-184` declares twelve
//! methods on the item-display mixin. Five of the six PLAN names
//! match the source verbatim; one (`RefreshSelectedCard`) is absent.
//! The mixin's frame-event dispatch (`OnEvent`), the EventRegistry
//! callback the panel mixin's `SetStoreFrontID` triggers
//! (`OnStoreFrontSet`), the mousewheel paging chain
//! (`OnMouseWheel`), the per-storeFrontID state initializer
//! (`InitializeStore`), and the page-clamping helpers (`GetMaxPage`,
//! `UpdateCurrencyAvailable`, `CreateItemRack`) were all
//! PLAN-omitted:
//!
//! | PLAN name             | Status                                                     |
//! |-----------------------|------------------------------------------------------------|
//! | `OnLoad`              | present (`Blizzard_AccountStoreItemDisplay.lua:32`)        |
//! | `OnShow`              | present (`Blizzard_AccountStoreItemDisplay.lua:62`)        |
//! | `OnHide`              | present (`Blizzard_AccountStoreItemDisplay.lua:70`)        |
//! | `OnCategorySelected`  | present (`Blizzard_AccountStoreItemDisplay.lua:114`)       |
//! | `RefreshSelectedCard` | absent — no method by this name. The closest functional analogue is `UpdateCurrencyAvailable` (`Blizzard_AccountStoreItemDisplay.lua:177`), which both refreshes the currency-available display and calls `self.currentItemRack:Refresh()` to re-render the visible cards (the `:Refresh()` call is the only "refresh selected card" semantic this mixin owns). PLAN's `RefreshSelectedCard` is a near-miss for either `UpdateCurrencyAvailable` or `SetPage` with `forceUpdate=true` (line 150), which re-runs the item-rack `:SetItems` chain. |
//! | `SetPage`             | present (`Blizzard_AccountStoreItemDisplay.lua:150`)       |
//!
//! Architecturally-critical PLAN-omitted-but-present methods (the
//! six the surface test pins as guards against silent removal):
//!
//! | Method                    | Source site                                                 | Role |
//! |---------------------------|-------------------------------------------------------------|------|
//! | `InitializeStore`         | `Blizzard_AccountStoreItemDisplay.lua:10`                   | Resets `categoryLastPage` / `currentPage` / `storeFrontID` on first load or store change, hides existing item-rack frames, and re-seeds `categoryTypeToItemRack` if missing. Called from `OnLoad` and from `OnStoreFrontSet`. |
//! | `OnEvent`                 | `Blizzard_AccountStoreItemDisplay.lua:80`                   | Dispatches the three registered frame events: `ACCOUNT_STORE_CURRENCY_AVAILABLE_UPDATED` re-runs `UpdateCurrencyAvailable` when the currencyID matches; `ACCOUNT_STORE_FRONT_UPDATED` flips `areItemsAvailable` and re-runs `OnCategorySelected` with `forceUpdate=true`; `ACCOUNT_STORE_TRANSACTION_ERROR` shows the static popup. Without this method the registered events would fire with no handler. |
//! | `OnMouseWheel`            | `Blizzard_AccountStoreItemDisplay.lua:100`                  | Drives the paging chain by calling `SetPage(currentPage ± 1)` based on wheel direction. Wired by the `AccountStoreBaseCardMixin:OnMouseWheel` ancestor-forwarding chain (`CallMethodOnNearestAncestor`). |
//! | `OnStoreFrontSet`         | `Blizzard_AccountStoreItemDisplay.lua:104`                  | Handler for the `"AccountStore.StoreFrontSet"` EventRegistry callback (registered in `OnLoad` line 58 via `AddStaticEventMethod`). Calls `InitializeStore`, then `C_AccountStore.RequestStoreFrontInfoUpdate` and seeds `currencyID` from `C_AccountStore.GetCurrencyIDForStore`. |
//! | `GetMaxPage`              | `Blizzard_AccountStoreItemDisplay.lua:146`                  | Returns `ceil(#categoryItems / currentItemRack:GetMaxCards())` — the upper bound `SetPage` clamps against. |
//! | `UpdateCurrencyAvailable` | `Blizzard_AccountStoreItemDisplay.lua:177`                  | Sets the Footer.CurrencyAvailable text via `AccountStoreUtil.FormatCurrencyDisplayWithWarning(currencyID)` and calls `currentItemRack:Refresh()` — the closest-name analogue to PLAN's `RefreshSelectedCard`. |
//! | `CreateItemRack`          | `Blizzard_AccountStoreItemDisplay.lua:137`                  | Lazy-creates an `AccountStoreItemRackTemplate` frame for a category type, anchors it edge-to-edge inside the StoreDisplay, and stores it in `categoryTypeToItemRack[categoryType]`. Called via `GenerateClosure` in `OnCategorySelected`. |

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AccountStore";
const ITEM_DISPLAY_MIXIN_NAME: &str = "AccountStoreItemDisplayMixin";

const ACTUAL_ITEM_DISPLAY_METHODS: &[(&str, &str)] = &[
    (
        "OnLoad",
        "Blizzard_AccountStoreItemDisplay.lua:32 — calls InitializeStore (no storeFrontID), \
         wires Footer.PrevPageButton / NextPageButton OnClick handlers to the SetPage chain \
         with ACCOUNT_STORE_PAGE_NAVIGATION sound, wires Footer.CurrencyAvailable OnEnter / \
         OnLeave to the currency-total tooltip, and registers the two EventRegistry static \
         callbacks (`AccountStore.StoreFrontSet` → OnStoreFrontSet, \
         `AccountStore.CategorySelected` → OnCategorySelected)",
    ),
    (
        "OnShow",
        "Blizzard_AccountStoreItemDisplay.lua:62 — delegates to CallbackRegistrantMixin.OnShow \
         (replays dynamic registrants), resets areItemsAvailable to false, then calls \
         FrameUtil.RegisterFrameForEvents for AccountStoreItemDisplayEvents \
         (ACCOUNT_STORE_CURRENCY_AVAILABLE_UPDATED + ACCOUNT_STORE_FRONT_UPDATED + \
         ACCOUNT_STORE_TRANSACTION_ERROR)",
    ),
    (
        "OnHide",
        "Blizzard_AccountStoreItemDisplay.lua:70 — delegates to CallbackRegistrantMixin.OnHide \
         (clears dynamic registrants), closes static popups via \
         AccountStoreUtil.CloseStaticPopups, clears the currentItemRack items list, and calls \
         FrameUtil.UnregisterFrameForEvents for the same three-event list",
    ),
    (
        "OnCategorySelected",
        "Blizzard_AccountStoreItemDisplay.lua:114 — handler for the \
         `AccountStore.CategorySelected` EventRegistry callback. Updates self.categoryID + \
         self.categoryItems via C_AccountStore.GetCategoryItems, lazy-creates an itemRack \
         frame for the category type via GetOrCreateTableEntryByCallback + GenerateClosure, \
         hides the previous itemRack, switches currentItemRack, and drives SetPage with the \
         remembered page from categoryLastPage",
    ),
    (
        "SetPage",
        "Blizzard_AccountStoreItemDisplay.lua:150 — clamps page against GetMaxPage, populates \
         the items array (page * maxCardsPerPage entries from categoryItems), calls \
         currentItemRack:SetItems, and updates Footer.PrevPageButton / NextPageButton enabled \
         state + Footer.PageText format string. The forceUpdate flag bypasses the no-op early \
         return when page hasn't changed",
    ),
    (
        "InitializeStore",
        "Blizzard_AccountStoreItemDisplay.lua:10 — PLAN-omitted-but-present. Resets \
         categoryLastPage / currentPage / storeFrontID on first load or store change, hides \
         existing itemRack frames, and re-seeds categoryTypeToItemRack if missing. Called \
         from OnLoad and OnStoreFrontSet",
    ),
    (
        "OnEvent",
        "Blizzard_AccountStoreItemDisplay.lua:80 — PLAN-omitted-but-present. Dispatches the \
         three registered frame events: ACCOUNT_STORE_CURRENCY_AVAILABLE_UPDATED re-runs \
         UpdateCurrencyAvailable when the currencyID matches, ACCOUNT_STORE_FRONT_UPDATED \
         flips areItemsAvailable + re-runs OnCategorySelected with forceUpdate=true, \
         ACCOUNT_STORE_TRANSACTION_ERROR shows the static popup",
    ),
    (
        "OnMouseWheel",
        "Blizzard_AccountStoreItemDisplay.lua:100 — PLAN-omitted-but-present. Drives the \
         paging chain by calling SetPage(currentPage ± 1) based on wheel direction. Wired by \
         the AccountStoreBaseCardMixin:OnMouseWheel ancestor-forwarding chain",
    ),
    (
        "OnStoreFrontSet",
        "Blizzard_AccountStoreItemDisplay.lua:104 — PLAN-omitted-but-present. Handler for \
         the `AccountStore.StoreFrontSet` EventRegistry callback. Calls InitializeStore, then \
         C_AccountStore.RequestStoreFrontInfoUpdate and seeds currencyID from \
         C_AccountStore.GetCurrencyIDForStore. Without this method, AccountStoreMixin's \
         SetStoreFrontID would fire the EventRegistry trigger and nothing would re-initialize \
         the per-store category state",
    ),
    (
        "GetMaxPage",
        "Blizzard_AccountStoreItemDisplay.lua:146 — PLAN-omitted-but-present. Returns \
         ceil(#categoryItems / currentItemRack:GetMaxCards()) — the upper bound SetPage \
         clamps against",
    ),
    (
        "UpdateCurrencyAvailable",
        "Blizzard_AccountStoreItemDisplay.lua:177 — PLAN-omitted-but-present. Sets the \
         Footer.CurrencyAvailable text via AccountStoreUtil.FormatCurrencyDisplayWithWarning \
         and calls currentItemRack:Refresh() — the closest-name analogue to PLAN's \
         `RefreshSelectedCard`",
    ),
    (
        "CreateItemRack",
        "Blizzard_AccountStoreItemDisplay.lua:137 — PLAN-omitted-but-present. Lazy-creates \
         an AccountStoreItemRackTemplate frame for a category type, anchors it edge-to-edge \
         inside the StoreDisplay, and stores it in categoryTypeToItemRack[categoryType]. \
         Called via GenerateClosure in OnCategorySelected",
    ),
];

const PLAN_NAMED_ITEM_DISPLAY_METHODS_ABSENT: &[(&str, &str)] = &[(
    "RefreshSelectedCard",
    "no method by this name on AccountStoreItemDisplayMixin. Verified absence by \
         `grep -rn RefreshSelectedCard Interface/BlizzardUI/Blizzard_AccountStore/` returning \
         zero matches. The closest functional analogue is UpdateCurrencyAvailable \
         (Blizzard_AccountStoreItemDisplay.lua:177), which calls currentItemRack:Refresh() — \
         the only `:Refresh()` semantic this mixin owns. SetPage with forceUpdate=true \
         (line 150) is the other near-miss: it re-runs the item-rack :SetItems chain to \
         force-render the currently-selected card list",
)];

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn account_store_item_display_mixin_methods_match_actual_source() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let mixin_type: String = env
            .eval(&format!("return type(_G[{ITEM_DISPLAY_MIXIN_NAME:?}])"))
            .expect("AccountStoreItemDisplayMixin global probe must run cleanly");

        assert_eq!(
            mixin_type, "table",
            "Expected `_G[{ITEM_DISPLAY_MIXIN_NAME:?}]` to be a table after `{ROOT}` loads, \
             got `{mixin_type}`. The mixin is declared at \
             `Blizzard_AccountStoreItemDisplay.lua:2` as \
             `AccountStoreItemDisplayMixin = {{}}` and gets twelve methods attached. The \
             item-display XML template at `Blizzard_AccountStoreItemDisplay.xml` references \
             the mixin via `mixin=\"AccountStoreItemDisplayMixin\"` on the virtual template, \
             which `AccountStoreFrame.StoreDisplay` (parentKey at \
             `Blizzard_AccountStore.xml:135`) inherits — so the XML loader's mixin-resolution \
             path must find the table by name in `_G` and copy its methods onto the \
             StoreDisplay frame instance. A nil reading means either the Lua file did not \
             execute (a regression in the addon load pipeline) or the global was shadowed \
             (a regression in the addon environment isolation)."
        );

        for (method_name, source_site) in ACTUAL_ITEM_DISPLAY_METHODS {
            let method_type: String = env
                .eval(&format!(
                    "return type(_G[{ITEM_DISPLAY_MIXIN_NAME:?}][{method_name:?}])"
                ))
                .unwrap_or_else(|error| {
                    panic!("failed to probe `AccountStoreItemDisplayMixin.{method_name}`: {error}")
                });

            assert_eq!(
                method_type, "function",
                "Expected `AccountStoreItemDisplayMixin.{method_name}` to be a function after \
                 `{ROOT}` loads ({source_site}), got `{method_type}`. The item-display \
                 mixin's lifecycle (OnLoad/OnShow/OnHide), frame-event dispatch (OnEvent), \
                 EventRegistry callback handlers (OnStoreFrontSet, OnCategorySelected), \
                 page-paging chain (SetPage, GetMaxPage, OnMouseWheel), per-store \
                 state-init (InitializeStore), category-rack lifecycle (CreateItemRack), \
                 and currency-display refresh (UpdateCurrencyAvailable) together form the \
                 entire item-display contract — losing any one would break a downstream \
                 store-front interaction. For example, dropping `OnStoreFrontSet` would \
                 leave the EventRegistry callback registered but with no handler, so \
                 AccountStoreMixin:SetStoreFrontID would fire the trigger and nothing would \
                 re-initialize the per-store category state."
            );
        }
    });
}
}

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn account_store_item_display_mixin_does_not_define_plan_named_methods_that_are_actually_absent() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for (absent_method, mismatch_reason) in PLAN_NAMED_ITEM_DISPLAY_METHODS_ABSENT {
            let method_type: String = env
                .eval(&format!(
                    "return type(_G[{ITEM_DISPLAY_MIXIN_NAME:?}][{absent_method:?}])"
                ))
                .unwrap_or_else(|error| {
                    panic!(
                        "failed to probe absence of \
                         `AccountStoreItemDisplayMixin.{absent_method}`: {error}"
                    )
                });

            assert_eq!(
                method_type, "nil",
                "Expected `AccountStoreItemDisplayMixin.{absent_method}` to be nil after \
                 `{ROOT}` loads (PLAN.md spec/source mismatch tripwire — {mismatch_reason}), \
                 got `{method_type}`. The PLAN.md task names `{absent_method}` as a method \
                 on `AccountStoreItemDisplayMixin`, but \
                 `Blizzard_AccountStoreItemDisplay.lua:2-184` declares the mixin as \
                 `AccountStoreItemDisplayMixin = {{}}` and attaches twelve methods (the \
                 lifecycle / dispatch surface plus the paging / state-init / refresh chain) \
                 — none named `{absent_method}`. A non-nil reading here means either (a) \
                 Blizzard added `RefreshSelectedCard` (or renamed `UpdateCurrencyAvailable` \
                 / `SetPage` to match), forcing a re-pin against the new refresh-chain \
                 contract, (b) some other addon monkey-patched the mixin (a layering \
                 violation worth investigating), or (c) the mixin's metatable started \
                 inheriting from a parent table that defines this method (a regression in \
                 the mixin's isolation)."
            );
        }
    });
}
}
