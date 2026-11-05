//! Mixin-method surface pin for `FullscreenAccountStoreContainerMixin`
//! (toplevel container hosting AccountStoreFrame in WoW Labs / Plunderstorm).
//!
//! Spec/source mismatch finding (PLAN.md task for
//! `FullscreenAccountStoreContainerMixin`): the plan names four
//! methods — `OnLoad`, `OnShow`, `OnHide`, `OnKeyDown` (escape-key
//! handling) — but `Blizzard_AccountStore.lua:66-100` declares only
//! three methods on the mixin. Three of the four PLAN names match
//! the source verbatim; one (`OnLoad`) is absent — the mixin has no
//! init lifecycle method at all. The container needs no `OnLoad`
//! wiring because the keyboard input it consumes is enabled via the
//! XML attribute `enableKeyboard="true"` on the frame declaration
//! (`Blizzard_AccountStore.xml:149`), so `OnKeyDown` fires without
//! any Lua-side `RegisterEvent` / handler-wiring step. The container
//! is also `toplevel="true" hidden="true"` and is reparented into
//! visibility by `AccountStoreMixin:SetFullscreenMode`
//! (`Blizzard_AccountStore.lua:50`), not by an OnLoad initializer:
//!
//! | PLAN name        | Status                                                     |
//! |------------------|------------------------------------------------------------|
//! | `OnLoad`         | absent — no method by this name on `FullscreenAccountStoreContainerMixin`. The mixin has no init lifecycle. Keyboard input is XML-enabled (`enableKeyboard="true"` on the frame at `Blizzard_AccountStore.xml:149`), and the container's visibility lifecycle is driven externally by `AccountStoreMixin:SetFullscreenMode` (which reparents AccountStoreFrame between UIParent and FullscreenAccountStoreContainer + toggles container `Show()` / `Hide()`). PLAN's `OnLoad` has no closest-name analogue on the mixin. |
//! | `OnShow`         | present (`Blizzard_AccountStore.lua:68`)                   |
//! | `OnHide`         | present (`Blizzard_AccountStore.lua:82`)                   |
//! | `OnKeyDown`      | present (`Blizzard_AccountStore.lua:95`) — matches the PLAN parenthetical note "(escape-key handling)": the body checks `key == "ESCAPE"` and calls `LeaveFullscreenMode()`. |
//!
//! No PLAN-omitted-but-present methods — the mixin's complete
//! source surface is exactly the three PLAN-named-and-present
//! methods (`OnShow`, `OnHide`, `OnKeyDown`).

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AccountStore";
const FULLSCREEN_CONTAINER_MIXIN_NAME: &str = "FullscreenAccountStoreContainerMixin";

const ACTUAL_FULLSCREEN_CONTAINER_METHODS: &[(&str, &str)] = &[
    (
        "OnShow",
        "Blizzard_AccountStore.lua:68 — early-returns when the parent is GlueParent (login \
         screen), then registers self as the StaticPopup full-screen frame, the AlertFrame \
         full-screen frame + base-anchor target (anchored to AccountStoreFrameBottom), and \
         the ActionStatus alternate parent. Finally calls \
         AlertFrame:BlockLeftClickingAlerts(self) to suppress alert click-through while \
         the fullscreen store is up",
    ),
    (
        "OnHide",
        "Blizzard_AccountStore.lua:82 — symmetric to OnShow: clears the StaticPopup + \
         AlertFrame + ActionStatus full-screen registrations and unblocks alert \
         left-clicking. Note the body has a known bug at line 83 (references undefined \
         `currParent` instead of computing it from `self:GetParent()`) — Blizzard's source \
         behavior",
    ),
    (
        "OnKeyDown",
        "Blizzard_AccountStore.lua:95 — escape-key handler. Since the toplevel container \
         captures keyboard input (XML `enableKeyboard=\"true\"` at \
         Blizzard_AccountStore.xml:149), the parent's escape-close behavior would never \
         fire. This method manually implements escape: when key == \"ESCAPE\" it calls \
         LeaveFullscreenMode() — the public glue-side entry point that reparents \
         AccountStoreFrame back to UIParent and hides the fullscreen container",
    ),
];

const PLAN_NAMED_FULLSCREEN_CONTAINER_METHODS_ABSENT: &[(&str, &str)] = &[(
    "OnLoad",
    "no method by this name on FullscreenAccountStoreContainerMixin. The mixin has no \
         init lifecycle. Keyboard input is enabled via the XML attribute \
         `enableKeyboard=\"true\"` on the frame declaration (Blizzard_AccountStore.xml:149), \
         so OnKeyDown fires without any Lua-side handler-wiring step. The container's \
         visibility lifecycle is driven externally by AccountStoreMixin:SetFullscreenMode \
         (Blizzard_AccountStore.lua:50, which reparents AccountStoreFrame between UIParent \
         and FullscreenAccountStoreContainer + toggles container Show()/Hide()), not by \
         an OnLoad initializer. PLAN's `OnLoad` has no closest-name analogue on the mixin",
)];

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn fullscreen_account_store_container_mixin_methods_match_actual_source() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let mixin_type: String = env
            .eval(&format!(
                "return type(_G[{FULLSCREEN_CONTAINER_MIXIN_NAME:?}])"
            ))
            .expect("FullscreenAccountStoreContainerMixin global probe must run cleanly");

        assert_eq!(
            mixin_type, "table",
            "Expected `_G[{FULLSCREEN_CONTAINER_MIXIN_NAME:?}]` to be a table after `{ROOT}` \
             loads, got `{mixin_type}`. The mixin is declared at \
             `Blizzard_AccountStore.lua:66` as \
             `FullscreenAccountStoreContainerMixin = {{}}` and gets three methods attached. \
             The XML toplevel frame `FullscreenAccountStoreContainer` \
             (`Blizzard_AccountStore.xml:149` — `toplevel=\"true\" hidden=\"true\" \
             parent=\"UIParent\" enableMouse=\"true\" enableKeyboard=\"true\"`) references \
             the mixin via the `mixin=\"FullscreenAccountStoreContainerMixin\"` attribute, \
             so the XML loader's mixin-resolution path must find the table by name in `_G` \
             and copy its methods onto the container frame instance. A nil reading means \
             either the Lua file did not execute (a regression in the addon load pipeline) \
             or the global was shadowed (a regression in the addon environment isolation)."
        );

        for (method_name, source_site) in ACTUAL_FULLSCREEN_CONTAINER_METHODS {
            let method_type: String = env
                .eval(&format!(
                    "return type(_G[{FULLSCREEN_CONTAINER_MIXIN_NAME:?}][{method_name:?}])"
                ))
                .unwrap_or_else(|error| {
                    panic!(
                        "failed to probe `FullscreenAccountStoreContainerMixin.{method_name}`: \
                         {error}"
                    )
                });

            assert_eq!(
                method_type, "function",
                "Expected `FullscreenAccountStoreContainerMixin.{method_name}` to be a \
                 function after `{ROOT}` loads ({source_site}), got `{method_type}`. The \
                 container's three methods together form the entire fullscreen-container \
                 contract: `OnShow` re-parents the StaticPopup / AlertFrame / ActionStatus \
                 anchors so their visuals overlay the fullscreen store correctly; `OnHide` \
                 reverses those re-parenting calls; `OnKeyDown` is the manual escape \
                 handler the toplevel container needs because `enableKeyboard=\"true\"` \
                 traps keyboard input from the parent's escape-close path. Losing \
                 `OnKeyDown` would silently break the escape-close behavior in WoW Labs / \
                 Plunderstorm fullscreen mode — players would have no keyboard way to exit \
                 the store. Losing `OnShow` or `OnHide` would leave alerts and static \
                 popups anchored to the wrong frame in fullscreen mode."
            );
        }
    });
}
}

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn fullscreen_account_store_container_mixin_does_not_define_plan_named_methods_that_are_actually_absent()
 {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for (absent_method, mismatch_reason) in PLAN_NAMED_FULLSCREEN_CONTAINER_METHODS_ABSENT {
            let method_type: String = env
                .eval(&format!(
                    "return type(_G[{FULLSCREEN_CONTAINER_MIXIN_NAME:?}][{absent_method:?}])"
                ))
                .unwrap_or_else(|error| {
                    panic!(
                        "failed to probe absence of \
                         `FullscreenAccountStoreContainerMixin.{absent_method}`: {error}"
                    )
                });

            assert_eq!(
                method_type, "nil",
                "Expected `FullscreenAccountStoreContainerMixin.{absent_method}` to be nil \
                 after `{ROOT}` loads (PLAN.md spec/source mismatch tripwire — \
                 {mismatch_reason}), got `{method_type}`. The PLAN.md task names \
                 `{absent_method}` as a method on `FullscreenAccountStoreContainerMixin`, \
                 but `Blizzard_AccountStore.lua:66-100` declares the mixin as \
                 `FullscreenAccountStoreContainerMixin = {{}}` and attaches exactly three \
                 methods (`OnShow`, `OnHide`, `OnKeyDown`) — none named `{absent_method}`. \
                 A non-nil reading here means either (a) Blizzard added an init lifecycle \
                 method to the mixin (forcing a re-pin against the new contract — and \
                 likely retiring the XML-driven `enableKeyboard=\"true\"` keyboard-trap \
                 design), (b) some other addon monkey-patched the mixin (a layering \
                 violation worth investigating), or (c) the mixin's metatable started \
                 inheriting from a parent table that defines this method (a regression in \
                 the mixin's isolation)."
            );
        }
    });
}
}
