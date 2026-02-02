//! Text measurement and rendering using iced canvas.
//!
//! This module provides text rendering with proper alignment for WoW UI frames.
//! Text measurement is handled by iced's text rendering via cosmic-text.

use iced::widget::canvas::{self, Frame};
use iced::{alignment, Color, Font, Pixels, Point, Rectangle};

use crate::widget::TextJustify;

/// Default WoW UI font (Friz Quadrata).
pub const WOW_FONT_DEFAULT: Font = Font::DEFAULT;

/// Text renderer with alignment capabilities.
pub struct TextRenderer;

impl TextRenderer {
    /// Draw text on a canvas frame with proper centering.
    ///
    /// Uses iced's built-in text centering via align_x and align_y.
    pub fn draw_centered_text(
        frame: &mut Frame,
        text: &str,
        bounds: Rectangle,
        font_size: f32,
        color: Color,
        font: Font,
    ) {
        if text.is_empty() {
            return;
        }

        // Position at center of bounds, let iced handle alignment
        let center = Point::new(
            bounds.x + bounds.width / 2.0,
            bounds.y + bounds.height / 2.0,
        );

        frame.fill_text(canvas::Text {
            content: text.to_string(),
            position: center,
            color,
            size: Pixels(font_size),
            line_height: iced::widget::text::LineHeight::default(),
            font,
            align_x: alignment::Horizontal::Center.into(),
            align_y: alignment::Vertical::Center,
            shaping: iced::widget::text::Shaping::Advanced,
            max_width: f32::INFINITY,
        });
    }

    /// Draw text on a canvas frame with WoW-style justification.
    pub fn draw_justified_text(
        frame: &mut Frame,
        text: &str,
        bounds: Rectangle,
        font_size: f32,
        color: Color,
        font: Font,
        justify_h: TextJustify,
        justify_v: TextJustify,
    ) {
        if text.is_empty() {
            return;
        }

        // Convert WoW justification to iced alignment
        let (align_x, x_pos) = match justify_h {
            TextJustify::Left => (alignment::Horizontal::Left, bounds.x),
            TextJustify::Center => (alignment::Horizontal::Center, bounds.x + bounds.width / 2.0),
            TextJustify::Right => (alignment::Horizontal::Right, bounds.x + bounds.width),
        };

        let (align_y, y_pos) = match justify_v {
            TextJustify::Left => (alignment::Vertical::Top, bounds.y),     // TOP
            TextJustify::Center => (alignment::Vertical::Center, bounds.y + bounds.height / 2.0), // MIDDLE
            TextJustify::Right => (alignment::Vertical::Bottom, bounds.y + bounds.height), // BOTTOM
        };

        frame.fill_text(canvas::Text {
            content: text.to_string(),
            position: Point::new(x_pos, y_pos),
            color,
            size: Pixels(font_size),
            line_height: iced::widget::text::LineHeight::default(),
            font,
            align_x: align_x.into(),
            align_y,
            shaping: iced::widget::text::Shaping::Advanced,
            max_width: bounds.width,
        });
    }
}

/// Map WoW font paths to system fonts.
/// Returns a Font that iced can use.
pub fn wow_font_to_iced(font_path: Option<&str>) -> Font {
    // For now, use the default font
    // In the future, we could load custom fonts via iced's font loading
    match font_path {
        Some(path) => {
            let path_upper = path.to_uppercase();
            if path_upper.contains("MONO") {
                Font::MONOSPACE
            } else {
                Font::DEFAULT
            }
        }
        None => Font::DEFAULT,
    }
}

/// Strip WoW texture/atlas markup from text (e.g., "|TInterface\ICONS\...:20:20:0:0|t")
/// Preserves plain text content while removing inline texture tags.
pub fn strip_wow_markup(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '|' {
            // Check for texture/atlas markup: |T...|t or |A...|a
            if let Some(&next) = chars.peek() {
                if next == 'T' || next == 'A' {
                    // Skip until we find |t or |a
                    let end_marker = if next == 'T' { 't' } else { 'a' };
                    chars.next(); // consume T or A
                    while let Some(ch) = chars.next() {
                        if ch == '|' {
                            if let Some(&marker) = chars.peek() {
                                if marker == end_marker {
                                    chars.next(); // consume the end marker
                                    break;
                                }
                            }
                        }
                    }
                    continue;
                }
                // Handle color markup: |c...|r - strip the |c...| and |r but keep content
                if next == 'c' {
                    // |cXXXXXXXX - skip 9 characters after |c
                    chars.next(); // consume 'c'
                    for _ in 0..8 {
                        chars.next();
                    }
                    continue;
                }
                if next == 'r' {
                    chars.next(); // consume 'r'
                    continue;
                }
            }
        }
        result.push(c);
    }

    result
}
