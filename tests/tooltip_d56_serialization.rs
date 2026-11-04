#![cfg(any(feature = "profile-retail", feature = "client-ptr"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn tooltip_d56_getter_copies_markup_and_publishes_packed_rgb() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().player.name = "Plain |cffff0000Red|r End".into();
    env.exec(r#"
        local first = C_TooltipInfo.GetUnit('player')
        local second = C_TooltipInfo.GetUnit('player')
        assert(first.lines[1].leftText == 'Plain |cffff0000Red|r End', 'D56 must copy markup')
        assert(first.lines[1].leftColorSegments == nil, 'D56 has no common segment arrays')
        assert(first ~= second and first.lines ~= second.lines and first.lines[1] ~= second.lines[1])
        assert(second.dataInstanceID == first.dataInstanceID + 1)
        local color = first.lines[1].leftColor
        assert(color and color.a == nil, 'D56 RGB has no alpha')
        assert(type(color.GetRGB) == 'function', 'ColorMixin methods missing')
        local r, g, b = color:GetRGB()
        for _, component in ipairs({r, g, b}) do
            local byte = math.floor(component * 255 + 0.5)
            assert(math.abs(component - byte * 0.003921568859368563) < 0.0000001, 'not packed RGB')
        end
        assert(first.lines[1].rightText == nil and first.lines[1].rightColor == nil)
        assert(first.lines[2].leftColor ~= nil, 'supplied text needs a color')
    "#).unwrap();
}

#[test]
fn tooltip_d56_populated_zero_identity_omits_id_without_changing_misses() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().shapeshift_forms[0].spell_id = 0;
    env.exec(r#"
        local data = C_TooltipInfo.GetShapeshift(1)
        assert(#data.lines > 0 and type(data.dataInstanceID) == 'number')
        assert(data.id == nil, 'D56 populated identity must omit zero')
        local miss = C_TooltipInfo.GetSpellByID(0)
        assert(#miss.lines == 0 and miss.id == 0 and miss.dataInstanceID == nil, 'miss seam changed')
    "#).unwrap();
}

#[test]
fn tooltip_d56_original_consumers_keep_markup_and_present_inline_colors() {
    let env = tooltip_full_env_helpers::setup_full_env();
    env.state().borrow_mut().player.name = "Plain |cffff0000Red|r End".into();
    env.exec(r#"
        GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
        assert(GameTooltip:ProcessInfo({getterName = 'GetUnit', getterArgs = {'player'}}))
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(data.lines[1].leftText == 'Plain |cffff0000Red|r End', 'ProcessInfo lost copied markup')
        local matches = TooltipUtil.FindLinesFromData({data.lines[1].type}, data)
        assert(matches[1] == data.lines[1], 'TooltipUtil must keep the published row')
        assert(GameTooltip:GetLeftLine(1):GetText() == data.lines[1].leftText)
        local name, unit = TooltipUtil.GetDisplayedUnit(GameTooltip)
        assert(name == data.lines[1].leftText and unit == 'player')
        assert(GameTooltip:IsShown() and GameTooltip:NumLines() >= 1)
    "#).unwrap();
    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("GameTooltipTextLeft1").unwrap();
    let frame = state.widgets.get(id).unwrap();
    assert_eq!(frame.text_stripped.as_deref(), Some("Plain Red End"));
    assert_eq!(frame.text_segments.len(), 3);
    assert_eq!(frame.text_segments[1].text, "Red");
    assert_eq!(frame.text_segments[1].color.r, 1.0);
    assert_eq!(frame.text_segments[1].color.g, 0.0);
    assert_eq!(frame.text_segments[1].color.b, 0.0);
}
