//! Surface-level globals pinned by the `Blizzard_AccountStore` lane.
//!
//! Spec/source mismatch finding (PLAN.md task for `AccountStoreUtil`):
//! the plan named `FormatCurrencyDisplayWithIcon` and
//! `FormatCurrencyTotalWithIcon`, but neither exists in this revision of
//! `Blizzard_AccountStoreUtil.lua`. The closest-named real functions are
//! `FormatCurrencyDisplay` (line 56 — formats amount with embedded icon
//! markup) and `FormatCurrencyDisplayWithWarning` (line 74 — same plus
//! threshold-based color wrapping). The other two PLAN-named functions
//! (`IsCurrencyAtWarningThreshold`, `AddCurrencyTotalTooltip`) match the
//! source verbatim. The `account_store_util_publishes_table_and_currency_functions`
//! test pins all four real functions, including the closest-name
//! substitutes — a future Blizzard rename to the PLAN-shaped names
//! would flip this test and force a re-pin against the new names.
//!
//! Spec/source mismatch finding (PLAN.md task for the
//! `ACCOUNT_STORE_TRANSACTION_ERROR` static popup): the plan named the
//! field set as `button1`, `OnAccept`, `text` — but
//! `Blizzard_AccountStoreUtil.lua:11-18` declares the popup with
//! `text`, `button1`, `showAlert`, `hideOnEscape`, `timeout`,
//! `whileDead` and NO `OnAccept` handler. This is the dismiss-only
//! confirmation pattern (the popup just informs the user that the
//! transaction failed; clicking OK closes it without dispatching a
//! follow-up action). The `account_store_transaction_error_popup_is_registered_with_expected_fields`
//! test pins `text` (string, populated), `button1` (string, populated),
//! `showAlert` (bool true), and absence-of-`OnAccept` (the spec/source
//! mismatch surfaces as a positive nil assertion so the test would flip
//! if Blizzard later adds a handler).
//!
//! Spec/source mismatch finding (PLAN.md task for `ToggleAccountStoreUI`):
//! the plan named `ToggleAccountStoreUI(storeFrontID)` as a global
//! that takes a store-front id, but no such global exists. The closest
//! match is a `local function ToggleAccountStoreUI()` (zero args) in
//! `Blizzard_CharacterSelectNavBar.lua:44` whose body is a one-line
//! redirect: `AccountStoreUtil.ToggleAccountStore()`. The actual
//! public toggle is `AccountStoreUtil.ToggleAccountStore()` at
//! `Blizzard_AccountStoreUtil.lua:52` — also zero-arg, it flips
//! `AccountStoreFrame:IsShown()` via
//! `SetAccountStoreShown(not AccountStoreFrame:IsShown())`. The
//! `account_store_toggle_is_callable_and_flips_frame_shown_state`
//! test pins the absence of the PLAN-named global and the callability +
//! state-flip behavior of the actual public toggle.
//!
//! Three tables are published at file scope by the lane's primary Lua
//! body (`Blizzard_AccountStore.lua`):
//!
//! | Global                                    | Defining line                          |
//! |-------------------------------------------|----------------------------------------|
//! | `AccountStoreMixin`                       | `Blizzard_AccountStore.lua:16` (`= {}`) |
//! | `FullscreenAccountStoreContainerMixin`    | `Blizzard_AccountStore.lua:66` (`= {}`) |
//! | `FullscreenLeaveAccountStoreButtonMixin`  | `Blizzard_AccountStore.lua:102` (`= {}`)|
//!
//! Why these three deserve a dedicated surface pin separate from the
//! load-smoke's general "lane file-scope globals exist" check: each
//! mixin is the entry point for a distinct frame in the addon's XML
//! (the root `AccountStoreFrame`, the fullscreen container, and the
//! fullscreen leave button respectively). The XML side wires
//! `mixin="AccountStoreMixin"` etc. on the frame definitions, so any
//! regression that drops a mixin at file scope would surface as a
//! template-resolution failure when the XML loads — but pinning the
//! table type at the global level catches the regression closer to the
//! source. A nil reading here means `Blizzard_AccountStore.lua` either
//! failed to execute (a regression in the load pipeline) or executed
//! but stopped before reaching the relevant assignment (a regression
//! in upstream globals like `Mixin` that the file uses indirectly).

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AccountStore";

const PUBLISHED_TABLES: &[&str] = &[
    "AccountStoreMixin",
    "FullscreenAccountStoreContainerMixin",
    "FullscreenLeaveAccountStoreButtonMixin",
];

const CATEGORY_LIST_TABLES: &[&str] =
    &["AccountStoreCategoryMixin", "AccountStoreCategoryListMixin"];

const CARD_TEMPLATE_TABLES: &[&str] = &[
    "AccountStoreBaseCardMixin",
    "AccountStoreCreatureCardMixin",
    "AccountStoreIconCardMixin",
    "AccountStoreTransmogSetCardMixin",
    "AccountStoreMountCardMixin",
];

const ITEM_VIEW_TABLES: &[(&str, &str)] = &[
    (
        "AccountStoreItemDisplayMixin",
        "Blizzard_AccountStoreItemDisplay.lua:2",
    ),
    (
        "AccountStoreItemRackMixin",
        "Blizzard_AccountStoreItemRack.lua:18",
    ),
];

const ACCOUNT_STORE_UTIL_FUNCTIONS: &[(&str, &str)] = &[
    (
        "IsCurrencyAtWarningThreshold",
        "Blizzard_AccountStoreUtil.lua:65",
    ),
    (
        "AddCurrencyTotalTooltip",
        "Blizzard_AccountStoreUtil.lua:108",
    ),
    ("FormatCurrencyDisplay", "Blizzard_AccountStoreUtil.lua:56"),
    (
        "FormatCurrencyDisplayWithWarning",
        "Blizzard_AccountStoreUtil.lua:74",
    ),
];

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn account_store_publishes_expected_global_mixin_tables() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for global in PUBLISHED_TABLES {
            let actual_type: String = env
                .eval(&format!("return type(_G[{global:?}])"))
                .unwrap_or_else(|error| panic!("failed to probe `{global}` type: {error}"));

            assert_eq!(
                actual_type, "table",
                "Expected `{global}` to publish as a table after `{ROOT}` loads, got \
                 `{actual_type}`. The mixin is declared at file scope in \
                 `Blizzard_AccountStore.lua` and consumed via `mixin=\"{global}\"` on the \
                 corresponding frame in `Blizzard_AccountStore.xml`. A nil here means either \
                 the Lua file did not execute (load-pipeline regression) or it stopped before \
                 the relevant `{global} = {{}}` assignment (upstream-global regression). \
                 Either way, the XML's mixin reference would break and the frame would not \
                 inherit any of its declared methods."
            );
        }
    });
}
}

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn account_store_category_list_publishes_expected_global_mixin_tables() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for global in CATEGORY_LIST_TABLES {
            let actual_type: String = env
                .eval(&format!("return type(_G[{global:?}])"))
                .unwrap_or_else(|error| panic!("failed to probe `{global}` type: {error}"));

            assert_eq!(
                actual_type, "table",
                "Expected `{global}` to publish as a table after `{ROOT}` loads, got \
                 `{actual_type}`. Both globals are declared at file scope in \
                 `Blizzard_AccountStoreCategoryList.lua` (lines 2 and 18 — `{global} = {{}}`) \
                 and consumed by the corresponding XML on the category-list scroll-box rows \
                 and the list container respectively. The TOC orders \
                 `Blizzard_AccountStoreCategoryList.lua` second (right after \
                 `Blizzard_AccountStoreUtil.lua`); a regression that drops this file from the \
                 TOC, or that fails to reach the assignment, would surface as a nil here. \
                 The category-list mixins are also a load-order canary for the rest of the \
                 lane: `Blizzard_AccountStore.lua:OnStoreFrontSet` reaches into \
                 `AccountStoreCategoryListMixin` indirectly via the XML's parentKey routing, \
                 so any regression dropping these globals would silently break the whole \
                 category-selection flow."
            );
        }
    });
}
}

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn account_store_card_templates_publishes_expected_global_mixin_tables() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for global in CARD_TEMPLATE_TABLES {
            let actual_type: String = env
                .eval(&format!("return type(_G[{global:?}])"))
                .unwrap_or_else(|error| panic!("failed to probe `{global}` type: {error}"));

            assert_eq!(
                actual_type, "table",
                "Expected `{global}` to publish as a table after `{ROOT}` loads, got \
                 `{actual_type}`. The five card mixins are declared at file scope in \
                 `Blizzard_AccountStoreCardTemplates.lua` (lines 13, 230, 252, 263, 367) and \
                 wired via `mixin=\"...\"` on the corresponding card frame templates in \
                 `Blizzard_AccountStoreCardTemplates.xml`. The XML's per-card-type templates \
                 build on `AccountStoreBaseCardMixin` for shared OnLoad/OnEnter/OnLeave/OnEvent \
                 handlers and override `UpdateCardDisplay` per type — a nil reading here means \
                 either the file did not execute or it stopped before the relevant assignment, \
                 which would leave every card frame in the lane without its display-update \
                 method."
            );
        }

        let mount_aliases_creature: bool = env
            .eval("return rawequal(AccountStoreMountCardMixin, AccountStoreCreatureCardMixin)")
            .expect("Mount/Creature alias-identity probe must run cleanly");

        assert!(
            mount_aliases_creature,
            "`AccountStoreMountCardMixin` MUST be the same table reference as \
             `AccountStoreCreatureCardMixin` (Blizzard_AccountStoreCardTemplates.lua:367 — \
             `AccountStoreMountCardMixin = AccountStoreCreatureCardMixin;`). The two card \
             types share their entire behavior surface (the mount card is rendered with the \
             same creature-display path as the creature card), so the addon expresses this by \
             aliasing the table directly rather than wrapping with `CreateFromMixins`. A \
             regression that converts the alias into a fresh empty table or a `CreateFromMixins` \
             wrap would break the shared-behavior contract: the mount card would lose access \
             to `UpdateCardDisplay` and any future Creature-mixin additions would silently \
             fail to propagate. The general type-only assertion above would pass in that \
             regression scenario (Mount is still a table), so this identity check is the only \
             thing keeping the alias contract honest."
        );
    });
}
}

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn account_store_item_view_publishes_expected_global_mixin_tables() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for (global, defining_site) in ITEM_VIEW_TABLES {
            let actual_type: String = env
                .eval(&format!("return type(_G[{global:?}])"))
                .unwrap_or_else(|error| panic!("failed to probe `{global}` type: {error}"));

            assert_eq!(
                actual_type, "table",
                "Expected `{global}` to publish as a table after `{ROOT}` loads, got \
                 `{actual_type}`. Declared at `{defining_site}` (`{global} = {{}}`) and \
                 consumed via `mixin=\"{global}\"` on the corresponding XML frame template. \
                 The two globals come from sibling Lua files \
                 (`Blizzard_AccountStoreItemRack.lua` runs third, \
                 `Blizzard_AccountStoreItemDisplay.lua` runs fourth in the TOC order); a \
                 regression that drops either file from the TOC, or that fails to reach the \
                 assignment, would surface as a nil here. The item-rack mixin renders the \
                 grid of available cards inside the selected category, and the item-display \
                 mixin renders the focused-card detail panel — both are required for the lane \
                 to function past the category-list selection."
            );
        }
    });
}
}

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn account_store_util_publishes_table_and_currency_functions() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let table_type: String = env
            .eval("return type(_G[\"AccountStoreUtil\"])")
            .expect("AccountStoreUtil type probe must run cleanly");

        assert_eq!(
            table_type, "table",
            "Expected `AccountStoreUtil` to publish as a table after `{ROOT}` loads, got \
             `{table_type}`. The namespace is declared at file scope in \
             `Blizzard_AccountStoreUtil.lua:1` (`AccountStoreUtil = {{}}`) and runs first in \
             the TOC ordering — every later file in the lane reaches `AccountStoreUtil.*` to \
             route currency formatting and toggle behavior, so a nil here would chain into \
             every downstream `AccountStoreUtil.X` call returning a nil-call error."
        );

        for (function_name, defining_site) in ACCOUNT_STORE_UTIL_FUNCTIONS {
            let actual_type: String = env
                .eval(&format!("return type(AccountStoreUtil[{function_name:?}])"))
                .unwrap_or_else(|error| {
                    panic!("failed to probe `AccountStoreUtil.{function_name}`: {error}")
                });

            assert_eq!(
                actual_type, "function",
                "Expected `AccountStoreUtil.{function_name}` to be a function after `{ROOT}` \
                 loads (declared at `{defining_site}`), got `{actual_type}`. A nil reading \
                 means the assignment was dropped — every downstream caller would surface \
                 this as a nil-call error against the AccountStoreUtil namespace."
            );
        }
    });
}
}

const TRANSACTION_ERROR_POPUP_KEY: &str = "ACCOUNT_STORE_TRANSACTION_ERROR";

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn account_store_transaction_error_popup_is_registered_with_expected_fields() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let popup_type: String = env
            .eval(&format!(
                "return type(StaticPopupDialogs[{TRANSACTION_ERROR_POPUP_KEY:?}])"
            ))
            .expect("StaticPopupDialogs lookup must run cleanly");

        assert_eq!(
            popup_type, "table",
            "Expected `StaticPopupDialogs[{TRANSACTION_ERROR_POPUP_KEY:?}]` to be a table after \
             `{ROOT}` loads, got `{popup_type}`. The popup is registered at file scope in \
             `Blizzard_AccountStoreUtil.lua:11-18` (the literal `StaticPopupDialogs[\"...\"] = {{ \
             ... }}` table assignment) and consumed by \
             `Blizzard_AccountStoreItemDisplay.lua:96` via `StaticPopup_Show(\"...\")` when the \
             `ACCOUNT_STORE_TRANSACTION_ERROR` event fires. A nil here means either the \
             `StaticPopupDialogs` global was not registered by Blizzard_StaticPopup (a missing \
             dependency), or the file-scope assignment did not run."
        );

        let text_is_string: bool = env
            .eval(&format!(
                "local p = StaticPopupDialogs[{TRANSACTION_ERROR_POPUP_KEY:?}]; \
                 return type(p.text) == \"string\" and #p.text > 0"
            ))
            .expect("popup `text` field probe must run cleanly");

        assert!(
            text_is_string,
            "Expected `StaticPopupDialogs[{TRANSACTION_ERROR_POPUP_KEY:?}].text` to be a \
             non-empty string after `{ROOT}` loads. The source declares it as \
             `NORMAL_FONT_COLOR:WrapTextInColorCode(ACCOUNT_STORE_INCOMPLETE_TRANSACTION)` at \
             `Blizzard_AccountStoreUtil.lua:12` — the wrap call evaluates eagerly at file-scope \
             load time, so this is a concrete string by the time the lane finishes loading. A \
             nil or empty reading means either the global locale string \
             `ACCOUNT_STORE_INCOMPLETE_TRANSACTION` was not registered, or `NORMAL_FONT_COLOR` \
             does not implement `WrapTextInColorCode` — both surface as the popup rendering \
             with no body text in real WoW."
        );

        let button1_is_string: bool = env
            .eval(&format!(
                "local p = StaticPopupDialogs[{TRANSACTION_ERROR_POPUP_KEY:?}]; \
                 return type(p.button1) == \"string\" and #p.button1 > 0"
            ))
            .expect("popup `button1` field probe must run cleanly");

        assert!(
            button1_is_string,
            "Expected `StaticPopupDialogs[{TRANSACTION_ERROR_POPUP_KEY:?}].button1` to be a \
             non-empty string after `{ROOT}` loads. The source declares it as the global locale \
             string `OKAY` at `Blizzard_AccountStoreUtil.lua:13`, which resolves to the localized \
             OK label at file-scope load time. A nil or empty reading means the `OKAY` global \
             was not registered by the locale tier — the popup would render with a blank button \
             label in real WoW."
        );

        let show_alert: bool = env
            .eval(&format!(
                "local p = StaticPopupDialogs[{TRANSACTION_ERROR_POPUP_KEY:?}]; \
                 return p.showAlert == true"
            ))
            .expect("popup `showAlert` field probe must run cleanly");

        assert!(
            show_alert,
            "Expected `StaticPopupDialogs[{TRANSACTION_ERROR_POPUP_KEY:?}].showAlert` to be \
             literally `true` after `{ROOT}` loads. The source sets it at \
             `Blizzard_AccountStoreUtil.lua:14`, which the StaticPopup framework reads to \
             render the alert icon (the warning triangle) on the popup frame. A regression \
             dropping this flag would render the popup as a plain dialog without the \
             error-severity visual cue."
        );

        let on_accept_is_nil: bool = env
            .eval(&format!(
                "local p = StaticPopupDialogs[{TRANSACTION_ERROR_POPUP_KEY:?}]; \
                 return p.OnAccept == nil"
            ))
            .expect("popup `OnAccept` absence probe must run cleanly");

        assert!(
            on_accept_is_nil,
            "Expected `StaticPopupDialogs[{TRANSACTION_ERROR_POPUP_KEY:?}].OnAccept` to be nil \
             after `{ROOT}` loads. PLAN.md's spec asks for an `OnAccept` handler, but the source \
             at `Blizzard_AccountStoreUtil.lua:11-18` does NOT define one — the popup is the \
             dismiss-only `text + button1 + showAlert + hideOnEscape + timeout + whileDead` \
             pattern (clicking OK closes the popup without dispatching follow-up action). This \
             positive nil assertion is the spec/source-mismatch tripwire: if a future Blizzard \
             revision adds an `OnAccept`, this test flips and forces a re-pin against the new \
             handler shape (and a corresponding behavior test to assert what `OnAccept` does)."
        );
    });
}
}

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn account_store_toggle_is_callable_and_flips_frame_shown_state() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let plan_named_global_type: String = env
            .eval("return type(_G[\"ToggleAccountStoreUI\"])")
            .expect("`ToggleAccountStoreUI` global probe must run cleanly");

        assert_eq!(
            plan_named_global_type, "nil",
            "Expected `_G[\"ToggleAccountStoreUI\"]` to be nil after `{ROOT}` loads, got \
             `{plan_named_global_type}`. PLAN.md's spec names `ToggleAccountStoreUI(storeFrontID)` \
             as a global, but the only `ToggleAccountStoreUI` symbol in the entire UI source is a \
             `local function ToggleAccountStoreUI()` (zero args, file-local scope) at \
             `Blizzard_CharacterSelectNavBar.lua:44`, whose body is a single redirect line: \
             `AccountStoreUtil.ToggleAccountStore()`. Local-scoped functions never escape into \
             `_G`, so the PLAN-named global is genuinely absent. This positive nil assertion is \
             the spec/source-mismatch tripwire: if Blizzard ever promotes the redirect to a \
             global with a `storeFrontID` parameter, this test flips and forces a re-pin against \
             the new public surface (and probably a behavior test for the parameter dispatch)."
        );

        let toggle_function_type: String = env
            .eval("return type(AccountStoreUtil.ToggleAccountStore)")
            .expect("`AccountStoreUtil.ToggleAccountStore` probe must run cleanly");

        assert_eq!(
            toggle_function_type, "function",
            "Expected `AccountStoreUtil.ToggleAccountStore` to be a function after `{ROOT}` \
             loads, got `{toggle_function_type}`. The actual public toggle entry point is \
             declared at `Blizzard_AccountStoreUtil.lua:52` and takes zero arguments — it flips \
             the frame state via `AccountStoreUtil.SetAccountStoreShown(not \
             AccountStoreFrame:IsShown())`. A nil reading here means the file-scope assignment \
             was dropped, which would break every external caller (the CharacterSelectNavBar \
             redirect, any addon that drives the account store from outside this lane)."
        );

        let initially_hidden: bool = env
            .eval("return AccountStoreFrame:IsShown() == false")
            .expect("initial `AccountStoreFrame:IsShown()` probe must run cleanly");

        assert!(
            initially_hidden,
            "Expected `AccountStoreFrame:IsShown()` to be false before any toggle is invoked. \
             The XML at `Blizzard_AccountStore.xml:78` declares the frame with `hidden=\"true\"` \
             so it MUST start hidden after `{ROOT}` finishes loading. A regression that auto-shows \
             the frame on load (e.g., a stray `:Show()` in OnLoad) would invert the toggle test \
             below — both toggle calls would still flip state, but the start-hidden contract \
             would be silently violated, and any addon that relies on `IsShown()` to gate UI \
             chrome would render incorrectly until the user manually closed the panel."
        );

        env.eval::<()>("AccountStoreUtil.ToggleAccountStore(); return")
            .expect("first `ToggleAccountStore()` call must run cleanly");

        let shown_after_first_toggle: bool = env
            .eval("return AccountStoreFrame:IsShown() == true")
            .expect("post-first-toggle `IsShown()` probe must run cleanly");

        assert!(
            shown_after_first_toggle,
            "Expected `AccountStoreFrame:IsShown()` to flip to true after the first \
             `AccountStoreUtil.ToggleAccountStore()` call. The toggle reads the frame's current \
             shown state (false at this point — pinned by the prior assertion) and routes to \
             `SetAccountStoreShown(true)`, which calls `ShowUIPanel(AccountStoreFrame)` if the \
             panel manager is available, falling back to `AccountStoreFrame:Show()` otherwise. A \
             false reading here means either the toggle no-opped (a regression in \
             `SetAccountStoreShown`'s if-already-equal early return), or `ShowUIPanel` failed \
             to mark the frame shown (the panel-manager baseline pre-loaded by the harness \
             should make this path work — `tests/common/panel_fixtures.rs:53-56`)."
        );

        env.eval::<()>("AccountStoreUtil.ToggleAccountStore(); return")
            .expect("second `ToggleAccountStore()` call must run cleanly");

        let hidden_after_second_toggle: bool = env
            .eval("return AccountStoreFrame:IsShown() == false")
            .expect("post-second-toggle `IsShown()` probe must run cleanly");

        assert!(
            hidden_after_second_toggle,
            "Expected `AccountStoreFrame:IsShown()` to flip back to false after the second \
             `AccountStoreUtil.ToggleAccountStore()` call. The double-toggle round-trip is the \
             core contract: a single toggle could spuriously land in the right state even if \
             the function were buggy (e.g., a stuck-on-true regression), but two toggles MUST \
             return the frame to its starting state. A true reading here means the hide path \
             (`HideUIPanel(AccountStoreFrame)` falling back to `AccountStoreFrame:Hide()`) \
             regressed — the frame would stay open after the user clicked the close button or \
             pressed escape, breaking the dismiss flow."
        );
    });
}
}
