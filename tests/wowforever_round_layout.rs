#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn forever_pixel_util_rounds_and_restores_recursive_region_layout() {
    let env = WowLuaEnv::new().unwrap();
    env.set_display_size(1024.0, 768.0).unwrap();
    let source = std::fs::read_to_string(
        wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
            .unwrap()
            .join("Blizzard_SharedXML/PixelUtil.lua"),
    )
    .unwrap();
    env.exec(&source).unwrap();
    env.exec(
        r#"
        local root = CreateFrame("Frame", nil, UIParent)
        local child = CreateFrame("Frame", nil, root)
        local texture = root:CreateTexture()
        local text = child:CreateFontString()
        local regions = {root, child, texture, text}
        for _, region in ipairs(regions) do
            region:SetSize(10.3, 20.3)
            assert(region:GetRoundLayoutToNearestPixel() == false)
            local methods = getmetatable(region).__index
            assert(type(methods.SetRoundLayoutToNearestPixel) == "function")
            assert(type(methods.GetRoundLayoutToNearestPixel) == "function")
        end
        PixelUtil.SetRoundLayoutToNearestPixelRecursively(root, true)
        for _, region in ipairs(regions) do
            assert(region:GetRoundLayoutToNearestPixel() == true)
            assert(math.abs(region:GetWidth() - 10) < 0.001)
            assert(math.abs(region:GetHeight() - 20) < 0.001)
        end
        PixelUtil.SetRoundLayoutToNearestPixelRecursively(root, false)
        for _, region in ipairs(regions) do
            assert(region:GetRoundLayoutToNearestPixel() == false)
            assert(math.abs(region:GetWidth() - 10.3) < 0.001)
            assert(math.abs(region:GetHeight() - 20.3) < 0.001)
        end
        "#,
    )
    .unwrap();
}
