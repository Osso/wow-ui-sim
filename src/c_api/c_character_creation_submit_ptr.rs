//! PTR R78 required arguments and neutral, host-only call observations.
//! No request serialization, outcomes, events, or inferred reader conversions.

use crate::c_api::helpers::bad_argument;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, table_set_static};
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
    /// R78c #282 VERIFIED byte write; no Lua reader or inferred gameplay effect.
    pub interaction_state: u8,
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
    update_camera_zoom_registration(state, namespace, is_glue)?;
    update_inferred_registration(state, namespace, is_glue)
}

/// R83 VERIFIED Glue registration, stored-state source, and no argument reads.
fn update_camera_zoom_registration(
    state: &mut LuaState,
    namespace: GcRef<Table>,
    is_glue: bool,
) -> LuaResult<()> {
    if is_glue {
        table_set_rust_fn_static(
            state,
            namespace,
            "GetCurrentCameraZoom",
            get_current_camera_zoom,
        )
    } else {
        table_set_static(
            state,
            Val::Table(namespace),
            "GetCurrentCameraZoom",
            Val::Nil,
        );
        Ok(())
    }
}

fn get_current_camera_zoom(state: &mut LuaState) -> LuaResult<u32> {
    let _zoom = borrow_state(state)?.character_create_camera_zoom;
    // VERIFIED-only scaffold: single-number Lua return bridge is INFERRED (R83)
    // and is deliberately deferred to its separate droppable commit.
    Ok(0)
}

// Existence INFERRED from unguarded consumer calls; catalog UNKNOWN (R78c Q1).
// Kept together in a droppable commit, separate from verified Glue registrations.
fn update_inferred_registration(
    state: &mut LuaState,
    namespace: GcRef<Table>,
    is_glue: bool,
) -> LuaResult<()> {
    if is_glue {
        table_set_rust_fn_static(
            state,
            namespace,
            "OnPlayerInteraction",
            on_player_interaction,
        )?;
        table_set_rust_fn_static(
            state,
            namespace,
            "SetPlayerModelHiddenState",
            set_player_model_hidden_state,
        )?;
    } else {
        table_set_static(
            state,
            Val::Table(namespace),
            "OnPlayerInteraction",
            Val::Nil,
        );
        table_set_static(
            state,
            Val::Table(namespace),
            "SetPlayerModelHiddenState",
            Val::Nil,
        );
    }
    Ok(())
}

fn on_player_interaction(state: &mut LuaState) -> LuaResult<u32> {
    // VERIFIED #282: internal byte is written to one. No Lua reader is known.
    borrow_state_mut(state)?
        .ptr_character_creation
        .interaction_state = 1;
    // Ordinary zero-result count remains INFERRED (#261), never asserted.
    Ok(0)
}

fn set_player_model_hidden_state(state: &mut LuaState) -> LuaResult<u32> {
    let argument = stack_val(state, 1);
    if matches!(argument, Val::Nil) {
        return Err(bad_argument(
            1,
            "SetPlayerModelHiddenState",
            "Usage: C_CharacterCreation.SetPlayerModelHiddenState(hiddenState)",
        ));
    }
    let _hidden = !matches!(argument, Val::Bool(false));
    // VERIFIED #282: without an owner/model nothing is written and no event is
    // emitted. This simulator has no character-create model; do not invent one.
    // Return count UNKNOWN: use no returns, with no count assertion.
    Ok(0)
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
