//! Tooltip state data structures.

/// A single line in a tooltip.
pub struct TooltipLine {
    pub left_text: String,
    pub left_color: (f32, f32, f32),
    pub right_text: Option<String>,
    pub right_color: (f32, f32, f32),
    pub wrap: bool,
}

/// State for a GameTooltip frame.
pub struct TooltipData {
    pub lines: Vec<TooltipLine>,
    pub owner_id: Option<u64>,
    pub anchor_type: String,
    pub min_width: f32,
    pub padding: f32,
}

impl Default for TooltipData {
    fn default() -> Self {
        Self {
            lines: Vec::new(),
            owner_id: None,
            anchor_type: "ANCHOR_NONE".to_string(),
            min_width: 0.0,
            padding: 0.0,
        }
    }
}
