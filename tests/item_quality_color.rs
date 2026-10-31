use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn item_quality_color_reads_current_globals() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local names = {'ITEM_POOR_COLOR', 'ITEM_STANDARD_COLOR', 'ITEM_GOOD_COLOR',
            'ITEM_SUPERIOR_COLOR', 'ITEM_EPIC_COLOR', 'ITEM_LEGENDARY_COLOR',
            'ITEM_ARTIFACT_COLOR', 'ITEM_SCALING_STAT_COLOR', 'ITEM_WOW_TOKEN_COLOR'}
        for i, name in ipairs(names) do
            _G[name] = CreateColor(i / 16, (i + 1) / 16, (i + 2) / 16, 1)
        end
        for i, name in ipairs(names) do
            local r, g, b, hex = C_Item.GetItemQualityColor(i - 1)
            local color = _G[name]
            assert(r == color.r and g == color.g and b == color.b, name)
            assert(type(hex) == 'string', name)
        end
        ITEM_EPIC_COLOR = CreateColor(0.25, 0.5, 0.75, 1)
        local r, g, b = C_Item.GetItemQualityColor(4)
        assert(r == 0.25 and g == 0.5 and b == 0.75, 'updated epic')
        ITEM_EPIC_COLOR = nil
        for _, quality in ipairs({-1, 4, 9}) do
            r, g, b = C_Item.GetItemQualityColor(quality)
            assert(r == 1 and g == 0 and b == 1, 'fallback ' .. quality)
        end
    "#,
    )
    .expect("item quality colors reflect current globals and magenta fallback");
}

#[cfg(feature = "client-mists")]
#[test]
fn mists_ui_parent_builds_colors_from_current_globals() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local names = {'ITEM_POOR_COLOR', 'ITEM_STANDARD_COLOR', 'ITEM_GOOD_COLOR',
            'ITEM_SUPERIOR_COLOR', 'ITEM_EPIC_COLOR', 'ITEM_LEGENDARY_COLOR',
            'ITEM_ARTIFACT_COLOR', 'ITEM_SCALING_STAT_COLOR', 'ITEM_WOW_TOKEN_COLOR'}
        for i, name in ipairs(names) do
            _G[name] = CreateColor(i / 16, (i + 1) / 16, (i + 2) / 16, 1)
        end
        NUM_LE_ITEM_QUALITYS = 9
    "#,
    )
    .unwrap();
    let root = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(
        env!("CARGO_MANIFEST_DIR"),
    ));
    let source =
        std::fs::read_to_string(root.join("Blizzard_UIParent/Mists/UIParent.lua")).unwrap();
    env.exec(&source)
        .expect("execute selected original Mists UIParent consumer");
    env.exec(r#"
        for i, name in ipairs({'ITEM_POOR_COLOR', 'ITEM_STANDARD_COLOR', 'ITEM_GOOD_COLOR',
            'ITEM_SUPERIOR_COLOR', 'ITEM_EPIC_COLOR', 'ITEM_LEGENDARY_COLOR',
            'ITEM_ARTIFACT_COLOR', 'ITEM_SCALING_STAT_COLOR', 'ITEM_WOW_TOKEN_COLOR'}) do
            local actual, expected = ITEM_QUALITY_COLORS[i - 1], _G[name]
            assert(actual.r == expected.r and actual.g == expected.g and actual.b == expected.b, name)
        end
    "#).expect("UIParent quality entries use named colors");
}
