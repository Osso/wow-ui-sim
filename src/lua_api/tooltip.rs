//! Tooltip state data structures.

use crate::widget::{Anchor, AnchorPoint};

/// Inline texture/atlas icon embedded in a tooltip line.
#[derive(Clone)]
pub enum TooltipTexture {
    FileDataId(u32),
    Atlas(String),
    /// Retail/PTR images attached to this existing row (WM63 S8L, WM84).
    Attachments(Vec<TooltipAttachment>),
}

/// `Enum.TooltipTextureAnchor.All`: two points span the selected text.
pub const TOOLTIP_TEXTURE_ANCHOR_ALL: i32 = 6;
/// `Enum.TooltipTextureRelativeRegion.RightLine`.
pub const TOOLTIP_TEXTURE_REGION_RIGHT: i32 = 1;

/// One texture or atlas anchored to a row's selected text region.
#[derive(Clone, Debug)]
pub struct TooltipAttachment {
    /// Pooled Texture region that displays this attachment.
    pub texture_id: u64,
    pub width: f32,
    pub height: f32,
    pub vertical_offset: f32,
    /// Left, right, top, bottom; parsed as floats and never clamped (WM84 §2).
    pub margins: [f32; 4],
    /// `Enum.TooltipTextureAnchor` ordinal, 0 through 6.
    pub anchor: i32,
    /// `Enum.TooltipTextureRelativeRegion`: 0 left line, 1 right line.
    pub region: i32,
}

impl TooltipAttachment {
    /// WM63 S8L anchor table: (texture point, text point, x offset) per constraint.
    pub fn anchor_points(&self) -> Vec<(AnchorPoint, AnchorPoint, f32)> {
        use AnchorPoint::*;
        let [left, right, _, _] = self.margins;
        match self.anchor {
            0 => vec![(TopRight, TopLeft, -right)],
            1 => vec![(Right, Left, -right)],
            2 => vec![(BottomRight, BottomLeft, -right)],
            3 => vec![(TopLeft, TopRight, left)],
            4 => vec![(Left, Right, left)],
            5 => vec![(BottomLeft, BottomRight, left)],
            _ => vec![(BottomRight, BottomRight, 0.0), (TopLeft, TopLeft, 0.0)],
        }
    }
}

/// Attachment reservations for one row from one layout pass (WM84 §1).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TooltipRowReservation {
    pub outward_left: f32,
    pub inward_left: f32,
    pub inward_right: f32,
    pub outward_right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl TooltipRowReservation {
    /// Total width reservation: the sum of the four horizontal groups.
    pub fn width(&self) -> f32 {
        self.outward_left + self.inward_left + self.inward_right + self.outward_right
    }
}

/// WM84 §1 horizontal groups. Outward groups take the first candidate, then
/// the maximum. Both inward groups start at zero and share one first-eligible
/// flag, so a negative first inward candidate replaces zero.
pub fn attachment_width_groups(attachments: &[TooltipAttachment]) -> TooltipRowReservation {
    let mut outward: [Option<f32>; 2] = [None; 2];
    let mut inward = [0.0_f32; 2];
    let mut inward_selected = false;
    for attachment in attachments {
        let [left, right, _, _] = attachment.margins;
        let width = attachment.width + left + right;
        let side = usize::from(attachment.region == TOOLTIP_TEXTURE_REGION_RIGHT);
        let (outward_candidate, inward_candidate) = match (attachment.anchor, side) {
            (0..=2, 0) => (Some(width), None),
            (0..=2, _) => (None, Some(width)),
            (3..=5, 0) => (None, Some(width)),
            (3..=5, _) => (Some(width), None),
            (_, 0) => (Some(left), Some(right)),
            (_, _) => (Some(right), Some(left)),
        };
        if let Some(candidate) = outward_candidate {
            outward[side] = Some(outward[side].map_or(candidate, |current| current.max(candidate)));
        }
        if let Some(candidate) = inward_candidate {
            if !inward_selected || candidate > inward[side] {
                inward[side] = candidate;
            }
            inward_selected = true;
        }
    }
    TooltipRowReservation {
        outward_left: outward[0].unwrap_or(0.0),
        inward_left: inward[0],
        inward_right: inward[1],
        outward_right: outward[1].unwrap_or(0.0),
        top: 0.0,
        bottom: 0.0,
    }
}

/// WM84 §1 top and bottom overflow, each at least zero. `left_height` and
/// `right_height` are the selected text regions' measured heights; a missing
/// text object contributes zero.
pub fn attachment_overflow(
    attachments: &[TooltipAttachment],
    left_height: f32,
    right_height: f32,
) -> (f32, f32) {
    attachments
        .iter()
        .fold((0.0_f32, 0.0_f32), |(top, bottom), attachment| {
            let [_, _, margin_top, margin_bottom] = attachment.margins;
            let text = if attachment.region == TOOLTIP_TEXTURE_REGION_RIGHT {
                right_height
            } else {
                left_height
            };
            let spare = attachment.height + margin_top + margin_bottom - text;
            let offset = attachment.vertical_offset;
            let (above, below) = match attachment.anchor {
                0 | 3 => (margin_top + offset, spare - offset),
                1 | 4 => (spare / 2.0 + offset, spare / 2.0 - offset),
                2 | 5 => (spare + offset, margin_bottom - offset),
                _ => (margin_top + offset, margin_bottom - offset),
            };
            (top.max(above), bottom.max(below))
        })
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
    /// Stored row inset: explicit row inset plus the tooltip-wide extra inset
    /// at append time (WM84 §3).
    pub inset: f32,
}

impl TooltipLine {
    pub fn attachments(&self) -> &[TooltipAttachment] {
        match &self.texture {
            Some(TooltipTexture::Attachments(attachments)) => attachments,
            _ => &[],
        }
    }
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
    pub force_min_width: bool,
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
    /// Tooltip-wide extra row inset sampled by each appended row (WM84 §3).
    /// Construction and ClearLines set it to zero. No writer of a nonzero
    /// value is established (WM84 UNKNOWN), so nothing else assigns it.
    pub extra_inset: f32,
    /// Pooled Retail/PTR attachment Texture regions, reused in attachment order.
    pub attachment_texture_ids: Vec<u64>,
    /// Attachment reservations from the latest layout pass, indexed like `lines`.
    pub row_reservations: Vec<TooltipRowReservation>,
}

/// One tooltip row's measured contributions to the WM63 L23 content width.
pub(crate) struct TooltipWidthRow {
    /// Stored row inset plus the row's attachment width reservation.
    pub(crate) reserved: f32,
    /// Measured intrinsic left text width; zero when the row is hidden.
    pub(crate) left: f32,
    /// Measured intrinsic right text width; zero when absent or hidden.
    pub(crate) right: f32,
    pub(crate) hidden: bool,
}

impl TooltipData {
    /// Attachments across the whole tooltip; the native limit is 30 (WM63 S8L).
    pub(crate) fn attachment_count(&self) -> usize {
        self.lines.iter().map(|line| line.attachments().len()).sum()
    }

    pub(crate) fn row_reservation(&self, index: usize) -> TooltipRowReservation {
        self.row_reservations
            .get(index)
            .copied()
            .unwrap_or_default()
    }

    pub(crate) fn left_line_is_hidden(
        &self,
        widgets: &crate::widget::WidgetRegistry,
        index: usize,
    ) -> bool {
        self.left_line_ids
            .get(index)
            .and_then(|id| widgets.get(*id))
            .is_some_and(|line| !line.visible)
    }

    pub(crate) fn right_line_is_hidden(
        &self,
        widgets: &crate::widget::WidgetRegistry,
        index: usize,
    ) -> bool {
        self.right_line_ids
            .get(index)
            .and_then(|id| widgets.get(*id))
            .is_some_and(|line| !line.visible)
    }

    pub(crate) fn double_column_reservation(&self, line: &TooltipLine) -> f32 {
        if matches!(
            crate::client_profile::ACTIVE,
            crate::client_profile::ClientProfile::Retail
                | crate::client_profile::ClientProfile::Ptr
        ) && !self.force_min_width
            && !line.wrap
        {
            if line.left_text.is_empty() { 0.0 } else { 38.4 }
        } else {
            20.0
        }
    }

    /// WM63 L23 section 2: the stored row wrap flag, or every row when a forced
    /// minimum width is set.
    pub(crate) fn row_wraps(&self, index: usize) -> bool {
        self.force_min_width || self.lines.get(index).is_some_and(|line| line.wrap)
    }

    /// WM63 L23 section 2: the content width selected before outer padding.
    ///
    /// A wrapped row contributes its intrinsic left width capped at the custom
    /// wrap value, or at 230.4 by default. Every other row contributes both
    /// columns plus the 38.4 column reservation and sets the floor together
    /// with the explicit minimum. `fitted_width` re-measures a wrapped row at
    /// its assigned text width; when shrink-to-fit is enabled the selected width
    /// shrinks to the widest fitted row without crossing the floor or growing.
    pub(crate) fn select_content_width(
        &self,
        rows: &[TooltipWidthRow],
        mut fitted_width: impl FnMut(usize, f32) -> f32,
    ) -> f32 {
        const WRAP_CAP: f32 = 230.4;
        const DOUBLE_COLUMN: f32 = 38.4;
        let cap = self.custom_word_wrap_min_width.unwrap_or(WRAP_CAP).max(0.0);
        let mut floor = self.min_width;
        let mut width = floor;
        for (index, row) in rows.iter().enumerate() {
            if self.row_wraps(index) {
                width = width.max((row.left + row.reserved).min(cap));
            } else {
                let gap = if row.left > 0.0 && row.right > 0.0 {
                    DOUBLE_COLUMN
                } else {
                    0.0
                };
                floor = floor.max(row.left + row.right + gap + row.reserved);
                width = width.max(floor);
            }
        }
        if self.shrink_to_fit_wrapped && !self.force_min_width {
            let mut fitted = floor;
            for (index, row) in rows.iter().enumerate() {
                if row.hidden || !self.row_wraps(index) {
                    continue;
                }
                let assigned = (width - row.reserved).max(0.0);
                fitted = fitted.max(fitted_width(index, assigned) + row.reserved);
            }
            width = width.min(fitted);
        }
        width
    }

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
            force_min_width: false,
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
            extra_inset: 0.0,
            attachment_texture_ids: Vec::new(),
            row_reservations: Vec::new(),
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

pub(crate) fn build_cursor_anchor(
    mx: f32,
    my: f32,
    x_offset: f32,
    y_offset: f32,
    anchor_type: &str,
    screen_height: f32,
    scale: f32,
) -> Anchor {
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    ) {
        let (point, x_offset, y_offset) = match anchor_type {
            "ANCHOR_CURSOR_LEFT" => (AnchorPoint::BottomRight, x_offset, y_offset),
            "ANCHOR_CURSOR_RIGHT" => (AnchorPoint::BottomLeft, x_offset, y_offset),
            _ => (AnchorPoint::Bottom, 0.0, 0.0),
        };
        return Anchor {
            point,
            relative_to: None,
            relative_to_id: None,
            relative_point: AnchorPoint::BottomLeft,
            x_offset: mx / scale + x_offset,
            y_offset: (screen_height - my) / scale + y_offset,
        };
    }
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
