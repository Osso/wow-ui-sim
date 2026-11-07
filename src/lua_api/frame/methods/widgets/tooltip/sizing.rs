//! Lightweight tooltip sizing for Lua-visible geometry.
//!
//! The renderer performs the final glyph-backed measurement, but Lua queries
//! such as `GameTooltip:GetWidth()` can run before the next draw. Keep the
//! frame's stored size plausible as soon as tooltip content changes.

use crate::lua_api::tooltip::{
    TooltipData, TooltipLine, TooltipRowReservation, TooltipTexture, attachment_overflow,
    attachment_width_groups,
};

const TOOLTIP_PADDING_H: f32 = 12.0;
const TOOLTIP_PADDING_V: f32 = 12.0;
const TOOLTIP_LINE_SPACING: f32 = 2.0;
const TOOLTIP_HEADER_FONT_SIZE: f32 = 14.0;
const TOOLTIP_BODY_FONT_SIZE: f32 = 12.0;
const APPROX_GLYPH_WIDTH: f32 = 0.55;
const TEXTURE_LINE_SIZE: f32 = 28.0;

pub(crate) fn refresh_tooltip_geometry(sim: &mut crate::lua_api::state::SimState, tooltip_id: u64) {
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Wrath {
        sim.widgets.mark_rect_dirty(tooltip_id);
        return;
    }
    let Some((width, height)) = estimated_tooltip_size(sim, tooltip_id) else {
        sim.widgets.mark_rect_dirty(tooltip_id);
        return;
    };
    if let Some(td) = sim.tooltips.get_mut(&tooltip_id) {
        td.row_reservations = estimated_row_reservations(td);
    }

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
        anchor_tooltip_lines(sim, tooltip_id, width);
        return;
    }
    refresh_tooltip_geometry(sim, tooltip_id);
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    ) {
        if let Some(width) = sim.widgets.get(tooltip_id).map(|frame| frame.width) {
            anchor_tooltip_lines(sim, tooltip_id, width);
        }
    }
}

/// 12340 `ShowThis` calls `CalculateSize` synchronously before showing a
/// tooltip that has an owner and at least one row
/// (handoff/laptop-labels-answers-20261018/tooltip-addtexture-12340.md).
pub(crate) fn size_wrath_tooltip_on_show(
    sim: &mut crate::lua_api::state::SimState,
    tooltip_id: u64,
) {
    #[cfg(feature = "gui")]
    {
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

fn anchor_tooltip_lines(sim: &mut crate::lua_api::state::SimState, tooltip_id: u64, width: f32) {
    let Some(td) = sim.tooltips.get(&tooltip_id) else {
        return;
    };
    let [_, _, left, top] = td.padding;
    let content_width = width - td.native_outer_padding().0;
    let spacing = td.line_spacing.unwrap_or(TOOLTIP_LINE_SPACING);
    let rows: Vec<_> = td
        .lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            (
                td.left_line_ids.get(index).copied(),
                td.right_line_ids.get(index).copied(),
                line.right_text
                    .as_ref()
                    .is_some_and(|text| !text.is_empty()),
                line.inset,
                td.row_reservation(index),
            )
        })
        .collect();
    let mut anchored = td.anchored_left_lines;
    for (index, &(line_id, right_id, has_right, inset, reservation)) in rows.iter().enumerate() {
        let Some(line_id) = line_id.filter(|&id| id != 0) else {
            break;
        };
        let previous = if index == 0 {
            tooltip_id
        } else if let Some(id) = rows[index - 1].0 {
            id
        } else {
            break;
        };
        if index >= anchored {
            // WM72/WM84: this row's top overflow and the preceding indexed
            // row's bottom overflow separate the rows.
            let top_offset = if index == 0 {
                -10.0 - top - reservation.top
            } else {
                -spacing - reservation.top - rows[index - 1].4.bottom
            };
            let relative_point = if index == 0 {
                crate::widget::AnchorPoint::Top
            } else {
                crate::widget::AnchorPoint::Bottom
            };
            if let Some(frame) = sim.widgets.get_mut_visual(line_id) {
                frame.anchors = vec![
                    tooltip_line_anchor(
                        crate::widget::AnchorPoint::Top,
                        previous,
                        relative_point,
                        0.0,
                        top_offset,
                    ),
                    tooltip_line_anchor(
                        crate::widget::AnchorPoint::Left,
                        tooltip_id,
                        crate::widget::AnchorPoint::Left,
                        10.0 + left + inset + reservation.outward_left,
                        0.0,
                    ),
                ];
                sim.widgets.mark_rect_dirty(line_id);
                anchored = index + 1;
            }
        }
        if has_right {
            if let Some(right_id) = right_id.filter(|&id| id != 0) {
                if let Some(frame) = sim.widgets.get_mut_visual(right_id)
                    && frame.visible
                {
                    frame.anchors = vec![tooltip_line_anchor(
                        crate::widget::AnchorPoint::Right,
                        line_id,
                        crate::widget::AnchorPoint::Left,
                        content_width
                            - inset
                            - reservation.outward_left
                            - reservation.outward_right,
                        0.0,
                    )];
                    sim.widgets.mark_rect_dirty(right_id);
                }
            }
        }
    }
    if let Some(td) = sim.tooltips.get_mut(&tooltip_id) {
        td.anchored_left_lines = anchored;
    }
}

fn tooltip_line_anchor(
    point: crate::widget::AnchorPoint,
    relative_id: u64,
    relative_point: crate::widget::AnchorPoint,
    x_offset: f32,
    y_offset: f32,
) -> crate::widget::Anchor {
    crate::widget::Anchor {
        point,
        relative_to: None,
        relative_to_id: Some(relative_id as usize),
        relative_point,
        x_offset,
        y_offset,
    }
}

fn estimated_tooltip_size(
    sim: &crate::lua_api::state::SimState,
    tooltip_id: u64,
) -> Option<(f32, f32)> {
    let td = sim.tooltips.get(&tooltip_id)?;
    if td.lines.is_empty() {
        return None;
    }

    let mut width = td.min_width.max(wrapped_min_width(td));
    let mut height = 0.0;
    let line_spacing = td.line_spacing.unwrap_or(TOOLTIP_LINE_SPACING);
    let reservations = estimated_row_reservations(td);
    for (index, line) in td.lines.iter().enumerate() {
        let font_size = tooltip_line_font_size(index);
        let reservation = reservations[index];
        width = width.max(
            estimated_line_width(td, line, font_size, &sim.widgets, index)
                + line.inset
                + reservation.width(),
        );
        let native = matches!(
            crate::client_profile::ACTIVE,
            crate::client_profile::ClientProfile::Retail
                | crate::client_profile::ClientProfile::Ptr
        );
        if native && td.left_line_is_hidden(&sim.widgets, index) {
            continue;
        }
        if (native && height > 0.0) || (!native && index > 0) {
            height += line_spacing;
        }
        height += estimated_line_height(line, font_size) + reservation.top + reservation.bottom;
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

fn estimated_line_width(
    td: &TooltipData,
    line: &TooltipLine,
    font_size: f32,
    widgets: &crate::widget::WidgetRegistry,
    index: usize,
) -> f32 {
    let native = matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    );
    let left = if native && td.left_line_is_hidden(widgets, index) {
        0.0
    } else {
        estimated_text_width(&line.left_text, font_size)
    };
    let right = if native && td.right_line_is_hidden(widgets, index) {
        0.0
    } else {
        line.right_text
            .as_deref()
            .map(|text| estimated_text_width(text, font_size))
            .unwrap_or(0.0)
    };
    let text_width = if right > 0.0 {
        left + right
            + if native && left == 0.0 {
                0.0
            } else {
                td.double_column_reservation(line)
            }
    } else {
        left
    };
    if has_texture_row(line) {
        text_width.max(TEXTURE_LINE_SIZE)
    } else {
        text_width
    }
}

fn estimated_text_width(text: &str, font_size: f32) -> f32 {
    crate::render::strip_wow_markup(text).chars().count() as f32 * font_size * APPROX_GLYPH_WIDTH
}

fn estimated_line_height(line: &TooltipLine, font_size: f32) -> f32 {
    if has_texture_row(line) {
        TEXTURE_LINE_SIZE
    } else {
        (font_size * 1.2).ceil()
    }
}

/// Profiles without native attachments append a separate texture row.
fn has_texture_row(line: &TooltipLine) -> bool {
    matches!(
        line.texture,
        Some(TooltipTexture::FileDataId(_) | TooltipTexture::Atlas(_))
    )
}

/// WM84 reservations from estimated text heights, for builds without glyph
/// measurement. The GUI layout replaces them with measured values.
fn estimated_row_reservations(td: &TooltipData) -> Vec<TooltipRowReservation> {
    td.lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            let text_height = (tooltip_line_font_size(index) * 1.2).ceil();
            let right_height = if line
                .right_text
                .as_deref()
                .is_some_and(|text| !text.is_empty())
            {
                text_height
            } else {
                0.0
            };
            let mut reservation = attachment_width_groups(line.attachments());
            (reservation.top, reservation.bottom) =
                attachment_overflow(line.attachments(), text_height, right_height);
            reservation
        })
        .collect()
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
            inset: 0.0,
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
