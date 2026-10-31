//! Supplied campaign chapter associations.

use crate::lua_api::methods::{borrow_state, create_table};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let ns = super::ensure_namespace(state, "C_CampaignInfo")?;
    table_set_rust_fn_static(state, ns, "GetChapterIDs", get_chapter_ids)
}

fn get_chapter_ids(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let chapters = borrow_state(state)?
        .quest_log_entries
        .campaign_chapter_ids
        .get(&id)
        .cloned();
    let Some(chapters) = chapters else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let result = create_table(state);
    for (index, chapter) in chapters.iter().enumerate() {
        super::helpers::set_table_array(
            state,
            result,
            (index + 1) as i64,
            Val::Num(f64::from(*chapter)),
        );
    }
    state.push(result);
    Ok(1)
}
