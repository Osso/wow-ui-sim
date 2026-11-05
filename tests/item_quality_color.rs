use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn item_quality_color_reads_current_globals() {
    let env = WowLuaEnv::new().unwrap();
    let getter = if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::Wrath { "GetItemQualityColor" } else { "C_Item.GetItemQualityColor" };
    env.exec(&format!("__test_quality_getter = {getter}")).unwrap();
    env.exec(
        r#"
        local names = {'ITEM_POOR_COLOR', 'ITEM_STANDARD_COLOR', 'ITEM_GOOD_COLOR',
            'ITEM_SUPERIOR_COLOR', 'ITEM_EPIC_COLOR', 'ITEM_LEGENDARY_COLOR',
            'ITEM_ARTIFACT_COLOR', 'ITEM_SCALING_STAT_COLOR', 'ITEM_WOW_TOKEN_COLOR'}
        for i, name in ipairs(names) do
            _G[name] = CreateColor(i / 16, (i + 1) / 16, (i + 2) / 16, 1)
        end
        local getter = __test_quality_getter
        for i, name in ipairs(names) do
            local r, g, b, hex = getter(i - 1)
            local color = _G[name]
            assert(r == color.r and g == color.g and b == color.b, name)
            assert(type(hex) == 'string', name)
        end
        ITEM_EPIC_COLOR = CreateColor(0.25, 0.5, 0.75, 1)
        local r, g, b = getter(4)
        assert(r == 0.25 and g == 0.5 and b == 0.75, 'updated epic')
        ITEM_EPIC_COLOR = nil
        r, g, b = getter(4)
        local cleared = _G.ITEM_EPIC_COLOR
        assert(r == cleared.r and g == cleared.g and b == cleared.b,
            'cleared epic follows global indexing')
        for _, quality in ipairs({-1, 9}) do
            r, g, b = getter(quality)
            assert(r == 1 and g == 0 and b == 1, 'fallback ' .. quality)
        end
    "#,
    )
    .expect("item quality colors reflect current globals and magenta fallback");
}

/// The native function resolves the named colour globals (see
/// `native-evidence-70688/GetItemQualityColor.md` sections 3 and 7), so a
/// consumer must observe the same value as a Lua read of the global.
#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
#[test]
fn merchant_product_info_reports_the_indexed_quality_color() {
    let env = WowLuaEnv::new().unwrap();
    // The consumer registers itself with the player interaction manager when
    // the file loads; that subsystem is outside this test's scope.
    env.exec("RegisterPlayerInteraction = function() end")
        .unwrap();

    let addons = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")),
    );
    let consumer_path = addons.join("Blizzard_UIPanels_Game/Mainline/MerchantFrame.lua");
    let source = std::fs::read_to_string(&consumer_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", consumer_path.display()));
    env.exec(&source)
        .expect("execute the original MerchantFrame_GetProductInfo consumer");

    let mismatch: Option<String> = env
        .eval(
            r#"
            local itemButton = {
                name = "Test Item",
                texture = 0,
                GetID = function() return 1 end,
            }
            local productInfo = MerchantFrame_GetProductInfo(itemButton)
            local actual = productInfo.color
            local expected = _G.ITEM_STANDARD_COLOR
            if actual[1] == expected.r and actual[2] == expected.g and actual[3] == expected.b then
                return nil
            end
            return string.format("expected (%.3f, %.3f, %.3f), got (%.3f, %.3f, %.3f)",
                expected.r, expected.g, expected.b, actual[1], actual[2], actual[3])
        "#,
        )
        .expect("merchant product colour must be readable");

    assert!(
        mismatch.is_none(),
        "MerchantFrame_GetProductInfo colour must match ITEM_STANDARD_COLOR: {}",
        mismatch.unwrap_or_default()
    );
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
