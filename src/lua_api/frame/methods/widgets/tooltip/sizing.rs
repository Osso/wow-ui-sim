//! Lightweight tooltip sizing for Lua-visible geometry.
//!
//! The renderer performs the final glyph-backed measurement, but Lua queries
//! such as `GameTooltip:GetWidth()` can run before the next draw. Keep the
//! frame's stored size plausible as soon as tooltip content changes.

use crate::lua_api::tooltip::{TooltipData, TooltipLine};

const TOOLTIP_PADDING_H: f32 = 12.0;
const TOOLTIP_PADDING_V: f32 = 12.0;
const TOOLTIP_LINE_SPACING: f32 = 2.0;
const TOOLTIP_HEADER_FONT_SIZE: f32 = 14.0;
const TOOLTIP_BODY_FONT_SIZE: f32 = 12.0;
const DOUBLE_LINE_GAP: f32 = 20.0;
const APPROX_GLYPH_WIDTH: f32 = 0.55;
const TEXTURE_LINE_SIZE: f32 = 28.0;

pub(crate) fn refresh_tooltip_geometry(sim: &mut crate::lua_api::state::SimState, tooltip_id: u64) {
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Wrath {
        sim.widgets.mark_rect_dirty(tooltip_id);
        return;
    }
    let Some((width, height)) = estimated_tooltip_size(sim.tooltips.get(&tooltip_id)) else {
        sim.widgets.mark_rect_dirty(tooltip_id);
        return;
    };

    if let Some(frame) = sim.widgets.get_mut_visual(tooltip_id) {
        frame.width = width;
        frame.height = height;
    }
    // Tooltip content refreshes can happen after the frame already has a cached
    // layout rect from a previous owner/anchor. The measured size may be the
    // same, but the cached rect can still be stale until the next resize/dump.
    sim.widgets.mark_rect_dirty(tooltip_id);
}

pub(crate) fn size_tooltip_on_native_trigger(
    sim: &mut crate::lua_api::state::SimState,
    tooltip_id: u64,
) {
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    ) && !sim
        .tooltips
        .get(&tooltip_id)
        .is_some_and(|tooltip| tooltip.owner_id.is_some())
    {
        return;
    }
    #[cfg(feature = "gui")]
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    ) {
        let mut font = crate::render::font::WowFontSystem::new();
        let (width, height) = crate::iced_app::tooltip::measure_tooltip(sim, tooltip_id, &mut font);
        if let Some(frame) = sim.widgets.get_mut_visual(tooltip_id) {
            frame.width = width;
            frame.height = height;
        }
        sim.widgets.mark_rect_dirty(tooltip_id);
        return;
    }
    refresh_tooltip_geometry(sim, tooltip_id);
}

fn estimated_tooltip_size(td: Option<&TooltipData>) -> Option<(f32, f32)> {
    let td = td?;
    if td.lines.is_empty() {
        return None;
    }

    let mut width = td.min_width.max(wrapped_min_width(td));
    let mut height = 0.0;
    let line_spacing = td.line_spacing.unwrap_or(TOOLTIP_LINE_SPACING);
    for (index, line) in td.lines.iter().enumerate() {
        let font_size = tooltip_line_font_size(index);
        width = width.max(estimated_line_width(line, font_size));
        if index > 0 {
            height += line_spacing;
        }
        height += estimated_line_height(line, font_size);
    }

    let (horizontal_padding, vertical_padding) = if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    ) {
        td.native_outer_padding()
    } else {
        (TOOLTIP_PADDING_H * 2.0, TOOLTIP_PADDING_V * 2.0)
    };
    Some((width + horizontal_padding, height + vertical_padding))
}

fn wrapped_min_width(td: &TooltipData) -> f32 {
    if td.lines.iter().any(|line| line.wrap) {
        td.custom_word_wrap_min_width.unwrap_or(0.0)
    } else {
        0.0
    }
}

fn estimated_line_width(line: &TooltipLine, font_size: f32) -> f32 {
    let left = estimated_text_width(&line.left_text, font_size);
    let right = line
        .right_text
        .as_deref()
        .map(|text| estimated_text_width(text, font_size))
        .unwrap_or(0.0);
    let text_width = if right > 0.0 {
        left + right + DOUBLE_LINE_GAP
    } else {
        left
    };
    if matches!(
        line.texture,
        Some(crate::lua_api::tooltip::TooltipTexture::PathAttachment(_))
    ) {
        text_width
    } else if line.texture.is_some() {
        text_width.max(TEXTURE_LINE_SIZE)
    } else {
        text_width
    }
}

fn estimated_text_width(text: &str, font_size: f32) -> f32 {
    crate::render::strip_wow_markup(text).chars().count() as f32 * font_size * APPROX_GLYPH_WIDTH
}

fn estimated_line_height(line: &TooltipLine, font_size: f32) -> f32 {
    if let Some(crate::lua_api::tooltip::TooltipTexture::PathAttachment(attachment)) = &line.texture
    {
        attachment.geometry(font_size).0
    } else if line.texture.is_some() {
        TEXTURE_LINE_SIZE
    } else {
        (font_size * 1.2).ceil()
    }
}

fn tooltip_line_font_size(index: usize) -> f32 {
    if index == 0 {
        TOOLTIP_HEADER_FONT_SIZE
    } else {
        TOOLTIP_BODY_FONT_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::refresh_tooltip_geometry;
    use crate::lua_api::state::SimState;
    use crate::lua_api::tooltip::{TooltipData, TooltipLine};
    use crate::widget::{Frame, WidgetType};

    fn tooltip_line(text: &str) -> TooltipLine {
        TooltipLine {
            left_text: text.to_string(),
            left_color: (1.0, 1.0, 1.0),
            left_segments: Vec::new(),
            right_text: None,
            right_color: (1.0, 1.0, 1.0),
            right_segments: Vec::new(),
            wrap: false,
            texture: None,
        }
    }

    #[test]
    fn refresh_tooltip_geometry_marks_rect_dirty_when_size_is_unchanged() {
        let mut sim = SimState::default();
        let tooltip_id = 42;
        let mut frame = Frame {
            id: tooltip_id,
            widget_type: WidgetType::GameTooltip,
            visible: true,
            ..Default::default()
        };
        frame.anchors.push(crate::widget::Anchor {
            point: crate::widget::AnchorPoint::Left,
            relative_to: None,
            relative_to_id: None,
            relative_point: crate::widget::AnchorPoint::Left,
            x_offset: 0.0,
            y_offset: 0.0,
        });
        sim.widgets.register(frame);
        sim.tooltips.insert(
            tooltip_id,
            TooltipData {
                lines: vec![tooltip_line("Stable tooltip")],
                ..Default::default()
            },
        );

        refresh_tooltip_geometry(&mut sim, tooltip_id);
        sim.widgets.clear_rect_dirty(tooltip_id);
        let (width, height) = sim
            .widgets
            .get(tooltip_id)
            .map(|frame| (frame.width, frame.height))
            .expect("tooltip frame should exist");
        if let Some(frame) = sim.widgets.get_mut(tooltip_id) {
            frame.layout_rect = Some(crate::LayoutRect {
                x: 100.0,
                y: 100.0,
                width: width + 500.0,
                height,
            });
        }

        refresh_tooltip_geometry(&mut sim, tooltip_id);

        assert!(
            sim.widgets.is_rect_dirty_self(tooltip_id),
            "tooltip content refresh should invalidate stale layout even when measured size is unchanged"
        );
    }
}
