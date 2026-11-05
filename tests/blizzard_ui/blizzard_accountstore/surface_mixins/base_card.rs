//! Mixin-method surface pin for `AccountStoreBaseCardMixin` (card template).
//!
//! Spec/source mismatch finding (PLAN.md task for
//! `AccountStoreBaseCardMixin`): the plan names five methods —
//! `SetItemID`, `SelectCard`, `RefreshDisplay`, `OnEnter`, `OnLeave`
//! — but `Blizzard_AccountStoreCardTemplates.lua:13-227` declares
//! thirteen methods on the base-card mixin. Four of the five PLAN
//! names match the source verbatim; one (`RefreshDisplay`) is absent.
//! The mixin's lifecycle / dispatch surface (`OnLoad`, `OnShow`,
//! `OnHide`, `OnEvent`, `OnUpdate`, `OnMouseWheel`) and the override
//! hook the derived card mixins specialize (`UpdateCardDisplay`) were
//! all PLAN-omitted:
//!
//! | PLAN name        | Status                                                     |
//! |------------------|------------------------------------------------------------|
//! | `SetItemID`      | present (`Blizzard_AccountStoreCardTemplates.lua:125`)     |
//! | `SelectCard`     | present (`Blizzard_AccountStoreCardTemplates.lua:172`)     |
//! | `RefreshDisplay` | absent — no method by this name. Closest functional analogue is `UpdateCardDisplay` (`Blizzard_AccountStoreCardTemplates.lua:225`), the abstract no-op the derived mixins (`AccountStoreCreatureCardMixin`, `AccountStoreIconCardMixin`, `AccountStoreTransmogSetCardMixin`, `AccountStoreMountCardMixin`) override to drive the model-scene / icon swap when item info changes. |
//! | `OnEnter`        | present (`Blizzard_AccountStoreCardTemplates.lua:60`)      |
//! | `OnLeave`        | present (`Blizzard_AccountStoreCardTemplates.lua:87`)      |
//!
//! Architecturally-critical PLAN-omitted-but-present methods (the
//! six the surface test pins as guards against silent removal):
//!
//! | Method                | Source site                                                  | Role |
//! |-----------------------|--------------------------------------------------------------|------|
//! | `OnLoad`              | `Blizzard_AccountStoreCardTemplates.lua:20`                  | Wires BuyButton OnEnter / OnLeave / OnClick handlers (the disabled-tooltip + SelectCard chain) and ModelScene OnMouseWheel / OnEnter / OnLeave handlers (the mousewheel-forwarding + tooltip-display + LockHighlight chain). |
//! | `OnShow`              | `Blizzard_AccountStoreCardTemplates.lua:50`                  | Calls `FrameUtil.RegisterFrameForEvents(self, AccountStoreBaseCardEvents)` for `UI_MODEL_SCENE_INFO_UPDATED` / `ACCOUNT_STORE_ITEM_INFO_UPDATED`, then drives `UpdateCardDisplay` to seed the visual. |
//! | `OnHide`              | `Blizzard_AccountStoreCardTemplates.lua:56`                  | Calls `FrameUtil.UnregisterFrameForEvents` for the same two-event list. |
//! | `OnEvent`             | `Blizzard_AccountStoreCardTemplates.lua:109`                 | Dispatches the two registered frame events: `UI_MODEL_SCENE_INFO_UPDATED` re-runs `UpdateCardDisplay`, `ACCOUNT_STORE_ITEM_INFO_UPDATED` re-runs `SetItemID` when the event itemID matches `self.itemInfo.id`. |
//! | `UpdateCardDisplay`   | `Blizzard_AccountStoreCardTemplates.lua:225`                 | Abstract no-op (the comment at line 226 says `Override in your derived Mixin.`) — the closest-name analogue to PLAN's `RefreshDisplay`. |
//! | `OnMouseWheel`        | `Blizzard_AccountStoreCardTemplates.lua:105`                 | Forwards mousewheel deltas to the nearest ancestor frame via `CallMethodOnNearestAncestor` — used to drive the StoreDisplay item-rack paging chain. |

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AccountStore";
const BASE_CARD_MIXIN_NAME: &str = "AccountStoreBaseCardMixin";

const ACTUAL_BASE_CARD_METHODS: &[(&str, &str)] = &[
    (
        "SetItemID",
        "Blizzard_AccountStoreCardTemplates.lua:125 — fetches itemInfo via \
         C_AccountStore.GetItemInfo, drives Name / OwnedCheckmark / New / BuyButton state from \
         it (text, enabled, shown, refund-vs-purchase wording), repositions BuyButton anchor, \
         and re-runs UpdateCardDisplay if the card is shown",
    ),
    (
        "SelectCard",
        "Blizzard_AccountStoreCardTemplates.lua:172 — plays ACCOUNT_STORE_ITEM_SELECT sound, \
         formats the refund-vs-purchase confirmation string, and shows the generic-confirmation \
         StaticPopup whose accept handler calls C_AccountStore.RefundItem (refundable) or \
         C_AccountStore.BeginPurchase (otherwise). Falls back to the legacy \
         ACCOUNT_STORE_BEGIN_PURCHASE_OR_REFUND popup when StaticPopup_Hide is unavailable",
    ),
    (
        "OnEnter",
        "Blizzard_AccountStoreCardTemplates.lua:60 — plays ACCOUNT_STORE_ITEM_HOVER sound \
         (gated by self.hoverSoundPlayed flag), shows the GameTooltip with item name + \
         description, and appends the ACCOUNT_STORE_NONREFUNDABLE_TOOLTIP error line when the \
         item is purchasable + nonrefundable + price > 0",
    ),
    (
        "OnLeave",
        "Blizzard_AccountStoreCardTemplates.lua:87 — clears self.hoverSoundPlayed when neither \
         the card nor its ModelScene has the mouse, then hides the GameTooltip. Pairs with \
         OnEnter to gate the hover-sound replay",
    ),
    (
        "OnLoad",
        "Blizzard_AccountStoreCardTemplates.lua:20 — PLAN-omitted-but-present. Wires \
         BuyButton OnEnter / OnLeave / OnClick handlers (the disabled-tooltip + SelectCard \
         chain) and ModelScene OnMouseWheel / OnEnter / OnLeave handlers (mousewheel \
         forwarding + tooltip-display + LockHighlight chain)",
    ),
    (
        "OnShow",
        "Blizzard_AccountStoreCardTemplates.lua:50 — PLAN-omitted-but-present. Calls \
         FrameUtil.RegisterFrameForEvents for the AccountStoreBaseCardEvents list \
         (UI_MODEL_SCENE_INFO_UPDATED + ACCOUNT_STORE_ITEM_INFO_UPDATED) and drives \
         UpdateCardDisplay to seed the visual",
    ),
    (
        "OnHide",
        "Blizzard_AccountStoreCardTemplates.lua:56 — PLAN-omitted-but-present. Calls \
         FrameUtil.UnregisterFrameForEvents for the same two-event list",
    ),
    (
        "OnEvent",
        "Blizzard_AccountStoreCardTemplates.lua:109 — PLAN-omitted-but-present. Dispatches \
         the two registered frame events: UI_MODEL_SCENE_INFO_UPDATED re-runs \
         UpdateCardDisplay, ACCOUNT_STORE_ITEM_INFO_UPDATED re-runs SetItemID when the event \
         itemID matches self.itemInfo.id. Without this method the registered events would \
         fire with no handler",
    ),
    (
        "UpdateCardDisplay",
        "Blizzard_AccountStoreCardTemplates.lua:225 — PLAN-omitted-but-present. Abstract no-op \
         (the comment at line 226 says `Override in your derived Mixin.`) — the closest-name \
         analogue to PLAN's `RefreshDisplay`. The four derived card mixins \
         (AccountStoreCreatureCardMixin, AccountStoreIconCardMixin, \
         AccountStoreTransmogSetCardMixin, AccountStoreMountCardMixin) each override this to \
         drive the model-scene / icon swap when item info changes",
    ),
    (
        "OnMouseWheel",
        "Blizzard_AccountStoreCardTemplates.lua:105 — PLAN-omitted-but-present. Forwards the \
         mousewheel delta to the nearest ancestor frame via CallMethodOnNearestAncestor — \
         used to drive the StoreDisplay item-rack paging chain when the user scrolls the \
         wheel over a card",
    ),
];

const PLAN_NAMED_BASE_CARD_METHODS_ABSENT: &[(&str, &str)] = &[(
    "RefreshDisplay",
    "no method by this name on AccountStoreBaseCardMixin. The closest functional analogue \
         is UpdateCardDisplay (Blizzard_AccountStoreCardTemplates.lua:225) — the abstract no-op \
         the derived card mixins (AccountStoreCreatureCardMixin, AccountStoreIconCardMixin, \
         AccountStoreTransmogSetCardMixin, AccountStoreMountCardMixin) override to drive the \
         model-scene / icon swap. PLAN's `RefreshDisplay` is a near-miss for `UpdateCardDisplay`",
)];

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn account_store_base_card_mixin_methods_match_actual_source() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let mixin_type: String = env
            .eval(&format!("return type(_G[{BASE_CARD_MIXIN_NAME:?}])"))
            .expect("AccountStoreBaseCardMixin global probe must run cleanly");

        assert_eq!(
            mixin_type, "table",
            "Expected `_G[{BASE_CARD_MIXIN_NAME:?}]` to be a table after `{ROOT}` loads, got \
             `{mixin_type}`. The mixin is declared at `Blizzard_AccountStoreCardTemplates.lua:13` \
             as `AccountStoreBaseCardMixin = {{}}` and immediately gets thirteen methods \
             attached. The base-card XML templates at \
             `Blizzard_AccountStoreCardTemplates.xml` reference the mixin via the \
             `mixin=\"AccountStoreBaseCardMixin\"` attribute on the virtual card templates \
             (CreatureCardTemplate, IconCardTemplate, TransmogSetCardTemplate, MountCardTemplate \
             via the `AccountStoreMountCardMixin = AccountStoreCreatureCardMixin` alias at line \
             367), so the XML loader's mixin-resolution path must find the table by name in \
             `_G` and copy its methods onto every card frame instance. A nil reading means \
             either the Lua file did not execute (a regression in the addon load pipeline) or \
             the global was shadowed (a regression in the addon environment isolation)."
        );

        for (method_name, source_site) in ACTUAL_BASE_CARD_METHODS {
            let method_type: String = env
                .eval(&format!(
                    "return type(_G[{BASE_CARD_MIXIN_NAME:?}][{method_name:?}])"
                ))
                .unwrap_or_else(|error| {
                    panic!("failed to probe `AccountStoreBaseCardMixin.{method_name}`: {error}")
                });

            assert_eq!(
                method_type, "function",
                "Expected `AccountStoreBaseCardMixin.{method_name}` to be a function after \
                 `{ROOT}` loads ({source_site}), got `{method_type}`. A nil reading means \
                 either the method definition was dropped from the Lua source or never reached \
                 because of an earlier syntax error. The base-card mixin's lifecycle \
                 (OnLoad/OnShow/OnHide), frame-event dispatch (OnEvent), override hook \
                 (UpdateCardDisplay), mousewheel forwarding (OnMouseWheel), and item-info \
                 chain (SetItemID, SelectCard, OnEnter, OnLeave) together form the entire \
                 base-card contract — losing any one would break a downstream card-mixin \
                 specialization. For example, dropping `OnEvent` would leave \
                 ACCOUNT_STORE_ITEM_INFO_UPDATED registered (via OnShow) but with no handler, \
                 so server-driven item-state changes would silently stop refreshing the card \
                 visuals."
            );
        }
    });
}
}

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn account_store_base_card_mixin_does_not_define_plan_named_methods_that_are_actually_absent() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for (absent_method, mismatch_reason) in PLAN_NAMED_BASE_CARD_METHODS_ABSENT {
            let method_type: String = env
                .eval(&format!(
                    "return type(_G[{BASE_CARD_MIXIN_NAME:?}][{absent_method:?}])"
                ))
                .unwrap_or_else(|error| {
                    panic!(
                        "failed to probe absence of \
                         `AccountStoreBaseCardMixin.{absent_method}`: {error}"
                    )
                });

            assert_eq!(
                method_type, "nil",
                "Expected `AccountStoreBaseCardMixin.{absent_method}` to be nil after `{ROOT}` \
                 loads (PLAN.md spec/source mismatch tripwire — {mismatch_reason}), got \
                 `{method_type}`. The PLAN.md task names `{absent_method}` as a method on \
                 `AccountStoreBaseCardMixin`, but `Blizzard_AccountStoreCardTemplates.lua:13-227` \
                 declares the mixin as `AccountStoreBaseCardMixin = {{}}` and attaches \
                 thirteen methods (the lifecycle / dispatch surface plus the item-info / \
                 hover / refund chain) — none named `{absent_method}`. A non-nil reading here \
                 means either (a) Blizzard renamed `UpdateCardDisplay` to `RefreshDisplay` \
                 (forcing a re-pin against the new override-hook contract for every derived \
                 card mixin), (b) some other addon monkey-patched the mixin (a layering \
                 violation worth investigating), or (c) the mixin's metatable started \
                 inheriting from a parent table that defines this method (a regression in the \
                 mixin's isolation)."
            );
        }
    });
}
}
