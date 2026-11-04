use super::{create_identified_tooltip, empty_tooltip, push_tooltip_line, table_get_int};
use crate::lua_api::methods::{create_table, table_get, table_set, val_to_string};
use rilua::{Lua, LuaApiMut, Val};

#[test]
fn tooltip_builder_creates_fresh_tables_and_incrementing_instances() {
    let mut lua = Lua::new().unwrap();
    let state = lua.state_mut();
    let first = create_identified_tooltip(state, 1.0, 42);
    state.push(first);
    let second = create_identified_tooltip(state, 1.0, 42);
    state.push(second);
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
    let no_identity = create_identified_tooltip(state, 1.0, 0);
    assert_eq!(table_get(state, no_identity, "id"), Val::Nil);
}

#[test]
fn tooltip_builder_copies_markup_and_publishes_packed_rgb() {
    let mut lua = Lua::new().unwrap();
    let state = lua.state_mut();
    let tooltip = empty_tooltip(state, 1.0);
    state.push(tooltip);
    let lines = table_get(state, tooltip, "lines");
    push_tooltip_line(
        state,
        lines,
        1,
        0.0,
        "|cffff0000Red|r",
        Some((128.0 / 255.0, 64.0 / 255.0, 1.0)),
        false,
    );
    let line = table_get_int(state, lines, 1);
    assert_eq!(
        val_to_string(state, table_get(state, line, "leftText")).as_deref(),
        Some("|cffff0000Red|r")
    );
    assert_eq!(table_get(state, line, "leftColorSegments"), Val::Nil);
    let color = table_get(state, line, "leftColor");
    assert_eq!(
        table_get(state, color, "r"),
        Val::Num(f64::from(128.0_f32 / 255.0))
    );
    assert_eq!(
        table_get(state, color, "g"),
        Val::Num(f64::from(64.0_f32 / 255.0))
    );
    assert_eq!(table_get(state, color, "b"), Val::Num(1.0));
    assert_eq!(table_get(state, color, "a"), Val::Nil);
    assert_eq!(table_get(state, line, "rightColor"), Val::Nil);
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
