//! Passive host-owned calendar records; no Lua projection or query behavior.

/// Owned calendar time fields; declared numbers retain their numeric representation.
#[derive(Debug, Clone, PartialEq)]
pub struct CalendarTimeRecord {
    pub month_day: u32,
    pub month: u32,
    pub weekday: u32,
    pub year: f64,
    pub hour: f64,
    pub minute: f64,
}

/// Owned day-event fields. Identifier strings are opaque, not numeric index-map keys.
#[derive(Debug, Clone, PartialEq)]
pub struct CalendarDayEventRecord {
    pub event_id: String,
    pub title: String,
    pub is_custom_title: bool,
    pub start_time: CalendarTimeRecord,
    pub end_time: CalendarTimeRecord,
    pub calendar_type: String,
    pub sequence_type: String,
    pub event_type: i32,
    pub icon_texture: Option<u32>,
    pub mod_status: String,
    /// Host-supplied pending-invite observation, independent of invite status.
    pub has_pending_invite: bool,
    pub invite_status: i32,
    pub invited_by: String,
    pub difficulty: f64,
    pub invite_type: i32,
    pub sequence_index: u32,
    pub num_sequence_days: f64,
    pub difficulty_name: String,
    pub dont_display_banner: bool,
    pub dont_display_end: bool,
    pub club_id: String,
    pub is_locked: bool,
}
