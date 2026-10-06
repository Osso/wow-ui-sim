#![cfg(feature = "retail-12-0-0")]
use wow_ui_sim::c_api::c_dye_color::DyeColorRecord;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::BagItem;

#[test]
fn p1127_dye_colors_read_catalog_inventory_and_fresh_color_snapshots() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.dye_colors
            .categories
            .extend([(17, "Warm".into()), (22, "Cool".into())]);
        for (id, category, item) in [
            (51, 17, Some(600001)),
            (52, 17, Some(600002)),
            (55, 22, Some(600003)),
            (56, 22, None),
        ] {
            sim.dye_colors.colors.insert(
                id,
                DyeColorRecord {
                    category_id: category,
                    name: format!("Dye {id}"),
                    sort_order: id - 50,
                    swatch_start: [0.2, 0.4, 0.6],
                    swatch_end: [0.8, 0.6, 0.4],
                    item_id: item,
                },
            );
        }
        sim.bag_items.insert(
            (0, 1),
            BagItem {
                item_id: 600001,
                stack_count: 2,
                hyperlink: None,
            },
        );
        sim.bag_items.insert(
            (-1, 1),
            BagItem {
                item_id: 600001,
                stack_count: 3,
                hyperlink: None,
            },
        );
        sim.bag_items.insert(
            (-1, 2),
            BagItem {
                item_id: 600003,
                stack_count: 4,
                hyperlink: None,
            },
        );
        sim.guild_bank_items.insert(
            (1, 1),
            BagItem {
                item_id: 600002,
                stack_count: 9,
                hyperlink: None,
            },
        );
    }
    env.exec(
        r#"
        assert(table.concat(C_DyeColor.GetAllDyeColorCategories(), ',') == '17,22')
        assert(table.concat(C_DyeColor.GetAllDyeColors(), ',') == '51,52,55,56')
        assert(table.concat(C_DyeColor.GetAllDyeColors(true), ',') == '51,55')
        assert(table.concat(C_DyeColor.GetDyeColorsInCategory(17), ',') == '51,52')
        assert(table.concat(C_DyeColor.GetDyeColorsInCategory(17, true), ',') == '51')
        assert(C_DyeColor.IsDyeColorOwned(51) and not C_DyeColor.IsDyeColorOwned(52))
        local category = C_DyeColor.GetDyeColorCategoryInfo(17)
        assert(category.ID == 17 and category.name == 'Warm')
        local original = CreateColor
        CreateColor = function(...) collectgarbage('collect'); return original(...) end
        local color = C_DyeColor.GetDyeColorInfo(51)
        assert(color.ID == 51 and color.dyeColorCategoryID == 17 and color.name == 'Dye 51')
        assert(color.sortOrder == 1 and color.itemID == 600001 and color.numOwned == 5)
        local r, g, b = color.swatchColorStart:GetRGB()
        assert(r == 0.2 and g == 0.4 and b == 0.6)
        r, g, b = color.swatchColorEnd:GetRGB()
        assert(r == 0.8 and g == 0.6 and b == 0.4)
        color.swatchColorStart.r = 0
        category.name = 'Mutation'
        assert(C_DyeColor.GetDyeColorCategoryInfo(17).name == 'Warm')
        assert(C_DyeColor.GetDyeColorInfo(51).swatchColorStart:GetRGB() == 0.2)
        assert(C_DyeColor.GetDyeColorInfo(56).itemID == nil)
        assert(C_DyeColor.GetDyeColorInfo(56).numOwned == 0)
        assert(not pcall(C_DyeColor.GetAllDyeColors, 1))
        CreateColor = function() error('color callback failure') end
        local ok, message = pcall(C_DyeColor.GetDyeColorInfo, 51)
        assert(not ok and string.find(message, 'color callback failure', 1, true))
        CreateColor = original
    "#,
    )
    .unwrap();
    env.state().borrow_mut().bag_items.clear();
    env.exec("assert(not C_DyeColor.IsDyeColorOwned(51)); assert(C_DyeColor.GetDyeColorInfo(51).numOwned == 0); assert(#C_DyeColor.GetAllDyeColors(true) == 0)").unwrap();
}

#[test]
fn p1127_dye_colors_empty_catalog_has_no_invented_records() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(#C_DyeColor.GetAllDyeColorCategories() == 0)
        assert(#C_DyeColor.GetAllDyeColors() == 0)
        assert(#C_DyeColor.GetAllDyeColors(true) == 0)
        assert(C_DyeColor.GetDyeColorCategoryInfo(81001) == nil)
        assert(C_DyeColor.GetDyeColorInfo(81002) == nil)
        assert(#C_DyeColor.GetDyeColorsInCategory(81001) == 0)
        assert(C_DyeColor.IsDyeColorOwned(81002) == false)
    "#,
    )
    .unwrap();
}
