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
        r##"
        local C = C_CurrencyInfo
        assert(C.GetCurrencyFilter() == 1)
        assert(C.DoesCurrentFilterRequireAccountCurrencyData() == false)
        assert(C.IsAccountCharacterCurrencyDataReady() == true)
        local updates = 0
        local watcher = CreateFrame("Frame")
        watcher:RegisterEvent("CURRENCY_DISPLAY_UPDATE")
        watcher:SetScript("OnEvent", function() updates = updates + 1 end)
        for _, case in ipairs({{0, false}, {1.9, false}, {2, true}, {2, true}, {false, false}, {{}, false}}) do
            assert(select("#", C.SetCurrencyFilter(case[1])) == 0)
            assert(C.DoesCurrentFilterRequireAccountCurrencyData() == case[2])
            TokenFrame:Update()
            B5AssertProvider()
        end
        assert(C.GetCurrencyFilter() == 0 and updates == 6)
        for _, invalid in ipairs({-1, 3, 2147483648}) do
            assert(not pcall(C.SetCurrencyFilter, invalid))
        end
        assert(not pcall(C.SetCurrencyFilter))
        assert(not pcall(C.SetCurrencyFilter, nil))
        assert(C.GetCurrencyFilter() == 0 and updates == 6)
        "##,
    );
}

#[test]
fn currency_filter_supplied_account_response_defers_projection() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.currency_list.account_currency_request = true;
        let mut info = state.currency_info.get(&2806).unwrap().clone();
        info.name = "Supplied account currency".into();
        state.currency_info.insert(990001, info);
    }
    env.exec(r#"
        local watcher = CreateFrame("Frame")
        watcher:RegisterEvent("ACCOUNT_CHARACTER_CURRENCY_DATA_RECEIVED")
        watcher:SetScript("OnEvent", function() AccountEvents = (AccountEvents or 0) + 1 end)
        C_CurrencyInfo.SetCurrencyFilter(2)
        assert(not C_CurrencyInfo.IsAccountCharacterCurrencyDataReady())
        assert(C_CurrencyInfo.GetCurrencyListSize() == 19)
        C_CurrencyInfo.SetCurrencyFilter(1)
        assert(not C_CurrencyInfo.IsAccountCharacterCurrencyDataReady())
        C_CurrencyInfo.SetCurrencyFilter(2)
    "#).unwrap();
    let mut row = *wow_ui_sim::lua_api::globals::currency_data::get_currency_list_entry(3).unwrap();
    row.currency_id = 990001;
    env.complete_account_currency_request(vec![row]).unwrap();
    env.exec(r#"
        assert(AccountEvents == 1)
        assert(C_CurrencyInfo.IsAccountCharacterCurrencyDataReady())
        assert(C_CurrencyInfo.GetCurrencyListSize() == 20)
        assert(C_CurrencyInfo.GetCurrencyListInfo(20).name == "Supplied account currency")
        C_CurrencyInfo.SetCurrencyFilter(1)
        assert(C_CurrencyInfo.GetCurrencyListSize() == 19)
    "#).unwrap();
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
