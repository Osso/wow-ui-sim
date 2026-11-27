//! Retail 12.1.5 `C_ProfSpecs.GetChildrenForPath` (R88as).

use crate::lua_api::methods::{borrow_state, create_table, table_set_num};
use crate::lua_bridge::table_set_rust_fn_static;
use crate::screen::ScreenKind;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const USAGE: &str = "Usage: local childIDs = C_ProfSpecs.GetChildrenForPath(pathID)";

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if borrow_state(state)?.screen_kind != ScreenKind::Game {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_ProfSpecs")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetChildrenForPath",
        get_children_for_path,
    )
}

fn read_path_id(state: &LuaState) -> LuaResult<i32> {
    super::helpers::read_untainted_signed32_argument(state, 1, "GetChildrenForPath", USAGE)
}

fn get_children_for_path(state: &mut LuaState) -> LuaResult<u32> {
    let path_id = read_path_id(state)?;
    let children = borrow_state(state)?
        .profession_spec_inputs
        .children_for_path(path_id);
    push_children_table(state, children)
}

fn push_children_table(state: &mut LuaState, children: Vec<i32>) -> LuaResult<u32> {
    let Val::Table(table) = create_table(state) else {
        unreachable!("create_table returns a table");
    };
    for (index, child_id) in children.into_iter().enumerate() {
        table_set_num(
            state,
            table,
            (index + 1) as f64,
            Val::Num(f64::from(child_id)),
        );
    }
    state.push(Val::Table(table));
    Ok(1)
}
