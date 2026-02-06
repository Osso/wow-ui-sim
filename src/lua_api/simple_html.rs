//! SimpleHTML frame state data structures.

use std::collections::HashMap;

/// Per-textType style (h1, h2, h3, p, etc.)
pub struct TextStyle {
    pub font: Option<String>,
    pub font_size: f32,
    pub font_object: Option<String>,
    pub text_color: (f32, f32, f32, f32),
    pub shadow_color: (f32, f32, f32, f32),
    pub shadow_offset: (f32, f32),
    pub spacing: f32,
    pub justify_h: String,
    pub justify_v: String,
    pub indented_word_wrap: bool,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font: None,
            font_size: 12.0,
            font_object: None,
            text_color: (1.0, 1.0, 1.0, 1.0),
            shadow_color: (0.0, 0.0, 0.0, 0.0),
            shadow_offset: (0.0, 0.0),
            spacing: 0.0,
            justify_h: "LEFT".to_string(),
            justify_v: "TOP".to_string(),
            indented_word_wrap: false,
        }
    }
}

/// State for a SimpleHTML frame.
pub struct SimpleHtmlData {
    pub hyperlink_format: String,
    pub hyperlinks_enabled: bool,
    pub text_styles: HashMap<String, TextStyle>,
}

impl Default for SimpleHtmlData {
    fn default() -> Self {
        Self {
            hyperlink_format: "|H%s|h%s|h".to_string(),
            hyperlinks_enabled: true,
            text_styles: HashMap::new(),
        }
    }
}
