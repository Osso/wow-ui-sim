use std::time::{Duration, Instant};

use iced::{Point, Rectangle, Size};
use wow_ui_sim::render::{GlyphAtlas, QuadBatch, WowFontSystem, emit_text_quads};
use wow_ui_sim::widget::{TextJustify, TextOutline};

const REPRESENTATIVE_TEXT_CASES: &[TextPerfCase] = &[
    TextPerfCase {
        text: "Game Menu",
        bounds: (220.0, 32.0),
        font_size: 16.0,
        word_wrap: false,
    },
    TextPerfCase {
        text: "Avenger's Shield",
        bounds: (260.0, 36.0),
        font_size: 18.0,
        word_wrap: false,
    },
    TextPerfCase {
        text: "Quest accepted: The Dark Portal",
        bounds: (320.0, 40.0),
        font_size: 15.0,
        word_wrap: false,
    },
    TextPerfCase {
        text: "This is a deliberately long tooltip body line used to exercise wrapped tooltip text shaping for the glyph atlas performance regression.",
        bounds: (320.0, 120.0),
        font_size: 14.0,
        word_wrap: true,
    },
    TextPerfCase {
        text: "123,456 / 789,012",
        bounds: (200.0, 28.0),
        font_size: 13.0,
        word_wrap: false,
    },
];

pub fn measure_glyph_text_shaping_for_representative_strings() -> Duration {
    let mut font_system = WowFontSystem::new();
    let mut glyph_atlas = GlyphAtlas::new();
    let mut batch = QuadBatch::new();

    let started = Instant::now();
    for (index, case) in REPRESENTATIVE_TEXT_CASES.iter().enumerate() {
        emit_text_quads(
            &mut batch,
            &mut font_system,
            &mut glyph_atlas,
            case.text,
            case.bounds_rect(index),
            None,
            case.font_size,
            [1.0, 1.0, 1.0, 1.0],
            TextJustify::Left,
            TextJustify::Left,
            0,
            None,
            (0.0, 0.0),
            TextOutline::None,
            case.word_wrap,
            0,
            0.0,
            None,
        );
    }
    let elapsed = started.elapsed();

    assert!(
        !batch.vertices.is_empty(),
        "representative glyph shaping should emit text vertices"
    );
    elapsed
}

struct TextPerfCase {
    text: &'static str,
    bounds: (f32, f32),
    font_size: f32,
    word_wrap: bool,
}

impl TextPerfCase {
    fn bounds_rect(&self, index: usize) -> Rectangle {
        Rectangle::new(
            Point::new(16.0, 16.0 + index as f32 * 48.0),
            Size::new(self.bounds.0, self.bounds.1),
        )
    }
}
