//! Legacy item globals.
//!
//! Plain global wrappers stay in the Lua globals layer; their modeled C_Item
//! implementation belongs to `c_api::item_spell`.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::global_colors::global_color_rgb;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{IntoStack, table_set_rust_fn_static};
use rilua::LuaResult;
use rilua::vm::state::LuaState;

const ITEM_STANDARD_QUALITY: u8 = 1;
const ITEM_SUPERIOR_QUALITY: u8 = 3;
const ITEM_EPIC_QUALITY: u8 = 4;
const ITEM_LEVEL_COLOR_FALLBACK: (f64, f64, f64) = (1.0, 0.0, 1.0);

pub(crate) fn register_legacy_item_globals(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE.has_legacy_item_globals() {
        table_set_rust_fn_static(
            state,
            state.global,
            "GetItemSpell",
            crate::c_api::item_spell::get_item_spell,
        )?;
    }
    table_set_rust_fn_static(
        state,
        state.global,
        "GetItemID",
        crate::c_api::item_spell::c_item_get_item_id,
    )?;
    if ACTIVE == ClientProfile::Retail {
        table_set_rust_fn_static(
            state,
            state.global,
            "GetItemLevelColor",
            get_item_level_color,
        )?;
    }
    Ok(())
}

fn get_item_level_color(state: &mut LuaState) -> LuaResult<u32> {
    let (condition_23, condition_24) = {
        let state = borrow_state(state)?;
        (
            state.item_level_color_condition_23,
            state.item_level_color_condition_24,
        )
    };
    let (quality, color_name) = selected_item_level_color(condition_23, condition_24);
    let (red, green, blue) =
        crate::c_api::c_color_overrides::color_override_rgb_for_quality(state, quality)
            .or_else(|| global_color_rgb(state, color_name))
            .unwrap_or(ITEM_LEVEL_COLOR_FALLBACK);
    (
        f64::from(red as f32),
        f64::from(green as f32),
        f64::from(blue as f32),
    )
        .into_stack(state)
}

fn selected_item_level_color(
    condition_23: Option<bool>,
    condition_24: Option<bool>,
) -> (u8, &'static str) {
    if condition_24 == Some(true) {
        (ITEM_EPIC_QUALITY, "ITEM_EPIC_COLOR")
    } else if condition_23 == Some(true) {
        (ITEM_SUPERIOR_QUALITY, "ITEM_SUPERIOR_COLOR")
    } else {
        (ITEM_STANDARD_QUALITY, "ITEM_STANDARD_COLOR")
    }
}
