//! Read-only exterior snapshots; serializers leave one rooted result on the VM stack.

use crate::c_api::c_housing::exterior::{
    EXTERIOR_CUSTOMIZATION_MODE, ExteriorFixtureOption, ExteriorFixturePoint, ExteriorSizeOption,
    ExteriorTypeOption,
};
use crate::c_api::helpers::set_table_array;
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set_static};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    let functions: &[(&str, rilua::RustFn)] = &[
        ("GetCurrentHouseExteriorSize", current_size),
        ("GetCurrentHouseExteriorType", current_type),
        ("GetHouseExteriorSizeOptions", size_options),
        ("GetHouseExteriorTypeOptions", type_options),
        ("GetSelectedFixturePointInfo", selected_point_info),
        ("HasSelectedFixturePoint", has_selected_point),
        (
            "IsAnyDecorAttachedToHouseExterior",
            exterior_has_attachments,
        ),
        (
            "IsAnyDecorAttachedToSelectedFixturePoint",
            selected_point_has_attachments,
        ),
    ];
    let base = state.top;
    state.push(Val::Table(namespace));
    let result = functions.iter().try_for_each(|&(name, function)| {
        table_set_rust_fn_static(state, namespace, name, function)
    });
    state.top = base;
    result
}

fn current_size(state: &mut LuaState) -> LuaResult<u32> {
    let size = borrow_state(state)?.housing.exterior.selected_size;
    state.push(size.map_or(Val::Nil, |size| Val::Num(f64::from(size))));
    Ok(1)
}

fn current_type(state: &mut LuaState) -> LuaResult<u32> {
    let (id, name) = {
        let sim = borrow_state(state)?;
        let exterior = &sim.housing.exterior;
        let id = exterior.selected_type_id;
        let name = id.and_then(|id| {
            exterior
                .type_options
                .iter()
                .find(|option| option.id == id)
                .map(|option| option.name.clone())
        });
        (id, name)
    };
    state.push(id.map_or(Val::Nil, |id| Val::Num(f64::from(id))));
    let name = name.map_or(Val::Nil, |name| create_string(state, &name));
    state.push(name);
    Ok(2)
}

fn size_options(state: &mut LuaState) -> LuaResult<u32> {
    let snapshot = {
        let sim = borrow_state(state)?;
        let exterior = &sim.housing.exterior;
        exterior
            .selected_size
            .map(|size| (size, exterior.size_options.clone()))
    };
    match snapshot {
        Some((size, options)) => {
            let row = push_table(state);
            set_number(state, row, "selectedSize", f64::from(size));
            let options = push_sequence(state, &options, push_size_option);
            attach_child(state, row, "options", options);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn type_options(state: &mut LuaState) -> LuaResult<u32> {
    let snapshot = {
        let sim = borrow_state(state)?;
        let exterior = &sim.housing.exterior;
        exterior
            .selected_type_id
            .map(|id| (id, exterior.type_options.clone()))
    };
    match snapshot {
        Some((id, options)) => {
            let row = push_table(state);
            set_number(state, row, "selectedExteriorType", f64::from(id));
            let options = push_sequence(state, &options, push_type_option);
            attach_child(state, row, "options", options);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn selected_point_info(state: &mut LuaState) -> LuaResult<u32> {
    let point = borrow_state(state)?
        .housing
        .exterior
        .selected_fixture_point
        .clone();
    match point {
        Some(point) => {
            push_fixture_point(state, &point);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn has_selected_point(state: &mut LuaState) -> LuaResult<u32> {
    let selected = borrow_state(state)?
        .housing
        .exterior
        .selected_fixture_point
        .is_some();
    state.push(Val::Bool(selected));
    Ok(1)
}

fn exterior_has_attachments(state: &mut LuaState) -> LuaResult<u32> {
    let attached = {
        let sim = borrow_state(state)?;
        let housing = &sim.housing;
        housing.inside_owned_plot
            && housing.active_house_editor_mode == EXTERIOR_CUSTOMIZATION_MODE
            && housing
                .exterior
                .decor
                .values()
                .any(|decor| decor.fixture_point_owner_hash.is_some())
    };
    state.push(Val::Bool(attached));
    Ok(1)
}

fn selected_point_has_attachments(state: &mut LuaState) -> LuaResult<u32> {
    let attached = {
        let sim = borrow_state(state)?;
        let exterior = &sim.housing.exterior;
        exterior
            .selected_fixture_point
            .as_ref()
            .is_some_and(|point| {
                point.selected_fixture_id.is_some()
                    && exterior
                        .decor
                        .values()
                        .any(|decor| decor.fixture_point_owner_hash == Some(point.owner_hash))
            })
    };
    state.push(Val::Bool(attached));
    Ok(1)
}

fn push_table(state: &mut LuaState) -> Val {
    let table = create_table(state);
    state.push(table);
    table
}

fn attach_child(state: &mut LuaState, parent: Val, field: &'static str, child: Val) {
    table_set_static(state, parent, field, child);
    state.top -= 1;
}

fn set_number(state: &mut LuaState, table: Val, field: &'static str, number: f64) {
    table_set_static(state, table, field, Val::Num(number));
}

fn set_text(state: &mut LuaState, table: Val, field: &'static str, text: &str) {
    let value = create_string(state, text);
    table_set_static(state, table, field, value);
}

fn push_sequence<T>(
    state: &mut LuaState,
    records: &[T],
    push_row: fn(&mut LuaState, &T) -> Val,
) -> Val {
    let sequence = push_table(state);
    for (index, record) in records.iter().enumerate() {
        let row = push_row(state, record);
        set_table_array(state, sequence, (index + 1) as i64, row);
        state.top -= 1;
    }
    sequence
}

fn push_size_option(state: &mut LuaState, option: &ExteriorSizeOption) -> Val {
    let row = push_table(state);
    set_number(state, row, "size", f64::from(option.size));
    set_text(state, row, "name", &option.name);
    table_set_static(state, row, "isLocked", Val::Bool(option.is_locked));
    row
}

fn push_type_option(state: &mut LuaState, option: &ExteriorTypeOption) -> Val {
    let row = push_table(state);
    set_number(state, row, "houseExteriorTypeID", f64::from(option.id));
    set_text(state, row, "name", &option.name);
    table_set_static(state, row, "isLocked", Val::Bool(option.is_locked));
    table_set_static(state, row, "isInvalid", Val::Bool(option.is_invalid));
    set_text(state, row, "reasonString", &option.reason);
    row
}

fn push_fixture_point(state: &mut LuaState, point: &ExteriorFixturePoint) -> Val {
    let row = push_table(state);
    set_number(state, row, "ownerHash", f64::from(point.owner_hash));
    if let Some(id) = point.selected_fixture_id {
        set_number(state, row, "selectedFixtureID", f64::from(id));
    }
    table_set_static(
        state,
        row,
        "canSelectionBeRemoved",
        Val::Bool(point.can_remove && point.selected_fixture_id.is_some()),
    );
    let options = push_sequence(state, &point.options, push_fixture_option);
    attach_child(state, row, "fixtureOptions", options);
    row
}

fn push_fixture_option(state: &mut LuaState, option: &ExteriorFixtureOption) -> Val {
    let row = push_table(state);
    for (field, value) in [
        ("fixtureID", option.id),
        ("typeID", option.type_id),
        ("colorID", option.color_id),
    ] {
        set_number(state, row, field, f64::from(value));
    }
    set_text(state, row, "name", &option.name);
    set_text(state, row, "typeName", &option.type_name);
    table_set_static(state, row, "isLocked", Val::Bool(option.is_locked));
    table_set_static(state, row, "isInvalid", Val::Bool(option.is_invalid));
    set_text(state, row, "reasonString", &option.reason);
    row
}
