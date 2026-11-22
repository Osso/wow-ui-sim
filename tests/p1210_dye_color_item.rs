#![cfg(any(feature = "retail-12-0-0", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn p1210_dye_colors_for_items_and_item_locations_use_supplied_lists() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        A_Admin.SetDyeColorsForItem(900001, '501,502')
        A_Admin.SetDyeColorsForItem(900002, '601')
        A_Admin.AddBagItem(0, 1, 900001, 1)
        A_Admin.AddBagItem(0, 2, 900002, 1)

        assert(table.concat(C_DyeColor.GetDyeColorsForItem(900001), ',') == '501,502')
        assert(table.concat(C_DyeColor.GetDyeColorsForItem(900002), ',') == '601')
        assert(table.concat(C_DyeColor.GetDyeColorsForItem(900003), ',') == '')
        assert(table.concat(C_DyeColor.GetDyeColorsForItemLocation({bagID = 1, slotIndex = 1}), ',') == '501,502')
        assert(table.concat(C_DyeColor.GetDyeColorsForItemLocation({bagID = 1, slotIndex = 2}), ',') == '601')
    "#,
    )
    .unwrap();
}
