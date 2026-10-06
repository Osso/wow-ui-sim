//! Per-addon runtime helpers: timing, taint names, and metrics counters.

use super::super::state::{AddonRuntimeMetrics, SimState};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

/// Increment threshold counters for a frame's addon time.
pub(crate) fn update_threshold_counters(rt: &mut AddonRuntimeMetrics, ms: f64) {
    if ms > 1.0 {
        rt.count_over_1ms += 1;
    }
    if ms > 5.0 {
        rt.count_over_5ms += 1;
    }
    if ms > 10.0 {
        rt.count_over_10ms += 1;
    }
    if ms > 50.0 {
        rt.count_over_50ms += 1;
    }
    if ms > 100.0 {
        rt.count_over_100ms += 1;
    }
    if ms > 500.0 {
        rt.count_over_500ms += 1;
    }
    if ms > 1000.0 {
        rt.count_over_1000ms += 1;
    }
}

/// Record per-addon timing from an Instant.
pub(crate) fn record_addon_time(state: &Rc<RefCell<SimState>>, idx: Option<u16>, start: &Instant) {
    if let Some(i) = idx {
        let ms = start.elapsed().as_secs_f64() * 1000.0;
        if let Some(addon) = state.borrow_mut().addons.get_mut(i as usize) {
            addon.runtime.current_frame_ms += ms;
        }
    }
}
