#![cfg(feature = "retail-12-0-0")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn p1200_rest_item_binding_uses_existing_metadata() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        -- Existing generated metadata: Veilroot Fountain has ItemBind value 9.
        assert(C_Item.GetItemInfo(253451) ~= nil)
        assert(C_Item.IsItemBindToAccount(253451) == true)
        assert(C_Item.IsItemBindToAccount('item:253451') == true)
        assert(C_Item.IsItemBindToAccount(19019) == false)
        assert(C_Item.IsItemBindToAccount(99999999) == false)
    "#,
    )
    .unwrap();
}
