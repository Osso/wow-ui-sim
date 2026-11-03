//! Tooltip state data structures.

use crate::widget::{Anchor, AnchorPoint};

/// Inline texture/atlas icon embedded in a tooltip line.
#[derive(Clone)]
pub enum TooltipTexture {
    FileDataId(u32),
    Atlas(String),
    PathAttachment(TooltipPathAttachment),
}

/// Path image attached to an existing wrapped blank tooltip row (Retail U1).
#[derive(Clone)]
pub struct TooltipPathAttachment {
    pub path: String,
    pub height: f32,
    /// Left, right, top, bottom margins in unscaled UI units.
    pub margins: [f32; 4],
}

impl TooltipPathAttachment {
    /// Selected U1 policy: retain the blank text row and center its image.
    /// Vertical margins do not change reservation or centering.
    /// Returns unscaled (row height, image offset from row top).
    pub fn geometry(&self, font_size: f32) -> (f32, f32) {
        let row_height = (font_size * 1.2).ceil();
        let image_offset = (row_height - self.height) / 2.0;
        (row_height, image_offset)
    }
}

/// A text segment with its own color inside a tooltip line.
#[derive(Clone)]
pub struct TooltipTextSegment {
    pub text: String,
    pub color: (f32, f32, f32),
}

/// A single line in a tooltip.
#[derive(Clone)]
pub struct TooltipLine {
    pub left_text: String,
    pub left_color: (f32, f32, f32),
    pub left_segments: Vec<TooltipTextSegment>,
    pub right_text: Option<String>,
    pub right_color: (f32, f32, f32),
    pub right_segments: Vec<TooltipTextSegment>,
    pub wrap: bool,
    /// Inline texture icon (from `AddTexture` / `AddAtlas`).
    pub texture: Option<TooltipTexture>,
}

/// State for a GameTooltip frame.
#[derive(Clone)]
pub struct TooltipData {
    pub lines: Vec<TooltipLine>,
    pub owner_id: Option<u64>,
    pub anchor_type: String,
    pub allow_show_with_no_lines: bool,
    pub frame_stack_index: usize,
    pub shrink_to_fit_wrapped: bool,
    pub anchor_x_offset: f32,
    pub anchor_y_offset: f32,
    pub custom_word_wrap_min_width: Option<f32>,
    pub min_width: f32,
    /// Right, bottom, left, top, in layout units.
    pub padding: [f32; 4],
    pub anchored_left_lines: usize,
    /// Spell ID set by `SetSpellByID`, returned by `GetSpell`.
    pub spell_id: Option<u32>,
    /// Unit token set by `SetUnit`, returned by `GetUnit`.
    pub unit_token: Option<String>,
    /// Unit display name set by `SetUnit`, returned by `GetUnit`.
    pub unit_name: Option<String>,
    /// Unit GUID set by `SetUnit`, returned by `GetUnit`.
    pub unit_guid: Option<String>,
    /// Custom line spacing set by `SetCustomLineSpacing` (default: 2px).
    pub line_spacing: Option<f32>,
    /// Widget IDs for left-side FontString children (`{Name}TextLeft{N}`).
    pub left_line_ids: Vec<u64>,
    /// Widget IDs for right-side FontString children (`{Name}TextRight{N}`).
    pub right_line_ids: Vec<u64>,
    /// Wrath template texture slots in attachment order: zero-based row, widget ID.
    pub wrath_texture_rows: Vec<(usize, u64)>,
}

impl TooltipData {
    /// WM63-L23: native outer allowance plus the explicitly stored sides.
    pub(crate) fn native_outer_padding(&self) -> (f32, f32) {
        const OUTER_PADDING: f32 = 20.48;
        let [right, bottom, left, top] = self.padding;
        (OUTER_PADDING + left + right, OUTER_PADDING + top + bottom)
    }
}

impl Default for TooltipData {
    fn default() -> Self {
        Self {
            lines: Vec::new(),
            owner_id: None,
            anchor_type: "ANCHOR_NONE".to_string(),
            allow_show_with_no_lines: false,
            frame_stack_index: 0,
            shrink_to_fit_wrapped: true,
            anchor_x_offset: 0.0,
            anchor_y_offset: 0.0,
            custom_word_wrap_min_width: None,
            min_width: 0.0,
            padding: [0.0; 4],
            anchored_left_lines: 0,
            spell_id: None,
            unit_token: None,
            unit_name: None,
            unit_guid: None,
            line_spacing: None,
            left_line_ids: Vec::new(),
            right_line_ids: Vec::new(),
            wrath_texture_rows: Vec::new(),
        }
    }
}

/// Clean-room report wow-tooltip-units-12340-facts.md, reconstruction0e4e9fe0.
/// Convert the raw DDC gap to local UI units; layout applies effective scale later.
pub(crate) fn wrath_texture_gap(width: f32, height: f32) -> f32 {
    const DDC_GAP: f32 = 0.0049999999;
    const REFERENCE_HEIGHT: f32 = 768.0;
    let aspect = width / height;
    DDC_GAP * REFERENCE_HEIGHT * (aspect * aspect + 1.0).sqrt()
}

pub(crate) const DEFAULT_CURSOR_Y_OFFSET: f32 = 20.0;

pub(crate) fn build_cursor_anchor(mx: f32, my: f32, x_offset: f32, y_offset: f32) -> Anchor {
    let cursor_y = if x_offset == 0.0 && y_offset == 0.0 {
        my + DEFAULT_CURSOR_Y_OFFSET
    } else {
        my + y_offset
    };
    Anchor {
        point: AnchorPoint::TopLeft,
        relative_to: None,
        relative_to_id: None,
        relative_point: AnchorPoint::TopLeft,
        x_offset: mx + x_offset,
        y_offset: cursor_y,
    }
}
