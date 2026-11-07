//! Full-environment tooltip tests requiring Blizzard addon loading.
//!
//! Tests that need the micro menu, OnEnter scripts, or the render pipeline
//! live here. Basic tooltip API tests stay in tooltip.rs.

mod tooltip_full_env_helpers;
mod tooltip_hover_helpers;

use tooltip_full_env_helpers::setup_full_env;
use tooltip_hover_helpers::{open_character_panel, refresh_buff_frame};
#[cfg(not(feature = "client-wrath"))]
use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn blizzard_tooltip_process_info_rejects_empty_or_ownerless_show() {
    let env = setup_full_env();
    env.exec(
        r#"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { leftText = "Previous content" },
        } } }))
        assert(GameTooltip:IsShown())
        local width, height = GameTooltip:GetSize()
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {} } }))
        assert(not GameTooltip:IsShown(), "empty ProcessInfo must hide the tooltip")
        assert(GameTooltip:GetOwner() == nil, "rejected Show must release its owner")
        assert(GameTooltip:GetWidth() == width and GameTooltip:GetHeight() == height,
            "rejected Show must retain the previous dimensions")

        GameTooltip:SetAllowShowWithNoLines(true)
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { leftText = "Ownerless content" },
        } } }))
        assert(not GameTooltip:IsShown(), "allow-empty does not permit ownerless Show")
        assert(GameTooltip:NumLines() == 0, "ownerless Show must clear pending lines")

        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {} } }))
        assert(GameTooltip:IsShown(), "owned allow-empty ProcessInfo must show")
        assert(GameTooltip:GetOwner() == UIParent)
        assert(math.abs(GameTooltip:GetWidth() - 20.48) < 0.01)
        assert(math.abs(GameTooltip:GetHeight() - 20.48) < 0.01)
        "#,
    )
    .expect("WM63 L4 rejected Show must use Hide, not resize empty content");
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn blizzard_backpack_tooltip_append_text_updates_pooled_title() {
    let env = setup_full_env();
    let addons = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    // Retail uses the Mainline TOC; pinned PTR uses the unified TOC and Shared Lua.
    let toc = wow_ui_sim::loader::find_toc_file(&addons.join("Blizzard_MainMenuBarBagButtons"))
        .expect("profile-selected original bag-button TOC");
    wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    env.exec(
        r#"
        assert(not KeybindFrames_InQuickKeybindMode())
        for _, binding in ipairs({ "CTRL-F8", "ALT-F9" }) do
            GameTooltip:Hide()
            assert(SetBinding(binding, "TOGGLEBACKPACK"))
            MainMenuBarBackpackButton:OnEnterInternal()
            local key = GetBindingText(GetBindingKey("TOGGLEBACKPACK"))
            local title = BACKPACK_TOOLTIP .. NORMAL_FONT_COLOR:WrapTextInColorCode(" (" .. key .. ")")
            assert(GameTooltip:IsShown())
            assert(GameTooltip:NumLines() == 2)
            local titleLine = GameTooltip:GetLeftLine(1)
            assert(titleLine:GetText() == title,
                "backpack title FontString lost the appended binding")
            assert(GameTooltip:GetWidth() + 0.01 >= titleLine:GetStringWidth() + 20.48,
                "backpack tooltip was sized using the title without its binding")
            assert(GameTooltip:GetLeftLine(2):GetText() == NUM_FREE_SLOTS:format(MainMenuBarBackpackButton.freeSlots or 0))
            assert(SetBinding(binding))
        end
        "#,
    )
    .expect("original backpack OnEnter must publish the appended title to its FontString");
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn blizzard_tooltip_append_text_uses_first_pooled_left_line_synchronously() {
    let env = setup_full_env();
    env.exec(
        r##"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { leftText = "Title" }, { leftText = "Footer" },
        } } }))
        local title = GameTooltip:GetLeftLine(1)
        title:SetText("Edited title")
        local oldWidth = GameTooltip:GetWidth()
        local suffix = string.rep("W", 30)
        GameTooltip:AppendText(suffix)
        assert(title:GetText() == "Edited title" .. suffix,
            "AppendText must update the first pooled left FontString")
        assert(GameTooltip:GetWidth() > oldWidth + 100,
            "AppendText must size the updated title before returning")
        assert(GameTooltip:GetLeftLine(1):GetText() == "Edited title" .. suffix,
            "AppendText must retain the edited title in stored line state")
        assert(GameTooltip:GetLeftLine(2):GetText() == "Footer",
            "AppendText must not select the last row")
        assert(GameTooltip:NumLines() == 2)

        title:SetText(string.rep("A", 250))
        GameTooltip:AppendText("123456789")
        assert(title:GetText() == string.rep("A", 250) .. "12345", "255-byte payload cap")
        title:SetText(string.rep("A", 254))
        GameTooltip:AppendText("é")
        assert(title:GetText() == string.rep("A", 254), "UTF-8 boundary emulation must not split a character")
        title:SetText("Before" .. string.char(0) .. "After")
        GameTooltip:AppendText("Suffix" .. string.char(0) .. "Ignored")
        assert(title:GetText() == "BeforeSuffix", "native text concatenation stops at NUL")
        title:SetText("Number")
        assert(select("#", GameTooltip:AppendText(17)) == 0)
        assert(title:GetText() == "Number17")

        GameTooltip:ClearLines()
        assert(select("#", GameTooltip:AppendText("No row")) == 0)
        assert(GameTooltip:NumLines() == 0)
        "##,
    )
    .expect("WM95 first-row pooled text, byte cap and synchronous layout");
    for invalid in ["nil", "false", "{}"] {
        assert!(
            env.exec(&format!("GameTooltip:AppendText({invalid})"))
                .is_err(),
            "empty tooltip must still reject AppendText({invalid})"
        );
    }
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn blizzard_default_tooltip_anchor_rejects_non_table_owner_without_clearing() {
    let env = setup_full_env();
    env.exec(
        r#"
        GameTooltip_SetDefaultAnchor(GameTooltip, UIParent)
        GameTooltip_AddNormalLine(GameTooltip, "Retained title")
        GameTooltip:Show()
        "#,
    )
    .unwrap();
    for invalid in ["nil", "false"] {
        let result = env.exec(&format!(
            "GameTooltip_SetDefaultAnchor(GameTooltip, {invalid})"
        ));
        assert!(result.is_err(), "WM95 rejects non-table owner {invalid}");
        assert!(result.unwrap_err().to_string().contains("SetOwner(region)"));
        env.exec(
            r#"
            assert(GameTooltip:GetOwner() == UIParent)
            assert(GameTooltip:IsShown())
            assert(GameTooltip:NumLines() == 1)
            assert(GameTooltip:GetLeftLine(1):GetText() == "Retained title")
            "#,
        )
        .expect("invalid owner must not clear or replace the existing tooltip");
    }
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn customization_tooltip_places_first_text_at_native_padding_anchor() {
    use iced::{Point, Rectangle, Size};
    use wow_ui_sim::iced_app::tooltip::{TooltipRender, build_tooltip_quads, collect_tooltip_data};
    use wow_ui_sim::render::glyph::emit_text_quads;
    use wow_ui_sim::render::{GlyphAtlas, QuadBatch, WowFontSystem};
    use wow_ui_sim::widget::{TextJustify, TextOutline};

    let env = setup_full_env();
    let addons = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    wow_ui_sim::loader::load_addon(
        &env.loader_env(),
        &addons.join("Blizzard_CustomizationUI/Blizzard_CustomizationUI.toc"),
    )
    .unwrap();
    env.exec(
        r#"
        local region = CreateFrame("Frame", nil, UIParent, "CustomizationFrameWithTooltipTemplate")
        region.tooltipAnchor = "ANCHOR_NONE"
        region.tooltipPadding = 5
        region:AddTooltipLine("Anchor")
        region:AddTooltipLine("Anchor")
        region:OnEnter()
        CustomizationNoHeaderTooltip:SetPadding(3, 5, 7, 11)
        local line = CustomizationNoHeaderTooltip:GetLeftLine(1)
        assert(line:GetNumPoints() == 2, "first left line must have two anchor points")
        local point, relative, relativePoint, x, y = line:GetPoint(1)
        assert(point == "TOP" and relative == CustomizationNoHeaderTooltip and relativePoint == "TOP" and x == 0 and y == -21, "first left vertical anchor")
        point, relative, relativePoint, x, y = line:GetPoint(2)
        assert(point == "LEFT" and relative == CustomizationNoHeaderTooltip and relativePoint == "LEFT" and x == 17 and y == 0, "first left horizontal anchor")
        line:SetPoint("TOP", CustomizationNoHeaderTooltip, "TOP", 0, -55)
        CustomizationNoHeaderTooltip:Show()
        local _, _, _, _, retainedY = line:GetPoint(1)
        assert(retainedY == -55, "repeated Show must retain the anchored left prefix")
        CustomizationNoHeaderTooltip:SetPadding(3, 5, 7, 11)
        local _, _, _, _, resetY = line:GetPoint(1)
        assert(resetY == -21, "SetPadding must invalidate and restore native anchors")
    "#,
    )
    .unwrap();
    let state = env.state().borrow();
    let id = state
        .widgets
        .get_id_by_name("CustomizationNoHeaderTooltip")
        .unwrap();
    let data = collect_tooltip_data(&state);
    let frame = state.widgets.get(id).unwrap();
    let mut fonts = WowFontSystem::new();
    let mut glyphs = GlyphAtlas::new();
    let mut actual = QuadBatch::new();
    let bounds = Rectangle::new(
        Point::new(100.0, 200.0),
        Size::new(frame.width, frame.height),
    );
    build_tooltip_quads(
        TooltipRender {
            batch: &mut actual,
            bounds,
            tooltip_data: Some(&data),
            id,
            eff_alpha: 1.0,
            eff_scale: 1.0,
            draw_background: false,
        },
        &mut Some((&mut fonts, &mut glyphs)),
    );
    // WM63 L23:42-44: each line's height comes from its FontString measurement.
    let first_size = data[&id].lines[0].font_size;
    let second_size = data[&id].lines[1].font_size;
    let first_height = fonts.measure_text_height("Anchor", None, first_size, None, 0.0, false);
    let second_height = fonts.measure_text_height("Anchor", None, second_size, None, 0.0, false);
    let mut expected = QuadBatch::new();
    emit_text_quads(
        &mut expected,
        &mut fonts,
        &mut glyphs,
        "Anchor",
        Rectangle::new(Point::new(117.0, 221.0), Size::new(200.0, first_height)),
        None,
        first_size,
        [1.0; 4],
        TextJustify::Left,
        TextJustify::Center,
        wow_ui_sim::render::shader::GLYPH_ATLAS_TEX_INDEX,
        None,
        (0.0, 0.0),
        TextOutline::None,
        false,
        0,
        0.0,
        false,
        None,
        None,
    );
    assert!(!actual.vertices.is_empty() && !expected.vertices.is_empty());
    let actual_first = actual
        .vertices
        .iter()
        .find(|vertex| vertex.tex_index == wow_ui_sim::render::shader::GLYPH_ATLAS_TEX_INDEX)
        .expect("tooltip glyph")
        .position;
    let expected_first = expected.vertices[0].position;
    let actual_second = actual
        .vertices
        .iter()
        .filter(|vertex| vertex.tex_index == wow_ui_sim::render::shader::GLYPH_ATLAS_TEX_INDEX)
        .nth(24)
        .expect("second line glyph")
        .position;
    let mut expected_second = QuadBatch::new();
    emit_text_quads(
        &mut expected_second,
        &mut fonts,
        &mut glyphs,
        "Anchor",
        Rectangle::new(
            Point::new(117.0, 221.0 + first_height + 2.0),
            Size::new(200.0, second_height),
        ),
        None,
        second_size,
        [1.0; 4],
        TextJustify::Left,
        TextJustify::Center,
        wow_ui_sim::render::shader::GLYPH_ATLAS_TEX_INDEX,
        None,
        (0.0, 0.0),
        TextOutline::None,
        false,
        0,
        0.0,
        false,
        None,
        None,
    );
    let expected_second = expected_second.vertices[0].position;
    assert!(
        (actual_second[0] - expected_second[0]).abs() < 0.01
            && (actual_second[1] - expected_second[1]).abs() < 0.01,
        "second glyph position {actual_second:?}, expected {expected_second:?}"
    );
    assert!(
        (actual_first[0] - expected_first[0]).abs() < 0.01
            && (actual_first[1] - expected_first[1]).abs() < 0.01,
        "first glyph position {actual_first:?}, expected {expected_first:?}"
    );
    drop(state);
    env.exec(
        r#"
        local line = CustomizationNoHeaderTooltip:GetLeftLine(1)
        line:SetPoint("TOP", CustomizationNoHeaderTooltip, "TOP", 0, -55)
        CustomizationNoHeaderTooltip:Show()
    "#,
    )
    .unwrap();
    let state = env.state().borrow();
    let data = collect_tooltip_data(&state);
    let mut moved = QuadBatch::new();
    build_tooltip_quads(
        TooltipRender {
            batch: &mut moved,
            bounds,
            tooltip_data: Some(&data),
            id,
            eff_alpha: 1.0,
            eff_scale: 1.0,
            draw_background: false,
        },
        &mut Some((&mut fonts, &mut glyphs)),
    );
    let moved_first = moved
        .vertices
        .iter()
        .find(|vertex| vertex.tex_index == wow_ui_sim::render::shader::GLYPH_ATLAS_TEX_INDEX)
        .expect("moved tooltip glyph")
        .position;
    assert!(
        (moved_first[1] - actual_first[1] - 34.0).abs() < 0.01,
        "render must follow retained first-line anchor: {moved_first:?} vs {actual_first:?}"
    );
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn blizzard_hidden_right_line_keeps_its_anchor_until_shown() {
    let env = setup_full_env();
    env.exec(r#"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { type = 0, leftText = "First", rightText = "Value" },
        } } }))
        local left, right = GameTooltip:GetLeftLine(1), GameTooltip:GetRightLine(1)
        assert(right:IsShown(), "right line starts shown")
        local bothWidth = GameTooltip:GetWidth()
        right:Hide()
        right:SetPoint("RIGHT", left, "LEFT", 123, 0)
        GameTooltip:Show()
        local _, _, _, x = right:GetPoint(1)
        assert(x == 123, "hidden right anchor must be retained")
        assert(GameTooltip:GetWidth() < bothWidth, "hidden right line must not contribute width")
        right:Show()
        GameTooltip:Show()
        local _, _, _, rebuilt = right:GetPoint(1)
        assert(math.abs(rebuilt - (GameTooltip:GetWidth() - 20.48)) < 0.01, "shown right anchor must be rebuilt")
    "#).unwrap();
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn blizzard_hidden_right_line_emits_no_glyphs() {
    use iced::{Point, Rectangle, Size};
    use wow_ui_sim::iced_app::tooltip::{TooltipRender, build_tooltip_quads, collect_tooltip_data};
    use wow_ui_sim::render::{GlyphAtlas, QuadBatch, WowFontSystem};

    let env = setup_full_env();
    env.exec(r#"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { type = 0, leftText = "Left", rightText = "Right", leftColor = CreateColor(1, 1, 1), rightColor = CreateColor(1, 0, 0) },
        } } }))
        GameTooltip:GetRightLine(1):Hide()
        GameTooltip:Show()
    "#).unwrap();
    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let frame = state.widgets.get(id).unwrap();
    let data = collect_tooltip_data(&state);
    let mut fonts = WowFontSystem::new();
    let mut glyphs = GlyphAtlas::new();
    let mut batch = QuadBatch::new();
    build_tooltip_quads(
        TooltipRender {
            batch: &mut batch,
            bounds: Rectangle::new(
                Point::new(100.0, 200.0),
                Size::new(frame.width, frame.height),
            ),
            tooltip_data: Some(&data),
            id,
            eff_alpha: 1.0,
            eff_scale: 1.0,
            draw_background: false,
        },
        &mut Some((&mut fonts, &mut glyphs)),
    );
    let glyphs: Vec<_> = batch
        .vertices
        .iter()
        .filter(|vertex| vertex.tex_index == wow_ui_sim::render::shader::GLYPH_ATLAS_TEX_INDEX)
        .collect();
    assert!(
        glyphs
            .iter()
            .any(|vertex| vertex.color == [1.0, 1.0, 1.0, 1.0]),
        "visible left glyphs must render"
    );
    assert!(
        !glyphs
            .iter()
            .any(|vertex| vertex.color == [1.0, 0.0, 0.0, 1.0]),
        "hidden right glyphs must not render"
    );
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn customization_hidden_first_row_is_not_rendered() {
    use iced::{Point, Rectangle, Size};
    use wow_ui_sim::iced_app::tooltip::{TooltipRender, build_tooltip_quads, collect_tooltip_data};
    use wow_ui_sim::render::{GlyphAtlas, QuadBatch, WowFontSystem};
    let env = setup_full_env();
    let addons = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    wow_ui_sim::loader::load_addon(
        &env.loader_env(),
        &addons.join("Blizzard_CustomizationUI/Blizzard_CustomizationUI.toc"),
    )
    .unwrap();
    env.exec(
        r#"
        local region = CreateFrame("Frame", nil, UIParent, "CustomizationFrameWithTooltipTemplate")
        region.tooltipAnchor = "ANCHOR_NONE"
        region:AddTooltipLine("First")
        region:AddTooltipLine("Second")
        region:OnEnter()
    "#,
    )
    .unwrap();
    let count_glyph_vertices = || {
        let state = env.state().borrow();
        let id = state
            .widgets
            .get_id_by_name("CustomizationNoHeaderTooltip")
            .unwrap();
        let frame = state.widgets.get(id).unwrap();
        let data = collect_tooltip_data(&state);
        let mut fonts = WowFontSystem::new();
        let mut glyphs = GlyphAtlas::new();
        let mut batch = QuadBatch::new();
        build_tooltip_quads(
            TooltipRender {
                batch: &mut batch,
                bounds: Rectangle::new(
                    Point::new(100.0, 200.0),
                    Size::new(frame.width, frame.height),
                ),
                tooltip_data: Some(&data),
                id,
                eff_alpha: 1.0,
                eff_scale: 1.0,
                draw_background: false,
            },
            &mut Some((&mut fonts, &mut glyphs)),
        );
        batch
            .vertices
            .iter()
            .filter(|vertex| vertex.tex_index == wow_ui_sim::render::shader::GLYPH_ATLAS_TEX_INDEX)
            .count()
    };
    let before = count_glyph_vertices();
    env.exec(
        "CustomizationNoHeaderTooltip:GetLeftLine(1):Hide(); CustomizationNoHeaderTooltip:Show()",
    )
    .unwrap();
    let after = count_glyph_vertices();
    assert!(
        after > 0 && after < before,
        "hidden first row must not emit glyphs: before {before}, after {after}"
    );
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn customization_hidden_first_row_keeps_indexed_anchor_but_not_height() {
    let env = setup_full_env();
    let addons = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    wow_ui_sim::loader::load_addon(
        &env.loader_env(),
        &addons.join("Blizzard_CustomizationUI/Blizzard_CustomizationUI.toc"),
    )
    .unwrap();
    env.exec(r#"
        local region = CreateFrame("Frame", nil, UIParent, "CustomizationFrameWithTooltipTemplate")
        region.tooltipAnchor = "ANCHOR_NONE"
        region:AddTooltipLine("First")
        region:AddTooltipLine("Second")
        region:OnEnter()
        local tip = CustomizationNoHeaderTooltip
        local first, second = tip:GetLeftLine(1), tip:GetLeftLine(2)
        assert(first:IsShown() and second:IsShown(), "original consumer shows both rows")
        local oldHeight = tip:GetHeight()
        first:Hide()
        tip:SetCustomLineSpacing(6)
        tip:Show()
        assert(tip:GetHeight() < oldHeight, "hidden first row must not contribute height")
        local point, relative, relativePoint, x, y = second:GetPoint(1)
        assert(point == "TOP" and relative == first and relativePoint == "BOTTOM" and x == 0 and y == -6, "second row still anchors to hidden predecessor")
    "#).unwrap();
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn blizzard_line_handler_only_adds_nonempty_rows() {
    let env = setup_full_env();
    env.exec(
        r#"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        GameTooltip:AddLineDataText({ leftText = "" })
        assert(GameTooltip:NumLines() == 0, "empty single line must not append")
        GameTooltip:AddLineDataText({ leftText = "", rightText = "" })
        assert(GameTooltip:NumLines() == 0, "empty double line must not append")
        GameTooltip:AddLineDataText({ leftText = " " })
        assert(GameTooltip:NumLines() == 1, "space is a nonempty line")
        GameTooltip:AddLineDataText({ leftText = "", rightText = "Right" })
        assert(GameTooltip:NumLines() == 2, "right-only line must append")
    "#,
    )
    .unwrap();
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn blizzard_double_line_nonempty_right_disables_requested_wrap() {
    let env = setup_full_env();
    env.exec(
        r#"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { type = 0, leftText = "First", rightText = "Value", wrapText = true },
        } } }))
    "#,
    )
    .unwrap();
    let width: f32 = env.eval("return GameTooltip:GetWidth()").unwrap();
    let mut font = wow_ui_sim::render::WowFontSystem::new();
    let expected = font.measure_text_width("First", None, 14.0)
        + font.measure_text_width("Value", None, 14.0)
        + 38.4
        + 20.48;
    assert!(
        (width - expected).abs() < 0.01,
        "nonempty right must disable wrapping: {width} vs {expected}"
    );
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn blizzard_double_line_reserves_native_column_gap() {
    let env = setup_full_env();
    env.exec(
        r#"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { type = 0, leftText = "First", rightText = "Value" },
        } } }))
    "#,
    )
    .unwrap();
    let width: f32 = env.eval("return GameTooltip:GetWidth()").unwrap();
    let mut font = wow_ui_sim::render::WowFontSystem::new();
    let left = font.measure_text_width("First", None, 14.0);
    let right = font.measure_text_width("Value", None, 14.0);
    let expected = left + right + 38.4 + 20.48;
    assert!(
        (width - expected).abs() < 0.01,
        "width {width} must include native two-column reservation {expected}"
    );
    use iced::{Point, Rectangle, Size};
    use wow_ui_sim::iced_app::tooltip::{TooltipRender, build_tooltip_quads, collect_tooltip_data};
    use wow_ui_sim::render::glyph::emit_text_quads;
    use wow_ui_sim::render::{GlyphAtlas, QuadBatch};
    use wow_ui_sim::widget::{TextJustify, TextOutline};
    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let data = collect_tooltip_data(&state);
    let height = state.widgets.get(id).unwrap().height;
    let mut glyphs = GlyphAtlas::new();
    let mut actual = QuadBatch::new();
    build_tooltip_quads(
        TooltipRender {
            batch: &mut actual,
            bounds: Rectangle::new(Point::new(100.0, 200.0), Size::new(width, height)),
            tooltip_data: Some(&data),
            id,
            eff_alpha: 1.0,
            eff_scale: 1.0,
            draw_background: false,
        },
        &mut Some((&mut font, &mut glyphs)),
    );
    let actual_right = actual
        .vertices
        .iter()
        .filter(|vertex| vertex.tex_index == wow_ui_sim::render::shader::GLYPH_ATLAS_TEX_INDEX)
        .nth(20)
        .expect("right glyph")
        .position;
    let mut reference = QuadBatch::new();
    let right_edge = 110.0 + width - 20.48;
    emit_text_quads(
        &mut reference,
        &mut font,
        &mut glyphs,
        "Value",
        Rectangle::new(
            Point::new(right_edge - 200.0, 210.0),
            Size::new(200.0, 17.0),
        ),
        None,
        14.0,
        [1.0; 4],
        TextJustify::Right,
        TextJustify::Center,
        wow_ui_sim::render::shader::GLYPH_ATLAS_TEX_INDEX,
        None,
        (0.0, 0.0),
        TextOutline::None,
        false,
        0,
        0.0,
        false,
        None,
        None,
    );
    let expected_right = reference.vertices[0].position;
    assert!(
        (actual_right[0] - expected_right[0]).abs() < 0.01
            && (actual_right[1] - expected_right[1]).abs() < 0.01,
        "right glyph position {actual_right:?}, expected {expected_right:?}"
    );
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn blizzard_line_handler_anchors_indexed_rows_and_right_column() {
    let env = setup_full_env();
    env.exec(
        r#"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { type = 0, leftText = "First" },
            { type = 0, leftText = "Second", rightText = "Value" },
        } } }))
        local first = GameTooltip:GetLeftLine(1)
        local second = GameTooltip:GetLeftLine(2)
        local right = GameTooltip:GetRightLine(2)
        assert(second:GetNumPoints() == 2, "second left anchor count")
        local point, relative, relativePoint, x, y = second:GetPoint(1)
        assert(point == "TOP" and relative == first and relativePoint == "BOTTOM" and x == 0 and y == -2, "second left follows preceding index")
        point, relative, relativePoint, x, y = second:GetPoint(2)
        assert(point == "LEFT" and relative == GameTooltip and relativePoint == "LEFT" and x == 10 and y == 0, "second left horizontal anchor")
        assert(right:GetNumPoints() == 1, "right anchor count")
        point, relative, relativePoint, x, y = right:GetPoint(1)
        assert(point == "RIGHT" and relative == second and relativePoint == "LEFT" and math.abs(x - (GameTooltip:GetWidth() - 20.48)) < 0.01 and y == 0, "right anchor uses selected content width")
        GameTooltip:SetCustomLineSpacing(6)
        GameTooltip:Show()
        point, relative, relativePoint, x, y = second:GetPoint(1)
        assert(relative == first and y == -6, "spacing invalidates later left anchors")
        "#,
    )
    .unwrap();
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn ringed_frame_tooltip_applies_native_outer_and_explicit_padding() {
    let env = setup_full_env();
    let addons = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    wow_ui_sim::loader::load_addon(
        &env.loader_env(),
        &addons.join("Blizzard_CustomizationUI/Blizzard_CustomizationUI.toc"),
    )
    .unwrap();
    env.exec(
        r#"
        local region = CreateFrame("Frame", nil, UIParent, "CustomizationFrameWithTooltipTemplate")
        local PaddingTooltip = CustomizationNoHeaderTooltip
        region.tooltipAnchor = "ANCHOR_NONE"
        region.tooltipMinWidth = 300
        region.tooltipPadding = 3
        region:AddTooltipLine("x")
        region:OnEnter()
        local width, height = PaddingTooltip:GetWidth(), PaddingTooltip:GetHeight()
        assert(math.abs(width - 326.48) < 0.01, "native outer width padding: " .. width .. " sides: " .. tostring(PaddingTooltip:GetPadding()))
        PaddingTooltip:SetPadding(3, 5, 7, 11)
        assert(math.abs(PaddingTooltip:GetWidth() - 330.48) < 0.01, "explicit horizontal sides")
        assert(math.abs(PaddingTooltip:GetHeight() - height - 10) < 0.01, "explicit vertical sides")
        local right, bottom, left, top = PaddingTooltip:GetPadding()
        assert(right == 3 and bottom == 5 and left == 7 and top == 11, "padding side order")
        PaddingTooltip:SetPadding(2, 4)
        right, bottom, left, top = PaddingTooltip:GetPadding()
        assert(right == 2 and bottom == 4 and left == 7 and top == 11, "omitted sides persist")
        assert(math.abs(PaddingTooltip:GetWidth() - 329.48) < 0.01, "preserved left padding")
        assert(math.abs(PaddingTooltip:GetHeight() - height - 9) < 0.01, "preserved top padding")
        "#,
    )
    .unwrap();
}

retail_game_tooltip_data_fixture! {
#[test]
fn action_button_hover_preserves_spell_identity_for_tooltip_postcall() {
    let env = setup_full_env();
    let button_id = {
        let mut state = env.state().borrow_mut();
        state.action_bars.insert(5, 45524);
        let id = state.widgets.get_id_by_name("ActionButton5").unwrap();
        state.hovered_frame = Some(id);
        id
    };
    env.exec(
        r#"
        ActionButton5.action = 5
        ActionButton5:Show()
        ClickedStylePostcall = { calls = 0 }
        TooltipDataProcessor.AddTooltipPostCall(Enum.TooltipDataType.Spell, function(tooltip)
            if tooltip:IsForbidden() or tooltip.GetSpell == nil then return end
            local name, spellId = tooltip:GetSpell()
            if spellId == nil or issecretvalue(spellId) then return end
            ClickedStylePostcall.calls = ClickedStylePostcall.calls + 1
            ClickedStylePostcall.id = spellId
            ClickedStylePostcall.name = name
            tooltip:AddLine("Bound to test key")
        end)
        "#,
    )
    .unwrap();
    env.state().borrow_mut().lua_errors.clear();

    env.fire_script_handler(button_id, "OnEnter", vec![])
        .expect("actual ActionButton OnEnter should finish");

    env.exec(
        r#"
        assert(ClickedStylePostcall.calls > 0, "spell postcall never completed")
        assert(ClickedStylePostcall.id == 45524, "postcall lost action spell identity")
        assert(ClickedStylePostcall.name == C_Spell.GetSpellName(45524))
        assert(GameTooltip:GetPrimaryTooltipData().id == 45524)
        assert(GameTooltip:IsShown())
        local found = false
        for i = 1, GameTooltip:NumLines() do
            if GameTooltip:GetLeftLine(i):GetText() == "Bound to test key" then found = true end
        end
        assert(found, "postcall could not append its binding line")
        "#,
    )
    .expect("native TooltipDataHandler and GetSpell preserve action identity for addon postcalls");
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "hover must not record a swallowed postcall error"
    );
}
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn blizzard_line_processing_defers_resize_until_show() {
    let env = setup_full_env();
    env.exec(
        r#"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { type = 0, leftText = "Short" },
        } } }))
        local width, height = GameTooltip:GetWidth(), GameTooltip:GetHeight()
        GameTooltip:AddLineDataText({ type = 0, leftText =
            "A substantially longer line that must not resize the tooltip before Show" })
        assert(GameTooltip:GetWidth() == width, "AddLineDataText resized width before Show")
        assert(GameTooltip:GetHeight() == height, "AddLineDataText resized height before Show")
        GameTooltip:Show()
        assert(GameTooltip:GetWidth() > width, "Show did not resize the tooltip")
        assert(GameTooltip:GetHeight() > height, "Show did not update tooltip height")
        "#,
    )
    .unwrap();
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn blizzard_double_line_processing_defers_resize_until_show() {
    let env = setup_full_env();
    env.exec(
        r#"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { type = 0, leftText = "Short" },
        } } }))
        local width, height = GameTooltip:GetWidth(), GameTooltip:GetHeight()
        GameTooltip:AddLineDataText({ type = 0, leftText = "A longer left column", rightText = "Right column" })
        assert(GameTooltip:GetWidth() == width, "double line resized width before Show")
        assert(GameTooltip:GetHeight() == height, "double line resized height before Show")
        GameTooltip:Show()
        assert(GameTooltip:GetWidth() > width, "Show did not resize the tooltip")
        assert(GameTooltip:GetHeight() > height, "Show did not update tooltip height")
        "#,
    )
    .unwrap();
}

// API-level timing probes: TooltipDataHandler's original line consumer supplies
// the pending line; each setter is invoked directly to isolate its timing.
#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
fn assert_setter_defers_tooltip_layout(setter: &str) {
    let env = setup_full_env();
    env.exec(
        r#"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { type = 0, leftText = "Short" },
        } } }))
        timingWidth, timingHeight = GameTooltip:GetWidth(), GameTooltip:GetHeight()
        GameTooltip:AddLineDataText({ type = 0, leftText = "A much longer pending tooltip row for layout timing" })
        "#,
    )
    .unwrap();
    env.exec(setter).unwrap();
    env.exec(
        r#"
        assert(GameTooltip:GetWidth() == timingWidth, "setter resized width before Show")
        assert(GameTooltip:GetHeight() == timingHeight, "setter resized height before Show")
        GameTooltip:Show()
        assert(GameTooltip:GetWidth() > timingWidth, "Show did not resize width")
        assert(GameTooltip:GetHeight() > timingHeight, "Show did not resize height")
        "#,
    )
    .unwrap();
}

// Original consumer cannot observe pending content: SetOwner clears lines before
// SetCustomWordWrapMinWidth (ProfessionsRecipeSchematicForm.lua:650-654;
// tooltip/owner.rs::record_tooltip_owner). The consumer is tested separately.
#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn api_wrap_width_defers_tooltip_layout() {
    assert_setter_defers_tooltip_layout("GameTooltip:SetCustomWordWrapMinWidth(350)");
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn api_line_spacing_defers_tooltip_layout() {
    assert_setter_defers_tooltip_layout("GameTooltip:SetCustomLineSpacing(9)");
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn api_shrink_to_fit_defers_tooltip_layout() {
    assert_setter_defers_tooltip_layout("GameTooltip:SetShrinkToFitWrapped(false)");
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn api_minimum_width_defers_tooltip_layout() {
    assert_setter_defers_tooltip_layout("GameTooltip:SetMinimumWidth(300, true)");
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn api_padding_synchronously_lays_out_pending_tooltip_lines() {
    let env = setup_full_env();
    env.exec(
        r#"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { type = 0, leftText = "Short" },
        } } }))
        local width, height = GameTooltip:GetWidth(), GameTooltip:GetHeight()
        GameTooltip:AddLineDataText({ type = 0, leftText = "A much longer pending tooltip row for layout timing" })
        GameTooltip:SetPadding(0, 6)
        assert(GameTooltip:GetWidth() > width, "SetPadding did not synchronously resize width")
        assert(GameTooltip:GetHeight() > height, "SetPadding did not synchronously resize height")
        "#,
    )
    .unwrap();
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn api_ownerless_padding_does_not_size_content() {
    let env = setup_full_env();
    env.exec(
        r#"
        GameTooltip:Hide()
        local width, height = GameTooltip:GetWidth(), GameTooltip:GetHeight()
        GameTooltip:AddLine("Ownerless text that must not trigger content sizing")
        GameTooltip:SetPadding(1, 2)
        assert(GameTooltip:GetWidth() == width, "ownerless SetPadding resized width")
        assert(GameTooltip:GetHeight() == height, "ownerless SetPadding resized height")
        "#,
    )
    .unwrap();
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn blizzard_money_helper_defers_minimum_width_layout() {
    let env = setup_full_env();
    env.exec(
        r#"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        GameTooltip:AddLine("Short")
        GameTooltip:Show()
        local width, height = GameTooltip:GetWidth(), GameTooltip:GetHeight()
        SetTooltipMoney(GameTooltip, 123456789, nil, "Price:")
        assert(GameTooltip:GetMinimumWidth() > 0, "money helper did not set minimum width")
        assert(GameTooltip:GetWidth() == width, "money helper resized width before Show")
        assert(GameTooltip:GetHeight() == height, "money helper resized height before Show")
        GameTooltip:Show()
        assert(GameTooltip:GetHeight() > height, "Show did not lay out money line")
        "#,
    )
    .unwrap();
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn blizzard_show_synchronously_uses_glyph_measurement() {
    let env = setup_full_env();
    env.exec(
        r#"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { type = 0, leftText = "WWWWWWWWWW" },
        } } }))
        assert(GameTooltip:IsShown(), "processed tooltip must be shown")
        assert(GameTooltip:GetOwner() == UIParent, "processed tooltip must retain its owner")
        shownWidth, shownHeight = GameTooltip:GetWidth(), GameTooltip:GetHeight()
        "#,
    )
    .unwrap();
    let mut font = wow_ui_sim::render::font::WowFontSystem::new();
    wow_ui_sim::iced_app::tooltip::prepare_tooltip_layout(&mut env.state().borrow_mut(), &mut font);
    env.exec(
        r#"
        assert(GameTooltip:GetWidth() == shownWidth, "Show did not synchronously measure glyph width")
        assert(GameTooltip:GetHeight() == shownHeight, "Show did not synchronously measure glyph height")
        GameTooltip:ClearLines()
        GameTooltip:AddLineDataText({ type = 0, leftText = "iiiiiiiiii" })
        GameTooltip:Show()
        assert(GameTooltip:GetWidth() < shownWidth, "equal-length glyphs must have distinct widths")
        "#,
    )
    .unwrap();
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn blizzard_line_processing_does_not_resize_on_render_before_show() {
    let env = setup_full_env();
    env.exec(
        r#"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { type = 0, leftText = "Short" },
        } } }))
        "#,
    )
    .unwrap();
    let mut font = wow_ui_sim::render::font::WowFontSystem::new();
    wow_ui_sim::iced_app::tooltip::prepare_tooltip_layout(&mut env.state().borrow_mut(), &mut font);
    env.exec(
        r#"
        renderWidth, renderHeight = GameTooltip:GetWidth(), GameTooltip:GetHeight()
        GameTooltip:AddLineDataText({ type = 0, leftText = "A much longer pending tooltip row that should wait for Show" })
        "#,
    )
    .unwrap();
    wow_ui_sim::iced_app::tooltip::prepare_tooltip_layout(&mut env.state().borrow_mut(), &mut font);
    env.exec(
        r#"
        assert(GameTooltip:GetWidth() == renderWidth, "render resized tooltip width before Show")
        assert(GameTooltip:GetHeight() == renderHeight, "render resized tooltip height before Show")
        GameTooltip:Show()
        assert(GameTooltip:GetWidth() > renderWidth, "Show did not resize tooltip")
        "#,
    )
    .unwrap();
}

retail_game_tooltip_data_fixture! {
#[test]
fn test_tooltip_mixins_load_in_full_env() {
    let env = setup_full_env();

    let tooltip_data_handler_mixin_type: String =
        env.eval("return type(TooltipDataHandlerMixin)").unwrap();
    let game_tooltip_data_mixin_type: String =
        env.eval("return type(GameTooltipDataMixin)").unwrap();
    let status_bar_onload_type: String = env
        .eval("return type(GameTooltipStatusBar.OnLoad)")
        .unwrap();
    let tooltip_onload_type: String = env.eval("return type(GameTooltip.OnLoad)").unwrap();

    assert_eq!(tooltip_data_handler_mixin_type, "table");
    assert_eq!(game_tooltip_data_mixin_type, "table");
    assert_eq!(status_bar_onload_type, "function");
    assert_eq!(tooltip_onload_type, "function");
}
}

retail_game_tooltip_data_fixture! {
#[test]
fn test_game_tooltip_process_info_with_constructed_tooltip_info_populates_lines() {
    let env = setup_full_env();

    let processed: bool = env
        .eval(
            r#"
            GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
            local tooltipInfo = {
                getterName = "GetItemByID",
                getterArgs = { [1] = 6948, n = 1 },
            }
            return GameTooltip:ProcessInfo(tooltipInfo)
        "#,
        )
        .unwrap();

    assert!(
        processed,
        "GameTooltip:ProcessInfo should succeed for a valid item tooltipInfo"
    );

    let num_lines: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert!(
        num_lines > 0,
        "GameTooltip:ProcessInfo should populate tooltip lines for a valid tooltipInfo"
    );

    let first_line: String = env
        .eval("return GameTooltip:GetLeftLine(1):GetText()")
        .unwrap();
    assert_eq!(
        first_line, "Hearthstone",
        "GameTooltip:ProcessInfo should set the first tooltip line from the constructed tooltipInfo"
    );
}
}

/// Test that hovering a micro menu button shows the tooltip with text.
///
/// Uses the full Blizzard UI environment so OnEnter scripts run properly.
#[test]
fn test_micro_menu_hover_shows_tooltip() {
    let env = setup_full_env();

    // The inherited OnEnter is gated on `self:IsEnabled() == 1`
    // (FrameXML/MainMenuBarMicroButtons.xml:13), so 12340 Button:IsEnabled must
    // return the number 1 when enabled and 0 when disabled. Enabled rows also
    // compare `== 1` and disabled rows `== 0` in UIPanelTemplates.xml:68-96,
    // LootFrame.xml:71, LFDFrame.xml:59, KnowledgeBaseFrame.xml:258,
    // ChannelFrame.lua:258, PaperDollFrame.xml:1440 and TutorialFrame.lua:835.
    if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::Wrath {
        let (enabled_type, enabled_is_one, disabled_is_zero): (String, bool, bool) = env
            .eval(
                r#"
                local disabled = CreateFrame('Button', 'WrathDisabledProbe', UIParent)
                disabled:Disable()
                return type(CharacterMicroButton:IsEnabled()),
                       CharacterMicroButton:IsEnabled() == 1,
                       disabled:IsEnabled() == 0
            "#,
            )
            .expect("IsEnabled values should be readable");
        assert_eq!(enabled_type, "number", "12340 IsEnabled returns a number");
        assert!(enabled_is_one, "an enabled button returns 1");
        assert!(disabled_is_zero, "a disabled button returns 0");
    }

    // Find the CharacterMicroButton frame ID
    let btn_id = {
        let state = env.state().borrow();
        state
            .widgets
            .get_id_by_name("CharacterMicroButton")
            .expect("CharacterMicroButton should exist")
    };

    // Set hovered_frame so IsMouseMotionFocus() returns true
    env.state().borrow_mut().hovered_frame = Some(btn_id);

    // Fire OnEnter (this is what handle_mouse_move does)
    env.fire_script_handler(btn_id, "OnEnter", vec![]).unwrap();

    // Check tooltip state
    let visible: bool = env.eval("return GameTooltip:IsVisible()").unwrap();
    let num_lines: i32 = env.eval("return GameTooltip:NumLines()").unwrap();

    assert!(
        visible,
        "GameTooltip should be visible after micro menu hover"
    );
    assert!(
        num_lines > 0,
        "GameTooltip should have at least one line, got {}",
        num_lines
    );

    // Verify the tooltip text content
    {
        let state = env.state().borrow();
        let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        let td = state
            .tooltips
            .get(&gt_id)
            .expect("tooltip data should exist");
        assert!(!td.lines.is_empty(), "tooltip should have line data");
        eprintln!("Tooltip text: {:?}", td.lines[0].left_text);
    }

    // Propagate effective_alpha via get_strata_buckets, then verify
    {
        let mut state = env.state().borrow_mut();
        let _ = state.get_strata_buckets();
    }
    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    assert!(
        state
            .widgets
            .get(gt_id)
            .is_some_and(|f| f.effective_alpha > 0.0),
        "GameTooltip should be ancestor-visible (effective_alpha > 0)"
    );

    // Check frame dimensions (tooltip should not be 0x0)
    let frame = state.widgets.get(gt_id).unwrap();
    eprintln!(
        "Tooltip frame: visible={}, width={}, height={}",
        frame.visible, frame.width, frame.height
    );
}

#[test]
fn test_character_slot_hover_shows_inventory_tooltip() {
    let env = setup_full_env();
    open_character_panel(&env);

    let slot_id = {
        let state = env.state().borrow();
        state
            .widgets
            .get_id_by_name("CharacterHeadSlot")
            .expect("CharacterHeadSlot should exist after opening character panel")
    };

    env.state().borrow_mut().hovered_frame = Some(slot_id);
    env.fire_script_handler(slot_id, "OnEnter", vec![]).unwrap();

    let visible: bool = env.eval("return GameTooltip:IsVisible()").unwrap();
    let num_lines: i32 = env.eval("return GameTooltip:NumLines()").unwrap();

    assert!(
        visible,
        "GameTooltip should be visible after hovering CharacterHeadSlot"
    );
    assert!(
        num_lines >= 3,
        "Character slot hover should populate item tooltip lines, got {}",
        num_lines
    );

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state
        .tooltips
        .get(&gt_id)
        .expect("tooltip data should exist after character slot hover");
    assert_eq!(td.lines[0].left_text, "Entombed Seraph's Casque");
    assert_eq!(td.lines[1].left_text, "Item Level 571");
    assert_eq!(td.lines[2].left_text, "Head");
}

#[test]
fn test_buff_icon_hover_shows_aura_tooltip() {
    let env = setup_full_env();
    refresh_buff_frame(&env);

    let expected_name: String = env
        .eval(
            if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::Wrath {
                // Wrath FrameXML/BuffFrame.xml:26-30 binds the original aura button script.
                r#"
                local button = assert(BuffButton1)
                assert(button:IsShown(), "first Wrath buff button must be shown")
                local name = assert(UnitBuff("player", button:GetID()))
                assert(button:GetScript("OnEnter"))(button)
                return name
                "#
            } else {
            r#"
            for _, button in ipairs(BuffFrame.auraFrames) do
                if button:IsShown() and button.buttonInfo and button.buttonInfo.index then
                    button:OnEnter()
                    return C_TooltipInfo.GetUnitAura("player", button.buttonInfo.index, button:GetFilter()).lines[1].leftText
                end
            end
            error("No visible buff icon with tooltip data")
            "#
            },
        )
        .unwrap();

    let visible: bool = env.eval("return GameTooltip:IsVisible()").unwrap();
    let num_lines: i32 = env.eval("return GameTooltip:NumLines()").unwrap();

    assert!(
        visible,
        "GameTooltip should be visible after hovering a buff icon"
    );
    assert!(
        num_lines >= 1,
        "Buff icon hover should populate aura tooltip lines, got {}",
        num_lines
    );

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state
        .tooltips
        .get(&gt_id)
        .expect("tooltip data should exist after buff hover");
    assert_eq!(td.lines[0].left_text, expected_name);
}

retail_game_tooltip_data_fixture! {
#[test]
fn test_world_loot_object_tooltip_shows_world_loot_data() {
    let env = setup_full_env();

    env.exec(
        r#"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        assert(GameTooltip.SetWorldLootObject, "GameTooltip:SetWorldLootObject should exist")
        local shown = GameTooltip:SetWorldLootObject("player")
        assert(shown == true, "SetWorldLootObject should report success for the supported unit")
        "#,
    )
    .expect("Failed to show the world loot object tooltip");

    let visible: bool = env.eval("return GameTooltip:IsVisible()").unwrap();
    let num_lines: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    let getter_name: String = env
        .eval(
            r#"
            local info = GameTooltip:GetPrimaryTooltipInfo()
            return info and info.getterName or ""
            "#,
        )
        .unwrap();

    assert!(
        visible,
        "GameTooltip should be visible after SetWorldLootObject"
    );
    assert!(
        num_lines >= 1,
        "GameTooltip should have world-loot tooltip lines, got {}",
        num_lines
    );
    assert_eq!(getter_name, "GetWorldLootObject");

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state
        .tooltips
        .get(&gt_id)
        .expect("tooltip data should exist after SetWorldLootObject");
    assert!(!td.lines[0].left_text.is_empty());
}
}

retail_game_tooltip_data_fixture! {
#[test]
fn test_tooltip_comparison_manager_get_comparison_item_data_uses_guid_lookup() {
    let env = setup_full_env();

    let has_real_tooltip: bool = env
        .eval(
            r#"
            assert(type(TooltipComparisonManager) == "table", "TooltipComparisonManager should be loaded")

            local guid = C_Item.GetItemGUID({ bagID = 0, slotIndex = 1 })
            local tooltipData = TooltipComparisonManager:GetComparisonItemData({ guid = guid })
            if not guid or guid == "" or not tooltipData or tooltipData.type ~= Enum.TooltipDataType.Item or not tooltipData.lines then
                return false
            end

            local nameLine = tooltipData.lines[1]
            return tooltipData.guid == guid
                and nameLine
                and nameLine.type == Enum.TooltipDataLineType.ItemName
                and nameLine.leftText == "Hearthstone"
            "#,
        )
        .unwrap();

    assert!(
        has_real_tooltip,
        "TooltipComparisonManager should resolve GUID-backed comparison items through C_TooltipInfo.GetItemByGUID",
    );
}
}

retail_game_tooltip_data_fixture! {
#[test]
fn test_tooltip_data_handler_set_owned_item_by_id_uses_owned_item_getter() {
    let env = setup_full_env();

    let has_real_tooltip: bool = env
        .eval(
            r#"
            assert(GameTooltip.SetOwnedItemByID, "GameTooltip:SetOwnedItemByID should exist")
            GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
            GameTooltip:SetOwnedItemByID(6948)

            local info = GameTooltip:GetPrimaryTooltipInfo()
            local tooltipData = GameTooltip:GetPrimaryTooltipData()
            if not info or not tooltipData or tooltipData.type ~= Enum.TooltipDataType.Item or not tooltipData.lines then
                return false
            end

            local nameLine = tooltipData.lines[1]
            return info.getterName == "GetOwnedItemByID"
                and nameLine
                and nameLine.type == Enum.TooltipDataLineType.ItemName
                and nameLine.leftText == "Hearthstone"
            "#,
        )
        .unwrap();

    assert!(
        has_real_tooltip,
        "TooltipDataHandler should populate GameTooltip:SetOwnedItemByID through C_TooltipInfo.GetOwnedItemByID",
    );
}
}

retail_game_tooltip_data_fixture! {
#[test]
fn test_tooltip_data_handler_set_inventory_item_uses_inventory_getter() {
    let env = setup_full_env();

    let has_real_tooltip = inventory_item_tooltip_uses_inventory_getter(&env);

    assert!(
        has_real_tooltip,
        "TooltipDataHandler should populate GameTooltip:SetInventoryItem through C_TooltipInfo.GetInventoryItem",
    );
}
}

#[cfg(not(feature = "client-wrath"))]
fn inventory_item_tooltip_uses_inventory_getter(env: &WowLuaEnv) -> bool {
    env.eval(
        r#"
        assert(GameTooltip.SetInventoryItem, "GameTooltip:SetInventoryItem should exist")
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        local ok = GameTooltip:SetInventoryItem("player", 1)
        if not ok then
            return false
        end

        local info = GameTooltip:GetPrimaryTooltipInfo()
        local tooltipData = GameTooltip:GetPrimaryTooltipData()
        if not info or not tooltipData or tooltipData.type ~= Enum.TooltipDataType.Item or not tooltipData.lines then
            return false
        end

        local nameLine = tooltipData.lines[1]
        return info.getterName == "GetInventoryItem"
            and nameLine
            and nameLine.type == Enum.TooltipDataLineType.ItemName
            and nameLine.leftText == "Entombed Seraph's Casque"
        "#,
    )
    .unwrap()
}

retail_game_tooltip_data_fixture! {
#[test]
fn test_tooltip_data_handler_set_recipe_result_item_uses_recipe_getters() {
    let env = setup_full_env();

    let recipe_result_ok: bool = env
        .eval(
            r#"
            assert(GameTooltip.SetRecipeResultItem, "GameTooltip:SetRecipeResultItem should exist")
            GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
            GameTooltip:SetRecipeResultItem(100005, {}, nil, nil, nil)

            local info = GameTooltip:GetPrimaryTooltipInfo()
            local tooltipData = GameTooltip:GetPrimaryTooltipData()
            if not info or not tooltipData or tooltipData.type ~= Enum.TooltipDataType.Item or not tooltipData.lines then
                return false
            end

            local nameLine = tooltipData.lines[1]
            return info.getterName == "GetRecipeResultItem"
                and nameLine
                and nameLine.type == Enum.TooltipDataLineType.ItemName
                and nameLine.leftText == "Ordained Forge Maul"
            "#,
        )
        .unwrap();

    let order_result_ok: bool = env
        .eval(
            r#"
            assert(GameTooltip.SetRecipeResultItemForOrder, "GameTooltip:SetRecipeResultItemForOrder should exist")
            GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
            GameTooltip:SetRecipeResultItemForOrder(100005, {}, 1, nil, nil)

            local info = GameTooltip:GetPrimaryTooltipInfo()
            local tooltipData = GameTooltip:GetPrimaryTooltipData()
            if not info or not tooltipData or tooltipData.type ~= Enum.TooltipDataType.Item or not tooltipData.lines then
                return false
            end

            local nameLine = tooltipData.lines[1]
            return info.getterName == "GetRecipeResultItemForOrder"
                and nameLine
                and nameLine.leftText == "Ordained Forge Maul"
            "#,
        )
        .unwrap();

    assert!(
        recipe_result_ok && order_result_ok,
        "TooltipDataHandler should populate both recipe result item accessors through C_TooltipInfo",
    );
}
}

retail_game_tooltip_data_fixture! {
#[test]
fn test_tooltip_data_handler_set_minimap_mouseover_uses_minimap_getter() {
    let env = setup_full_env();

    let has_real_tooltip: bool = env
        .eval(
            r#"
            assert(GameTooltip.SetMinimapMouseover, "GameTooltip:SetMinimapMouseover should exist")
            GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
            GameTooltip:SetMinimapMouseover()

            local info = GameTooltip:GetPrimaryTooltipInfo()
            local tooltipData = GameTooltip:GetPrimaryTooltipData()
            if not info or not tooltipData or tooltipData.type ~= Enum.TooltipDataType.MinimapMouseover or not tooltipData.lines then
                return false
            end

            local zoneLine = tooltipData.lines[1]
            local subZoneLine = tooltipData.lines[2]
            return info.getterName == "GetMinimapMouseover"
                and zoneLine
                and zoneLine.leftText == "Stormwind City"
                and subZoneLine
                and subZoneLine.leftText == "Trade District"
            "#,
        )
        .unwrap();

    assert!(
        has_real_tooltip,
        "TooltipDataHandler should populate GameTooltip:SetMinimapMouseover through C_TooltipInfo.GetMinimapMouseover",
    );
}
}

retail_game_tooltip_data_fixture! {
#[test]
fn test_tooltip_data_handler_set_upgrade_item_uses_upgrade_getter() {
    let env = setup_full_env();

    let has_real_tooltip: bool = env
        .eval(
            r#"
            C_ItemUpgrade.SetItemUpgradeFromLocation({ bagID = 0, slotIndex = 1 })
            assert(GameTooltip.SetUpgradeItem, "GameTooltip:SetUpgradeItem should exist")
            GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
            GameTooltip:SetUpgradeItem()

            local info = GameTooltip:GetPrimaryTooltipInfo()
            local tooltipData = GameTooltip:GetPrimaryTooltipData()
            if not info or not tooltipData or tooltipData.type ~= Enum.TooltipDataType.Item or not tooltipData.lines then
                return false
            end

            local nameLine = tooltipData.lines[1]
            local itemLevelLine = tooltipData.lines[2]
            return info.getterName == "GetUpgradeItem"
                and nameLine
                and nameLine.leftText == "Hearthstone"
                and itemLevelLine
                and itemLevelLine.leftText == "Item Level 1"
            "#,
        )
        .unwrap();

    assert!(
        has_real_tooltip,
        "TooltipDataHandler should populate GameTooltip:SetUpgradeItem through C_TooltipInfo.GetUpgradeItem",
    );
}
}
