use iced::{Point, Rectangle, Size};

use super::*;
use crate::lua_api::tooltip::{TooltipData, TooltipLine, TooltipTextSegment};
use crate::render::{GlyphAtlas, QuadBatch, TextureRequest, WowFontSystem};
use crate::widget::{Frame, WidgetType};

fn request_bounds(batch: &QuadBatch, request: &TextureRequest) -> (f32, f32, f32, f32) {
    let start = request.vertex_start as usize;
    let end = start + request.vertex_count as usize;
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for vertex in &batch.vertices[start..end] {
        min_x = min_x.min(vertex.position[0]);
        min_y = min_y.min(vertex.position[1]);
        max_x = max_x.max(vertex.position[0]);
        max_y = max_y.max(vertex.position[1]);
    }

    (min_x, min_y, max_x, max_y)
}

fn union_bounds(
    bounds: impl Iterator<Item = (f32, f32, f32, f32)>,
) -> Option<(f32, f32, f32, f32)> {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    let mut found = false;

    for (x0, y0, x1, y1) in bounds {
        min_x = min_x.min(x0);
        min_y = min_y.min(y0);
        max_x = max_x.max(x1);
        max_y = max_y.max(y1);
        found = true;
    }

    found.then_some((min_x, min_y, max_x, max_y))
}

fn glyph_bounds(batch: &QuadBatch) -> Option<(f32, f32, f32, f32)> {
    let glyph_tex_index = GLYPH_ATLAS_TEX_INDEX;
    union_bounds(batch.vertices.iter().filter_map(|vertex| {
        (vertex.tex_index == glyph_tex_index).then_some((
            vertex.position[0],
            vertex.position[1],
            vertex.position[0],
            vertex.position[1],
        ))
    }))
}

fn has_glyph_color(batch: &QuadBatch, color: [f32; 4]) -> bool {
    batch
        .vertices
        .iter()
        .any(|vertex| vertex.tex_index == GLYPH_ATLAS_TEX_INDEX && vertex.color == color)
}

fn has_solid_color(batch: &QuadBatch, color: [f32; 4]) -> bool {
    batch
        .vertices
        .iter()
        .any(|vertex| vertex.tex_index == -1 && vertex.color == color)
}

fn glyph_bounds_for_color(batch: &QuadBatch, color: [f32; 4]) -> Option<(f32, f32, f32, f32)> {
    union_bounds(batch.vertices.iter().filter_map(|vertex| {
        (vertex.tex_index == GLYPH_ATLAS_TEX_INDEX && vertex.color == color).then_some((
            vertex.position[0],
            vertex.position[1],
            vertex.position[0],
            vertex.position[1],
        ))
    }))
}

fn request_bounds_by_base_path(batch: &QuadBatch, base_path: &str) -> Option<(f32, f32, f32, f32)> {
    batch
        .texture_requests
        .iter()
        .find(|request| request.path.starts_with(base_path))
        .map(|request| request_bounds(batch, request))
}

fn render_single_line_tooltip_batch(
    bounds: Rectangle,
) -> (QuadBatch, (f32, f32, f32, f32), (f32, f32, f32, f32)) {
    let batch = build_single_line_tooltip_batch(bounds);
    let border_bounds = tooltip_border_bounds(&batch);
    let glyph_bounds = glyph_bounds(&batch).expect("tooltip text should emit glyph vertices");

    (batch, border_bounds, glyph_bounds)
}

fn build_single_line_tooltip_batch(bounds: Rectangle) -> QuadBatch {
    build_single_line_tooltip_batch_with_scale(bounds, 1.0)
}

fn build_single_line_tooltip_batch_with_scale(bounds: Rectangle, eff_scale: f32) -> QuadBatch {
    let data = TooltipRenderData {
        lines: vec![single_header_tooltip_line()],
        line_spacing: TOOLTIP_LINE_SPACING,
        left_text_offsets: vec![Some([10.0; 2])],
    };
    let mut batch = QuadBatch::new();
    let mut font_sys = WowFontSystem::new();
    let mut glyph_atlas = GlyphAtlas::new();
    let tooltip_data = HashMap::from([(42_u64, data)]);
    let mut text_ctx = Some((&mut font_sys, &mut glyph_atlas));

    build_tooltip_quads(
        TooltipRender {
            batch: &mut batch,
            bounds,
            tooltip_data: Some(&tooltip_data),
            id: 42,
            eff_alpha: 1.0,
            eff_scale,
            draw_background: true,
        },
        &mut text_ctx,
    );

    batch
}

fn single_header_tooltip_line() -> TooltipLineRender {
    TooltipLineRender {
        left_text: "Header".to_string(),
        left_color: [1.0, 1.0, 1.0, 1.0],
        left_segments: Vec::new(),
        right_text: None,
        right_color: [1.0, 1.0, 1.0, 1.0],
        right_segments: Vec::new(),
        font_size: TOOLTIP_HEADER_FONT_SIZE,
        wrap: false,
        measured_height: (TOOLTIP_HEADER_FONT_SIZE * 1.2).ceil(),
        path_attachment: None,
    }
}

fn tooltip_border_bounds(batch: &QuadBatch) -> (f32, f32, f32, f32) {
    union_bounds(
        batch
            .texture_requests
            .iter()
            .map(|request| request_bounds(batch, request)),
    )
    .expect("tooltip border should emit texture requests")
}

fn assert_text_origin_matches_nine_slice_corner(bounds: Rectangle, batch: &QuadBatch) {
    let ns = tooltip_nine_slice().expect("tooltip nine-slice info should exist");
    let tl_corner_bounds = request_bounds_by_base_path(batch, ns.corner_tl.file)
        .expect("tooltip top-left corner texture should emit one request");
    let text_origin = (
        bounds.x + tooltip_text_insets().left,
        bounds.y + tooltip_text_insets().top,
    );

    assert_eq!(
        tl_corner_bounds,
        (
            bounds.x,
            bounds.y,
            bounds.x + ns.corner_tl.width as f32,
            bounds.y + ns.corner_tl.height as f32,
        ),
        "top-left corner texture should match the tooltip's top-left corner coords"
    );
    assert!(
        (text_origin.0 - tl_corner_bounds.2 - (TOOLTIP_PADDING_H - TOOLTIP_CENTER_OVERLAP)).abs()
            <= f32::EPSILON,
        "text x origin should be offset from the top-left corner's right edge by padding minus overlap: text_origin={text_origin:?} tl_corner_bounds={tl_corner_bounds:?}"
    );
    assert!(
        (text_origin.1 - tl_corner_bounds.3 - (TOOLTIP_PADDING_V - TOOLTIP_CENTER_OVERLAP)).abs()
            <= f32::EPSILON,
        "text y origin should be offset from the top-left corner's bottom edge by padding minus overlap: text_origin={text_origin:?} tl_corner_bounds={tl_corner_bounds:?}"
    );
}

#[test]
fn collect_tooltip_data_applies_alpha_and_font_sizes() {
    let mut state = SimState::default();
    let mut frame = Frame::new(WidgetType::Frame, Some("GameTooltip".to_string()), None);
    frame.id = 42;
    frame.visible = true;
    frame.alpha = 0.35;
    state.widgets.register(frame);
    state.tooltips.insert(
        42,
        TooltipData {
            lines: vec![
                TooltipLine {
                    left_text: "Header".to_string(),
                    left_color: (1.0, 0.5, 0.25),
                    left_segments: Vec::new(),
                    right_text: Some("Right".to_string()),
                    right_color: (0.2, 0.3, 0.4),
                    right_segments: Vec::new(),
                    wrap: false,
                    texture: None,
                },
                TooltipLine {
                    left_text: "Body".to_string(),
                    left_color: (0.1, 0.2, 0.3),
                    left_segments: Vec::new(),
                    right_text: None,
                    right_color: (0.0, 0.0, 0.0),
                    right_segments: Vec::new(),
                    wrap: true,
                    texture: None,
                },
            ],
            ..TooltipData::default()
        },
    );

    let data = collect_tooltip_data(&state);
    let lines = &data.get(&42).unwrap().lines;

    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].font_size, TOOLTIP_HEADER_FONT_SIZE);
    assert_eq!(lines[1].font_size, TOOLTIP_BODY_FONT_SIZE);
    assert_eq!(lines[0].left_color, [1.0, 0.5, 0.25, 0.35]);
    assert_eq!(lines[0].right_color, [0.2, 0.3, 0.4, 0.35]);
    assert_eq!(lines[1].left_color, [0.1, 0.2, 0.3, 0.35]);
    assert!(lines[1].wrap);
}

#[test]
fn collect_tooltip_data_preserves_inline_color_segments() {
    let mut state = SimState::default();
    let mut frame = Frame::new(WidgetType::Frame, Some("GameTooltip".to_string()), None);
    frame.id = 42;
    frame.visible = true;
    frame.alpha = 0.5;
    state.widgets.register(frame);
    state.tooltips.insert(
        42,
        TooltipData {
            lines: vec![TooltipLine {
                left_text: "Plain Hot Plain".to_string(),
                left_color: (1.0, 0.82, 0.0),
                left_segments: vec![
                    TooltipTextSegment {
                        text: "Plain ".to_string(),
                        color: (1.0, 0.82, 0.0),
                    },
                    TooltipTextSegment {
                        text: "Hot".to_string(),
                        color: (1.0, 1.0, 1.0),
                    },
                ],
                right_text: None,
                right_color: (1.0, 1.0, 1.0),
                right_segments: Vec::new(),
                wrap: false,
                texture: None,
            }],
            ..TooltipData::default()
        },
    );

    let data = collect_tooltip_data(&state);
    let segments = &data.get(&42).unwrap().lines[0].left_segments;

    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].color, [1.0, 0.82, 0.0, 0.5]);
    assert_eq!(segments[1].color, [1.0, 1.0, 1.0, 0.5]);
}

#[test]
fn tooltip_renderer_emits_inline_color_segments() {
    let data = TooltipRenderData {
        lines: vec![TooltipLineRender {
            left_text: "AB".to_string(),
            left_color: [1.0, 0.82, 0.0, 1.0],
            left_segments: vec![
                TooltipTextSegmentRender {
                    text: "A".to_string(),
                    color: [1.0, 0.0, 0.0, 1.0],
                },
                TooltipTextSegmentRender {
                    text: "B".to_string(),
                    color: [0.0, 1.0, 0.0, 1.0],
                },
            ],
            right_text: None,
            right_color: [1.0, 1.0, 1.0, 1.0],
            right_segments: Vec::new(),
            font_size: TOOLTIP_HEADER_FONT_SIZE,
            wrap: false,
            measured_height: (TOOLTIP_HEADER_FONT_SIZE * 1.2).ceil(),
            path_attachment: None,
        }],
        line_spacing: TOOLTIP_LINE_SPACING,
        left_text_offsets: vec![Some([10.0; 2])],
    };

    let mut batch = QuadBatch::new();
    let mut font_sys = WowFontSystem::new();
    let mut glyph_atlas = GlyphAtlas::new();
    let tooltip_data = HashMap::from([(42_u64, data)]);
    let mut text_ctx = Some((&mut font_sys, &mut glyph_atlas));

    build_tooltip_quads(
        TooltipRender {
            batch: &mut batch,
            bounds: Rectangle::new(Point::new(100.0, 200.0), Size::new(80.0, 47.0)),
            tooltip_data: Some(&tooltip_data),
            id: 42,
            eff_alpha: 1.0,
            eff_scale: 1.0,
            draw_background: false,
        },
        &mut text_ctx,
    );

    assert!(has_glyph_color(&batch, [1.0, 0.0, 0.0, 1.0]));
    assert!(has_glyph_color(&batch, [0.0, 1.0, 0.0, 1.0]));
}

#[test]
fn tooltip_renderer_wraps_inline_color_segments() {
    let data = TooltipRenderData {
        lines: vec![TooltipLineRender {
            left_text: "Alpha Bravo Charlie Delta".to_string(),
            left_color: [1.0, 0.82, 0.0, 1.0],
            left_segments: vec![
                TooltipTextSegmentRender {
                    text: "Alpha Bravo ".to_string(),
                    color: [1.0, 0.0, 0.0, 1.0],
                },
                TooltipTextSegmentRender {
                    text: "Charlie Delta".to_string(),
                    color: [0.0, 1.0, 0.0, 1.0],
                },
            ],
            right_text: None,
            right_color: [1.0, 1.0, 1.0, 1.0],
            right_segments: Vec::new(),
            font_size: TOOLTIP_BODY_FONT_SIZE,
            wrap: true,
            measured_height: (TOOLTIP_BODY_FONT_SIZE * 3.0).ceil(),
            path_attachment: None,
        }],
        line_spacing: TOOLTIP_LINE_SPACING,
        left_text_offsets: vec![Some([10.0; 2])],
    };

    let mut batch = QuadBatch::new();
    let mut font_sys = WowFontSystem::new();
    let mut glyph_atlas = GlyphAtlas::new();
    let tooltip_data = HashMap::from([(42_u64, data)]);
    let mut text_ctx = Some((&mut font_sys, &mut glyph_atlas));
    let bounds = Rectangle::new(Point::new(100.0, 200.0), Size::new(95.0, 90.0));

    build_tooltip_quads(
        TooltipRender {
            batch: &mut batch,
            bounds,
            tooltip_data: Some(&tooltip_data),
            id: 42,
            eff_alpha: 1.0,
            eff_scale: 1.0,
            draw_background: false,
        },
        &mut text_ctx,
    );

    let insets = tooltip_text_insets();
    let right_edge = bounds.x + bounds.width - insets.right;
    let red_bounds = glyph_bounds_for_color(&batch, [1.0, 0.0, 0.0, 1.0]).unwrap();
    let green_bounds = glyph_bounds_for_color(&batch, [0.0, 1.0, 0.0, 1.0]).unwrap();

    assert!(red_bounds.2 <= right_edge + 1.0);
    assert!(green_bounds.2 <= right_edge + 1.0);
    assert!(
        green_bounds.1 > red_bounds.1,
        "later colored segment should wrap below earlier text, red={red_bounds:?} green={green_bounds:?}"
    );
}

#[test]
fn tooltip_measurement_ignores_hidden_wow_markup_payload() {
    let visible_name = "[Seal of the Silent Vigil]";
    let raw_item_link = "|cffa335ee|Hitem:238036::::::::80:70::13:1:3524:8:40:1279:38:8:46:224073:47:231756:48:226024:49:231768:50:231756:51:231756:52:231756:53:231756|h[Seal of the Silent Vigil]|h|r";
    let tooltip = TooltipData {
        lines: vec![TooltipLine {
            left_text: raw_item_link.to_string(),
            left_color: (0.64, 0.21, 0.93),
            left_segments: Vec::new(),
            right_text: None,
            right_color: (1.0, 1.0, 1.0),
            right_segments: Vec::new(),
            wrap: false,
            texture: None,
        }],
        ..TooltipData::default()
    };
    let mut font_sys = WowFontSystem::new();
    let measured = measure_tooltip_content_width(&tooltip, &mut font_sys);
    let visible_width = font_sys.measure_text_width(visible_name, None, TOOLTIP_HEADER_FONT_SIZE);

    assert!(
        measured <= visible_width + 1.0,
        "tooltip sizing should measure displayed item text, not hidden hyperlink payload: measured={measured}, visible_width={visible_width}"
    );
}

#[test]
fn tooltip_text_insets_account_for_tooltip_nine_slice_overlap() {
    let insets = tooltip_text_insets();
    assert_eq!(
        insets,
        TooltipTextInsets {
            left: 15.0,
            right: 15.0,
            top: 15.0,
            bottom: 15.0,
        }
    );
}

#[test]
fn tooltip_text_quads_start_inside_rendered_border_bounds() {
    let bounds = Rectangle::new(Point::new(100.0, 200.0), Size::new(80.0, 47.0));
    let (batch, border_bounds, glyph_bounds) = render_single_line_tooltip_batch(bounds);

    assert_eq!(
        border_bounds,
        (
            bounds.x,
            bounds.y,
            bounds.x + bounds.width,
            bounds.y + bounds.height
        ),
        "rendered tooltip border should match tooltip bounds"
    );
    assert_text_origin_matches_nine_slice_corner(bounds, &batch);

    let left_inset = glyph_bounds.0 - border_bounds.0;
    let top_inset = glyph_bounds.1 - border_bounds.1;

    assert!(
        left_inset >= 15.0,
        "glyphs should start at or inside the 15px left inset: glyphs={glyph_bounds:?} border={border_bounds:?}"
    );
    assert!(
        top_inset >= 15.0,
        "glyphs should start at or inside the 15px top inset: glyphs={glyph_bounds:?} border={border_bounds:?}"
    );
}

#[test]
fn tooltip_renderer_scales_text_with_effective_scale() {
    let mut font_sys = WowFontSystem::new();
    let line = single_header_tooltip_line();
    let insets = tooltip_text_insets();
    let text_width = font_sys.measure_text_width(&line.left_text, None, line.font_size);
    let unscaled_width = text_width + insets.left + insets.right;
    let scaled_bounds = Rectangle::new(
        Point::new(100.0, 200.0),
        Size::new(unscaled_width * 0.5, 30.0),
    );

    let batch = build_single_line_tooltip_batch_with_scale(scaled_bounds, 0.5);
    let glyph_bounds = glyph_bounds(&batch).expect("tooltip text should emit glyph vertices");

    assert!(
        glyph_bounds.2 <= scaled_bounds.x + scaled_bounds.width,
        "scaled tooltip text should stay inside the scaled tooltip width"
    );
}

#[test]
fn tooltip_renderer_skips_fallback_background_when_lua_nineslice_exists() {
    let data = TooltipRenderData {
        lines: vec![TooltipLineRender {
            left_text: "Header".to_string(),
            left_color: [1.0, 1.0, 1.0, 1.0],
            left_segments: Vec::new(),
            right_text: None,
            right_color: [1.0, 1.0, 1.0, 1.0],
            right_segments: Vec::new(),
            font_size: TOOLTIP_HEADER_FONT_SIZE,
            wrap: false,
            measured_height: (TOOLTIP_HEADER_FONT_SIZE * 1.2).ceil(),
            path_attachment: None,
        }],
        line_spacing: TOOLTIP_LINE_SPACING,
        left_text_offsets: vec![Some([10.0; 2])],
    };

    let mut batch = QuadBatch::new();
    let mut font_sys = WowFontSystem::new();
    let mut glyph_atlas = GlyphAtlas::new();
    let tooltip_data = HashMap::from([(42_u64, data)]);
    let mut text_ctx = Some((&mut font_sys, &mut glyph_atlas));

    build_tooltip_quads(
        TooltipRender {
            batch: &mut batch,
            bounds: Rectangle::new(Point::new(100.0, 200.0), Size::new(80.0, 47.0)),
            tooltip_data: Some(&tooltip_data),
            id: 42,
            eff_alpha: 1.0,
            eff_scale: 1.0,
            draw_background: false,
        },
        &mut text_ctx,
    );

    assert!(
        batch.texture_requests.is_empty(),
        "Lua-owned tooltip NineSlice should suppress Rust fallback background requests"
    );
    assert!(
        glyph_bounds(&batch).is_some(),
        "Skipping the fallback background must still render tooltip text"
    );
}

#[test]
fn tooltip_renderer_keeps_opaque_center_when_lua_nineslice_exists() {
    let data = TooltipRenderData {
        lines: vec![TooltipLineRender {
            left_text: "Header".to_string(),
            left_color: [1.0, 1.0, 1.0, 1.0],
            left_segments: Vec::new(),
            right_text: None,
            right_color: [1.0, 1.0, 1.0, 1.0],
            right_segments: Vec::new(),
            font_size: TOOLTIP_HEADER_FONT_SIZE,
            wrap: false,
            measured_height: (TOOLTIP_HEADER_FONT_SIZE * 1.2).ceil(),
            path_attachment: None,
        }],
        line_spacing: TOOLTIP_LINE_SPACING,
        left_text_offsets: vec![Some([10.0; 2])],
    };

    let mut batch = QuadBatch::new();
    let mut font_sys = WowFontSystem::new();
    let mut glyph_atlas = GlyphAtlas::new();
    let tooltip_data = HashMap::from([(42_u64, data)]);
    let mut text_ctx = Some((&mut font_sys, &mut glyph_atlas));

    build_tooltip_quads(
        TooltipRender {
            batch: &mut batch,
            bounds: Rectangle::new(Point::new(100.0, 200.0), Size::new(160.0, 80.0)),
            tooltip_data: Some(&tooltip_data),
            id: 42,
            eff_alpha: 1.0,
            eff_scale: 1.0,
            draw_background: false,
        },
        &mut text_ctx,
    );

    assert!(
        has_solid_color(&batch, [0.0, 0.0, 0.0, 1.0]),
        "Lua-owned tooltip NineSlice should still get an opaque black center fill"
    );
}
