#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn transfer_calculation_apis_handle_unselected_currency() {
    let env = WowLuaEnv::new().expect("simulator should initialize");
    env.exec(
        r#"
            assert(C_CurrencyInfo.GetMaxTransferableAmountFromQuantity(nil, 10) == nil,
                "an unselected currency has no maximum")
            assert(C_CurrencyInfo.GetMaxTransferableAmountFromQuantity(1560, 100) == 76,
                "currency 1560 uses its supplied transfer rate")
            assert(C_CurrencyInfo.GetMaxTransferableAmountFromQuantity(1792, 100) == 79,
                "currency 1792 uses its supplied transfer rate")

            local costOk, costError = pcall(C_CurrencyInfo.GetCostToTransferCurrency, nil, 10)
            assert(not costOk)
            assert(string.find(
                costError,
                "Usage: local totalQuantityConsumed = C_CurrencyInfo.GetCostToTransferCurrency(currencyID, quantity)",
                1,
                true
            ))
        "#,
    )
    .expect("max transfer should handle the unselected currency and supplied rates");
}
