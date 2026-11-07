#![cfg(feature = "gui")]

#[cfg(not(feature = "client-wrath"))]
#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

#[cfg(not(feature = "client-wrath"))]
use tooltip_full_env_helpers::setup_full_env;
#[cfg(not(feature = "client-wrath"))]
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

/// Retail/PTR attach images to the last existing row (WM63 S8L); other
/// non-Wrath profiles still append a separate texture row.
fn native_attachments() -> bool {
    matches!(
        wow_ui_sim::client_profile::ACTIVE,
        wow_ui_sim::client_profile::ClientProfile::Retail
            | wow_ui_sim::client_profile::ClientProfile::Ptr
    )
}

/// Wrath 12340 uses the template-backed `<name>TextureN` slots instead.
fn wrath_template_attachments() -> bool {
    wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::Wrath
}

/// Widget IDs of the attachment textures stored on GameTooltip rows.
fn attachment_texture_ids(env: &WowLuaEnv) -> Vec<u64> {
    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    state.tooltips[&gt_id]
        .lines
        .iter()
        .flat_map(|line| line.attachments().iter().map(|a| a.texture_id))
        .collect()
}

/// Names of the Wrath template texture slots that AddTexture filled, in order.
fn wrath_texture_slot_names(env: &WowLuaEnv) -> Vec<String> {
    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    state.tooltips[&gt_id]
        .wrath_texture_rows
        .iter()
        .filter_map(|&(_, id)| state.widgets.get(id).and_then(|frame| frame.name.clone()))
        .collect()
}

/// Stored filename of a Wrath template texture slot is not observable in the bare
/// environment: the `TextureN` slots live in Blizzard XML, so `widgets_b6_u1.rs`
/// owns the slot coverage. The Wrath branch here rejects the legacy FileDataID row.
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
    if native_attachments() {
        assert_eq!(count, 1, "AddTexture attaches to the existing row");
        assert_eq!(attachment_texture_ids(&env).len(), 1);
    } else if wrath_template_attachments() {
        // 12340 AddTexture rejects fewer than two rows, so this one-row
        // tooltip keeps its single row and records no texture slot
        // (handoff/laptop-labels-answers-20261018/tooltip-addtexture-12340.md).
        assert_eq!(count, 1, "a one-row tooltip has no row to attach to");
        assert_eq!(wrath_texture_slot_names(&env), Vec::<String>::new());
    } else {
        assert_eq!(count, 2, "AddTexture should add a line");
    }
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
    if native_attachments() {
        assert_eq!(count, 1, "AddAtlas attaches to the existing row");
        assert_eq!(attachment_texture_ids(&env).len(), 1);
    } else {
        assert_eq!(count, 2, "AddAtlas should add a line");
    }
}

#[test]
fn test_add_texture_stores_file_data_id() {
    let env = WowLuaEnv::new().unwrap();

    if native_attachments() {
        env.exec("GameTooltip:AddLine('Header'); GameTooltip:AddTexture(136243)")
            .unwrap();
        let ids = attachment_texture_ids(&env);
        let state = env.state().borrow();
        let texture = state.widgets.get(ids[0]).unwrap();
        assert_eq!(texture.texture_file_data_id, Some(136243));
        return;
    }
    if wrath_template_attachments() {
        // 12340 reads the argument with lua_tolstring, so a number is a FILENAME
        // and is never resolved as a FileDataID (tooltip-addtexture-12340.md).
        // The filled `TextureN` slot is asserted in tests/widgets_b6_u1.rs.
        env.exec(
            "GameTooltip:AddLine('Header'); GameTooltip:AddLine('Body'); GameTooltip:AddTexture(136243)",
        )
        .unwrap();
        let state = env.state().borrow();
        let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        assert!(
            state.tooltips[&gt_id]
                .lines
                .iter()
                .all(|line| line.texture.is_none()),
            "a numeric AddTexture argument must not become a FileDataID row"
        );
        return;
    }
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

    if native_attachments() {
        env.exec(
            r#"GameTooltip:AddLine("Header"); GameTooltip:AddAtlas("groupfinder-icon-friend")"#,
        )
        .unwrap();
        let ids = attachment_texture_ids(&env);
        let state = env.state().borrow();
        let texture = state.widgets.get(ids[0]).unwrap();
        assert_eq!(texture.atlas.as_deref(), Some("groupfinder-icon-friend"));
        return;
    }
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
    if native_attachments() || wrath_template_attachments() {
        assert_eq!(count, 0, "an empty tooltip has no row to attach to");
    } else {
        assert_eq!(count, 1, "AddTexture with string ID should add a line");
    }
}

#[test]
fn test_clearlines_clears_texture_lines() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameTooltip:AddLine("Header")
        GameTooltip:AddTexture(136243)
        GameTooltip:AddAtlas("test-atlas")
        GameTooltip:ClearLines()
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert_eq!(count, 0, "ClearLines should remove texture lines too");
    assert!(attachment_texture_ids(&env).is_empty());
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

    // WM63 L23:40: mainline returns no values without a spacing override.
    let (count, spacing): (i32, Option<f64>) = env
        .eval("return select('#', GameTooltip:GetCustomLineSpacing()), GameTooltip:GetCustomLineSpacing()")
        .unwrap();
    if matches!(
        wow_ui_sim::client_profile::ACTIVE,
        wow_ui_sim::client_profile::ClientProfile::Retail
            | wow_ui_sim::client_profile::ClientProfile::Ptr
    ) {
        assert_eq!((count, spacing), (0, None));
    } else {
        assert_eq!((count, spacing), (1, Some(0.0)));
    }
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

fn prepare_tooltip_layout(env: &WowLuaEnv) {
    use wow_ui_sim::render::font::WowFontSystem;

    let mut font_sys = WowFontSystem::new();
    let mut state = env.state().borrow_mut();
    wow_ui_sim::iced_app::tooltip::prepare_tooltip_layout(&mut state, &mut font_sys);
}

const LONG_WRAP_LINE: &str = "This is a very very very very very very very very very long line that should word-wrap within the tooltip width rather than expanding it to be extremely wide";
const WIDTH_PROBE_ROW: &str = "Short";

/// Text inset of the profile's tooltip border, derived from one known unwrapped row
/// (`WrapWidthProbe`) instead of a hard-coded border size.
fn tooltip_text_inset(env: &WowLuaEnv, fonts: &mut wow_ui_sim::render::font::WowFontSystem) -> f32 {
    let width = {
        let state = env.state().borrow();
        let id = state
            .widgets
            .get_id_by_name("WrapWidthProbe")
            .expect("WrapWidthProbe should exist");
        state.widgets.get(id).expect("probe geometry").width
    };
    width - fonts.measure_text_width(WIDTH_PROBE_ROW, None, 14.0)
}

#[test]
fn test_wrapped_line_does_not_expand_width() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(&format!(
        r#"
        local owner = CreateFrame("Frame", "WrapWidthOwner", UIParent)
        local probe = CreateFrame("GameTooltip", "WrapWidthProbe")
        probe:SetOwner(owner, "ANCHOR_NONE")
        probe:AddLine("{WIDTH_PROBE_ROW}")
        probe:Show()
        GameTooltip:SetOwner(owner, "ANCHOR_NONE")
        GameTooltip:AddLine("{WIDTH_PROBE_ROW}")
        GameTooltip:AddLine("{LONG_WRAP_LINE}", 1, 1, 1, true)
        GameTooltip:Show()
        assert(GameTooltip:IsShown())
    "#
    ))
    .unwrap();

    prepare_tooltip_layout(&env);

    let (frame_width, line_width) = {
        let state = env.state().borrow();
        let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        let line_id = state.tooltips[&gt_id].left_line_ids[1];
        (
            state.widgets.get(gt_id).unwrap().width,
            state.widgets.get(line_id).unwrap().width,
        )
    };

    if matches!(
        wow_ui_sim::client_profile::ACTIVE,
        wow_ui_sim::client_profile::ClientProfile::Retail
            | wow_ui_sim::client_profile::ClientProfile::Ptr
    ) {
        // WM63 L23:16-22: wrapped text contributes up to the default 230.4 cap.
        let content_width = frame_width - 20.48;
        assert!(content_width > 150.0 && content_width <= 230.4);
        assert!((line_width - content_width).abs() < 0.01);
    } else if wrath_template_attachments() {
        // 12340 refines the shared width from each wrapped row up to the 230.4
        // candidate and never to the full string, and the row's left line takes
        // the shared width (batch8-tooltip-precision.md).
        let mut fonts = wow_ui_sim::render::font::WowFontSystem::new();
        let inset = tooltip_text_inset(&env, &mut fonts);
        let content_width = frame_width - inset;
        let full_width = fonts.measure_text_width(LONG_WRAP_LINE, None, 12.0);
        assert!(
            content_width > full_width * 0.1,
            "the wrapped row must expand the width, got {content_width} of a {full_width} px line"
        );
        assert!(
            content_width < full_width * 0.5,
            "the wrapped row must not expand the width to the whole line, got {content_width}"
        );
        assert!((line_width - content_width).abs() < 0.01);
    } else {
        // Legacy profiles retain the non-wrapped width rule.
        assert!(
            frame_width < 200.0,
            "Tooltip width should be small (non-wrapping lines only), got {frame_width}"
        );
    }
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

    prepare_tooltip_layout(&env);

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
        GameTooltip:Show()
    "#,
    )
    .unwrap();
    prepare_tooltip_layout(&env);
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
        GameTooltip:Show()
    "#,
    )
    .unwrap();
    prepare_tooltip_layout(&env);

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
        GameTooltip:Show()
    "#,
    )
    .unwrap();
    prepare_tooltip_layout(&env);
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
        GameTooltip:Show()
    "#,
    )
    .unwrap();
    prepare_tooltip_layout(&env);
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
    prepare_tooltip_layout(&env);

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let frame = state.widgets.get(gt_id).unwrap();

    // WM63-L23 specifies a 20.48 outer addition for Retail/PTR. Other
    // profiles retain the renderer's previous NineSlice-based estimate.
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
    let outer_padding = if cfg!(any(feature = "client-retail", feature = "client-ptr")) {
        20.48
    } else {
        30.0
    };
    let expected_height = (14.0_f32 * 1.2).ceil() + outer_padding;
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
        GameTooltip:Show()
    "#,
    )
    .unwrap();
    prepare_tooltip_layout(&env);
    let h1 = {
        let state = env.state().borrow();
        let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        state.widgets.get(gt_id).unwrap().height
    };

    env.exec(r#"GameTooltip:AddLine("Line 2"); GameTooltip:Show()"#)
        .unwrap();
    prepare_tooltip_layout(&env);
    let h2 = {
        let state = env.state().borrow();
        let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        state.widgets.get(gt_id).unwrap().height
    };

    env.exec(r#"GameTooltip:AddLine("Line 3"); GameTooltip:Show()"#)
        .unwrap();
    prepare_tooltip_layout(&env);
    let h3 = {
        let state = env.state().borrow();
        let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        state.widgets.get(gt_id).unwrap().height
    };

    assert!(h2 > h1, "2 lines should be taller than 1: h1={h1}, h2={h2}");
    assert!(h3 > h2, "3 lines should be taller than 2: h2={h2}, h3={h3}");
}

#[cfg(not(feature = "client-wrath"))]
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

    prepare_tooltip_layout(&env);

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

    prepare_tooltip_layout(&env);

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
        GameTooltip:Show()
    "#,
    )
    .unwrap();
    prepare_tooltip_layout(&env);

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let frame = state.widgets.get(gt_id).unwrap();

    // WM63-L23 places the outer addition outside the content minimum.
    let outer_padding = if cfg!(any(feature = "client-retail", feature = "client-ptr")) {
        20.48
    } else {
        24.0
    };
    assert!(
        frame.width >= 300.0 + outer_padding - 0.01,
        "Width should respect min_width (300+padding), got {}",
        frame.width
    );
}
