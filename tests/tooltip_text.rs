#![cfg(feature = "gui")]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use tooltip_full_env_helpers::setup_full_env;
use wow_ui_sim::iced_app::compute_frame_rect;
use wow_ui_sim::lua_api::WowLuaEnv;

// --- GetLeftLine / GetRightLine tests ---

#[test]
fn test_get_left_line_returns_fontstring() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"GameTooltip:AddLine("Hello")"#).unwrap();

    let obj_type: String = env
        .eval("return GameTooltip:GetLeftLine(1):GetObjectType()")
        .unwrap();
    assert_eq!(obj_type, "FontString");
}

#[test]
fn test_get_left_line_has_correct_text() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameTooltip:AddLine("First line")
        GameTooltip:AddLine("Second line")
    "#,
    )
    .unwrap();

    let text1: String = env
        .eval("return GameTooltip:GetLeftLine(1):GetText()")
        .unwrap();
    assert_eq!(text1, "First line");

    let text2: String = env
        .eval("return GameTooltip:GetLeftLine(2):GetText()")
        .unwrap();
    assert_eq!(text2, "Second line");
}

#[test]
fn test_get_right_line_has_correct_text() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"GameTooltip:AddDoubleLine("Left", "Right")"#)
        .unwrap();

    let right_text: String = env
        .eval("return GameTooltip:GetRightLine(1):GetText()")
        .unwrap();
    assert_eq!(right_text, "Right");
}

#[test]
fn test_add_line_without_color_uses_normal_font_color() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"GameTooltip:AddLine("Click here to view calendar")"#)
        .unwrap();

    let (expected_r, expected_g, expected_b): (f32, f32, f32) = env
        .eval(
            r#"
            local r, g, b = NORMAL_FONT_COLOR:GetRGB()
            return r, g, b
            "#,
        )
        .unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    let (r, g, b) = td.lines[0].left_color;

    const EPSILON: f32 = 0.0001;
    assert!((r - expected_r).abs() < EPSILON);
    assert!((g - expected_g).abs() < EPSILON);
    assert!((b - expected_b).abs() < EPSILON);
}

#[test]
fn test_add_double_line_without_color_uses_normal_font_color_for_both_sides() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"GameTooltip:AddDoubleLine("Left", "Right")"#)
        .unwrap();

    let (expected_r, expected_g, expected_b): (f32, f32, f32) = env
        .eval(
            r#"
            local r, g, b = NORMAL_FONT_COLOR:GetRGB()
            return r, g, b
            "#,
        )
        .unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    let line = &td.lines[0];

    const EPSILON: f32 = 0.0001;
    assert!((line.left_color.0 - expected_r).abs() < EPSILON);
    assert!((line.left_color.1 - expected_g).abs() < EPSILON);
    assert!((line.left_color.2 - expected_b).abs() < EPSILON);
    assert!((line.right_color.0 - expected_r).abs() < EPSILON);
    assert!((line.right_color.1 - expected_g).abs() < EPSILON);
    assert!((line.right_color.2 - expected_b).abs() < EPSILON);
}

#[test]
fn test_get_left_line_out_of_range_returns_nil() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"GameTooltip:AddLine("Only one")"#).unwrap();

    let is_nil: bool = env
        .eval("return GameTooltip:GetLeftLine(5) == nil")
        .unwrap();
    assert!(is_nil, "Out-of-range index should return nil");

    let zero_nil: bool = env
        .eval("return GameTooltip:GetLeftLine(0) == nil")
        .unwrap();
    assert!(zero_nil, "Index 0 should return nil");
}

#[test]
fn test_tooltip_fontstring_globals_exist() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameTooltip:AddLine("Line 1")
        GameTooltip:AddLine("Line 2")
        -- Force FontString creation by accessing them
        local _ = GameTooltip:GetLeftLine(1)
        local _ = GameTooltip:GetLeftLine(2)
    "#,
    )
    .unwrap();

    let exists1: bool = env.eval("return GameTooltipTextLeft1 ~= nil").unwrap();
    assert!(exists1, "GameTooltipTextLeft1 global should exist");

    let exists2: bool = env.eval("return GameTooltipTextLeft2 ~= nil").unwrap();
    assert!(exists2, "GameTooltipTextLeft2 global should exist");

    let text: String = env.eval("return GameTooltipTextLeft1:GetText()").unwrap();
    assert_eq!(text, "Line 1");
}

#[test]
fn test_get_left_line_after_set_item_by_id() {
    let env = WowLuaEnv::new().unwrap();

    env.exec("GameTooltip:SetItemByID(229181)").unwrap();

    let text: String = env
        .eval("return GameTooltip:GetLeftLine(1):GetText()")
        .unwrap();
    assert_eq!(text, "Ordained Forge Maul");
}

#[test]
fn test_get_right_line_no_right_text_returns_nil_text() {
    let env = WowLuaEnv::new().unwrap();

    // Single-text line has no right text
    env.exec(r#"GameTooltip:AddLine("Left only")"#).unwrap();

    // GetRightLine still returns a FontString, but with nil text
    let right_text_nil: bool = env
        .eval(
            r#"
        local fs = GameTooltip:GetRightLine(1)
        return fs:GetText() == nil or fs:GetText() == ""
    "#,
        )
        .unwrap();
    assert!(
        right_text_nil,
        "Right line text should be nil/empty for single-text lines"
    );
}

// --- AddTexture / AddAtlas tests ---

#[test]
fn test_add_texture_increments_numlines() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameTooltip:AddLine("Header")
        GameTooltip:AddTexture(136243)
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert_eq!(count, 2, "AddTexture should add a line");
}

#[test]
fn test_add_atlas_increments_numlines() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameTooltip:AddLine("Header")
        GameTooltip:AddAtlas("groupfinder-icon-friend", {})
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert_eq!(count, 2, "AddAtlas should add a line");
}

#[test]
fn test_add_texture_stores_file_data_id() {
    let env = WowLuaEnv::new().unwrap();

    env.exec("GameTooltip:AddTexture(136243)").unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    assert_eq!(td.lines.len(), 1);
    match &td.lines[0].texture {
        Some(wow_ui_sim::lua_api::tooltip::TooltipTexture::FileDataId(id)) => {
            assert_eq!(*id, 136243);
        }
        other => panic!("Expected FileDataId, got {:?}", other.is_some()),
    }
}

#[test]
fn test_add_atlas_stores_atlas_name() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"GameTooltip:AddAtlas("groupfinder-icon-friend")"#)
        .unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    assert_eq!(td.lines.len(), 1);
    match &td.lines[0].texture {
        Some(wow_ui_sim::lua_api::tooltip::TooltipTexture::Atlas(name)) => {
            assert_eq!(name, "groupfinder-icon-friend");
        }
        other => panic!("Expected Atlas, got {:?}", other.is_some()),
    }
}

#[test]
fn test_add_texture_with_string_id() {
    let env = WowLuaEnv::new().unwrap();

    // Some addons pass texture path as string
    env.exec(r#"GameTooltip:AddTexture("136243")"#).unwrap();

    let count: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert_eq!(count, 1, "AddTexture with string ID should add a line");
}

#[test]
fn test_clearlines_clears_texture_lines() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameTooltip:AddTexture(136243)
        GameTooltip:AddAtlas("test-atlas")
        GameTooltip:ClearLines()
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert_eq!(count, 0, "ClearLines should remove texture lines too");
}

// --- SetCustomLineSpacing / GetCustomLineSpacing tests ---

#[test]
fn test_set_custom_line_spacing_and_get() {
    let env = WowLuaEnv::new().unwrap();

    env.exec("GameTooltip:SetCustomLineSpacing(8)").unwrap();

    let spacing: f64 = env
        .eval("return GameTooltip:GetCustomLineSpacing()")
        .unwrap();
    assert!((spacing - 8.0).abs() < 0.01);
}

#[test]
fn test_get_custom_line_spacing_default_is_zero() {
    let env = WowLuaEnv::new().unwrap();

    let spacing: f64 = env
        .eval("return GameTooltip:GetCustomLineSpacing()")
        .unwrap();
    assert_eq!(
        spacing, 0.0,
        "Default should be 0 (meaning use default 2px)"
    );
}

#[test]
fn test_set_custom_line_spacing_stores_in_tooltip_data() {
    let env = WowLuaEnv::new().unwrap();

    env.exec("GameTooltip:SetCustomLineSpacing(5)").unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    assert_eq!(td.line_spacing, Some(5.0));
}

#[test]
fn test_set_custom_line_spacing_on_custom_tooltip() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local tt = CreateFrame("GameTooltip", "SpacingTestTooltip", UIParent)
        tt:SetCustomLineSpacing(12)
    "#,
    )
    .unwrap();

    let spacing: f64 = env
        .eval("return SpacingTestTooltip:GetCustomLineSpacing()")
        .unwrap();
    assert!((spacing - 12.0).abs() < 0.01);
}

// --- Text wrapping tests ---

#[test]
fn test_addline_wrap_flag_stored() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameTooltip:AddLine("Short title")
        GameTooltip:AddLine("This is a long description that should wrap", 1, 1, 1, true)
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    assert!(!td.lines[0].wrap, "First line should not wrap");
    assert!(td.lines[1].wrap, "Second line (with wrap=true) should wrap");
}

fn update_tooltip_sizes(env: &WowLuaEnv) {
    use wow_ui_sim::render::font::WowFontSystem;

    let mut font_sys = WowFontSystem::new();
    let mut state = env.state().borrow_mut();
    wow_ui_sim::iced_app::tooltip::update_tooltip_sizes(&mut state, &mut font_sys);
}

#[test]
fn test_wrapped_line_does_not_expand_width() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "WrapWidthOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_NONE")
        GameTooltip:AddLine("Short")
        GameTooltip:AddLine("This is a very very very very very very very very very long line that should word-wrap within the tooltip width rather than expanding it to be extremely wide", 1, 1, 1, true)
        GameTooltip:Show()
        assert(GameTooltip:IsShown())
    "#,
    )
    .unwrap();

    update_tooltip_sizes(&env);

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let frame = state.widgets.get(gt_id).unwrap();

    // Width should be determined by the short line, not the long wrapped one.
    // "Short" at 14px ~ 40-60px + 24px padding = ~64-84px total.
    assert!(
        frame.width < 200.0,
        "Tooltip width should be small (non-wrapping lines only), got {}",
        frame.width
    );
}

#[test]
fn test_wrapped_only_line_still_sets_tooltip_width() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "WrapOnlyOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_NONE")
        GameTooltip:ClearLines()
        GameTooltip:AddLine("Professions", 1, 1, 1, true)
    "#,
    )
    .unwrap();

    update_tooltip_sizes(&env);

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let frame = state.widgets.get(gt_id).unwrap();

    assert!(
        frame.width > 100.0,
        "Wrapped-only title text should still size tooltip width, got {}",
        frame.width
    );
}

#[test]
fn test_wrapped_line_increases_height() {
    let env = WowLuaEnv::new().unwrap();

    // Single non-wrapping line
    env.exec(
        r#"
        local owner = CreateFrame("Frame", "WrapHeightOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_NONE")
        GameTooltip:AddLine("Short")
    "#,
    )
    .unwrap();
    update_tooltip_sizes(&env);
    let height_one_line = {
        let state = env.state().borrow();
        let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        state.widgets.get(gt_id).unwrap().height
    };

    // Clear and add a short line + a long wrapping line
    env.exec(
        r#"
        GameTooltip:ClearLines()
        GameTooltip:AddLine("Short")
        GameTooltip:AddLine("This is a long description that should definitely wrap onto multiple lines when rendered within the narrow tooltip width constraint", 1, 1, 1, true)
    "#,
    )
    .unwrap();
    update_tooltip_sizes(&env);

    let height_with_wrap = {
        let state = env.state().borrow();
        let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        state.widgets.get(gt_id).unwrap().height
    };

    assert!(
        height_with_wrap > height_one_line,
        "Wrapped text should increase tooltip height: one_line={}, with_wrap={}",
        height_one_line,
        height_with_wrap
    );
}

#[test]
fn test_double_line_width_includes_gap() {
    let env = WowLuaEnv::new().unwrap();

    // A single-text line for baseline width
    env.exec(
        r#"
        local owner = CreateFrame("Frame", "GapTestOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_NONE")
        GameTooltip:AddLine("Left text only")
    "#,
    )
    .unwrap();
    update_tooltip_sizes(&env);
    let width_single = {
        let state = env.state().borrow();
        let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        state.widgets.get(gt_id).unwrap().width
    };

    // Now a double line — should be wider due to right text + gap
    env.exec(
        r#"
        GameTooltip:ClearLines()
        GameTooltip:AddDoubleLine("Left text only", "Right text")
    "#,
    )
    .unwrap();
    update_tooltip_sizes(&env);
    let width_double = {
        let state = env.state().borrow();
        let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        state.widgets.get(gt_id).unwrap().width
    };

    assert!(
        width_double > width_single,
        "Double-line tooltip should be wider than single: single={}, double={}",
        width_single,
        width_double
    );
}

// --- Tooltip NineSlice sizing tests ---

#[test]
fn test_tooltip_sizing_includes_padding() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "PadTestOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_NONE")
        GameTooltip:AddLine("X")
        GameTooltip:Show()
        assert(GameTooltip:IsShown())
    "#,
    )
    .unwrap();
    update_tooltip_sizes(&env);

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let frame = state.widgets.get(gt_id).unwrap();

    // The tooltip text inset is the 12px content padding plus the 3px
    // NineSlice inner border inset on each side (30 total per axis).
    // Single line at 14px header: height = ceil(14*1.2) = 17 + 30 inset = 47.
    // Width = text width + 24 padding.
    assert!(
        frame.width > 24.0,
        "Width should be > padding alone, got {}",
        frame.width
    );
    assert!(
        frame.height > 24.0,
        "Height should be > padding alone, got {}",
        frame.height
    );
    // Single header line (14pt * 1.2 = ~17px) + 30px vertical inset = ~47px
    let expected_height = (14.0_f32 * 1.2).ceil() + 30.0;
    assert!(
        (frame.height - expected_height).abs() < 1.0,
        "Expected height ~{}, got {}",
        expected_height,
        frame.height
    );
}

#[test]
fn test_tooltip_height_grows_with_lines() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "GrowOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_NONE")
        GameTooltip:AddLine("Line 1")
    "#,
    )
    .unwrap();
    update_tooltip_sizes(&env);
    let h1 = {
        let state = env.state().borrow();
        let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        state.widgets.get(gt_id).unwrap().height
    };

    env.exec(r#"GameTooltip:AddLine("Line 2")"#).unwrap();
    update_tooltip_sizes(&env);
    let h2 = {
        let state = env.state().borrow();
        let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        state.widgets.get(gt_id).unwrap().height
    };

    env.exec(r#"GameTooltip:AddLine("Line 3")"#).unwrap();
    update_tooltip_sizes(&env);
    let h3 = {
        let state = env.state().borrow();
        let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        state.widgets.get(gt_id).unwrap().height
    };

    assert!(h2 > h1, "2 lines should be taller than 1: h1={h1}, h2={h2}");
    assert!(h3 > h2, "3 lines should be taller than 2: h2={h2}, h3={h3}");
}

#[test]
fn test_tooltip_nineslice_child_accessible() {
    let env = setup_full_env();

    let (gt_id, ns_id) = {
        let state = env.state().borrow();
        let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        let frame = state.widgets.get(gt_id).unwrap();
        let ns_id = *frame
            .children_keys
            .get("NineSlice")
            .expect("NineSlice should be in Rust children_keys");
        (gt_id, ns_id)
    };

    let has_ns: bool = env.eval("return GameTooltip.NineSlice ~= nil").unwrap();
    assert!(
        has_ns,
        "GameTooltip.NineSlice should be accessible from Lua"
    );

    let obj_type: String = env
        .eval("return GameTooltip.NineSlice:GetObjectType()")
        .unwrap();
    assert_eq!(obj_type, "Frame", "NineSlice child should be a Frame");

    let ns_num_points: i32 = env
        .eval("return GameTooltip.NineSlice:GetNumPoints()")
        .unwrap();
    assert!(
        ns_num_points >= 2,
        "GameTooltip.NineSlice should use fill-parent anchors, got {ns_num_points} points"
    );

    let state = env.state().borrow();
    let tooltip_rect = compute_frame_rect(&state.widgets, gt_id, 1024.0, 768.0);
    let nine_slice_rect = compute_frame_rect(&state.widgets, ns_id, 1024.0, 768.0);
    assert!(
        (nine_slice_rect.x - tooltip_rect.x).abs() < 1.0,
        "GameTooltip.NineSlice x {} should match tooltip x {}",
        nine_slice_rect.x,
        tooltip_rect.x
    );
    assert!(
        (nine_slice_rect.y - tooltip_rect.y).abs() < 1.0,
        "GameTooltip.NineSlice y {} should match tooltip y {}",
        nine_slice_rect.y,
        tooltip_rect.y
    );
    assert!(
        (nine_slice_rect.width - tooltip_rect.width).abs() < 1.0,
        "GameTooltip.NineSlice width {} should match tooltip width {}",
        nine_slice_rect.width,
        tooltip_rect.width
    );
    assert!(
        (nine_slice_rect.height - tooltip_rect.height).abs() < 1.0,
        "GameTooltip.NineSlice height {} should match tooltip height {}",
        nine_slice_rect.height,
        tooltip_rect.height
    );
}

#[test]
fn test_blank_unwrapped_line_does_not_collapse_wrapped_tooltip_width() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "SpacerWrapOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_NONE")
        -- Matches Blizzard tooltip composition pattern: wrapped title + blank spacer + wrapped instruction.
        GameTooltip:AddLine("Council of Dornogal", 1, 1, 1, true)
        GameTooltip:AddLine(" ")
        GameTooltip:AddLine("<Click to view options>", 0, 1, 0, true)
        GameTooltip:Show()
    "#,
    )
    .unwrap();

    update_tooltip_sizes(&env);

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let frame = state.widgets.get(gt_id).unwrap();
    assert!(
        frame.width > 100.0,
        "blank spacer lines should not collapse wrapped-tooltip width; got {}",
        frame.width
    );
}

#[test]
fn test_tooltip_sizing_skipped_when_hidden() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameTooltip:Hide()
        GameTooltip:AddLine("Hidden line")
    "#,
    )
    .unwrap();

    let width_before_render = {
        let state = env.state().borrow();
        let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        let frame = state.widgets.get(gt_id).unwrap();
        assert!(
            !frame.visible,
            "tooltip must be hidden before render sizing"
        );
        frame.width
    };

    update_tooltip_sizes(&env);

    let width_after_render = {
        let state = env.state().borrow();
        let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        state.widgets.get(gt_id).unwrap().width
    };

    assert_eq!(
        width_before_render, width_after_render,
        "render sizing should skip hidden tooltips"
    );
}

#[test]
fn test_tooltip_min_width_respected() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "MinWOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_NONE")
        GameTooltip:SetMinimumWidth(300)
        GameTooltip:AddLine("Short")
    "#,
    )
    .unwrap();
    update_tooltip_sizes(&env);

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let frame = state.widgets.get(gt_id).unwrap();

    // min_width=300 + 24px padding = 324
    assert!(
        frame.width >= 324.0,
        "Width should respect min_width (300+padding), got {}",
        frame.width
    );
}
