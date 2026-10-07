//! Cached Game ordering for the page's color picker migration.
#![cfg(feature = "client-retail")]
use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
    fn patch_10_2_5_cached_color_picker_hex_consumer(env: &WowLuaEnv) {
        let errors_before = env.state().borrow().lua_errors.len();
        env.exec(r#"
            local frame = ColorPickerFrame
            local calls = 0
            frame:SetupColorPickerAndShow({r=0.25, g=0.5, b=0.75,
                swatchFunc=function() calls = calls + 1 end})
            assert(frame:IsShown())
            assert(frame.Content.HexBox:GetText() == '4080bf')
            assert(calls == 1)
            frame.Content.HexBox:SetText('204080')
            frame.Content.HexBox:OnEnterPressed()
            local r, g, b = frame:GetColorRGB()
            assert(math.abs(r - 32/255) < 0.001)
            assert(math.abs(g - 64/255) < 0.001)
            assert(math.abs(b - 128/255) < 0.001)
            assert(calls == 2)
            frame:Hide()
        "#).unwrap();
        assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
    }
}
