//! API-level coverage: no retail Blizzard consumer calls C_Item.ItemHasRange.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn item_range_selects_the_use_spell_range_index_and_requires_the_player() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().player.present = true;
    env.exec(
        r#"
        -- Sky Chain: spell 346772 is neither helpful nor harmful, so index 0
        -- of range 318 applies: (3, 10). Index 1 is (0, 0).
        assert(C_Item.ItemHasRange(184586))
        -- Lifestone: helpful spell 17712 on range 1, (0, 0) at both indexes.
        assert(not C_Item.ItemHasRange(833))
        -- Harmful spell 13424 on range 4: index 0 is (0, 30).
        assert(C_Item.ItemHasRange('item:1434'))
        assert(not C_Item.ItemHasRange(6948))
        -- No on-use spell.
        assert(not C_Item.ItemHasRange(211993))
        assert(select('#', C_Item.ItemHasRange(184586)) == 1)
        assert(not pcall(C_Item.ItemHasRange))
    "#,
    )
    .unwrap();
    env.state().borrow_mut().player.present = false;
    env.exec("assert(not C_Item.ItemHasRange(184586))").unwrap();
}
