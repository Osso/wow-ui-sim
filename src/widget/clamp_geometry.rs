//! Retail R88n.5r-q3 screen geometry, separate from arbitrary UIParent state.

use crate::client_profile::{ACTIVE, ClientProfile};

pub(crate) const fn uses_retail_clamp_producers() -> bool {
    matches!(ACTIVE, ClientProfile::Retail)
}

/// Reserved geometry target; widget IDs are allocated from 1. The native
/// shared coordinate root is false and guarded, so it never accepts a request.
/// This is not a Lua script object and is never published as a relative frame.
pub(crate) const COORDINATE_ROOT: usize = 0;
pub(crate) const COORDINATE_ROOT_PROPAGATION: bool = false;

// R88n.5r-q3.2 initialization of the read-only shared unit extents.
const HORIZONTAL_EXTENT: f32 = 1.0;
const VERTICAL_EXTENT: f32 = 0.8;
const UNIT_DENOMINATOR: f32 = 1024.0;
// R88n.5r-q4.1: strict comparison also rejects unordered (NaN) inputs.
const ROUNDING_EFFECTIVE_SCALE_THRESHOLD: f32 = f32::EPSILON;

pub(crate) fn inset_to_storage(public: f32) -> f32 {
    if !uses_retail_clamp_producers() {
        return public;
    }
    let coefficient = (1.0 / UNIT_DENOMINATOR) / HORIZONTAL_EXTENT;
    (public * coefficient) * VERTICAL_EXTENT
}

pub(crate) fn inset_to_public(stored: f32) -> f32 {
    if !uses_retail_clamp_producers() {
        return stored;
    }
    let horizontal = HORIZONTAL_EXTENT * UNIT_DENOMINATOR;
    (horizontal * stored) / VERTICAL_EXTENT
}

/// The rest of the simulator stores anchors and dimensions in public units.
/// Enter and leave the same native unit space used by the inset methods.
pub(crate) fn round_retail_layout_value(
    public: f32,
    scale: f32,
    display_scale: f32,
    is_dimension: bool,
) -> f32 {
    if (is_dimension && public == 0.0)
        || !(scale > ROUNDING_EFFECTIVE_SCALE_THRESHOLD && display_scale > 0.0)
    {
        return public;
    }
    let stored = inset_to_storage(public);
    let horizontal = HORIZONTAL_EXTENT * UNIT_DENOMINATOR;
    let pixels = ((((horizontal * stored) / VERTICAL_EXTENT) * scale) / display_scale).round();
    let pixels = if is_dimension && public > 0.0 {
        pixels.max(1.0)
    } else if is_dimension && public < 0.0 {
        pixels.min(-1.0)
    } else {
        pixels
    };
    let rounded = ((pixels * display_scale) / (horizontal * scale)) * VERTICAL_EXTENT;
    inset_to_public(rounded)
}

/// Distinct geometry auxiliary word. The native numeric bit position is not
/// published; only the observed gate is represented, without aliases.
#[derive(Debug, Default)]
pub struct ClampGeometryAuxWord {
    pub ancestor_notification_gate: bool,
}

/// Host-only inputs for a simulated setter call frame. Never read actual VM
/// secret/taint state or change the existing authenticated argument readers.
#[derive(Debug)]
pub struct ClampInsetAccessInput {
    pub tracking_enabled: bool,
    pub initial_byte: u8,
    pub marked_reads: [bool; 4],
}

impl Default for ClampInsetAccessInput {
    fn default() -> Self {
        Self {
            tracking_enabled: true,
            initial_byte: 0,
            marked_reads: [false; 4],
        }
    }
}

impl ClampInsetAccessInput {
    pub(crate) fn begin(&self) -> ClampInsetReadState {
        ClampInsetReadState {
            byte: if self.tracking_enabled {
                0
            } else {
                self.initial_byte
            },
            restore_nonzero: self.tracking_enabled && self.initial_byte != 0,
            marked_reads: self.marked_reads,
        }
    }
}

/// Call-local model; this is discarded at return, never a persistent VM flag.
pub(crate) struct ClampInsetReadState {
    byte: u8,
    restore_nonzero: bool,
    marked_reads: [bool; 4],
}

impl ClampInsetReadState {
    pub(crate) fn accepted_dimension(&mut self, index: usize) {
        if self.marked_reads[index] {
            self.byte = 1;
        }
    }

    pub(crate) fn sample_then_restore(&mut self) -> u8 {
        let sample = self.byte;
        if self.restore_nonzero {
            self.byte = 1;
        }
        sample
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_access_samples_before_restoring_saved_nonzero_byte() {
        let input = ClampInsetAccessInput {
            initial_byte: 9,
            ..Default::default()
        };
        let mut call = input.begin();
        assert_eq!(call.sample_then_restore(), 0);
        assert_eq!(call.byte, 1);

        let mut call = ClampInsetAccessInput {
            initial_byte: 9,
            marked_reads: [false, false, true, false],
            ..Default::default()
        }
        .begin();
        call.accepted_dimension(2);
        assert_eq!(call.sample_then_restore(), 1);
        assert_eq!(call.byte, 1);

        let mut call = ClampInsetAccessInput {
            tracking_enabled: false,
            initial_byte: 9,
            ..Default::default()
        }
        .begin();
        assert_eq!(call.sample_then_restore(), 9);
        assert_eq!(call.byte, 9);
    }
    #[test]
    fn native_clamp_rounding_threshold_is_strict_and_dimensions_bypass_zero() {
        let threshold = f32::EPSILON;
        let next = f32::from_bits(threshold.to_bits() + 1);
        for dimension in [false, true] {
            for scale in [f32::NAN, 0.0, threshold] {
                assert_eq!(round_retail_layout_value(0.25, scale, 1.0, dimension), 0.25);
            }
            assert_ne!(round_retail_layout_value(0.25, next, 1.0, dimension), 0.25);
        }
        assert_eq!(round_retail_layout_value(0.0, next, 1.0, true), 0.0);
    }
}
