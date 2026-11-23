//! Supplied skill-line and tab queries for `C_ProfSpecs`.
//!
//! Query defaults are INFERRED (R88ez, native evidence pending).

use super::profession_spec_inputs::{
    ProfessionSpecSkillLine, ProfessionSpecTabInfo, ProfessionSpecializationCurrencyInfo,
    ProfessionSpecializationTabInfo,
};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set_static,
};
use crate::lua_bridge::{FromStack, TableBuilder, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const CONFIG_ID_USAGE: &str =
    "Usage: local configID = C_ProfSpecs.GetConfigIDForSkillLine(skillLineID)";
const CURRENCY_INFO_USAGE: &str =
    "Usage: local info = C_ProfSpecs.GetCurrencyInfoForSkillLine(skillLineID)";
const SPEC_TAB_IDS_USAGE: &str =
    "Usage: local specTabIDs = C_ProfSpecs.GetSpecTabIDsForSkillLine(skillLineID)";
const HAS_SPECIALIZATION_USAGE: &str =
    "Usage: local hasSpecialization = C_ProfSpecs.SkillLineHasSpecialization(skillLineID)";
const TAB_INFO_USAGE: &str = "Usage: local tabInfo = C_ProfSpecs.GetTabInfo(tabTreeID)";

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        return Ok(());
    }

    let namespace = super::ensure_namespace(state, "C_ProfSpecs")?;
    table_set_rust_fn_static(state, namespace, "GetConfigIDForSkillLine", get_config_id)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetCurrencyInfoForSkillLine",
        get_currency_info,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetDefaultSpecSkillLine",
        get_default_skill_line,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetSpecTabIDsForSkillLine",
        get_spec_tab_ids,
    )?;
    table_set_rust_fn_static(state, namespace, "GetSpecTabInfo", get_spec_tab_info)?;
    table_set_rust_fn_static(state, namespace, "GetTabInfo", get_tab_info)?;
    table_set_rust_fn_static(state, namespace, "ShouldShowSpecTab", should_show_spec_tab)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "SkillLineHasSpecialization",
        skill_line_has_specialization,
    )
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    let builder = builder
        .set_function(
            "SetProfessionSpecSkillLines",
            set_profession_spec_skill_lines,
        )?
        .set_function("SetProfessionSpecTabInfo", set_profession_spec_tab_info)?;
    Ok(builder)
}

fn read_skill_line_id(state: &LuaState, api: &'static str, usage: &'static str) -> LuaResult<u32> {
    let value = super::helpers::read_untainted_signed32_argument(state, 1, api, usage)?;
    u32::try_from(value).map_err(|_| runtime_error("skillLineID must be non-negative"))
}

fn default_skill_line(state: &LuaState) -> LuaResult<Option<u32>> {
    let sim = borrow_state(state)?;
    let current_skill_line = sim
        .crafting
        .selected_profession_id
        .and_then(|id| u32::try_from(id).ok());
    Ok(sim
        .profession_spec_inputs
        .default_skill_line(current_skill_line))
}

fn get_config_id(state: &mut LuaState) -> LuaResult<u32> {
    let skill_line = read_skill_line_id(state, "GetConfigIDForSkillLine", CONFIG_ID_USAGE)?;
    let config_id = borrow_state(state)?
        .profession_spec_inputs
        .skill_line(skill_line)
        .map(|record| record.config_id)
        .unwrap_or_default();
    state.push(Val::Num(f64::from(config_id)));
    Ok(1)
}

fn get_currency_info(state: &mut LuaState) -> LuaResult<u32> {
    let skill_line = read_skill_line_id(state, "GetCurrencyInfoForSkillLine", CURRENCY_INFO_USAGE)?;
    let currency = borrow_state(state)?
        .profession_spec_inputs
        .skill_line(skill_line)
        .map(|record| record.currency_info.clone())
        .unwrap_or_default();
    let table = create_table(state);
    table_set_static(
        state,
        table,
        "numAvailable",
        Val::Num(f64::from(currency.num_available)),
    );
    let currency_name = create_string(state, &currency.currency_name);
    table_set_static(state, table, "currencyName", currency_name);
    state.push(table);
    Ok(1)
}

fn get_default_skill_line(state: &mut LuaState) -> LuaResult<u32> {
    state.push(default_skill_line(state)?.map_or(Val::Nil, |id| Val::Num(f64::from(id))));
    Ok(1)
}

fn get_spec_tab_ids(state: &mut LuaState) -> LuaResult<u32> {
    let skill_line = read_skill_line_id(state, "GetSpecTabIDsForSkillLine", SPEC_TAB_IDS_USAGE)?;
    let tab_ids = borrow_state(state)?
        .profession_spec_inputs
        .skill_line(skill_line)
        .map(|record| record.spec_tab_ids.clone())
        .unwrap_or_default();
    let Val::Table(table) = create_table(state) else {
        unreachable!("create_table returns a table");
    };
    for (index, tab_id) in tab_ids.into_iter().enumerate() {
        crate::lua_api::methods::table_set_num(
            state,
            table,
            (index + 1) as f64,
            Val::Num(f64::from(tab_id)),
        );
    }
    state.push(Val::Table(table));
    Ok(1)
}

fn get_spec_tab_info(state: &mut LuaState) -> LuaResult<u32> {
    let info = if let Some(skill_line) = default_skill_line(state)? {
        borrow_state(state)?
            .profession_spec_inputs
            .skill_line(skill_line)
            .map(|record| {
                record
                    .spec_tab_info
                    .clone()
                    .unwrap_or(ProfessionSpecializationTabInfo {
                        enabled: true,
                        error_reason: String::new(),
                    })
            })
    } else {
        None
    };
    let info = info.unwrap_or(ProfessionSpecializationTabInfo {
        enabled: false,
        error_reason: String::new(),
    });
    let table = create_table(state);
    table_set_static(state, table, "enabled", Val::Bool(info.enabled));
    let error_reason = create_string(state, &info.error_reason);
    table_set_static(state, table, "errorReason", error_reason);
    state.push(table);
    Ok(1)
}

fn get_tab_info(state: &mut LuaState) -> LuaResult<u32> {
    let tab_id =
        super::helpers::read_untainted_signed32_argument(state, 1, "GetTabInfo", TAB_INFO_USAGE)?;
    let tab_info = match u32::try_from(tab_id) {
        Ok(id) => borrow_state(state)?
            .profession_spec_inputs
            .tab_infos
            .get(&id)
            .cloned(),
        Err(_) => None,
    };
    let Some(tab_info) = tab_info else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let table = create_table(state);
    table_set_static(
        state,
        table,
        "rootNodeID",
        Val::Num(f64::from(tab_info.root_node_id)),
    );
    let name = create_string(state, &tab_info.name);
    table_set_static(state, table, "name", name);
    let description = create_string(state, &tab_info.description);
    table_set_static(state, table, "description", description);
    table_set_static(
        state,
        table,
        "rootIconID",
        Val::Num(f64::from(tab_info.root_icon_id)),
    );
    let Val::Table(highlights) = create_table(state) else {
        unreachable!("create_table returns a table");
    };
    for (index, description) in tab_info.highlights.into_iter().enumerate() {
        let Val::Table(highlight) = create_table(state) else {
            unreachable!("create_table returns a table");
        };
        let description = create_string(state, &description);
        table_set_static(state, Val::Table(highlight), "description", description);
        crate::lua_api::methods::table_set_num(
            state,
            highlights,
            (index + 1) as f64,
            Val::Table(highlight),
        );
    }
    table_set_static(state, table, "highlights", Val::Table(highlights));
    state.push(table);
    Ok(1)
}

fn should_show_spec_tab(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(default_skill_line(state)?.is_some()));
    Ok(1)
}

fn skill_line_has_specialization(state: &mut LuaState) -> LuaResult<u32> {
    let skill_line = read_skill_line_id(
        state,
        "SkillLineHasSpecialization",
        HAS_SPECIALIZATION_USAGE,
    )?;
    let has_specialization = borrow_state(state)?
        .profession_spec_inputs
        .skill_line(skill_line)
        .is_some();
    state.push(Val::Bool(has_specialization));
    Ok(1)
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
