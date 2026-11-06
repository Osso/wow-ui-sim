//! Item quality colors resolve the current named Lua color globals.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::global_colors::global_color_rgb;
use crate::lua_api::methods::create_string;
use crate::lua_bridge::{FromStack, IntoStack, table_set_rust_fn_static};
use rilua::LuaResult;
use rilua::vm::state::LuaState;

const COLOR_NAMES: [&str; 9] = [
    "ITEM_POOR_COLOR",
    "ITEM_STANDARD_COLOR",
    "ITEM_GOOD_COLOR",
    "ITEM_SUPERIOR_COLOR",
    "ITEM_EPIC_COLOR",
    "ITEM_LEGENDARY_COLOR",
    "ITEM_ARTIFACT_COLOR",
    "ITEM_SCALING_STAT_COLOR",
    "ITEM_WOW_TOKEN_COLOR",
];

/// Opaque magenta for an unusable quality, as the native function returns.
const FALLBACK_RGB: (f64, f64, f64) = (1.0, 0.0, 1.0);

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE.has_legacy_item_globals() {
        table_set_rust_fn_static(state, state.global, "GetItemQualityColor", get_color)?;
    }
    if ACTIVE != ClientProfile::Wrath {
        let item = super::helpers::ensure_namespace(state, "C_Item")?;
        table_set_rust_fn_static(state, item, "GetItemQualityColor", get_color)?;
    }
    Ok(())
}

fn get_color(state: &mut LuaState) -> LuaResult<u32> {
    let quality = i32::from_stack(state, 1)?;
    let (r, g, b) = named_quality_color(state, quality).unwrap_or(FALLBACK_RGB);
    let hex = create_string(state, &rgb_hex(r, g, b));
    (r, g, b, hex).into_stack(state)
}

/// `ffRRGGBB` of a quality, the colour `GetItemQualityColor` reports.
pub(crate) fn quality_hex(state: &mut LuaState, quality: i32) -> String {
    let (r, g, b) = named_quality_color(state, quality).unwrap_or(FALLBACK_RGB);
    rgb_hex(r, g, b)
}

fn rgb_hex(r: f64, g: f64, b: f64) -> String {
    format!(
        "ff{:02x}{:02x}{:02x}",
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8
    )
}

/// The named global is read through the Lua global index, so the simulator's
/// synthesized colour constants are visible to the consumer.
fn named_quality_color(state: &mut LuaState, quality: i32) -> Option<(f64, f64, f64)> {
    let name = COLOR_NAMES.get(quality as usize)?;
    global_color_rgb(state, name)
}
