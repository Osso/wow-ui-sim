//! Mists corpse coordinates from explicit observations and existing death state;
//! Retail, PTR and Forever self-resurrect options from host inputs.
//!
//! `C_DeathInfo.GetSelfResurrectOptions` (DeathInfoDocumentation.lua:55 on
//! Retail and Forever, `MayReturnNothing`) returns the `SelfResurrectOption`
//! records supplied by `A_Admin.AddSelfResurrectOption(name, optionType, id,
//! canUse, isLimited, priority)` in supply order; `A_Admin.ClearSelfResurrectOptions()`
//! removes them. INFERRED R88g9 (native evidence pending): without a supplied
//! option the call returns nothing, which the DEATH dialog treats as no option.
//!
//! `C_DeathInfo.GetCorpseMapPosition(uiMapID)` (DeathInfoDocumentation.lua:10-22,
//! byte-identical on Retail and Forever) reads the corpse observations supplied by
//! `A_Admin.SetCorpseMapPosition(uiMapID, x, y)`; `A_Admin.ClearCorpse()` removes
//! the corpse. INFERRED R88g9 (native evidence pending): a supplied position also
//! marks the corpse as available, the same flag `RetrieveCorpse` clears.

use crate::c_api::helpers::set_table_array;
use crate::c_api::{ensure_namespace, global_val};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, create_string, create_table, table_set,
};
use crate::lua_bridge::{FromStack, TableBuilder, table_set_rust_fn_static};
use rilua::{LuaResult, Val, vm::state::LuaState};

/// One supplied `SelfResurrectOption` record.
#[derive(Clone, Debug, PartialEq)]
pub struct SelfResurrectOption {
    pub name: String,
    pub option_type: i32,
    pub id: i32,
    pub can_use: bool,
    pub is_limited: bool,
    pub priority: i32,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        let namespace = ensure_namespace(state, "C_DeathInfo")?;
        table_set_rust_fn_static(state, namespace, "GetSelfResurrectOptions", query_options)?;
    }
    if !matches!(
        ACTIVE,
        ClientProfile::Retail
            | ClientProfile::Ptr
            | ClientProfile::WowForever
            | ClientProfile::Mists
    ) {
        return Ok(());
    }
    let namespace = ensure_namespace(state, "C_DeathInfo")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetCorpseMapPosition",
        query_corpse_position,
    )
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder
        .set_function("AddSelfResurrectOption", add_option)?
        .set_function("ClearSelfResurrectOptions", |state| {
            borrow_state_mut(state)?.self_resurrect_options.clear();
            Ok(0)
        })?
        .set_function("SetCorpseMapPosition", set_corpse_map_position)?
        .set_function("ClearCorpse", clear_corpse)
}

/// `A_Admin.SetCorpseMapPosition(uiMapID, x, y)`: a positive map ID and
/// normalized map coordinates in `[0, 1]`.
fn set_corpse_map_position(state: &mut LuaState) -> LuaResult<u32> {
    let map_id = i32::from_stack(state, 1)?;
    let x = f64::from_stack(state, 2)?;
    let y = f64::from_stack(state, 3)?;
    if map_id <= 0 || ![x, y].iter().all(|value| (0.0..=1.0).contains(value)) {
        return Err(rilua::runtime_error(
            "SetCorpseMapPosition: uiMapID must be positive and x, y within [0, 1]",
        ));
    }
    let mut sim = borrow_state_mut(state)?;
    sim.corpse_map_positions.insert(map_id, (x, y));
    sim.corpse_available = true;
    Ok(0)
}

fn clear_corpse(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    sim.corpse_map_positions.clear();
    sim.corpse_available = false;
    Ok(0)
}

fn add_option(state: &mut LuaState) -> LuaResult<u32> {
    let option = SelfResurrectOption {
        name: String::from_stack(state, 1)?,
        option_type: i32::from_stack(state, 2)?,
        id: i32::from_stack(state, 3)?,
        can_use: bool::from_stack(state, 4)?,
        is_limited: bool::from_stack(state, 5)?,
        priority: i32::from_stack(state, 6)?,
    };
    borrow_state_mut(state)?.self_resurrect_options.push(option);
    Ok(0)
}

fn query_options(state: &mut LuaState) -> LuaResult<u32> {
    let options = borrow_state(state)?.self_resurrect_options.clone();
    if options.is_empty() {
        return Ok(0);
    }
    let list = create_table(state);
    for (index, option) in options.iter().enumerate() {
        let row = push_option(state, option);
        set_table_array(state, list, index as i64 + 1, row);
    }
    state.push(list);
    Ok(1)
}

fn push_option(state: &mut LuaState, option: &SelfResurrectOption) -> Val {
    let row = create_table(state);
    let name = create_string(state, &option.name);
    table_set(state, row, "name", name);
    for (key, value) in [
        ("optionType", option.option_type),
        ("id", option.id),
        ("priority", option.priority),
    ] {
        table_set(state, row, key, Val::Num(f64::from(value)));
    }
    table_set(state, row, "canUse", Val::Bool(option.can_use));
    table_set(state, row, "isLimited", Val::Bool(option.is_limited));
    row
}

fn query_corpse_position(state: &mut LuaState) -> LuaResult<u32> {
    let map_id = i32::from_stack(state, 1)?;
    let position = {
        let sim = borrow_state(state)?;
        let has_corpse = sim.player.health <= 0 && sim.corpse_available;
        has_corpse
            .then(|| sim.corpse_map_positions.get(&map_id).copied())
            .flatten()
    };
    let Some((x, y)) = position else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let factory = global_val(state, "CreateVector2D");
    let vector = call_function_state(state, factory, &[Val::Num(x), Val::Num(y)])?;
    state.push(vector);
    Ok(1)
}
