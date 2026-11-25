#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn transfer_calculation_apis_require_currency_id() {
    let env = WowLuaEnv::new().expect("simulator should initialize");
    env.exec(
        r#"
            local maxOk, maxError = pcall(
                C_CurrencyInfo.GetMaxTransferableAmountFromQuantity,
                nil,
                10
            )
            assert(not maxOk)
            assert(string.find(
                maxError,
                "Usage: local maxTransferableAmount = C_CurrencyInfo.GetMaxTransferableAmountFromQuantity(currencyID, requestedQuantity)",
                1,
                true
            ))

            local costOk, costError = pcall(
                C_CurrencyInfo.GetCostToTransferCurrency,
                nil,
                10
            )
            assert(not costOk)
            assert(string.find(
                costError,
                "Usage: local totalQuantityConsumed = C_CurrencyInfo.GetCostToTransferCurrency(currencyID, quantity)",
                1,
                true
            ))
        "#,
    )
    .expect("both transfer calculation APIs should reject a missing currency ID");
}
