//! INFERRED (R88ef): Forever reincarnation fields supplied by the glue flow.
//! Native evidence is pending; the selected consumers establish the four fields.

use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Default)]
pub(crate) struct ReincarnationState {
    active: bool,
    character: Option<ReincarnatingCharacter>,
}

struct ReincarnatingCharacter {
    guid: String,
    name: String,
    class_name: String,
    level: u32,
}

impl ReincarnationState {
    fn start(&mut self, character: ReincarnatingCharacter) {
        self.active = true;
        self.character = Some(character);
    }
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_Reincarnation")?;
    table_set_rust_fn_static(state, namespace, "StartReincarnation", start_reincarnation)?;
    table_set_rust_fn_static(state, namespace, "GetReincarnatingCharacter", get_character)?;
    table_set_rust_fn_static(state, namespace, "IsReincarnating", is_reincarnating)
}

fn start_reincarnation(state: &mut LuaState) -> LuaResult<u32> {
    let character = read_character(state)?;
    borrow_state_mut(state)?.reincarnation.start(character);
    Ok(0)
}

fn read_character(state: &LuaState) -> LuaResult<ReincarnatingCharacter> {
    Ok(ReincarnatingCharacter {
        guid: String::from_stack(state, 1)?,
        name: String::from_stack(state, 2)?,
        class_name: String::from_stack(state, 3)?,
        level: u32::from_stack(state, 4)?,
    })
}

fn get_character(state: &mut LuaState) -> LuaResult<u32> {
    let character = borrow_state(state)?
        .reincarnation
        .character
        .as_ref()
        .map(|c| {
            (
                c.guid.clone(),
                c.name.clone(),
                c.class_name.clone(),
                c.level,
            )
        });
    let Some((guid, name, class_name, level)) = character else {
        for _ in 0..4 {
            state.push(Val::Nil);
        }
        return Ok(4);
    };
    let guid = create_string(state, &guid);
    state.push(guid);
    let name = create_string(state, &name);
    state.push(name);
    let class_name = create_string(state, &class_name);
    state.push(class_name);
    state.push(Val::Num(f64::from(level)));
    Ok(4)
}

fn is_reincarnating(state: &mut LuaState) -> LuaResult<u32> {
    let active = borrow_state(state)?.reincarnation.active;
    state.push(Val::Bool(active));
    Ok(1)
}
