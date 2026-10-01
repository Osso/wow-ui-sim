//! Cooldown widget quad emitters.

#[cfg(all(test, feature = "retail-12-0-5"))]
#[path = "quad_builders_cooldown/countdown_formatter_tests.rs"]
mod countdown_formatter_tests;

use iced::Rectangle;

use crate::render::shader::FLAG_CIRCLE_CLIP;
use crate::render::{BlendMode, QuadBatch};
use crate::widget::AttributeValue;

use super::{
    FrameQuadEmit, WidgetTextLayout, WidgetTextRenderer, color_with_alpha, emit_widget_text_quads,
};

/// Emit all cooldown quads (swipe, edge, bling, countdown text).
pub(super) fn emit_cooldown_quads(
    batch: &mut QuadBatch,
    text_ctx: &mut Option<(
        &mut crate::render::font::WowFontSystem,
        &mut crate::render::glyph::GlyphAtlas,
    )>,
    frame: &FrameQuadEmit<'_>,
) {
    build_cooldown_quads(
        batch,
        frame.bounds,
        frame.widget,
        frame.elapsed_secs,
        frame.eff_alpha,
    );
    emit_cooldown_edge_overlay(batch, frame);
    emit_cooldown_bling_overlay(batch, frame);
    emit_cooldown_countdown_text(batch, text_ctx, frame);
}

/// Build a cooldown swipe overlay quad.
pub(super) fn build_cooldown_quads(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    f: &crate::widget::Frame,
    elapsed_secs: f64,
    alpha: f32,
) {
    if !f.cooldown_draw_swipe || f.cooldown_duration <= 0.0 {
        return;
    }
    let elapsed_since_start = cooldown_elapsed_since_start(f, elapsed_secs);
    let progress = (elapsed_since_start / f.cooldown_duration).clamp(0.0, 1.0);
    if progress >= 1.0 {
        return; // Cooldown finished, no overlay
    }
    let swipe_progress = if f.cooldown_reverse {
        1.0 - progress
    } else {
        progress
    } as f32;
    let mut color = parse_swipe_color(f);
    color[3] *= alpha;
    if let Some(path) = f.cooldown_swipe_texture.as_deref() {
        batch.push_cooldown_swipe_path(
            bounds,
            swipe_progress,
            path,
            color,
            f.cooldown_tex_coord_range,
        );
    } else {
        batch.push_cooldown_swipe(bounds, swipe_progress, color);
    }
}

/// Parse the swipe color from the frame's `__swipe_color` attribute, or return default.
pub(super) fn parse_swipe_color(f: &crate::widget::Frame) -> [f32; 4] {
    if let Some(AttributeValue::String(s)) = f.attributes.get("__swipe_color") {
        let parts: Vec<f32> = s.split(',').filter_map(|p| p.trim().parse().ok()).collect();
        if parts.len() == 4 {
            return [parts[0], parts[1], parts[2], parts[3]];
        }
    }
    [0.0, 0.0, 0.0, 0.62] // WoW default: semi-transparent black
}

pub(super) fn emit_cooldown_edge_overlay(batch: &mut QuadBatch, frame: &FrameQuadEmit<'_>) {
    let f = frame.widget;
    if !f.cooldown_draw_edge {
        return;
    }
    let Some(path) = f.cooldown_edge_texture.as_deref() else {
        return;
    };
    let overlay_bounds = scale_rect_from_center(frame.bounds, f.cooldown_edge_scale as f32);
    batch.push_textured_path(
        overlay_bounds,
        path,
        color_with_alpha(&f.cooldown_edge_color, frame.eff_alpha),
        BlendMode::Alpha,
    );
    if f.cooldown_use_circular_edge {
        batch.set_extra_flags(4, FLAG_CIRCLE_CLIP);
    }
}

pub(super) fn emit_cooldown_bling_overlay(batch: &mut QuadBatch, frame: &FrameQuadEmit<'_>) {
    let f = frame.widget;
    if !f.cooldown_draw_bling || f.cooldown_duration <= 0.0 {
        return;
    }
    let elapsed_since_start = cooldown_elapsed_since_start(f, frame.elapsed_secs);
    let progress = (elapsed_since_start / f.cooldown_duration).clamp(0.0, 1.0);
    if progress < 1.0 {
        return;
    }
    let Some(path) = f.cooldown_bling_texture.as_deref() else {
        return;
    };
    batch.push_textured_path(
        frame.bounds,
        path,
        [1.0, 1.0, 1.0, frame.eff_alpha],
        BlendMode::Alpha,
    );
}

pub(super) fn emit_cooldown_countdown_text(
    batch: &mut QuadBatch,
    text_ctx: &mut Option<(
        &mut crate::render::font::WowFontSystem,
        &mut crate::render::glyph::GlyphAtlas,
    )>,
    frame: &FrameQuadEmit<'_>,
) {
    let f = frame.widget;
    let Some(remaining) = cooldown_remaining_seconds(f, frame.elapsed_secs) else {
        return;
    };
    let Some((font_sys, glyph_atlas)) = text_ctx.as_mut() else {
        return;
    };
    let Some(text) = cooldown_countdown_text(f, remaining) else {
        return;
    };
    let Some(countdown_id) = f.cooldown_countdown_font_string_id else {
        return;
    };
    let Some(countdown_frame) = frame.registry.get(countdown_id) else {
        return;
    };
    let mut text_renderer = WidgetTextRenderer {
        batch,
        font_sys,
        glyph_atlas,
    };
    emit_widget_text_quads(
        &mut text_renderer,
        countdown_frame,
        WidgetTextLayout {
            text: &text,
            bounds: frame.bounds,
            justify_h: countdown_frame.justify_h,
            justify_v: countdown_frame.justify_v,
            word_wrap: false,
            max_lines: 1,
            alpha: frame.eff_alpha,
        },
    );
}

fn cooldown_remaining_seconds(f: &crate::widget::Frame, elapsed_secs: f64) -> Option<f64> {
    f.cooldown_remaining_seconds(elapsed_secs)
}

fn cooldown_elapsed_since_start(f: &crate::widget::Frame, elapsed_secs: f64) -> f64 {
    f.cooldown_elapsed_since_start(elapsed_secs)
}

pub(super) fn cooldown_countdown_text(f: &crate::widget::Frame, remaining: f64) -> Option<String> {
    if f.cooldown_has_countdown_formatter {
        return f.cooldown_formatted_countdown_text.clone();
    }
    let threshold = f.cooldown_countdown_abbrev_threshold_seconds;
    if f.cooldown_use_aura_display_time {
        return Some(format!("{}", remaining.ceil() as i64));
    }
    if threshold > 0.0 && remaining >= threshold {
        return Some(format!("{}s", remaining.ceil() as i64));
    }
    let decimal_threshold = f.cooldown_countdown_milliseconds_threshold_seconds;
    if remaining < decimal_threshold {
        Some(format!("{remaining:.1}"))
    } else {
        Some(format!("{}", remaining.ceil() as i64))
    }
}

pub(super) fn scale_rect_from_center(bounds: Rectangle, scale: f32) -> Rectangle {
    use iced::{Point, Size};
    if (scale - 1.0).abs() < f32::EPSILON {
        return bounds;
    }
    let scaled_width = bounds.width * scale;
    let scaled_height = bounds.height * scale;
    Rectangle::new(
        Point::new(
            bounds.x + (bounds.width - scaled_width) * 0.5,
            bounds.y + (bounds.height - scaled_height) * 0.5,
        ),
        Size::new(scaled_width, scaled_height),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        build_cooldown_quads, cooldown_countdown_text, cooldown_remaining_seconds,
        emit_cooldown_bling_overlay, emit_cooldown_edge_overlay,
    };
    use crate::iced_app::quad_builders::FrameQuadEmit;
    use crate::render::{QuadBatch, shader::FLAG_CIRCLE_CLIP};
    use crate::widget::{Frame, WidgetRegistry, WidgetType};
    use iced::{Point, Rectangle, Size};

    #[test]
    fn cooldown_swipe_uses_normalized_local_uvs_and_configured_sample_uvs() {
        let mut batch = QuadBatch::new();
        let mut cooldown = Frame::new(WidgetType::Cooldown, None, None);
        cooldown.cooldown_duration = 10.0;
        cooldown.cooldown_start = 0.0;
        cooldown.cooldown_tex_coord_range = Some((0.2, 0.3, 0.8, 0.9));
        cooldown.cooldown_swipe_texture = Some("Interface/CooldownSwipe".to_string());

        build_cooldown_quads(
            &mut batch,
            Rectangle::new(Point::new(10.0, 20.0), Size::new(40.0, 50.0)),
            &cooldown,
            2.5,
            1.0,
        );

        assert_eq!(
            batch.vertices.len(),
            4,
            "cooldown should emit one swipe quad"
        );
        assert_eq!(batch.vertices[0].local_uv, [0.0, 0.0]);
        assert_eq!(batch.vertices[1].local_uv, [1.0, 0.0]);
        assert_eq!(batch.vertices[2].local_uv, [1.0, 1.0]);
        assert_eq!(batch.vertices[3].local_uv, [0.0, 1.0]);
        assert_eq!(batch.vertices[0].mask_tex_coords, [0.2, 0.3]);
        assert_eq!(batch.vertices[1].mask_tex_coords, [0.8, 0.3]);
        assert_eq!(batch.vertices[2].mask_tex_coords, [0.8, 0.9]);
        assert_eq!(batch.vertices[3].mask_tex_coords, [0.2, 0.9]);
        assert_eq!(batch.texture_requests.len(), 1);
        assert_eq!(batch.texture_requests[0].path, "Interface/CooldownSwipe");
    }

    #[test]
    fn cooldown_swipe_progress_uses_mod_rate() {
        let bounds = Rectangle::new(Point::new(10.0, 20.0), Size::new(40.0, 50.0));
        let mut cooldown = Frame::new(WidgetType::Cooldown, None, None);
        cooldown.cooldown_duration = 10.0;
        cooldown.cooldown_start = 0.0;

        let mut fast_batch = QuadBatch::new();
        cooldown.cooldown_mod_rate = 2.0;
        build_cooldown_quads(&mut fast_batch, bounds, &cooldown, 2.5, 1.0);
        assert_eq!(
            fast_batch.vertices[0].tex_coords[0], 0.5,
            "double mod rate should make 2.5s of real time advance 5s into a 10s cooldown"
        );

        let mut slow_batch = QuadBatch::new();
        cooldown.cooldown_mod_rate = 0.5;
        build_cooldown_quads(&mut slow_batch, bounds, &cooldown, 2.5, 1.0);
        assert_eq!(
            slow_batch.vertices[0].tex_coords[0], 0.125,
            "half mod rate should make 2.5s of real time advance 1.25s into a 10s cooldown"
        );
    }

    #[test]
    fn cooldown_bling_completion_uses_mod_rate() {
        let mut batch = QuadBatch::new();
        let mut cooldown = Frame::new(WidgetType::Cooldown, None, None);
        cooldown.cooldown_duration = 10.0;
        cooldown.cooldown_start = 0.0;
        cooldown.cooldown_mod_rate = 2.0;
        cooldown.cooldown_draw_bling = true;
        cooldown.cooldown_bling_texture = Some("Interface/CooldownBling".to_string());

        let registry = WidgetRegistry::default();
        let frame = FrameQuadEmit {
            id: cooldown.id,
            widget: &cooldown,
            bounds: Rectangle::new(Point::new(10.0, 20.0), Size::new(40.0, 40.0)),
            clip_bounds: None,
            bar_fill: None,
            pressed_frame: None,
            hovered_frame: None,
            message_frames: None,
            tooltip_data: None,
            quest_blobs: None,
            registry: &registry,
            elapsed_secs: 5.0,
            eff_alpha: 1.0,
        };

        emit_cooldown_bling_overlay(&mut batch, &frame);

        assert_eq!(batch.texture_requests.len(), 1);
        assert_eq!(batch.texture_requests[0].path, "Interface/CooldownBling");
    }

    #[test]
    fn cooldown_countdown_remaining_uses_mod_rate() {
        let mut cooldown = Frame::new(WidgetType::Cooldown, None, None);
        cooldown.cooldown_duration = 10.0;
        cooldown.cooldown_start = 0.0;
        cooldown.cooldown_mod_rate = 2.0;
        cooldown.cooldown_display_duration_ms = 10_000.0;

        assert_eq!(cooldown_remaining_seconds(&cooldown, 2.0), Some(6.0));
    }

    #[test]
    fn cooldown_edge_and_bling_overlays_emit_textured_quads() {
        let mut batch = QuadBatch::new();
        let mut cooldown = Frame::new(WidgetType::Cooldown, None, None);
        cooldown.cooldown_duration = 10.0;
        cooldown.cooldown_start = 0.0;
        cooldown.cooldown_draw_edge = true;
        cooldown.cooldown_edge_texture = Some("Interface/CooldownEdge".to_string());
        cooldown.cooldown_use_circular_edge = true;
        cooldown.cooldown_edge_scale = 1.25;
        cooldown.cooldown_draw_bling = true;
        cooldown.cooldown_bling_texture = Some("Interface/CooldownBling".to_string());

        let registry = WidgetRegistry::default();
        let frame = FrameQuadEmit {
            id: cooldown.id,
            widget: &cooldown,
            bounds: Rectangle::new(Point::new(10.0, 20.0), Size::new(40.0, 40.0)),
            clip_bounds: None,
            bar_fill: None,
            pressed_frame: None,
            hovered_frame: None,
            message_frames: None,
            tooltip_data: None,
            quest_blobs: None,
            registry: &registry,
            elapsed_secs: 10.0,
            eff_alpha: 1.0,
        };

        emit_cooldown_edge_overlay(&mut batch, &frame);
        emit_cooldown_bling_overlay(&mut batch, &frame);

        assert_eq!(batch.texture_requests.len(), 2);
        assert_eq!(batch.texture_requests[0].path, "Interface/CooldownEdge");
        assert_eq!(batch.texture_requests[1].path, "Interface/CooldownBling");
        assert!(
            batch.vertices[..4]
                .iter()
                .all(|vertex| (vertex.flags & FLAG_CIRCLE_CLIP) != 0),
            "circular edge should set the circle clip flag on the edge quad"
        );
    }

    #[test]
    fn cooldown_decimal_threshold_changes_rendered_text_at_exact_boundary() {
        let env = crate::lua_api::WowLuaEnv::new().expect("Lua environment");
        env.exec(
            r#"
                local cooldown = CreateFrame("Cooldown", "DecimalThresholdCooldown")
                cooldown:SetCountdownMillisecondsThreshold(20)
            "#,
        )
        .expect("configure threshold through Lua API");
        let sim = env.state().borrow();
        let id = sim
            .widgets
            .get_id_by_name("DecimalThresholdCooldown")
            .unwrap();
        let cooldown = sim.widgets.get(id).unwrap();
        for (remaining, expected) in [(19.9, "19.9"), (20.0, "20"), (20.1, "21")] {
            assert_eq!(
                cooldown_countdown_text(cooldown, remaining).as_deref(),
                Some(expected),
                "remaining={remaining}"
            );
        }
    }

    #[test]
    fn cooldown_decimal_threshold_zero_and_updates_control_fractional_text() {
        let env = crate::lua_api::WowLuaEnv::new().expect("Lua environment");
        env.exec("decimalCooldown = CreateFrame('Cooldown', 'UpdatedDecimalCooldown')")
            .expect("create cooldown");
        for (threshold, expected) in [(0.0, "3"), (5.0, "2.4"), (2.0, "3"), (0.0, "3")] {
            env.exec(&format!(
                "decimalCooldown:SetCountdownMillisecondsThreshold({threshold})"
            ))
            .expect("update threshold through Lua API");
            let sim = env.state().borrow();
            let id = sim
                .widgets
                .get_id_by_name("UpdatedDecimalCooldown")
                .unwrap();
            let cooldown = sim.widgets.get(id).unwrap();
            assert_eq!(
                cooldown_countdown_text(cooldown, 2.4).as_deref(),
                Some(expected),
                "threshold={threshold}"
            );
        }
    }

    #[test]
    fn cooldown_countdown_text_uses_aura_and_abbrev_modes() {
        let mut cooldown = Frame::new(WidgetType::Cooldown, None, None);
        cooldown.cooldown_countdown_abbrev_threshold_seconds = 5.0;
        cooldown.cooldown_countdown_milliseconds_threshold_seconds = 10.0;

        assert_eq!(
            cooldown_countdown_text(&cooldown, 8.2).as_deref(),
            Some("9s")
        );
        assert_eq!(
            cooldown_countdown_text(&cooldown, 2.4).as_deref(),
            Some("2.4")
        );

        cooldown.cooldown_use_aura_display_time = true;
        assert_eq!(
            cooldown_countdown_text(&cooldown, 8.2).as_deref(),
            Some("9")
        );
    }
}
