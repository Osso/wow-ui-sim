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
}
