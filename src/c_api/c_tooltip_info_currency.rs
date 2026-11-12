//! Explicit host inputs for Retail `C_TooltipInfo` currency producers.

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
    /// Ordered host-supplied CurrencyContainer text rows for this currency.
    pub text_ranges: Vec<CurrencyTooltipTextRange>,
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
