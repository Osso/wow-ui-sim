use super::{
    create_identified_tooltip, empty_tooltip, push_tooltip_line, stamp_data_instance, table_get_int,
};
use crate::lua_api::methods::{create_table, table_get, table_set};
use rilua::{Lua, LuaApiMut, Val};

#[test]
fn tooltip_builder_creates_fresh_tables_and_incrementing_instances() {
    let mut lua = Lua::new().unwrap();
    let state = lua.state_mut();
    let first = create_identified_tooltip(state, 1.0, 42);
    state.push(first);
    stamp_data_instance(state, first);
    let second = create_identified_tooltip(state, 1.0, 42);
    state.push(second);
    stamp_data_instance(state, second);
    let first_lines = table_get(state, first, "lines");
    let second_lines = table_get(state, second, "lines");
    assert_ne!(first, second);
    assert_ne!(first_lines, second_lines);
    let Val::Num(first_id) = table_get(state, first, "dataInstanceID") else {
        panic!("first tooltip has no dataInstanceID");
    };
    let Val::Num(second_id) = table_get(state, second, "dataInstanceID") else {
        panic!("second tooltip has no dataInstanceID");
    };
    assert_eq!(second_id, first_id + 1.0);
    // Miss shape is unchanged: WM82 leaves miss publication UNKNOWN.
    let miss = create_identified_tooltip(state, 1.0, 0);
    assert_eq!(table_get(state, miss, "id"), Val::Num(0.0));
    assert_eq!(table_get(state, miss, "dataInstanceID"), Val::Nil);
}

#[test]
fn tooltip_builder_remaps_error_and_disabled_line_types() {
    let mut lua = Lua::new().unwrap();
    let state = lua.state_mut();
    let globals = Val::Table(state.global);
    for (name, color) in [
        ("ERROR_COLOR", (1.0, 0.0, 0.0)),
        ("DISABLED_FONT_COLOR", (0.5, 0.5, 0.5)),
    ] {
        let value = create_table(state);
        table_set(state, globals, name, value);
        table_set(state, value, "r", Val::Num(color.0));
        table_set(state, value, "g", Val::Num(color.1));
        table_set(state, value, "b", Val::Num(color.2));
    }
    let tooltip = empty_tooltip(state, 1.0);
    state.push(tooltip);
    let lines = table_get(state, tooltip, "lines");
    for (index, color, expected) in [
        (1, (1.0, 0.0, 0.0), 41.0),
        (2, (0.5, 0.5, 0.5), 42.0),
        (3, (0.0, 1.0, 0.0), 0.0),
    ] {
        push_tooltip_line(state, lines, index, 0.0, "text", Some(color), false);
        let line = table_get_int(state, lines, index as i32);
        assert_eq!(table_get(state, line, "type"), Val::Num(expected));
    }
}
