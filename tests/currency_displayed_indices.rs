#![cfg(feature = "client-retail")]

use crate::common;
use common::blizzard_addon_harness::with_blizzard_addon_smoke_shape_in_dir;

// Original cached TokenUI.lua:23-31 (header), 363-404 (refresh), 459-475 (popup).
// No vendor overrides, copied handlers, or namespace replacements.
pub(crate) fn with_token_ui(probe: &str) {
    let cache = dirs::home_dir()
        .expect("home directory")
        .join(".cache/wow-ui-sim/blizzard-ui/retail/AddOns");
    assert!(
        cache
            .join("Blizzard_TokenUI/Blizzard_TokenUI.lua")
            .is_file()
    );
    let probe = probe.to_owned();
    common::with_perf_lock(|| {
        common::with_timeout(90, move || {
            with_blizzard_addon_smoke_shape_in_dir(
                &cache,
                &["Blizzard_TokenUI"],
                &[],
                |env, loaded| {
                    assert!(loaded.iter().any(|name| name == "Blizzard_TokenUI"));
                    env.exec(include_str!("fixtures/currency_displayed_indices.lua"))
                        .expect("original TokenUI fixture setup");
                    env.exec(&probe).expect("currency consumer assertions");
                },
            );
        });
    });
}

#[test]
fn currency_filter_account_data_query_allows_original_token_ui_update() {
    with_token_ui(
        r#"
        assert(type(C_CurrencyInfo.DoesCurrentFilterRequireAccountCurrencyData()) == "boolean")
        TokenFrame:Update()
        B5AssertProvider()
        "#,
    );
}

#[test]
fn currency_displayed_indices_original_headers_collapse_shift_and_restore() {
    with_token_ui(
        r#"
        local before = B5Snapshot()
        assert(C_CurrencyInfo.GetCurrencyListSize() == 19)
        local first = B5Header(1)
        TokenFrame.selectedToken = C_CurrencyInfo.GetCurrencyListInfo(2).name
        TokenFrame.selectedID = 2
        first:ToggleCollapsed()
        assert(C_CurrencyInfo.GetCurrencyListSize() == 14, "B5: children must disappear")
        assert(C_CurrencyInfo.GetCurrencyListInfo(1).isHeaderExpanded == false)
        assert(C_CurrencyInfo.GetCurrencyListInfo(2).name == "Player vs. Player")
        assert(C_CurrencyInfo.GetCurrencyListInfo(2).currencyListDepth == 0)
        assert(C_CurrencyInfo.GetCurrencyListInfo(3).currencyListDepth == 1)
        assert(C_CurrencyInfo.GetCurrencyListInfo(3).currencyTypesID == 1792)
        assert(TokenFrame.selectedToken == nil and TokenFrame.selectedID == nil)
        assert(not TokenFramePopup:IsShown())
        assert(C_CurrencyInfo.GetCurrencyInfo(2245).quantity == 1847)
        assert(C_CurrencyInfo.GetBackpackCurrencyInfo(1).currencyTypesID == 2245)
        B5AssertProvider()

        -- Resolve the second header from CURRENT displayed index, not catalog index 7.
        local second = B5Header(2)
        second:ToggleCollapsed()
        assert(C_CurrencyInfo.GetCurrencyListSize() == 8)
        assert(C_CurrencyInfo.GetCurrencyListInfo(3).name == "Miscellaneous")
        assert(C_CurrencyInfo.GetCurrencyListInfo(4).currencyTypesID == 1191)
        B5AssertProvider()
        B5Header(2):ToggleCollapsed()
        assert(C_CurrencyInfo.GetCurrencyListSize() == 14)
        B5Header(1):ToggleCollapsed()
        assert(B5Snapshot() == before, "B5: expansion must restore exact rows")
        B5AssertProvider()
        "#,
    );
}

#[test]
fn currency_displayed_indices_noop_targets_and_environment_isolation() {
    with_token_ui(
        r#"
        local before = B5Snapshot()
        for _, index in ipairs({0, -1, 20, 2}) do
            C_CurrencyInfo.ExpandCurrencyList(index, false)
        end
        C_CurrencyInfo.ExpandCurrencyList(1, true)
        assert(B5Snapshot() == before)
        C_CurrencyInfo.ExpandCurrencyList(1, false)
        local collapsed = B5Snapshot()
        C_CurrencyInfo.ExpandCurrencyList(1, false)
        assert(B5Snapshot() == collapsed)
        assert(C_CurrencyInfo.GetCurrencyListInfo(0) == nil)
        assert(C_CurrencyInfo.GetCurrencyListInfo(15) == nil)
        "#,
    );
    // A fresh environment must not inherit expansion state from the preceding one.
    with_token_ui(
        r#"
        assert(C_CurrencyInfo.GetCurrencyListSize() == 19)
        assert(C_CurrencyInfo.GetCurrencyListInfo(1).isHeaderExpanded == true)
        "#,
    );
}
