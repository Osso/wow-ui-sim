//! MessageFrame / ScrollingMessageFrame state data structures.

/// A single message in a MessageFrame.
pub struct Message {
    pub text: String,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub message_id: Option<i64>,
    /// GetTime() value when the message was added (for fading calculations).
    pub timestamp: f64,
}

/// State for a MessageFrame or ScrollingMessageFrame.
pub struct MessageFrameData {
    pub messages: Vec<Message>,
    pub max_lines: usize,
    pub fading: bool,
    pub time_visible: f64,
    pub fade_duration: f64,
    pub fade_power: f64,
    pub insert_mode: String, // "TOP" or "BOTTOM"
    pub scroll_offset: i32,
    pub scroll_allowed: bool,
    pub text_copyable: bool,
}

impl Default for MessageFrameData {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            max_lines: 120,
            fading: true,
            time_visible: 10.0,
            fade_duration: 3.0,
            fade_power: 1.0,
            insert_mode: "BOTTOM".to_string(),
            scroll_offset: 0,
            scroll_allowed: true,
            text_copyable: false,
        }
    }
}
