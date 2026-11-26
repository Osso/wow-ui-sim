//! R80: host-supplied ordinary item stat amounts and producer metadata; R88aw: bag-item status snapshots.
//! Native item-mod amount calculation and calculated SellPrice values remain UNKNOWN.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::{TableBuilder, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

/// One host-supplied instance socket row. Native row selection remains UNKNOWN.
#[derive(Clone, Debug, Default)]
pub struct ItemSocketProjectionRow {
    pub socket_type: Option<String>,
    pub gem_icon: Option<String>,
    pub gem_item_id: Option<u32>,
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if ACTIVE != ClientProfile::Retail {
        return Ok(builder);
    }
    builder.set_function(
        "SetItemTooltipSellPriceQuantities",
        set_item_tooltip_sell_price_quantities,
    )
}

fn set_item_tooltip_sell_price_quantities(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = read_u32(state, 1, "itemID")?;
    let current = read_optional_u32(state, 2, "currentQuantity")?;
    let prepared_max = read_optional_u32(state, 3, "preparedMaxQuantity")?;
    let mut sim = borrow_state_mut(state)?;
    match (current, prepared_max) {
        (Some(current), Some(prepared_max)) => {
            sim.item_tooltip_sell_price_quantities
                .insert(item_id, (current, prepared_max));
        }
        _ => {
            sim.item_tooltip_sell_price_quantities.remove(&item_id);
        }
    }
    Ok(0)
}

fn read_optional_u32(state: &LuaState, position: i32, field: &str) -> LuaResult<Option<u32>> {
    if stack_val(state, position).is_nil() {
        Ok(None)
    } else {
        read_u32(state, position, field).map(Some)
    }
}

fn read_u32(state: &LuaState, position: i32, field: &str) -> LuaResult<u32> {
    match stack_val(state, position) {
        Val::Num(value)
            if value.is_finite()
                && value.fract() == 0.0
                && (0.0..=f64::from(u32::MAX)).contains(&value) =>
        {
            Ok(value as u32)
        }
        _ => Err(runtime_error(format!(
            "A_Admin.SetItemTooltipSellPriceQuantities: {field} must be an unsigned integer"
        ))),
    }
}

#[derive(Clone, Debug)]
pub struct ItemTooltipStat {
    pub label: String,
    pub amount: i32,
}

/// Inputs for a catalog item at one resolved working level. This is not a native
/// DB2 mapping. Hosts must not reuse a row for unrepresented instance variants.
#[derive(Clone, Debug, Default)]
pub struct ItemTooltipInput {
    pub stats: Vec<ItemTooltipStat>,
    pub guid: Option<String>,
    pub hyperlink: Option<String>,
    pub override_item_level: Option<u32>,
    pub is_corrupted_item: Option<bool>,
    pub is_azerite_empowered_item: Option<bool>,
    pub is_azerite_item: Option<bool>,
}

/// Explicit status inputs for the Retail bag-item SellPrice gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SellPriceStatusInput {
    StatusRecord {
        flags: u32,
        recorded_time: i64,
        client_clock: i64,
    },
    NoStatusRecord {
        instance_marker_flags: u32,
    },
}
