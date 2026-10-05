//! UI situation metadata from host catalog, with live pending/saved option values.

use super::model::{SituationCategory, SituationGroup, SituationKey};
use crate::lua_api::methods::{
    borrow_state, create_string, create_table, table_set, table_set_num,
};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn get_categories(state: &mut LuaState) -> LuaResult<u32> {
    let categories = borrow_state(state)?
        .transmog_outfits
        .situation_categories
        .clone();
    let array = create_table(state);
    let Val::Table(table) = array else {
        unreachable!()
    };
    for (index, category) in categories.iter().enumerate() {
        let row = build_category(state, category)?;
        table_set_num(state, table, (index + 1) as f64, row);
    }
    state.push(array);
    Ok(1)
}

fn build_category(state: &mut LuaState, category: &SituationCategory) -> LuaResult<Val> {
    let row = create_table(state);
    let name = create_string(state, &category.name);
    let description = create_string(state, &category.description);
    let groups = create_table(state);
    let Val::Table(table) = groups else {
        unreachable!()
    };
    for (index, group) in category.groups.iter().enumerate() {
        let group = build_group(state, group)?;
        table_set_num(state, table, (index + 1) as f64, group);
    }
    table_set(
        state,
        row,
        "triggerID",
        Val::Num(category.trigger_id as f64),
    );
    table_set(state, row, "name", name);
    table_set(state, row, "description", description);
    table_set(
        state,
        row,
        "isRadioButton",
        Val::Bool(category.is_radio_button),
    );
    table_set(state, row, "groupData", groups);
    Ok(row)
}

fn build_group(state: &mut LuaState, group: &SituationGroup) -> LuaResult<Val> {
    let row = create_table(state);
    let options = create_table(state);
    let Val::Table(table) = options else {
        unreachable!()
    };
    for (index, (name, key)) in group.options.iter().enumerate() {
        let option = build_option(state, name, *key)?;
        table_set_num(state, table, (index + 1) as f64, option);
    }
    table_set(state, row, "groupID", Val::Num(group.group_id as f64));
    table_set(
        state,
        row,
        "secondaryID",
        Val::Num(group.secondary_id as f64),
    );
    table_set(state, row, "optionData", options);
    Ok(row)
}

fn build_option(state: &mut LuaState, name: &str, key: SituationKey) -> LuaResult<Val> {
    let value = super::situations::read_option_value(state, key)?;
    let row = create_table(state);
    let option = create_table(state);
    let name = create_string(state, name);
    for (field, value) in [
        ("situationID", key.0),
        ("specID", key.1),
        ("loadoutID", key.2),
        ("equipmentSetID", key.3),
    ] {
        table_set(state, option, field, Val::Num(value as f64));
    }
    table_set(state, row, "name", name);
    table_set(state, row, "value", Val::Bool(value));
    table_set(state, row, "option", option);
    Ok(row)
}
