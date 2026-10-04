//! INFERRED explicit profiling snapshots, zero by default.

#[derive(Default)]
pub struct PerformanceInputs {
    /// INFERRED milliseconds and cumulative call count; zero default.
    pub event_time: f64,
    pub event_count: f64,
    pub function_time: f64,
    pub function_count: f64,
    pub script_usage: f64,
}
