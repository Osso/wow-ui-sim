// C_GamePad.GetAllConfigIDs: GamePadDocumentation.lua:117.
use super::{
    borrow_state, borrow_state_mut, create_table, read_input, register_methods, set_table_array,
};
use crate::lua_api::methods::create_table_with_fields;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(super) struct ConfigId {
    #[serde(rename = "vendorID")]
    vendor_id: Option<u32>,
    #[serde(rename = "productID")]
    product_id: Option<u32>,
}

pub(super) fn supply(state: &mut LuaState) -> LuaResult<u32> {
    let records = read_input(state)?;
    borrow_state_mut(state)?
        .scenario_observations
        .gamepad_configuration_ids = records;
    Ok(0)
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    register_methods(
        state,
        "C_GamePad",
        &[("GetAllConfigIDs", configuration_ids)],
    )
}

fn configuration_ids(state: &mut LuaState) -> LuaResult<u32> {
    let ids = borrow_state(state)?
        .scenario_observations
        .gamepad_configuration_ids
        .clone();
    let result = create_table(state);
    for (index, id) in ids.iter().enumerate() {
        let row = create_table_with_fields(
            state,
            &[
                (
                    "vendorID",
                    id.vendor_id.map_or(Val::Nil, |id| Val::Num(f64::from(id))),
                ),
                (
                    "productID",
                    id.product_id.map_or(Val::Nil, |id| Val::Num(f64::from(id))),
                ),
            ],
        );
        set_table_array(state, result, index as i64 + 1, row);
    }
    state.push(result);
    Ok(1)
}
