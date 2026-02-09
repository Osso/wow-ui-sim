//! UI style functions for iced widgets.

use iced::widget::{button, pick_list, text_input};
use iced::{Border, Color, Theme};

// WoW-inspired color palette
pub mod palette {
    use iced::Color;

    pub const BG_DARK: Color = Color::from_rgb(0.05, 0.05, 0.08);
    pub const BG_PANEL: Color = Color::from_rgb(0.12, 0.12, 0.14);
    pub const BG_INPUT: Color = Color::from_rgb(0.06, 0.06, 0.08);
    pub const GOLD: Color = Color::from_rgb(0.85, 0.65, 0.13);
    pub const GOLD_DIM: Color = Color::from_rgb(0.55, 0.42, 0.10);
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.92, 0.90, 0.85);
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.60, 0.58, 0.55);
    pub const TEXT_MUTED: Color = Color::from_rgb(0.45, 0.43, 0.40);
    pub const BORDER: Color = Color::from_rgb(0.25, 0.23, 0.20);
    pub const BORDER_HIGHLIGHT: Color = Color::from_rgb(0.40, 0.35, 0.25);
    pub const CONSOLE_TEXT: Color = Color::from_rgb(0.70, 0.85, 0.70);
}

/// Style for event buttons (ADDON_LOADED, PLAYER_LOGIN, etc.).
pub fn event_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let (bg, text_color) = match status {
        button::Status::Active => (palette::BG_PANEL, palette::TEXT_SECONDARY),
        button::Status::Hovered => (palette::BORDER_HIGHLIGHT, palette::GOLD),
        button::Status::Pressed => (palette::GOLD_DIM, palette::TEXT_PRIMARY),
        button::Status::Disabled => (palette::BG_DARK, palette::TEXT_MUTED),
    };

    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color,
        border: Border {
            color: palette::BORDER,
            width: 1.0,
            radius: 3.0.into(),
        },
        ..Default::default()
    }
}

/// Style for the Run button.
pub fn run_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let (bg, text_color, border_color) = match status {
        button::Status::Active => (palette::GOLD_DIM, palette::TEXT_PRIMARY, palette::GOLD),
        button::Status::Hovered => (palette::GOLD, Color::BLACK, palette::GOLD),
        button::Status::Pressed => (palette::GOLD_DIM, Color::BLACK, palette::GOLD_DIM),
        button::Status::Disabled => (palette::BG_DARK, palette::TEXT_MUTED, palette::BORDER),
    };

    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color,
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 3.0.into(),
        },
        ..Default::default()
    }
}

/// Style for the command input field.
pub fn input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Active => palette::BORDER,
        text_input::Status::Hovered => palette::BORDER_HIGHLIGHT,
        text_input::Status::Focused { is_hovered: _ } => palette::GOLD_DIM,
        text_input::Status::Disabled => palette::BG_DARK,
    };

    text_input::Style {
        background: iced::Background::Color(palette::BG_INPUT),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 3.0.into(),
        },
        icon: palette::TEXT_MUTED,
        placeholder: palette::TEXT_MUTED,
        value: palette::TEXT_PRIMARY,
        selection: palette::GOLD_DIM,
    }
}

/// Style for pick_list dropdowns (class/race selectors).
pub fn pick_list_style(_theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let border_color = match status {
        pick_list::Status::Active => palette::BORDER,
        pick_list::Status::Hovered => palette::BORDER_HIGHLIGHT,
        pick_list::Status::Opened { .. } => palette::GOLD_DIM,
    };

    pick_list::Style {
        text_color: palette::TEXT_PRIMARY,
        placeholder_color: palette::TEXT_MUTED,
        handle_color: palette::TEXT_SECONDARY,
        background: iced::Background::Color(palette::BG_INPUT),
        border: Border { color: border_color, width: 1.0, radius: 3.0.into() },
    }
}
