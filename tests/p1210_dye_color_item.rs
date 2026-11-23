#![cfg(any(feature = "retail-12-0-0", feature = "client-wowforever"))]

#[test]
fn p1210_dye_colors_for_items_and_item_locations_use_supplied_lists() {
    let env = crate::common::panel_fixtures::setup_env();
    let ui = crate::common::panel_fixtures::blizzard_ui_dir();
    crate::common::load_required_blizzard_addon(&env, &ui, "Blizzard_Deprecated");
    env.exec(
        r#"
        A_Admin.SetDyeColorsForItem(900001, {501, 502})
        A_Admin.SetDyeColorsForItem(900002, {601})
        A_Admin.AddBagItem(0, 1, 900001, 1)
        A_Admin.AddBagItem(0, 2, 900002, 1)

        assert(table.concat(C_DyeColor.GetDyeColorsForItem(900001), ',') == '501,502')
        assert(table.concat(C_DyeColor.GetDyeColorsForItem(900002), ',') == '601')
        assert(table.concat(C_DyeColor.GetDyeColorsForItem(900003), ',') == '')
        assert(C_DyeColor.GetDyeColorForItem(900001) == 501)
        assert(C_DyeColor.GetDyeColorForItem(900002) == 601)
        local first = ItemLocation:CreateFromBagAndSlot(0, 1)
        local second = ItemLocation:CreateFromBagAndSlot(0, 2)
        assert(table.concat(C_DyeColor.GetDyeColorsForItemLocation(first), ',') == '501,502')
        assert(table.concat(C_DyeColor.GetDyeColorsForItemLocation(second), ',') == '601')
        assert(C_DyeColor.GetDyeColorForItemLocation(first) == 501)
        assert(C_DyeColor.GetDyeColorForItemLocation(second) == 601)
    "#,
    )
    .unwrap();
}
