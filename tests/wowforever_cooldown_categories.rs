//! Forever cooldown category publication and the real settings data provider.

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_cooldown_categories_initialize_vendor_provider() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local names = {"Essential", "Utility", "TrackedBuff", "TrackedBar",
            "GroupBuff", "SpecAgnosticEssential", "SpecAgnosticTracked",
            "EquipSlotEssential", "EquipSlotTracked"}
        for index, name in ipairs(names) do
            assert(Enum.CooldownViewerCategory[name] == index - 1, name)
        end
        assert(Enum.CooldownViewerCategoryMeta.MinValue == 0)
        assert(Enum.CooldownViewerCategoryMeta.MaxValue == 8)
        assert(Enum.CooldownViewerCategoryMeta.NumValues == 9)
        "#,
    )
    .unwrap();
    let root = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    for file in [
        "Blizzard_SharedXMLBase/TableUtil.lua",
        "Blizzard_CooldownViewer/CooldownViewerSettingsConstants.lua",
        "Blizzard_CooldownViewer/CooldownViewerSettingsDataProvider.lua",
    ] {
        env.exec(&std::fs::read_to_string(root.join(file)).unwrap())
            .unwrap();
    }
    env.exec(
        r#"
        local categories = CooldownViewerSettingsDataProvider_GetCategories()
        local expected = {0, 1, 2, 3, 7, 8, 5, 6}
        assert(#categories == #expected)
        for index, category in ipairs(expected) do
            assert(categories[index] == category)
        end
        assert(Enum.CooldownViewerCategory.HiddenActive == -1)
        assert(Enum.CooldownViewerCategory.HiddenPassive == -2)
        "#,
    )
    .unwrap();
}

#[test]
#[cfg(all(not(feature = "client-wowforever"), feature = "retail-12-1-0"))]
fn forever_cooldown_categories_preserve_retail_12_1_0_epoch() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(Enum.CooldownViewerCategory.Essential == 0)
        assert(Enum.CooldownViewerCategory.Utility == 1)
        assert(Enum.CooldownViewerCategory.TrackedBuff == 2)
        assert(Enum.CooldownViewerCategory.TrackedBar == 3)
        assert(Enum.CooldownViewerCategory.GroupBuff == 4)
        assert(Enum.CooldownViewerCategory.SpecAgnosticEssential == 5)
        assert(Enum.CooldownViewerCategory.SpecAgnosticTracked == 6)
        assert(Enum.CooldownViewerCategory.EquipSlotEssential == 7)
        assert(Enum.CooldownViewerCategory.EquipSlotTracked == 8)
        assert(Enum.CooldownViewerCategoryMeta.MinValue == 0)
        assert(Enum.CooldownViewerCategoryMeta.MaxValue == 8)
        assert(Enum.CooldownViewerCategoryMeta.NumValues == 9)
        "#,
    )
    .unwrap();
}

#[test]
#[cfg(all(not(feature = "client-wowforever"), not(feature = "retail-12-1-0")))]
fn forever_cooldown_categories_preserve_other_profiles() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(Enum.CooldownViewerCategory.Essential == 0)
        assert(Enum.CooldownViewerCategory.TrackedBar == 3)
        assert(Enum.CooldownViewerCategory.EquipSlotEssential == nil)
        assert(Enum.CooldownViewerCategoryMeta.MinValue == 0)
        assert(Enum.CooldownViewerCategoryMeta.MaxValue == 3)
        assert(Enum.CooldownViewerCategoryMeta.NumValues == 4)
        "#,
    )
    .unwrap();
}
