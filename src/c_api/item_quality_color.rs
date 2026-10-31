//! Item quality colors resolve the current named Lua color globals.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{create_string, table_get};
use crate::lua_bridge::{FromStack, IntoStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

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

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    table_set_rust_fn_static(state, state.global, "GetItemQualityColor", get_color)?;
    if ACTIVE != ClientProfile::Wrath {
        let item = super::helpers::ensure_namespace(state, "C_Item")?;
        table_set_rust_fn_static(state, item, "GetItemQualityColor", get_color)?;
    }
    Ok(())
}

fn get_color(state: &mut LuaState) -> LuaResult<u32> {
    let quality = i32::from_stack(state, 1)?;
    let color = COLOR_NAMES
        .get(quality as usize)
        .map(|name| table_get(state, Val::Table(state.global), name))
        .unwrap_or(Val::Nil);
    let (r, g, b) = match color {
        Val::Table(_) => (
            channel(state, color, "r"),
            channel(state, color, "g"),
            channel(state, color, "b"),
        ),
        _ => (1.0, 0.0, 1.0),
    };
    let hex = format!(
        "ff{:02x}{:02x}{:02x}",
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8
    );
    let hex = create_string(state, &hex);
    (r, g, b, hex).into_stack(state)
}

fn channel(state: &mut LuaState, color: Val, name: &str) -> f64 {
    match table_get(state, color, name) {
        Val::Num(value) => value,
        _ => 0.0,
    }
}
