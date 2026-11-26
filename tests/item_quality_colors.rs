const SET_OVERRIDES_LUA: &str = r#"
assert(type(ColorManager) == "table", "original ColorManager must load")
local qualities = { Enum.ItemQuality.Rare, Enum.ItemQuality.Epic }
local colors = { CreateColor(32 / 255, 64 / 255, 96 / 255), CreateColor(128 / 255, 160 / 255, 192 / 255) }
for index, quality in ipairs(qualities) do
    local override = ITEM_QUALITY_OVERRIDES[quality]
    assert(override ~= nil, "quality must map to a color override")
    C_ColorOverrides.SetColorOverride(override, colors[index])
end
"#;

const ASSERT_PUBLISHED_COLORS_LUA: &str = r#"
local qualities = { Enum.ItemQuality.Rare, Enum.ItemQuality.Epic }
local colors = { CreateColor(32 / 255, 64 / 255, 96 / 255), CreateColor(128 / 255, 160 / 255, 192 / 255) }
for index, quality in ipairs(qualities) do
    local data = ITEM_QUALITY_COLORS[quality]
    assert(type(data) == "table", "original ColorManager must publish table data")
    assert(data.GetRGB == nil and data.GetRGBA == nil, "entry must remain plain data")
    local red, green, blue = data.color:GetRGB()
    local expected = colors[index]
    assert(math.abs(red - expected.r) < 0.000001)
    assert(math.abs(green - expected.g) < 0.000001)
    assert(math.abs(blue - expected.b) < 0.000001)
    assert(data.hex == expected:GenerateHexColorMarkup())
    assert(ColorManager.GetColorDataForItemQuality(quality) == data)
end
"#;

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn original_color_manager_publishes_plain_item_quality_data_for_overrides() {
    crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape(
        &[],
        &[],
        |env, _| {
            let (loaded, reason): (bool, Option<String>) = env
                .eval(r#"return C_AddOns.LoadAddOn("Blizzard_ArtifactUI")"#)
                .expect("Blizzard_ArtifactUI must load through C_AddOns");
            assert!(loaded, "Blizzard_ArtifactUI should load: {reason:?}");
            env.exec(SET_OVERRIDES_LUA)
                .expect("C_ColorOverrides must accept the two supplied colors");
            env.fire_event("ACCOUNT_CVARS_LOADED")
                .expect("ColorManager must refresh on ACCOUNT_CVARS_LOADED");
            env.exec(ASSERT_PUBLISHED_COLORS_LUA)
                .expect("original ColorManager must publish the supplied quality colors");
        },
    );
}
