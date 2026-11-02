//! Retail MerchantFrame_GetProductInfo consumes the filtered specialization list.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn merchant_product_info_returns_class_filtered_specs(env: &WowLuaEnv) {
    {
        let mut sim = env.state().borrow_mut();
        sim.player.level = 80;
        sim.player.class_index = 2;
    }
    env.exec(r#"
        C_AddOns.LoadAddOn('Blizzard_MerchantUI')
        local source = debug.getinfo(MerchantFrame_GetProductInfo, 'S').source:gsub('\\', '/')
        assert(source:find('Blizzard_UIPanels_Game/Mainline/MerchantFrame.lua', 1, true), source)
        local button = CreateFrame('Button')
        button:SetID(1)
        button.link = select(2, C_Item.GetItemInfo(80773))
        assert(button.link)
        local product, specs = MerchantFrame_GetProductInfo(button)
        assert(product.link == button.link)
        assert(#specs == 1 and specs[1] == 65)
    "#).unwrap();
}
}
