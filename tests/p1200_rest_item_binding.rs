#![cfg(feature = "retail-12-0-0")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn p1200_rest_item_binding_uses_existing_metadata() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        -- ItemSparse 12.1.5.70391 enUS: Glowing Zangarshroom Cup has Bonding 9.
        assert(C_Item.GetItemInfo(247776) ~= nil)
        assert(C_Item.IsItemBindToAccount(247776) == true)
        assert(C_Item.IsItemBindToAccount('item:247776') == true)
        assert(C_Item.IsItemBindToAccount(19019) == false)
        assert(C_Item.IsItemBindToAccount(99999999) == false)
    "#,
    )
    .unwrap();
}
