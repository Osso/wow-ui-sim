use crate::lua_api::WowLuaEnv;

#[cfg(feature = "client-retail")]
#[test]
fn test_patch_12_1_5_curio_rarity_preserves_retail() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local rarity = Enum.CurioRarity
        assert(rarity.Common == 1)
        assert(rarity.Uncommon == 2)
        assert(rarity.Rare == 3)
        assert(rarity.Epic == 4)
        assert(rarity.EpicTier2 == nil)
        local count = 0
        for _ in pairs(rarity) do count = count + 1 end
        assert(count == 4)
        local metadata = Enum.CurioRarityMeta
        assert(metadata.MinValue == 1)
        assert(metadata.MaxValue == 4)
        assert(metadata.NumValues == 4)
        "#,
    )
    .unwrap();
}
