//! Event-registration surface pins for the `Blizzard_AccountStore` lane.
//!
//! Spec/source mismatch finding (PLAN.md task for `AccountStoreFrame`
//! event registration after OnLoad): the plan says `AccountStoreFrame`
//! is registered for `ACCOUNT_STORE_FRONT_UPDATED` and
//! `STORE_FRONT_STATE_UPDATED` after OnLoad — but neither claim holds:
//!
//! 1. `AccountStoreMixin:OnLoad` (Blizzard_AccountStore.lua:18-24) only
//!    calls `self:SetPortraitToAsset(...)` and writes
//!    `UIPanelWindows["AccountStoreFrame"] = {...}`. It does NOT call
//!    `RegisterEvent` for any event. So `AccountStoreFrame` itself does
//!    not register either PLAN-named event after OnLoad.
//!
//! 2. `ACCOUNT_STORE_FRONT_UPDATED` IS registered in this lane, but on
//!    a different frame: `AccountStoreFrame.StoreDisplay` — the
//!    `AccountStoreItemDisplayMixin` instance declared at
//!    `Blizzard_AccountStore.xml:135` (parentKey `StoreDisplay`,
//!    inheriting `AccountStoreItemDisplayTemplate`). Registration
//!    happens in `AccountStoreItemDisplayMixin:OnShow`
//!    (`Blizzard_AccountStoreItemDisplay.lua:62-68`) via
//!    `FrameUtil.RegisterFrameForEvents(self, AccountStoreItemDisplayEvents)`
//!    — at OnShow time, NOT at OnLoad. The event list at lines 4-8
//!    contains exactly three events:
//!    `ACCOUNT_STORE_CURRENCY_AVAILABLE_UPDATED`,
//!    `ACCOUNT_STORE_FRONT_UPDATED`,
//!    `ACCOUNT_STORE_TRANSACTION_ERROR`.
//!
//! 3. `STORE_FRONT_STATE_UPDATED` is NOT registered anywhere in the
//!    `Blizzard_AccountStore` lane. Verified via
//!    `grep -rn STORE_FRONT_STATE_UPDATED Interface/BlizzardUI/Blizzard_AccountStore/`
//!    — zero matches. The event is registered by
//!    `Blizzard_CharacterSelectNavBar` (line 207),
//!    `Blizzard_PVPUI/Mainline/Blizzard_PVPUI.lua` (line 2439), and
//!    `Blizzard_PlunderstormBasics/Blizzard_PlunderstormBasics.lua`
//!    (line 42) — three sibling lanes, none of them this one.
//!
//! Two tests pin both halves of the contract:
//!
//! - `account_store_frame_does_not_register_plan_named_events_after_onload`
//!   walks the PLAN-named event list against `AccountStoreFrame` and
//!   asserts each is reported NOT registered. This is the spec/source
//!   mismatch tripwire — if Blizzard ever moves event registration onto
//!   `AccountStoreFrame` itself, this flips and the test forces a re-pin
//!   against the new contract.
//!
//! - `account_store_item_display_onshow_registers_actual_event_list`
//!   directly invokes `AccountStoreItemDisplayMixin.OnShow(StoreDisplay)`
//!   to drive the actual addon code path (rather than relying on
//!   simulator-side OnShow firing semantics, which depend on default
//!   visibility plus parent-chain shown state). Then it asserts the
//!   three actually-registered events from `AccountStoreItemDisplayEvents`
//!   are reported registered and `STORE_FRONT_STATE_UPDATED` is reported
//!   NOT registered (lane-absence tripwire). Direct invocation pins the
//!   addon's contract — the mixin's OnShow body — without taking a
//!   dependency on `:Show()` reliably firing OnShow in this simulator
//!   build.
//!
//! Spec/source mismatch finding (PLAN.md task for
//! `AccountStoreItemDisplayMixin` EventRegistry callbacks): the plan
//! names four events as EventRegistry callbacks for
//! `AccountStoreItemDisplayMixin` —
//! `ACCOUNT_STORE_CURRENCY_AVAILABLE_UPDATED`,
//! `ACCOUNT_STORE_FRONT_UPDATED`, `ACCOUNT_STORE_TRANSACTION_ERROR`,
//! `ACCOUNT_STORE_ITEM_INFO_UPDATED`. All four halves of the claim are
//! wrong:
//!
//! 1. The first three (`ACCOUNT_STORE_CURRENCY_AVAILABLE_UPDATED`,
//!    `ACCOUNT_STORE_FRONT_UPDATED`, `ACCOUNT_STORE_TRANSACTION_ERROR`)
//!    are FRAME events registered via
//!    `FrameUtil.RegisterFrameForEvents(self, AccountStoreItemDisplayEvents)`
//!    in `OnShow` (`Blizzard_AccountStoreItemDisplay.lua:67`) — NOT
//!    `EventRegistry` callbacks. Frame events fire through the
//!    `:OnEvent` script-handler path; `EventRegistry` callbacks fire
//!    through `EventRegistry:TriggerEvent`. The two systems are
//!    separate.
//!
//! 2. `ACCOUNT_STORE_ITEM_INFO_UPDATED` is a frame event on a DIFFERENT
//!    mixin: `AccountStoreBaseCardMixin`
//!    (`Blizzard_AccountStoreCardTemplates.lua:15-18`,
//!    `AccountStoreBaseCardEvents` list) — NOT on
//!    `AccountStoreItemDisplayMixin`. It also is not an EventRegistry
//!    callback.
//!
//! 3. `AccountStoreItemDisplayMixin`'s actual EventRegistry callbacks
//!    (registered in OnLoad via
//!    `self:AddStaticEventMethod(EventRegistry, ...)` at
//!    `Blizzard_AccountStoreItemDisplay.lua:58-59`) are two custom
//!    dot-namespaced events:
//!    - `"AccountStore.StoreFrontSet"` → `self.OnStoreFrontSet`
//!    - `"AccountStore.CategorySelected"` → `self.OnCategorySelected`
//!
//!    Triggered by `AccountStoreMixin:SetStoreFrontID` (line 47:
//!    `EventRegistry:TriggerEvent("AccountStore.StoreFrontSet", ...)`)
//!    and `AccountStoreMixin:CategorySelected`. None of the four
//!    PLAN-named events match either of these two actual events.
//!
//! Two tests pin both halves of this third mismatch:
//!
//! - `account_store_item_display_event_registry_callbacks_match_actual_event_names`
//!   asserts `EventRegistry:HasRegistrantsForEvent(actual_event)` is
//!   true for both `"AccountStore.StoreFrontSet"` and
//!   `"AccountStore.CategorySelected"` after addon load. This pins the
//!   actual contract — that AddStaticEventMethod ran during OnLoad and
//!   the callbacks are registered on the global EventRegistry.
//!
//! - `account_store_item_display_does_not_register_event_registry_callbacks_for_plan_named_events`
//!   asserts `EventRegistry:HasRegistrantsForEvent(plan_event)` is
//!   false for all four PLAN-named events. This is the spec/source
//!   mismatch tripwire — `HasRegistrantsForEvent` walks the entire
//!   `EventRegistry` callback table, so a true reading would mean
//!   either (a) some addon registered an EventRegistry callback for
//!   the PLAN-named event (unlikely — the names follow client-event
//!   conventions, not dot-namespaced EventRegistry conventions), or
//!   (b) Blizzard added the PLAN-named events to the EventRegistry
//!   callback contract (forcing a re-pin against the new shape).

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AccountStore";
const FRAME_NAME: &str = "AccountStoreFrame";
const ITEM_DISPLAY_PARENT_KEY: &str = "StoreDisplay";

const PLAN_NAMED_EVENTS_ABSENT_ON_FRAME: &[(&str, &str)] = &[
    (
        "ACCOUNT_STORE_FRONT_UPDATED",
        "actually registered on AccountStoreFrame.StoreDisplay (the AccountStoreItemDisplayMixin \
         instance) at OnShow time via FrameUtil.RegisterFrameForEvents — NOT on AccountStoreFrame \
         at OnLoad",
    ),
    (
        "STORE_FRONT_STATE_UPDATED",
        "not registered anywhere in the Blizzard_AccountStore lane — registered by sibling lanes \
         Blizzard_CharacterSelectNavBar (line 207), Blizzard_PVPUI/Mainline (line 2439), and \
         Blizzard_PlunderstormBasics (line 42)",
    ),
];

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn account_store_frame_does_not_register_plan_named_events_after_onload() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for (event_name, mismatch_reason) in PLAN_NAMED_EVENTS_ABSENT_ON_FRAME {
            let registered: bool = env
                .eval(&format!(
                    "return _G[{FRAME_NAME:?}]:IsEventRegistered({event_name:?})"
                ))
                .unwrap_or_else(|error| {
                    panic!(
                        "failed to probe `AccountStoreFrame:IsEventRegistered({event_name:?})`: \
                         {error}"
                    )
                });

            assert!(
                !registered,
                "Expected `AccountStoreFrame:IsEventRegistered({event_name:?})` to return false \
                 after `{ROOT}` loads (PLAN.md spec/source mismatch tripwire — {mismatch_reason}), \
                 got true. The PLAN.md task asserts this event is registered on \
                 `AccountStoreFrame` after OnLoad, but `AccountStoreMixin:OnLoad` \
                 (`Blizzard_AccountStore.lua:18-24`) only calls `SetPortraitToAsset` and writes \
                 `UIPanelWindows[\"AccountStoreFrame\"]` — it never invokes `RegisterEvent`. A \
                 true reading here would mean either (a) Blizzard moved event registration onto \
                 `AccountStoreFrame` (forcing a re-pin against the new OnLoad contract), or (b) \
                 the simulator's event-routing leaked a child's RegisterEvent onto the parent (a \
                 regression in `register_event` / `registered_events` storage)."
            );
        }
    });
}
}

const ACTUAL_ITEM_DISPLAY_EVENTS: &[(&str, &str)] = &[
    (
        "ACCOUNT_STORE_CURRENCY_AVAILABLE_UPDATED",
        "AccountStoreItemDisplayEvents[1] (Blizzard_AccountStoreItemDisplay.lua:5)",
    ),
    (
        "ACCOUNT_STORE_FRONT_UPDATED",
        "AccountStoreItemDisplayEvents[2] (Blizzard_AccountStoreItemDisplay.lua:6)",
    ),
    (
        "ACCOUNT_STORE_TRANSACTION_ERROR",
        "AccountStoreItemDisplayEvents[3] (Blizzard_AccountStoreItemDisplay.lua:7)",
    ),
];

const LANE_ABSENT_EVENT: &str = "STORE_FRONT_STATE_UPDATED";

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn account_store_item_display_onshow_registers_actual_event_list() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let (has_mixin, has_register_event): (bool, bool) = env
            .eval(&format!(
                r#"
                local store_display = _G[{FRAME_NAME:?}][{ITEM_DISPLAY_PARENT_KEY:?}]
                return type(AccountStoreItemDisplayMixin) == "table",
                       type(store_display.RegisterEvent) == "function"
                "#
            ))
            .expect("mixin and RegisterEvent precondition probe must run cleanly");

        assert!(
            has_mixin,
            "Precondition: `AccountStoreItemDisplayMixin` must be a table (defined at \
             `Blizzard_AccountStoreItemDisplay.lua:2`) before this test invokes its OnShow"
        );
        assert!(
            has_register_event,
            "Precondition: `StoreDisplay:RegisterEvent` must be callable before this test \
             drives the OnShow chain"
        );

        env.eval::<()>(&format!(
            r#"
            local store_display = _G[{FRAME_NAME:?}][{ITEM_DISPLAY_PARENT_KEY:?}]
            AccountStoreItemDisplayMixin.OnShow(store_display)
            "#
        ))
        .expect(
            "Direct invocation of `AccountStoreItemDisplayMixin.OnShow(StoreDisplay)` must run \
             cleanly — the mixin OnShow at `Blizzard_AccountStoreItemDisplay.lua:62-68` calls \
             CallbackRegistrantMixin.OnShow(self) (which iterates the per-frame dynamic \
             registrant handlers populated by AddDynamicEventMethod, empty here since OnLoad \
             only added static methods) followed by FrameUtil.RegisterFrameForEvents",
        );

        for (event_name, source_site) in ACTUAL_ITEM_DISPLAY_EVENTS {
            let registered: bool = env
                .eval(&format!(
                    "return _G[{FRAME_NAME:?}][{ITEM_DISPLAY_PARENT_KEY:?}]:IsEventRegistered({event_name:?})"
                ))
                .unwrap_or_else(|error| {
                    panic!(
                        "failed to probe \
                         `AccountStoreFrame.StoreDisplay:IsEventRegistered({event_name:?})`: \
                         {error}"
                    )
                });

            assert!(
                registered,
                "Expected `AccountStoreFrame.StoreDisplay:IsEventRegistered({event_name:?})` to \
                 return true after a forced OnShow cycle (Hide → Show) on the StoreDisplay child. \
                 The event is declared in `AccountStoreItemDisplayEvents` ({source_site}) and \
                 registered by `AccountStoreItemDisplayMixin:OnShow` \
                 (`Blizzard_AccountStoreItemDisplay.lua:62-68`) via \
                 `FrameUtil.RegisterFrameForEvents(self, AccountStoreItemDisplayEvents)`, which \
                 walks the list and calls `frame:RegisterEvent(events[i])` for each entry \
                 (`shared_bootstrap.lua:268-277`). A false reading means either (a) the OnShow \
                 script chain dropped the mixin's OnShow handler (a regression in the \
                 mixin-to-script registration path), (b) `FrameUtil.RegisterFrameForEvents` \
                 stopped delegating to `RegisterEvent` (a regression in the bootstrap fallback), \
                 or (c) the StoreDisplay child wasn't transitioned through Hide → Show cleanly \
                 (a regression in the simulator's :Show() / OnShow firing semantics)."
            );
        }

        let lane_absent_registered: bool = env
            .eval(&format!(
                "return _G[{FRAME_NAME:?}][{ITEM_DISPLAY_PARENT_KEY:?}]:IsEventRegistered({LANE_ABSENT_EVENT:?})"
            ))
            .expect(
                "STORE_FRONT_STATE_UPDATED probe on StoreDisplay must run cleanly",
            );

        assert!(
            !lane_absent_registered,
            "Expected `AccountStoreFrame.StoreDisplay:IsEventRegistered({LANE_ABSENT_EVENT:?})` \
             to return false after a forced OnShow cycle, got true. \
             `STORE_FRONT_STATE_UPDATED` is NOT in `AccountStoreItemDisplayEvents` \
             (`Blizzard_AccountStoreItemDisplay.lua:4-8` lists only \
             ACCOUNT_STORE_CURRENCY_AVAILABLE_UPDATED, ACCOUNT_STORE_FRONT_UPDATED, \
             ACCOUNT_STORE_TRANSACTION_ERROR), so OnShow does not register it on StoreDisplay. \
             This is the lane-absence tripwire: if a future change adds \
             `STORE_FRONT_STATE_UPDATED` to the AccountStoreItemDisplayEvents list, this flips \
             and forces a re-pin against the new event contract. The PLAN.md task names this \
             event as one of two registered on `AccountStoreFrame` after OnLoad — but the event \
             actually lives in three sibling lanes (Blizzard_CharacterSelectNavBar line 207, \
             Blizzard_PVPUI/Mainline line 2439, Blizzard_PlunderstormBasics line 42), none of \
             them this one."
        );
    });
}
}

const ACTUAL_EVENT_REGISTRY_CALLBACKS: &[(&str, &str)] = &[
    (
        "AccountStore.StoreFrontSet",
        "Blizzard_AccountStoreItemDisplay.lua:58 — `self:AddStaticEventMethod(EventRegistry, \
         \"AccountStore.StoreFrontSet\", self.OnStoreFrontSet)`. Triggered by \
         AccountStoreMixin:SetStoreFrontID at Blizzard_AccountStore.lua:47.",
    ),
    (
        "AccountStore.CategorySelected",
        "Blizzard_AccountStoreItemDisplay.lua:59 — `self:AddStaticEventMethod(EventRegistry, \
         \"AccountStore.CategorySelected\", self.OnCategorySelected)`. Triggered by \
         AccountStoreMixin:CategorySelected.",
    ),
];

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn account_store_item_display_event_registry_callbacks_match_actual_event_names() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for (callback_event_name, source_site) in ACTUAL_EVENT_REGISTRY_CALLBACKS {
            let has_registrants: bool = env
                .eval(&format!(
                    "return EventRegistry:HasRegistrantsForEvent({callback_event_name:?})"
                ))
                .unwrap_or_else(|error| {
                    panic!(
                        "failed to probe \
                         `EventRegistry:HasRegistrantsForEvent({callback_event_name:?})`: {error}"
                    )
                });

            assert!(
                has_registrants,
                "Expected `EventRegistry:HasRegistrantsForEvent({callback_event_name:?})` to \
                 return true after `{ROOT}` loads ({source_site}), got false. \
                 `AccountStoreItemDisplayMixin:OnLoad` (`Blizzard_AccountStoreItemDisplay.lua:32-60`) \
                 calls `self:AddStaticEventMethod(EventRegistry, ..., handler)` at lines 58-59 to \
                 wire two custom EventRegistry callbacks. `AddStaticEventMethod` \
                 (`CallbackRegistrant.lua:29-32`) inserts the handler into the static registrant \
                 table AND immediately calls `RegisterFromRegistrationInfo`, which delegates to \
                 `EventRegistry:RegisterCallback`. So after the OnLoad chain runs, EventRegistry's \
                 callback table for the event MUST be populated. A false reading means either (a) \
                 OnLoad never reached the `AddStaticEventMethod` calls (likely an error earlier \
                 in OnLoad — e.g. Footer subwidgets missing), (b) `AddStaticEventMethod` failed \
                 to delegate to `RegisterCallback` (a regression in CallbackRegistrantMixin), or \
                 (c) the simulator's EventRegistry singleton was not bootstrapped as a \
                 CallbackRegistryMixin (a regression in `runtime_surface_bootstrap.lua:12143-12146`)."
            );
        }
    });
}
}

const PLAN_NAMED_EVENT_REGISTRY_CALLBACKS_ABSENT: &[(&str, &str)] = &[
    (
        "ACCOUNT_STORE_CURRENCY_AVAILABLE_UPDATED",
        "frame event in AccountStoreItemDisplayEvents (Blizzard_AccountStoreItemDisplay.lua:5), \
         registered via FrameUtil.RegisterFrameForEvents in OnShow — NOT an EventRegistry \
         callback. The frame-event surface is verified by the prior \
         `account_store_item_display_onshow_registers_actual_event_list` test.",
    ),
    (
        "ACCOUNT_STORE_FRONT_UPDATED",
        "frame event in AccountStoreItemDisplayEvents (Blizzard_AccountStoreItemDisplay.lua:6), \
         registered via FrameUtil.RegisterFrameForEvents in OnShow — NOT an EventRegistry \
         callback.",
    ),
    (
        "ACCOUNT_STORE_TRANSACTION_ERROR",
        "frame event in AccountStoreItemDisplayEvents (Blizzard_AccountStoreItemDisplay.lua:7), \
         registered via FrameUtil.RegisterFrameForEvents in OnShow — NOT an EventRegistry \
         callback.",
    ),
    (
        "ACCOUNT_STORE_ITEM_INFO_UPDATED",
        "frame event on a DIFFERENT mixin (AccountStoreBaseCardMixin), declared at \
         Blizzard_AccountStoreCardTemplates.lua:17 and registered via \
         FrameUtil.RegisterFrameForEvents in `AccountStoreBaseCardMixin:OnShow` — NOT an \
         EventRegistry callback, and NOT on AccountStoreItemDisplayMixin at all.",
    ),
];

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn account_store_item_display_does_not_register_event_registry_callbacks_for_plan_named_events() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for (plan_event, mismatch_reason) in PLAN_NAMED_EVENT_REGISTRY_CALLBACKS_ABSENT {
            let has_registrants: bool = env
                .eval(&format!(
                    "return EventRegistry:HasRegistrantsForEvent({plan_event:?})"
                ))
                .unwrap_or_else(|error| {
                    panic!(
                        "failed to probe \
                         `EventRegistry:HasRegistrantsForEvent({plan_event:?})`: {error}"
                    )
                });

            assert!(
                !has_registrants,
                "Expected `EventRegistry:HasRegistrantsForEvent({plan_event:?})` to return false \
                 after `{ROOT}` loads (PLAN.md spec/source mismatch tripwire — {mismatch_reason}), \
                 got true. `HasRegistrantsForEvent` walks the entire EventRegistry callback table \
                 (`CallbackRegistry.lua:96-104`), so a true reading means either (a) some other \
                 addon registered an EventRegistry callback for the PLAN-named event — unlikely \
                 because the names follow client-event conventions (UPPER_SNAKE_CASE) rather than \
                 the dot-namespaced convention EventRegistry uses (e.g. `AccountStore.StoreFrontSet`), \
                 or (b) Blizzard added these events to the EventRegistry callback contract for \
                 `AccountStoreItemDisplayMixin`, forcing a re-pin against the new shape. Note: \
                 the actual EventRegistry callbacks registered by `AccountStoreItemDisplayMixin:OnLoad` \
                 are `\"AccountStore.StoreFrontSet\"` and `\"AccountStore.CategorySelected\"` — \
                 verified by the companion \
                 `account_store_item_display_event_registry_callbacks_match_actual_event_names` \
                 test."
            );
        }
    });
}
}
