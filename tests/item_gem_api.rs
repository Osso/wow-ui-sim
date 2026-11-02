//! API-level coverage: the pinned retail UI has no GetItemGem consumer.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn item_gem_returns_selected_cached_gem_and_no_values_for_missing(env: &WowLuaEnv) {
    env.exec(r#"
        local source = '|cffff0000|Hitem:211993:7:22459:23094:0:0:0:0:80:70:0:12:1:999999:0:0:0:0::0|h[Fake Parent Label]|h|r'
        local name1, link1 = C_Item.GetItemGem(source, 1)
        assert(name1 == 'Void Sphere', tostring(name1))
        assert(link1:find('|cnIQ4:|Hitem:22459:', 1, true), link1)
        assert(not link1:find('Fake Parent Label', 1, true), link1)
        assert(not link1:find('999999', 1, true), link1)
        local name2, link2 = C_Item.GetItemGem(source, 2)
        assert(name2 == 'Brilliant Blood Garnet', tostring(name2))
        assert(link2:find('|cnIQ2:|Hitem:23094:', 1, true), link2)
        assert(select('#', C_Item.GetItemGem(source, 3)) == 0)
        assert(select('#', C_Item.GetItemGem(source, 4)) == 0)
        assert(select('#', C_Item.GetItemGem('item:1:0:999999', 1)) == 0)
        assert(not pcall(C_Item.GetItemGem, source, -1))
    "#).unwrap();
}
}
