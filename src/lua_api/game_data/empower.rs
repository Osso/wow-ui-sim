//! Empower timing attached to active casting state.
use super::CastingState;

/// Simulator empower inputs use seconds; Lua stage queries expose milliseconds.
#[derive(Clone, Debug)]
pub struct EmpowerTiming {
    pub stage_durations: Vec<f64>,
    pub hold_at_max: f64,
}

impl CastingState {
    pub fn empower_stage_count(&self) -> usize {
        self.empower
            .as_ref()
            .map_or(0, |timing| timing.stage_durations.len())
    }

    pub fn completion_time(&self) -> f64 {
        self.end_time
            + self
                .empower
                .as_ref()
                .map_or(0.0, |timing| timing.hold_at_max)
    }
}
