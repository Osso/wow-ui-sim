use super::super::profession_spec_inputs::{
    ProfessionSpecSkillLine, ProfessionSpecTabInfo, ProfessionSpecializationCurrencyInfo,
    ProfessionSpecializationTabInfo,
};
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::{FromStack, TableBuilder};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    let builder = builder
        .set_function(
            "SetProfessionSpecSkillLines",
            set_profession_spec_skill_lines,
        )?
        .set_function("SetProfessionSpecTabInfo", set_profession_spec_tab_info)?;
    Ok(builder)
}

fn set_profession_spec_skill_lines(state: &mut LuaState) -> LuaResult<u32> {
    let records = table_argument(state, 1)?;
    let record_count = state
        .gc
        .tables
        .get(records)
        .map_or(0, |table| table.len(&state.gc.string_arena));
    let skill_lines = (1..=record_count)
        .map(|index| read_skill_line_record(state, records, index as i64))
        .collect::<LuaResult<Vec<_>>>()?;
    borrow_state_mut(state)?.profession_spec_inputs.skill_lines = skill_lines;
    Ok(0)
}

fn read_skill_line_record(
    state: &mut LuaState,
    records: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
    index: i64,
) -> LuaResult<ProfessionSpecSkillLine> {
    let row = state
        .gc
        .tables
        .get(records)
        .map(|table| table.get_int(index))
        .unwrap_or(Val::Nil);
    let Val::Table(row) = row else {
        return Err(runtime_error(
            "profession spec skill-line entries must be tables",
        ));
    };
    let row_value = Val::Table(row);
    let skill_line_id = number_field(state, row_value, "skillLineID") as u32;
    let config_id = number_field(state, row_value, "configID") as i32;
    let spec_tab_ids = read_number_array(state, row, "specTabIDs")?
        .into_iter()
        .map(|id| id as u32)
        .collect();
    let currency_table = table_field(state, row, "currency");
    let currency_info = ProfessionSpecializationCurrencyInfo {
        num_available: number_field(state, currency_table, "numAvailable") as i32,
        currency_name: string_field(state, currency_table, "currencyName"),
    };
    let spec_tab_info = match table_field(state, row, "specTabInfo") {
        value @ Val::Table(_) => Some(ProfessionSpecializationTabInfo {
            enabled: bool_field(state, value, "enabled"),
            error_reason: string_field(state, value, "errorReason"),
        }),
        _ => None,
    };
    Ok(ProfessionSpecSkillLine {
        skill_line_id,
        config_id,
        spec_tab_ids,
        currency_info,
        spec_tab_info,
    })
}

fn set_profession_spec_tab_info(state: &mut LuaState) -> LuaResult<u32> {
    let tab_id = i32::from_stack(state, 1)?;
    let info = Val::from_stack(state, 2)?;
    let Val::Table(info) = info else {
        return Err(runtime_error("profession spec tab info must be a table"));
    };
    let tab_id =
        u32::try_from(tab_id).map_err(|_| runtime_error("tabTreeID must be non-negative"))?;
    let record = ProfessionSpecTabInfo {
        root_node_id: number_field(state, Val::Table(info), "rootNodeID") as i32,
        name: string_field(state, Val::Table(info), "name"),
        description: string_field(state, Val::Table(info), "description"),
        root_icon_id: number_field(state, Val::Table(info), "rootIconID") as i32,
        highlights: read_string_array_field(state, info, "highlights")?,
    };
    borrow_state_mut(state)?
        .profession_spec_inputs
        .tab_infos
        .insert(tab_id, record);
    Ok(0)
}

fn table_argument(
    state: &mut LuaState,
    index: i32,
) -> LuaResult<rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>> {
    match Val::from_stack(state, index)? {
        Val::Table(table) => Ok(table),
        _ => Err(runtime_error(
            "profession spec skill-line records must be a table",
        )),
    }
}

fn table_field(
    state: &mut LuaState,
    table: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
    key: &str,
) -> Val {
    crate::lua_api::methods::table_get(state, Val::Table(table), key)
}

fn number_field(state: &mut LuaState, table: Val, key: &str) -> f64 {
    match table {
        Val::Table(table) => {
            match crate::lua_api::methods::table_get(state, Val::Table(table), key) {
                Val::Num(value) => value,
                _ => 0.0,
            }
        }
        _ => 0.0,
    }
}

fn string_field(state: &mut LuaState, table: Val, key: &str) -> String {
    match table {
        Val::Table(table) => {
            match crate::lua_api::methods::table_get(state, Val::Table(table), key) {
                value @ Val::Str(_) => {
                    crate::lua_api::methods::val_to_string(state, value).unwrap_or_default()
                }
                _ => String::new(),
            }
        }
        _ => String::new(),
    }
}

fn bool_field(state: &mut LuaState, table: Val, key: &str) -> bool {
    match table {
        Val::Table(table) => matches!(
            crate::lua_api::methods::table_get(state, Val::Table(table), key),
            Val::Bool(true)
        ),
        _ => false,
    }
}

fn read_number_array(
    state: &mut LuaState,
    table: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
    key: &str,
) -> LuaResult<Vec<f64>> {
    let value = table_field(state, table, key);
    let Val::Table(array) = value else {
        return Ok(Vec::new());
    };
    let length = state
        .gc
        .tables
        .get(array)
        .map_or(0, |array| array.len(&state.gc.string_arena));
    (1..=length)
        .map(|index| {
            match state
                .gc
                .tables
                .get(array)
                .map(|array| array.get_int(index as i64))
            {
                Some(Val::Num(value)) => Ok(value),
                _ => Err(runtime_error("profession spec tab IDs must be numbers")),
            }
        })
        .collect()
}

fn read_string_array_field(
    state: &mut LuaState,
    table: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
    key: &str,
) -> LuaResult<Vec<String>> {
    let value = table_field(state, table, key);
    let Val::Table(array) = value else {
        return Ok(Vec::new());
    };
    let length = state
        .gc
        .tables
        .get(array)
        .map_or(0, |array| array.len(&state.gc.string_arena));
    (1..=length)
        .map(|index| {
            match state
                .gc
                .tables
                .get(array)
                .map(|array| array.get_int(index as i64))
            {
                Some(value @ Val::Str(_)) => {
                    Ok(crate::lua_api::methods::val_to_string(state, value).unwrap_or_default())
                }
                _ => Err(runtime_error(
                    "profession spec highlight descriptions must be strings",
                )),
            }
        })
        .collect()
}
