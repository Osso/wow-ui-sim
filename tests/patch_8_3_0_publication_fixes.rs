//! Removed namespace members stay absent under repeated retail lookup.
#![cfg(feature = "client-retail")]

pub(crate) const RETIREMENT_ASSERTIONS: &str = r#"
assert(rawget(_G, "GetAuctionHouseDepositRate") == nil, "GetAuctionHouseDepositRate raw")
assert(GetAuctionHouseDepositRate == nil, "GetAuctionHouseDepositRate lookup")
assert(GetAuctionHouseDepositRate == nil, "GetAuctionHouseDepositRate repeat")
for _, symbol in ipairs({
    "C_ProductChoice.GetChoices",
    "C_ProductChoice.GetNumSuppressed",
    "C_ProductChoice.GetProducts",
    "C_ProductChoice.MakeSelection",
    "C_WowTokenPublic.SellToken",
}) do
    local namespace, member = string.match(symbol, "^([^.]+)%.(.+)$")
    local table = _G[namespace]
    assert(type(table) == "table", namespace)
    assert(rawget(table, member) == nil, symbol .. " raw")
    assert(table[member] == nil, symbol .. " lookup")
    assert(table[member] == nil, symbol .. " repeat")
end
"#;

#[test]
fn patch_8_3_0_unused_members_stay_absent() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(RETIREMENT_ASSERTIONS).unwrap();
}
