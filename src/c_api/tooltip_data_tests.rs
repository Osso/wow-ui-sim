use super::{assign_identity, publish_rgb, publish_side, stamp_instance};
use crate::lua_api::methods::{create_table, table_get, table_set};
use rilua::{Lua, LuaApiMut, Val};

#[test]
fn tooltip_d56_nested_identity_retains_primary_and_order() {
    let mut lua = Lua::new().unwrap();
    let state = lua.state_mut();
    let tooltip = create_table(state);
    state.push(tooltip);
    let lines = create_table(state);
    table_set(state, tooltip, "lines", lines);
    let color = create_table(state);
    state.push(color);
    publish_rgb(state, color, (0.25, 0.75, 0.5));
    assign_identity(state, tooltip, 0.0, 6948, color);
    stamp_instance(state, tooltip);
    assign_identity(state, tooltip, 1.0, 19750, color);
    assign_identity(state, tooltip, 2.0, 0, color);
    assert_eq!(table_get(state, tooltip, "type"), Val::Num(0.0));
    assert_eq!(table_get(state, tooltip, "id"), Val::Num(6948.0));
    let Val::Table(lines_ref) = lines else {
        unreachable!()
    };
    for (index, kind, id) in [(1, 1.0, Val::Num(19750.0)), (2, 2.0, Val::Nil)] {
        let line = state.gc.tables.get(lines_ref).unwrap().get_int(index);
        assert_eq!(table_get(state, line, "type"), Val::Num(19.0));
        assert_eq!(table_get(state, line, "tooltipType"), Val::Num(kind));
        assert_eq!(table_get(state, line, "tooltipID"), id);
        assert_eq!(table_get(state, line, "leftColor"), color);
        let text = table_get(state, line, "leftText");
        let empty = crate::lua_api::methods::val_to_string(state, text);
        assert_eq!(empty.as_deref(), Some(""));
        assert_eq!(table_get(state, line, "rightColor"), Val::Nil);
    }
}

#[test]
fn tooltip_d56_supplied_empty_and_absent_sides_are_distinct() {
    let mut lua = Lua::new().unwrap();
    let state = lua.state_mut();
    let color = create_table(state);
    state.push(color);
    table_set(state, color, "a", Val::Num(1.0));
    publish_rgb(state, color, (64.0 / 255.0, 128.0 / 255.0, 192.0 / 255.0));
    assert_eq!(table_get(state, color, "a"), Val::Nil);
    assert_eq!(
        table_get(state, color, "r"),
        Val::Num(f64::from(64.0_f32 * (1.0_f32 / 255.0)))
    );
    let line = create_table(state);
    state.push(line);
    publish_side(state, line, false, None, color);
    publish_side(state, line, true, Some(""), color);
    assert_eq!(table_get(state, line, "leftText"), Val::Nil);
    assert_eq!(table_get(state, line, "leftColor"), Val::Nil);
    assert_eq!(table_get(state, line, "rightColor"), color);
    let text = table_get(state, line, "rightText");
    assert_eq!(
        crate::lua_api::methods::val_to_string(state, text).as_deref(),
        Some("")
    );
}
