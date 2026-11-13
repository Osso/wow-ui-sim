//! Explicit host inputs for Retail `C_TooltipInfo` currency producers.

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct CurrencyTooltipInputs {
    /// Supplied `CurrencyTypes.Flags_0`; absence means no flag data was supplied.
    pub flags_0: Option<u32>,
    /// Supplied `CurrencyTypes.Flags_1`; absence means no flag data was supplied.
    pub flags_1: Option<u32>,
    /// The low byte read from `CurrencyTypes` field 6.
    pub field_6_low_byte: Option<u8>,
    /// `CurrencyTypes` field 11 (`XpQuestDifficulty`).
    pub field_11_xp_quest_difficulty: Option<i32>,
    /// `CurrencyTypes` field 16 transfer percentage.
    pub field_16_transfer_percentage: Option<f64>,
    /// Whether a local player-currency record was supplied.
    pub local_currency_record: Option<bool>,
    /// Ordered CurrencyContainer rows with host-resolved title and description text.
    pub text_ranges: Vec<CurrencyTooltipTextRange>,
    /// Local currency quantity used by weekly-cap rows.
    pub local_quantity: Option<i32>,
    /// Local weekly maximum override; selected when Flags_0 bit 2 is set.
    pub weekly_maximum_override: Option<i32>,
    /// CurrencyTypes field 7, used when Flags_0 bit 2 is clear.
    pub field_7_weekly_maximum: Option<i32>,
    /// Local total-maximum override, selected by the verified flags.
    pub local_total_maximum_override: Option<i32>,
    /// Resolved value for CurrencyTypes field 13's world-state maximum.
    pub field_13_world_state_maximum: Option<i32>,
    /// Value read from CurrencyTypes field 6 by the total-maximum fallback.
    pub field_6_progression_maximum: Option<i32>,
    /// Local seasonal quantity.
    pub seasonal_quantity: Option<i32>,
    /// Opaque native balance-helper result; its formula is not modeled.
    pub balance_helper_result: Option<i32>,
    /// Verified `|c` prefix supplied for total and seasonal rows.
    pub total_row_color_prefix: Option<String>,
    /// Host-formatted integer strings keyed by the exact value passed to the formatter.
    /// The native runtime formatter mode is UNKNOWN, so no locale mode is inferred here.
    pub formatted_numbers: HashMap<i32, String>,
}

/// One CurrencyContainer row selected by signed quantity.
#[derive(Debug, Clone)]
pub struct CurrencyTooltipTextRange {
    pub min_quantity: i32,
    /// Zero means that the range has no upper bound.
    pub max_quantity: i32,
    pub name: String,
    pub description: String,
}
