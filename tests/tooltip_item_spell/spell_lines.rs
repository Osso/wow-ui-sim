use super::*;

#[test]
fn test_set_spell_by_id_get_left_line_uses_tooltip_line_color() {
    let env = WowLuaEnv::new().unwrap();

    env.exec("GameTooltip:SetSpellByID(53600)").unwrap();

    let (line_index, expected_color) = {
        let state = env.state().borrow();
        let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        let td = state.tooltips.get(&gt_id).unwrap();
        let description_index = td
            .lines
            .iter()
            .position(|line| line.left_text.contains("Armor by"))
            .expect("Shield of the Righteous tooltip should include armor description")
            + 1;
        (
            description_index,
            td.lines[description_index - 1].left_color,
        )
    };
    let actual_color: (f32, f32, f32) = env
        .eval(&format!(
            "local line = GameTooltip:GetLeftLine({line_index}); return line:GetTextColor()"
        ))
        .unwrap();

    assert!(
        (actual_color.0 - expected_color.0).abs() < 0.01
            && (actual_color.1 - expected_color.1).abs() < 0.01
            && (actual_color.2 - expected_color.2).abs() < 0.01,
        "GetLeftLine should apply tooltip line color; expected rgb=({:.3},{:.3},{:.3}), got rgb=({:.3},{:.3},{:.3})",
        expected_color.0,
        expected_color.1,
        expected_color.2,
        actual_color.0,
        actual_color.1,
        actual_color.2
    );
}

#[test]
fn test_set_spell_by_id_get_left_line_does_not_invent_value_color_segments() {
    let env = WowLuaEnv::new().unwrap();

    env.exec("GameTooltip:SetSpellByID(53600)").unwrap();

    let line_index = {
        let state = env.state().borrow();
        let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        let td = state.tooltips.get(&gt_id).unwrap();
        let line_index = td
            .lines
            .iter()
            .position(|line| line.left_text.contains("Armor by"))
            .expect("Shield of the Righteous tooltip should include armor description");
        line_index + 1
    };
    env.exec(&format!("GameTooltip:GetLeftLine({line_index})"))
        .unwrap();
    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    let left_line_id = td.left_line_ids[line_index - 1];
    let line_frame = state.widgets.get(left_line_id).unwrap();
    assert!(
        line_frame.text_segments.is_empty(),
        "rendered numeric values should not invent inline color segments"
    );
}

#[test]
fn test_add_line_does_not_invent_processing_info_value_color_segments() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local info = { tooltipData = C_TooltipInfo.GetAction(5) }
        GameTooltip.processingInfo = info
        GameTooltip:ClearLines()
        for _, lineData in ipairs(info.tooltipData.lines) do
            local color = lineData.leftColor or NORMAL_FONT_COLOR
            local r, g, b = color:GetRGB()
            GameTooltip:AddLine(lineData.leftText, r, g, b, lineData.wrapText)
        end
        GameTooltip.processingInfo = nil
        "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    let description_line = td
        .lines
        .iter()
        .find(|line| line.left_text.contains("Armor by"))
        .expect("action tooltip should include Shield of the Righteous description");
    assert!(
        description_line.left_segments.is_empty(),
        "data-handler tooltip should not invent color segments for resolved numeric values"
    );
}

#[test]
fn test_add_line_preserves_explicit_processing_info_inline_color_segments() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local info = {
            tooltipData = {
                lines = {
                    {
                        leftText = "Requires Clearcasting",
                        leftColor = NORMAL_FONT_COLOR,
                        leftColorSegments = {
                            { text = "Requires ", color = NORMAL_FONT_COLOR },
                            { text = "Clearcasting", color = HIGHLIGHT_FONT_COLOR },
                        },
                    },
                },
            },
        }
        GameTooltip.processingInfo = info
        GameTooltip:ClearLines()
        local lineData = info.tooltipData.lines[1]
        local r, g, b = lineData.leftColor:GetRGB()
        GameTooltip:AddLine(lineData.leftText, r, g, b, lineData.wrapText)
        GameTooltip.processingInfo = nil
        "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    let line = td.lines.first().unwrap();
    let highlighted = line
        .left_segments
        .iter()
        .find(|segment| segment.text == "Clearcasting")
        .expect("AddLine should preserve explicit processingInfo color segments");

    assert!(
        (highlighted.color.0 - 1.0).abs() < 0.01
            && (highlighted.color.1 - 1.0).abs() < 0.01
            && (highlighted.color.2 - 1.0).abs() < 0.01,
        "explicit highlighted segment should stay white, got rgb=({:.3},{:.3},{:.3})",
        highlighted.color.0,
        highlighted.color.1,
        highlighted.color.2
    );
}

#[test]
fn test_set_spell_by_id_applies_full_line_inline_color_markup() {
    let env = WowLuaEnv::new().unwrap();

    env.exec("GameTooltip:SetSpellByID(5143)").unwrap();

    let (description_index, description_color, description_text) = {
        let state = env.state().borrow();
        let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        let td = state.tooltips.get(&gt_id).unwrap();
        let index = td
            .lines
            .iter()
            .position(|line| line.left_text.contains("Requires Clearcasting"))
            .expect("Arcane Missiles tooltip should include its colored requirement")
            + 1;
        let line = &td.lines[index - 1];
        (index, line.left_color, line.left_text.clone())
    };
    let line_frame_color: (f32, f32, f32) = env
        .eval(&format!(
            "local line = GameTooltip:GetLeftLine({description_index}); return line:GetTextColor()"
        ))
        .unwrap();

    assert_eq!(description_text, "Requires Clearcasting");
    assert!(
        (description_color.0 - 1.0).abs() < 0.01
            && (description_color.1 - 1.0).abs() < 0.01
            && (description_color.2 - 1.0).abs() < 0.01,
        "full-line |cFFFFFFFF markup should make tooltip data white, got rgb=({:.3},{:.3},{:.3})",
        description_color.0,
        description_color.1,
        description_color.2
    );
    assert!(
        (line_frame_color.0 - 1.0).abs() < 0.01
            && (line_frame_color.1 - 1.0).abs() < 0.01
            && (line_frame_color.2 - 1.0).abs() < 0.01,
        "full-line |cFFFFFFFF markup should make tooltip FontString white, got rgb=({:.3},{:.3},{:.3})",
        line_frame_color.0,
        line_frame_color.1,
        line_frame_color.2
    );
}

#[test]
fn test_set_spell_by_id_applies_named_inline_color_markup() {
    let env = WowLuaEnv::new().unwrap();

    env.exec("GameTooltip:SetSpellByID(448287)").unwrap();

    let (description_color, description_text) = {
        let state = env.state().borrow();
        let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        let td = state.tooltips.get(&gt_id).unwrap();
        let line = td
            .lines
            .iter()
            .find(|line| line.left_text.contains("Season 1"))
            .expect("tooltip should include the named-color description line");
        (line.left_color, line.left_text.clone())
    };
    let expected: (f32, f32, f32) = env
        .eval("local r,g,b = GREEN_FONT_COLOR:GetRGB(); return r,g,b")
        .unwrap();

    assert_eq!(description_text, "Season 1");
    assert!(
        (description_color.0 - expected.0).abs() < 0.01
            && (description_color.1 - expected.1).abs() < 0.01
            && (description_color.2 - expected.2).abs() < 0.01,
        "named |cnGREEN_FONT_COLOR markup should make tooltip data green; expected rgb=({:.3},{:.3},{:.3}), got rgb=({:.3},{:.3},{:.3})",
        expected.0,
        expected.1,
        expected.2,
        description_color.0,
        description_color.1,
        description_color.2
    );
}

#[test]
fn test_set_spell_by_id_preserves_partial_inline_color_segments() {
    let env = WowLuaEnv::new().unwrap();

    env.exec("GameTooltip:SetSpellByID(1223268)").unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    let line = td
        .lines
        .iter()
        .find(|line| line.left_text.contains("Scale of the Earth-Warder"))
        .expect("tooltip should include the partially colored description line");
    let highlighted = line
        .left_segments
        .iter()
        .find(|segment| segment.text == "Scale of the Earth-Warder")
        .expect("partial inline color markup should create a segment for the highlighted text");

    assert!(
        line.left_segments.len() > 1,
        "partial inline color markup should split the line into colored runs, got {}",
        line.left_segments.len()
    );
    assert!(
        (highlighted.color.0 - 1.0).abs() < 0.01
            && (highlighted.color.1 - 0.8).abs() < 0.01
            && (highlighted.color.2 - 0.6).abs() < 0.01,
        "highlighted segment should use |cFFFFCC99 color, got rgb=({:.3},{:.3},{:.3})",
        highlighted.color.0,
        highlighted.color.1,
        highlighted.color.2
    );
}

#[test]
fn test_set_spell_by_id_makes_tooltip_visible() {
    let env = WowLuaEnv::new().unwrap();

    let initially_visible: bool = env.eval("return GameTooltip:IsVisible()").unwrap();
    assert!(!initially_visible);

    env.exec("GameTooltip:SetSpellByID(19750)").unwrap();

    let visible: bool = env.eval("return GameTooltip:IsVisible()").unwrap();
    assert!(visible, "SetSpellByID should make tooltip visible");
}

#[test]
fn test_set_spell_by_id_fires_on_tooltip_set_spell() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        _G.spell_set_count = 0
        GameTooltip:SetScript("OnTooltipSetSpell", function()
            _G.spell_set_count = _G.spell_set_count + 1
        end)
        GameTooltip:SetSpellByID(19750)
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return _G.spell_set_count").unwrap();
    assert_eq!(count, 1, "OnTooltipSetSpell should fire once");
}

#[test]
fn test_get_spell_returns_spell_data_after_set() {
    let env = WowLuaEnv::new().unwrap();

    env.exec("GameTooltip:SetSpellByID(19750)").unwrap();

    let name: String = env
        .eval("local name, id = GameTooltip:GetSpell(); return name")
        .unwrap();
    assert_eq!(name, "Flash of Light");

    let id: i32 = env
        .eval("local name, id = GameTooltip:GetSpell(); return id")
        .unwrap();
    assert_eq!(id, 19750);
}

#[test]
fn test_get_spell_returns_nil_when_no_spell() {
    let env = WowLuaEnv::new().unwrap();

    let is_nil: bool = env
        .eval("local name, id = GameTooltip:GetSpell(); return name == nil and id == nil")
        .unwrap();
    assert!(
        is_nil,
        "GetSpell should return nil,nil when no spell is set"
    );
}

#[test]
fn test_clear_lines_clears_spell_id() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameTooltip:SetSpellByID(19750)
        GameTooltip:ClearLines()
    "#,
    )
    .unwrap();

    let is_nil: bool = env
        .eval("local name, id = GameTooltip:GetSpell(); return name == nil")
        .unwrap();
    assert!(is_nil, "ClearLines should clear spell_id");
}

#[test]
fn test_set_spell_by_id_unknown_spell_is_noop() {
    let env = WowLuaEnv::new().unwrap();

    env.exec("GameTooltip:SetSpellByID(999999999)").unwrap();

    let count: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert_eq!(count, 0, "Unknown spell should not add lines");
}
