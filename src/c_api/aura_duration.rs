//! Explicit per-spell aura duration inputs; no production records are seeded.

/// Recast metadata, independent of an active aura's current duration.
/// Consumers must reject nonfinite or negative fields before public output.
/// The carryover cap is an INFERRED simulator policy, not native-verified data.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpellAuraDuration {
    pub base_duration_seconds: f64,
    pub max_carryover_seconds: f64,
}
