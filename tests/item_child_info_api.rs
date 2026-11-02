//! API-level test: no pinned retail Blizzard consumer calls GetItemChildInfo.
#![cfg(feature = "client-retail")]

use wow_ui_sim::c_api::item_spell::item_child::ItemChildEquipment;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn item_child_info_selects_supplied_rows_and_returns_flat_pairs() {
    let env = WowLuaEnv::new().unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.item_children.insert(
            6948,
            vec![
                ItemChildEquipment {
                    slot_index: 0,
                    child_item_id: 101,
                },
                ItemChildEquipment {
                    slot_index: 1,
                    child_item_id: 102,
                },
            ],
        );
    }
    env.exec(
        r#"
        local all = C_Item.GetItemChildInfo('item:6948')
        assert(#all == 4 and all[1] == 101 and all[2] == 0 and all[3] == 102 and all[4] == 1)
        local second = C_Item.GetItemChildInfo(6948, 2)
        assert(#second == 2 and second[1] == 102 and second[2] == 1)
        assert(#C_Item.GetItemChildInfo(6948, 3) == 0)
        assert(#C_Item.GetItemChildInfo(7005) == 0)
        assert(not pcall(C_Item.GetItemChildInfo))
    "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .item_children
        .get_mut(&6948)
        .unwrap()[1]
        .child_item_id = 103;
    env.exec("local v = C_Item.GetItemChildInfo(6948, 2); assert(v[1] == 103 and v[2] == 1)")
        .unwrap();
}
