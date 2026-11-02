//! API-level test: no pinned retail Blizzard consumer calls GetItemFamily.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn item_family_distinguishes_supplied_zero_nonzero_and_cache_miss() {
    let env = WowLuaEnv::new().unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.item_families.insert(6948, 0);
        sim.item_families.insert(7005, 136);
    }
    env.exec(
        r#"
        assert(C_Item.GetItemFamily(6948) == 0)
        assert(C_Item.GetItemFamily('item:7005') == 136)
        assert(select('#', C_Item.GetItemFamily(999999)) == 1)
        assert(C_Item.GetItemFamily(999999) == nil)
        assert(not pcall(C_Item.GetItemFamily))
    "#,
    )
    .unwrap();
    env.state().borrow_mut().item_families.insert(7005, 8);
    env.exec("assert(C_Item.GetItemFamily(7005) == 8)").unwrap();
}
