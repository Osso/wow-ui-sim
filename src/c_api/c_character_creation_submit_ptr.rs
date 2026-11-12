//! PTR R78 required arguments and neutral, host-only call observations.
//! No request serialization, outcomes, events, or inferred reader conversions.

use crate::c_api::helpers::bad_argument;
use crate::lua_api::methods::{borrow_state_mut, table_set_static};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

const USAGE: &str =
    "Usage: C_CharacterCreation.CreateCharacter(name, surname, useNPE [, raceFaction])";

/// Owned input observations, not the native reader's output or wire fields.
/// Opaque values retain no VM references and expose no Lua object to the host.
#[derive(Clone, Debug, PartialEq)]
pub enum ObservedArgument {
    String(Vec<u8>),
    Number(f64),
    Boolean(bool),
    Opaque,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CreateCharacterCall {
    pub name: ObservedArgument,
    pub surname: ObservedArgument,
    pub use_npe: ObservedArgument,
    pub race_faction: Option<ObservedArgument>,
}

#[derive(Debug, Default)]
pub struct PtrCharacterCreationState {
    pub last_call: Option<CreateCharacterCall>,
}

/// R78c #282 VERIFIED: these members belong to the PTR Glue registrar only.
pub(crate) fn update_registration(
    state: &mut LuaState,
    namespace: GcRef<Table>,
    is_glue: bool,
) -> LuaResult<()> {
    if is_glue {
        table_set_rust_fn_static(state, namespace, "CreateCharacter", create_character)?;
        table_set_rust_fn_static(
            state,
            namespace,
            "StopAllSpellVisualKitsOnCharacter",
            stop_all_spell_visual_kits,
        )?;
    } else {
        table_set_static(state, Val::Table(namespace), "CreateCharacter", Val::Nil);
        table_set_static(
            state,
            Val::Table(namespace),
            "StopAllSpellVisualKitsOnCharacter",
            Val::Nil,
        );
    }
    Ok(())
}

fn stop_all_spell_visual_kits(_state: &mut LuaState) -> LuaResult<u32> {
    // R78 PTR/#261: no model and no pending error produce no Lua-visible effect.
    // Do not invent a pending-error input or a model/visual-kit flag.
    // Ordinary zero-result count is INFERRED; no test asserts it.
    Ok(0)
}

fn observe(state: &LuaState, value: Val) -> ObservedArgument {
    match value {
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .map(|string| ObservedArgument::String(string.data().to_vec()))
            .unwrap_or(ObservedArgument::Opaque),
        Val::Num(number) => ObservedArgument::Number(number),
        Val::Bool(value) => ObservedArgument::Boolean(value),
        _ => ObservedArgument::Opaque,
    }
}

fn create_character(state: &mut LuaState) -> LuaResult<u32> {
    let arguments = [
        stack_val(state, 1),
        stack_val(state, 2),
        stack_val(state, 3),
    ];
    for (index, argument) in arguments.iter().enumerate() {
        if matches!(argument, Val::Nil) {
            return Err(bad_argument(index as i32 + 1, "CreateCharacter", USAGE));
        }
    }
    let faction = stack_val(state, 4);
    let race_faction = match faction {
        Val::Nil => None,
        Val::Str(_) | Val::Num(_) => Some(observe(state, faction)),
        // UNKNOWN accepted types: do not copy RF rejection or interpret them.
        // Coordinator #268 explicitly permits this opaque host-only marker.
        _ => Some(ObservedArgument::Opaque),
    };
    let call = CreateCharacterCall {
        name: observe(state, arguments[0]),
        surname: observe(state, arguments[1]),
        use_npe: observe(state, arguments[2]),
        race_faction,
    };
    borrow_state_mut(state)?.ptr_character_creation.last_call = Some(call);
    // Retain the existing simulator's no-return convention. An ordinary native
    // zero-result call is only INFERRED (#265); no test asserts this count.
    Ok(0)
}
