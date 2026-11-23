//! Supplied skill-line and tab queries for `C_ProfSpecs`.
//!
//! Query defaults are INFERRED (R88ez, native evidence pending).

use super::profession_spec_inputs::{ProfessionSpecTabInfo, ProfessionSpecializationTabInfo};
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
mod admin_input;
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set_static};
use crate::lua_bridge::table_set_rust_fn_static;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) use admin_input::register_admin;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
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
    register_skill_line_queries(state, namespace)?;
    register_tab_queries(state, namespace)
}

fn register_skill_line_queries(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
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
        "GetSpecTabIDsForSkillLine",
        get_spec_tab_ids,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "SkillLineHasSpecialization",
        skill_line_has_specialization,
    )
}

fn register_tab_queries(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "GetDefaultSpecSkillLine",
        get_default_skill_line,
    )?;
    table_set_rust_fn_static(state, namespace, "GetSpecTabInfo", get_spec_tab_info)?;
    table_set_rust_fn_static(state, namespace, "GetTabInfo", get_tab_info)?;
    table_set_rust_fn_static(state, namespace, "ShouldShowSpecTab", should_show_spec_tab)
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
    let tab_info = supplied_tab_info(state, tab_id)?;
    let value = tab_info.map_or(Val::Nil, |info| build_tab_info(state, info));
    state.push(value);
    Ok(1)
}

fn supplied_tab_info(state: &LuaState, tab_id: i32) -> LuaResult<Option<ProfessionSpecTabInfo>> {
    let Ok(tab_id) = u32::try_from(tab_id) else {
        return Ok(None);
    };
    Ok(borrow_state(state)?
        .profession_spec_inputs
        .tab_infos
        .get(&tab_id)
        .cloned())
}

fn build_tab_info(state: &mut LuaState, tab_info: ProfessionSpecTabInfo) -> Val {
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
    let highlights = build_tab_highlights(state, tab_info.highlights);
    table_set_static(state, table, "highlights", highlights);
    table
}

fn build_tab_highlights(state: &mut LuaState, descriptions: Vec<String>) -> Val {
    let Val::Table(highlights) = create_table(state) else {
        unreachable!("create_table returns a table");
    };
    for (index, description) in descriptions.into_iter().enumerate() {
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
    Val::Table(highlights)
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
