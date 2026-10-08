//! Mists legacy lookup and product-choice defaults remain intact.
#![cfg(feature = "client-mists")]

#[test]
fn patch_8_3_0_mists_preserves_legacy_lookup() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(r#"
assert(type(C_ProductChoice.GetChoices()) == "table")
assert(type(C_ProductChoice.GetProducts(123)) == "table")
assert(C_ProductChoice.GetNumSuppressed() == 0)
assert(C_ProductChoice.MakeSelection() == false)
assert(type(C_WowTokenPublic.SellToken) == "function")
assert(type(GetAuctionHouseDepositRate) == "function")
assert(GetAuctionHouseDepositRate() == 0)
"#).unwrap();
}
