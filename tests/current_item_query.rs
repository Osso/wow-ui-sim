//! API-level coverage: no selected Retail FrameXML consumer was found.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn current_item_query_reads_supplied_resolved_item_list() {
    let env = WowLuaEnv::new().unwrap();
    {
        let state = env.state();
        state.borrow_mut().current_item_ids.insert(6948);
    }
    env.exec(
        r#"
        assert(select('#', C_Item.IsCurrentItem(6948)) == 1)
        assert(C_Item.IsCurrentItem(6948) == true)
        assert(C_Item.IsCurrentItem('item:6948') == true)
        assert(C_Item.IsCurrentItem('Hearthstone') == true)
        assert(C_Item.IsCurrentItem(0) == false)
        assert(C_Item.IsCurrentItem(159) == false)
        assert(C_Item.IsCurrentItem('no such item WM23') == false)
    "#,
    )
    .unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.current_item_ids.clear();
        sim.current_item_ids.insert(159);
    }
    env.exec(
        r#"
        assert(C_Item.IsCurrentItem(6948) == false)
        assert(C_Item.IsCurrentItem(159) == true)
    "#,
    )
    .unwrap();
}
