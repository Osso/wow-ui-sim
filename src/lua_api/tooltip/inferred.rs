//! Simulator policies not established by R88n.7.

use super::TooltipData;
use rilua::{LuaError, runtime_error};

/// INFERRED: use one fixed bad-self envelope. R88n.7 leaves native caller-frame
/// decoration UNKNOWN because it depends on runtime debug state, which is not modeled here.
pub(crate) fn inferred_bad_self_error(
    method_name: Option<&str>,
    usage: &str,
    reason: &str,
) -> LuaError {
    match method_name {
        Some(name) => runtime_error(format!("calling '{name}' on bad self ({reason})")),
        None => runtime_error(format!("{usage} - {reason}")),
    }
}

/// INFERRED: ClearLines and the simulator's Hide path retain FontString pairs
/// and aligned state. R88n.7 leaves later pair lifetime UNKNOWN.
pub(crate) fn inferred_clear_lines_preserving_font_string_state(
    tooltip: &mut TooltipData,
    is_wrath: bool,
) {
    tooltip.lines.clear();
    tooltip.anchored_left_lines = 0;
    tooltip.extra_inset = 0.0;
    if is_wrath {
        tooltip.min_width = 0.0;
        tooltip.wrath_texture_rows.clear();
    }
    tooltip.spell_id = None;
    tooltip.unit_token = None;
    tooltip.unit_name = None;
    tooltip.unit_guid = None;
}

/// INFERRED: the simulator's CopyTooltip source clone also copies FontString
/// pair state. R88n.7 leaves native pair copying and later lifetime UNKNOWN.
pub(crate) fn inferred_copy_tooltip_source_with_font_string_state(
    source: &TooltipData,
) -> TooltipData {
    source.clone()
}
