//! INFERRED Retail/Forever combat-log history input.
//!
//! The host supplies records explicitly. This model invents no combat events.

#[derive(Clone, Debug, PartialEq)]
pub enum CombatLogValue {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CombatLogEntry {
    pub values: Vec<CombatLogValue>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CombatLogHistory {
    pub entries: Vec<CombatLogEntry>,
    pub current_index: Option<usize>,
    pub current_entry: i64,
    pub retention_time: f64,
    pub filtered_events_enabled: bool,
    pub message_limit: i64,
}

impl Default for CombatLogHistory {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            current_index: None,
            current_entry: 0,
            retention_time: 300.0,
            filtered_events_enabled: false,
            message_limit: 300,
        }
    }
}
