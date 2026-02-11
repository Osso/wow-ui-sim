//! MessageFrame text rendering.

use iced::Rectangle;

use crate::lua_api::message_frame::MessageFrameData;
use crate::render::font::WowFontSystem;
use crate::render::glyph::{emit_text_quads, measure_text_height, GlyphAtlas};
use crate::render::shader::GLYPH_ATLAS_TEX_INDEX;
use crate::render::QuadBatch;
use crate::widget::TextJustify;

/// Render stored messages for a MessageFrame, bottom-aligned within bounds.
///
/// Messages are word-wrapped to the frame width. We work backwards from
/// the most recent message, measuring each one's wrapped height, until
/// the available vertical space is exhausted.
#[allow(clippy::too_many_arguments)]
pub fn emit_message_frame_text(
    batch: &mut QuadBatch,
    font_sys: &mut WowFontSystem,
    glyph_atlas: &mut GlyphAtlas,
    f: &crate::widget::Frame,
    id: u64,
    bounds: Rectangle,
    mf_map: &std::collections::HashMap<u64, MessageFrameData>,
    alpha: f32,
) {
    let Some(data) = mf_map.get(&id) else { return };
    if data.messages.is_empty() || bounds.width <= 0.0 || bounds.height <= 0.0 {
        return;
    }

    let total = data.messages.len();
    let scroll = data.scroll_offset.max(0) as usize;
    let end = total.saturating_sub(scroll);
    if end == 0 {
        return;
    }

    // Pre-measure wrapped heights from newest to oldest, stopping when
    // we've filled the available vertical space.
    let measured = measure_visible_messages(
        font_sys, f, &data.messages[..end], bounds.width, bounds.height,
    );

    // Render bottom-aligned: walk measured messages from oldest to newest
    let mut y = bounds.y + bounds.height;
    for &(msg_idx, height) in measured.iter().rev() {
        y -= height;
        render_message(
            batch, font_sys, glyph_atlas, f, bounds,
            &data.messages[msg_idx], y, height, alpha,
        );
    }
}

/// Measure messages from newest to oldest, returning (index, height) pairs
/// in newest-first order, until available height is filled.
fn measure_visible_messages(
    font_sys: &mut WowFontSystem,
    f: &crate::widget::Frame,
    messages: &[crate::lua_api::message_frame::Message],
    width: f32,
    available_height: f32,
) -> Vec<(usize, f32)> {
    let mut result = Vec::new();
    let mut used_height = 0.0;

    for i in (0..messages.len()).rev() {
        let h = measure_text_height(
            font_sys, &messages[i].text,
            f.font.as_deref(), f.font_size, width, true,
        );
        if h <= 0.0 {
            continue;
        }
        if used_height + h > available_height {
            break;
        }
        used_height += h;
        result.push((i, h));
    }
    result
}

/// Render a single message at the given y position with word wrapping.
#[allow(clippy::too_many_arguments)]
fn render_message(
    batch: &mut QuadBatch,
    font_sys: &mut WowFontSystem,
    glyph_atlas: &mut GlyphAtlas,
    f: &crate::widget::Frame,
    bounds: Rectangle,
    msg: &crate::lua_api::message_frame::Message,
    y: f32,
    height: f32,
    alpha: f32,
) {
    let line_bounds = Rectangle {
        x: bounds.x,
        y,
        width: bounds.width,
        height,
    };
    let color = [msg.r, msg.g, msg.b, msg.a * alpha];
    let shadow = Some([0.0, 0.0, 0.0, alpha]);

    emit_text_quads(
        batch, font_sys, glyph_atlas,
        &msg.text, line_bounds,
        f.font.as_deref(), f.font_size, color,
        TextJustify::Left, TextJustify::Left, // top-aligned within slot
        GLYPH_ATLAS_TEX_INDEX,
        shadow, (1.0, 1.0),
        f.font_outline,
        true, 0, // word_wrap=true, no line limit
    );
}
