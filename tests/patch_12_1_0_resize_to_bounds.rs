//! 12.1.0 `Frame:ResizeToBoundsRect()`: "an addon-safe API ... to resize a frame
//! to match the bounds of its children."
#![cfg(feature = "retail-12-1-0")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn resize_to_bounds_rect_sizes_frame_to_shown_descendant_union() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local function addon()
            local box = CreateFrame('Frame', 'BoundsBox', UIParent)
            box:SetPoint('TOPLEFT', 50, -50)
            box:SetSize(10, 10)
            local a = CreateFrame('Frame', nil, box)
            a:SetPoint('TOPLEFT', box, 'TOPLEFT', 0, 0)
            a:SetSize(40, 20)
            local b = box:CreateTexture(nil, 'ARTWORK')
            b:SetPoint('TOPLEFT', box, 'TOPLEFT', 30, -15)
            b:SetSize(50, 25)
            local nested = CreateFrame('Frame', nil, a)
            nested:SetPoint('TOPLEFT', a, 'TOPLEFT', 5, -50)
            nested:SetSize(10, 10)
            local hidden = CreateFrame('Frame', nil, box)
            hidden:SetPoint('TOPLEFT', box, 'TOPLEFT', 0, 0)
            hidden:SetSize(500, 500)
            hidden:Hide()
            box:ResizeToBoundsRect()
            return box:GetSize()
        end
        debug.setobjecttaint(addon, 'BoundsProbe')
        BoundsW, BoundsH = addon()
        "#,
    )
    .unwrap();
    let (width, height): (f64, f64) = env.eval("return BoundsW, BoundsH").unwrap();
    assert_eq!(
        (width, height),
        (80.0, 60.0),
        "x spans 0..80 (texture), y spans 0..60 (nested grandchild); the hidden child is ignored"
    );
    let left: f64 = env.eval("return BoundsBox:GetLeft()").unwrap();
    assert_eq!(left, 50.0, "anchors and position are kept");

    env.exec(
        r#"
        local empty = CreateFrame('Frame', 'BoundsEmpty', UIParent)
        empty:SetSize(7, 9)
        empty:ResizeToBoundsRect()
        assert(empty:GetWidth() == 7 and empty:GetHeight() == 9, 'no children: unchanged')
        "#,
    )
    .unwrap();
}
