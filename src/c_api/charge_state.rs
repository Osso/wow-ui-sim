//! Explicit spell-keyed charge input; no automatic charge progression.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpellChargeState {
    pub current_charges: u32,
    pub max_charges: u32,
    pub recharge_start: f64,
    pub recharge_duration: f64,
    pub charge_mod_rate: f64,
}
