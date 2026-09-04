//! Text measurement and rendering using iced canvas.
//!
//! This module provides text rendering with proper alignment for WoW UI frames.
//! Text measurement is handled by iced's text rendering via cosmic-text.

use iced::widget::canvas::{self, Frame};
use iced::{Color, Font, Pixels, Point, Rectangle, alignment};

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
    pub fn draw_justified_text(frame: &mut Frame, text: JustifiedText<'_>) {
        if text.content.is_empty() {
            return;
        }

        let horizontal = horizontal_text_anchor(text.bounds, text.justify_h);
        let vertical = vertical_text_anchor(text.bounds, text.justify_v);

        frame.fill_text(canvas::Text {
            content: text.content.to_string(),
            position: Point::new(horizontal.position, vertical.position),
            color: text.color,
            size: Pixels(text.font_size),
            line_height: iced::widget::text::LineHeight::default(),
            font: text.font,
            align_x: horizontal.alignment.into(),
            align_y: vertical.alignment,
            shaping: iced::widget::text::Shaping::Advanced,
            max_width: text.bounds.width,
        });
    }
}

struct HorizontalTextAnchor {
    alignment: alignment::Horizontal,
    position: f32,
}

struct VerticalTextAnchor {
    alignment: alignment::Vertical,
    position: f32,
}

fn horizontal_text_anchor(bounds: Rectangle, justify: TextJustify) -> HorizontalTextAnchor {
    match justify {
        TextJustify::Left => HorizontalTextAnchor {
            alignment: alignment::Horizontal::Left,
            position: bounds.x,
        },
        TextJustify::Center => HorizontalTextAnchor {
            alignment: alignment::Horizontal::Center,
            position: bounds.x + bounds.width / 2.0,
        },
        TextJustify::Right => HorizontalTextAnchor {
            alignment: alignment::Horizontal::Right,
            position: bounds.x + bounds.width,
        },
    }
}

fn vertical_text_anchor(bounds: Rectangle, justify: TextJustify) -> VerticalTextAnchor {
    match justify {
        TextJustify::Left => VerticalTextAnchor {
            alignment: alignment::Vertical::Top,
            position: bounds.y,
        },
        TextJustify::Center => VerticalTextAnchor {
            alignment: alignment::Vertical::Center,
            position: bounds.y + bounds.height / 2.0,
        },
        TextJustify::Right => VerticalTextAnchor {
            alignment: alignment::Vertical::Bottom,
            position: bounds.y + bounds.height,
        },
    }
}

pub struct JustifiedText<'a> {
    pub content: &'a str,
    pub bounds: Rectangle,
    pub font_size: f32,
    pub color: Color,
    pub font: Font,
    pub justify_h: TextJustify,
    pub justify_v: TextJustify,
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

#[cfg(test)]
mod tests {
    use crate::render::strip_wow_markup;

    #[test]
    fn plain_text_unchanged() {
        assert_eq!(strip_wow_markup("Hello World"), "Hello World");
    }

    #[test]
    fn strips_color_codes() {
        assert_eq!(strip_wow_markup("|cFF00FF00Green|r text"), "Green text");
    }

    #[test]
    fn strips_inline_texture() {
        assert_eq!(
            strip_wow_markup("Icon|TInterface\\Icons\\Spell:16|tEnd"),
            "IconEnd"
        );
    }

    #[test]
    fn strips_inline_atlas() {
        assert_eq!(strip_wow_markup("Before|Aatlasname|aAfter"), "BeforeAfter");
    }

    #[test]
    fn strips_hyperlink_keeps_text() {
        assert_eq!(strip_wow_markup("|Hitem:12345|hCool Sword|h"), "Cool Sword");
    }

    #[test]
    fn strips_nested_color_in_hyperlink() {
        assert_eq!(
            strip_wow_markup("|cFF0070DD|Hitem:123|h[Blade]|h|r"),
            "[Blade]"
        );
    }

    #[test]
    fn strips_named_color_code() {
        assert_eq!(strip_wow_markup("|cnIQ3:Rare|r item"), "Rare item");
    }

    #[test]
    fn strips_named_color_before_hyperlink() {
        assert_eq!(
            strip_wow_markup(
                "Use |cnIQ3:|Hitem:202046::::::::80:70:::::::::|h[Lucky Tortollan Charm]|h|r now"
            ),
            "Use [Lucky Tortollan Charm] now"
        );
    }

    #[test]
    fn named_color_without_colon_stops_at_the_next_escape() {
        assert_eq!(strip_wow_markup("|cnBroken|r tail"), " tail");
    }

    #[test]
    fn converts_wow_newline_escape() {
        assert_eq!(strip_wow_markup("First|nSecond"), "First\nSecond");
    }

    #[test]
    fn strips_spell_link_before_wow_newline_escape() {
        assert_eq!(
            strip_wow_markup("|cFF2959D3|Hspell:1225135|h[Suppression Zones]|h|r|nNext"),
            "[Suppression Zones]\nNext"
        );
    }

    #[test]
    fn empty_string() {
        assert_eq!(strip_wow_markup(""), "");
    }

    #[test]
    fn lone_pipe_preserved() {
        assert_eq!(strip_wow_markup("a|b"), "a|b");
    }

    #[test]
    fn pipe_at_end_preserved() {
        assert_eq!(strip_wow_markup("text|"), "text|");
    }
}
