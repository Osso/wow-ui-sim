//! WM82 / WM63 D56 common publication rules, bounded to Retail/PTR.
//!
//! Getter miss decisions remain with their producers. No GUID is synthesized.
use crate::lua_api::methods::{
    create_string, create_table, registry_get, registry_set, table_get, table_set,
};
use rilua::{Val, vm::state::LuaState};

#[cfg(all(test, any(feature = "profile-retail", feature = "client-ptr")))]
#[path = "tooltip_data_tests.rs"]
mod tests;

pub(crate) fn enabled() -> bool {
    matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    )
}

pub(crate) fn stamp_instance(state: &mut LuaState, tooltip: Val) {
    let counter = match registry_get(state, "__tooltip_data_instance_counter") {
        Val::Num(counter) => counter + 1.0,
        _ => 1.0,
    };
    registry_set(state, "__tooltip_data_instance_counter", Val::Num(counter));
    table_set(state, tooltip, "dataInstanceID", Val::Num(counter));
    if table_get(state, tooltip, "id") == Val::Num(0.0) {
        table_set(state, tooltip, "id", Val::Nil);
    }
}

/// EMULATED input seam: scenario colors are floats, not native packed bytes.
/// Round to the nearest saturated byte. D56 establishes byte-to-float output,
/// not the native float-to-byte rule. Keep that uncertainty in this one seam.
fn scenario_color_byte(value: f64) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn packed_component(value: f64) -> f64 {
    f64::from(f32::from(scenario_color_byte(value)) * (1.0_f32 / 255.0))
}

/// Keep the existing ColorMixin factory, but publish only the D56 RGB fields.
/// Complete native ColorMixin contents are UNKNOWN (D56).
pub(crate) fn publish_rgb(state: &mut LuaState, color: Val, rgb: (f64, f64, f64)) {
    table_set(state, color, "r", Val::Num(packed_component(rgb.0)));
    table_set(state, color, "g", Val::Num(packed_component(rgb.1)));
    table_set(state, color, "b", Val::Num(packed_component(rgb.2)));
    table_set(state, color, "a", Val::Nil);
}

/// Empty text is supplied; absent text publishes neither a text nor a color key.
pub(crate) fn publish_side(
    state: &mut LuaState,
    line: Val,
    right: bool,
    text: Option<&str>,
    color: Val,
) {
    let Some(text) = text else {
        return;
    };
    let (text_key, color_key) = if right {
        ("rightText", "rightColor")
    } else {
        ("leftText", "leftColor")
    };
    let value = create_string(state, text);
    table_set(state, line, text_key, value);
    table_set(state, line, color_key, color);
}

/// A second identity assignment appends a nested block, never replaces identity.
/// Eligibility for invoking a second assignment is getter-specific and UNKNOWN.
pub(crate) fn assign_identity(
    state: &mut LuaState,
    tooltip: Val,
    tooltip_type: f64,
    id: u32,
    left_color: Val,
) {
    if table_get(state, tooltip, "type") == Val::Nil {
        table_set(state, tooltip, "type", Val::Num(tooltip_type));
        if id != 0 {
            table_set(state, tooltip, "id", Val::Num(f64::from(id)));
        }
        return;
    }
    let lines = table_get(state, tooltip, "lines");
    let Val::Table(lines_ref) = lines else {
        return;
    };
    let mut index = 1;
    while state
        .gc
        .tables
        .get(lines_ref)
        .is_some_and(|table| table.get_int(index) != Val::Nil)
    {
        index += 1;
    }
    let line = create_table(state);
    if let Some(table) = state.gc.tables.get_mut(lines_ref) {
        table
            .raw_set(Val::Num(index as f64), line, &state.gc.string_arena)
            .expect("valid line index");
    }
    state.gc.barrier_back(lines_ref);
    table_set(state, line, "type", Val::Num(19.0));
    publish_side(state, line, false, Some(""), left_color);
    table_set(state, line, "tooltipType", Val::Num(tooltip_type));
    if id != 0 {
        table_set(state, line, "tooltipID", Val::Num(f64::from(id)));
    }
}
