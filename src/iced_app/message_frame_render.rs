//! MessageFrame text rendering.

use iced::Rectangle;

use crate::lua_api::message_frame::MessageFrameData;
use crate::render::QuadBatch;
use crate::render::font::WowFontSystem;
use crate::render::glyph::{GlyphAtlas, emit_text_quads, measure_text_height};
use crate::render::shader::GLYPH_ATLAS_TEX_INDEX;
use crate::widget::TextJustify;

/// Render stored messages for a MessageFrame, bottom-aligned within bounds.
///
/// Messages are word-wrapped to the frame width. We work backwards from
/// the most recent message, measuring each one's wrapped height, until
/// the available vertical space is exhausted.
pub fn emit_message_frame_text(
    render: &mut MessageFrameTextRenderer<'_>,
    f: &crate::widget::Frame,
    id: u64,
    bounds: Rectangle,
    mf_map: &std::collections::HashMap<u64, MessageFrameData>,
    alpha: f32,
    elapsed_secs: f64,
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
        render.font_sys,
        render.glyph_atlas,
        f,
        &data.messages[..end],
        bounds.width,
        bounds.height,
    );

    // Render bottom-aligned: walk measured messages from oldest to newest
    let mut y = bounds.y + bounds.height;
    for &(msg_idx, height) in measured.iter().rev() {
        y -= height;
        let msg_alpha = message_fade_alpha(data, &data.messages[msg_idx], elapsed_secs);
        if msg_alpha <= 0.0 {
            continue;
        }
        render_message(
            render,
            f,
            MessageLine {
                bounds,
                msg: &data.messages[msg_idx],
                y,
                height,
                alpha: alpha * msg_alpha,
            },
        );
    }
}

pub struct MessageFrameTextRenderer<'a> {
    pub batch: &'a mut QuadBatch,
    pub font_sys: &'a mut WowFontSystem,
    pub glyph_atlas: &'a mut GlyphAtlas,
}

/// Compute fade alpha for a single message.
///
/// WoW MessageFrame fading: after `time_visible` seconds the message starts
/// fading out over `fade_duration` seconds, with optional `fade_power` curve.
/// Returns 1.0 (fully visible) → 0.0 (fully faded).
fn message_fade_alpha(
    data: &MessageFrameData,
    msg: &crate::lua_api::message_frame::Message,
    now: f64,
) -> f32 {
    if !data.fading {
        return 1.0;
    }
    let fade_timestamp = msg.timestamp.max(data.override_fade_timestamp);
    let age = now - fade_timestamp;
    if age <= data.time_visible {
        return 1.0;
    }
    let fade_elapsed = age - data.time_visible;
    if data.fade_duration <= 0.0 || fade_elapsed >= data.fade_duration {
        return 0.0;
    }
    let t = 1.0 - (fade_elapsed / data.fade_duration);
    t.powf(data.fade_power) as f32
}

/// Measure messages from newest to oldest, returning (index, height) pairs
/// in newest-first order, until available height is filled.
fn measure_visible_messages(
    font_sys: &mut WowFontSystem,
    glyph_atlas: &mut GlyphAtlas,
    f: &crate::widget::Frame,
    messages: &[crate::lua_api::message_frame::Message],
    width: f32,
    available_height: f32,
) -> Vec<(usize, f32)> {
    let mut result = Vec::new();
    let mut used_height = 0.0;

    for i in (0..messages.len()).rev() {
        let h = measure_text_height(
            font_sys,
            glyph_atlas,
            &messages[i].text,
            f.font.as_deref(),
            f.font_size,
            width,
            true,
            0.0,
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
fn render_message(
    render: &mut MessageFrameTextRenderer<'_>,
    f: &crate::widget::Frame,
    line: MessageLine<'_>,
) {
    let line_bounds = Rectangle {
        x: line.bounds.x,
        y: line.y,
        width: line.bounds.width,
        height: line.height,
    };
    let color = [line.msg.r, line.msg.g, line.msg.b, line.msg.a * line.alpha];
    let shadow = Some([0.0, 0.0, 0.0, line.alpha]);

    emit_text_quads(
        render.batch,
        render.font_sys,
        render.glyph_atlas,
        &line.msg.text,
        line_bounds,
        f.font.as_deref(),
        f.font_size,
        color,
        TextJustify::Left,
        TextJustify::Left, // top-aligned within slot
        GLYPH_ATLAS_TEX_INDEX,
        shadow,
        (1.0, 1.0),
        f.font_outline,
        true,
        0, // word_wrap=true, no line limit
        0.0,
        None, // message frames don't pre-strip
    );
}

struct MessageLine<'a> {
    bounds: Rectangle,
    msg: &'a crate::lua_api::message_frame::Message,
    y: f32,
    height: f32,
    alpha: f32,
}
