//! Host housing-market policy and house-finder plot selection.

use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState, housing: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        housing,
        "IsHousingMarketShopEnabled",
        is_market_shop_enabled,
    )?;
    #[cfg(feature = "retail-12-0-5")]
    table_set_rust_fn_static(
        state,
        housing,
        "IsHousingMarketCartFullRemoveEnabled",
        |s| {
            let enabled = borrow_state(s)?.housing.market_cart_full_remove_enabled;
            s.push(Val::Bool(enabled));
            Ok(1)
        },
    )?;
    table_set_rust_fn_static(state, housing, "OnHouseFinderClickPlot", select_plot)
}

fn is_market_shop_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?.housing.market_shop_enabled;
    state.push(Val::Bool(enabled));
    Ok(1)
}

fn select_plot(state: &mut LuaState) -> LuaResult<u32> {
    let plot_id = i32::from_stack(state, 1)?;
    // INFERRED: record the UI selection only. The contract documents no return
    // or event; do not fabricate a purchase, teleport, or service response.
    borrow_state_mut(state)?.housing.house_finder_selected_plot = Some(plot_id);
    Ok(0)
}
