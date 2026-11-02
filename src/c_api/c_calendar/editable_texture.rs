//! Local texture selection on the supplied editable Calendar event.
use super::{editable_access::can_edit, editable_date::read_index};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
use rilua::vm::state::LuaState;

pub(super) fn register(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "EventSetTextureID", set_texture)
}

fn set_texture(state: &mut LuaState) -> LuaResult<u32> {
    let index = read_index(
        state,
        1,
        "Usage: C_Calendar.EventSetTextureID(textureIndex)",
    )?;
    let selected = {
        let host = borrow_state(state)?;
        if can_edit(&host) {
            let event = host
                .calendar_context_policy
                .editable
                .as_ref()
                .expect("editable event checked above");
            let list = match event.event_type {
                0 => Some(0),
                1 | 5 => Some(1),
                _ => None,
            };
            list.and_then(|list| {
                usize::try_from(index)
                    .ok()
                    .and_then(|index| host.calendar_texture_lists[list].get(index))
            })
            .map(|texture| texture.id)
        } else {
            None
        }
    };
    if let Some(id) = selected {
        let mut host = borrow_state_mut(state)?;
        let event = host
            .calendar_context_policy
            .editable
            .as_mut()
            .expect("editable event checked above");
        if event.texture_id != Some(id) {
            event.texture_id = Some(id);
            event.dirty = true;
        }
    }
    Ok(0)
}
