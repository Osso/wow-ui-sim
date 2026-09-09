//! Earlier-retail method absence; PTR geometry proof lives in pixel_rounding_probe.

use crate::lua_api::WowLuaEnv;

#[test]
fn round_layout_methods_remain_absent_on_retail() {
    let env = WowLuaEnv::new().unwrap();
    assert_retail_region_methods(&env);
    crate::ptr::compat_bootstrap::apply_post_load(&env);
    assert_retail_region_methods(&env);
}

fn assert_retail_region_methods(env: &WowLuaEnv) {
    env.exec(
        r#"
        local frame = CreateFrame("Frame")
        local regions = { frame, frame:CreateTexture(), frame:CreateFontString() }
        for _, region in ipairs(regions) do
            assert(region.GetRoundLayoutToNearestPixel == nil, region:GetObjectType())
            assert(region.SetRoundLayoutToNearestPixel == nil, region:GetObjectType())
            region:SetSize(48, 24)
            assert(region:GetWidth() == 48 and region:GetHeight() == 24)
            region:SetAlpha(0.5)
            assert(region:GetAlpha() == 0.5)
            region:Hide()
            assert(not region:IsShown())
            region:Show()
            assert(region:IsShown())
        end
        "#,
    )
    .expect("retail regions retain existing methods without PTR rounding controls");
}
