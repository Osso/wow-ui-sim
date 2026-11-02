//! Item reward classifiers: IsAnimaItemByID and IsArtifactPowerItem.

use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use sim_data::item_classifiers::{ARTIFACT_POWER_SPELLS, ITEM_CURRENCY_VALUES};
use sim_data::item_effects::ITEM_EFFECTS;

const RESERVOIR_ANIMA_CURRENCY: u32 = 1813;
/// Spell extraction rejects this trigger and any trigger above 10 (WM34-A).
const REJECTED_TRIGGER: i32 = 8;
const MAX_TRIGGER: i32 = 10;

fn item_argument(state: &mut LuaState, api: &str) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    super::parse_item_id_key(state, value)
        .ok_or_else(|| runtime_error(format!("C_Item.{api}: expected itemInfo")))
}

/// Signed 32-bit wrapping total of anima-linked ItemCurrencyValue rows (WM45).
fn anima_value(item: u32) -> i32 {
    let start = ITEM_CURRENCY_VALUES.partition_point(|row| row.0 < item);
    let end = ITEM_CURRENCY_VALUES.partition_point(|row| row.0 <= item);
    ITEM_CURRENCY_VALUES[start..end]
        .iter()
        .filter(|(_, value, currencies)| {
            *value > 0 && currencies.contains(&RESERVOIR_ANIMA_CURRENCY)
        })
        .fold(0_i32, |total, (_, value, _)| total.wrapping_add(*value))
}

/// Whether any collected item-effect spell's effect index 0 grants artifact power (WM34-A).
fn grants_artifact_power(item: u32) -> bool {
    if sim_data::items::get_item(item).is_none() {
        return false;
    }
    let start = ITEM_EFFECTS.partition_point(|row| row.0 < item);
    let end = ITEM_EFFECTS.partition_point(|row| row.0 <= item);
    ITEM_EFFECTS[start..end]
        .iter()
        .any(|&(_, _, spell, trigger, ..)| {
            trigger != REJECTED_TRIGGER
                && trigger <= MAX_TRIGGER
                && ARTIFACT_POWER_SPELLS.binary_search(&spell).is_ok()
        })
}

pub(super) fn is_anima_item_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let item = item_argument(state, "IsAnimaItemByID")?;
    state.push(Val::Bool(anima_value(item) > 0));
    Ok(1)
}

pub(super) fn is_artifact_power_item(state: &mut LuaState) -> LuaResult<u32> {
    let item = item_argument(state, "IsArtifactPowerItem")?;
    state.push(Val::Bool(grants_artifact_power(item)));
    Ok(1)
}
