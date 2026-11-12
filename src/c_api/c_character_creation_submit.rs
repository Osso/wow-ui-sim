//! R78 character-creation inputs. Host delivery is separate from submission:
//! neither network serialization nor a roster refresh is inferred here.

use crate::c_api::helpers::bad_argument;
use crate::lua_api::methods::{borrow_state_mut, table_set_static};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

#[cfg(feature = "client-retail")]
const CREATE_USAGE: &str =
    "Usage: C_CharacterCreation.CreateCharacter(name, useNPE [, raceFaction])";
#[cfg(feature = "client-wowforever")]
const CREATE_USAGE: &str =
    "Usage: C_CharacterCreation.CreateCharacter(name, surname, useNPE [, raceFaction])";

/// Preserve optional faction input without inferring native conversion (#210).
#[derive(Clone, Debug, PartialEq)]
pub enum FactionArgument {
    String(Vec<u8>),
    Number(f64),
}

/// Simulator call record; native request fields and serialization UNKNOWN (R78 Q3).
/// Text retains Lua bytes. This is host-only and has no effect on result delivery.
#[derive(Clone, Debug, PartialEq)]
pub struct CreateCharacterCall {
    pub name: Vec<u8>,
    pub surname: Option<Vec<u8>>,
    pub use_npe: bool,
    pub race_faction: Option<FactionArgument>,
}

/// Observable native interaction/model state, not a rendered character model.
/// Defaults describe an empty simulator scenario, not native initialization.
#[derive(Debug, Default)]
pub struct CharacterCreationState {
    pub last_call: Option<CreateCharacterCall>,
    /// R78 Q8 writes one; the byte's readers and meaning remain unknown.
    pub interaction_state: u8,
    /// R78 Q9 clears this even without a model.
    pub visual_kit_active: bool,
    /// R78 Q9/#208: native qword is cleared with the flag.
    pub visual_kit_value: u64,
}

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
            "OnPlayerInteraction",
            on_player_interaction,
        )?;
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
            "OnPlayerInteraction",
            Val::Nil,
        );
        table_set_static(
            state,
            Val::Table(namespace),
            "StopAllSpellVisualKitsOnCharacter",
            Val::Nil,
        );
    }
    Ok(())
}

fn text_argument(state: &LuaState, index: i32) -> LuaResult<Vec<u8>> {
    let value = stack_val(state, index);
    match value {
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .map(|s| s.data().to_vec()),
        // Val's formatter uses the VM's Lua numeric-text representation.
        Val::Num(_) => Some(value.to_string().into_bytes()),
        _ => None,
    }
    .ok_or_else(|| bad_argument(index, "CreateCharacter", CREATE_USAGE))
}

fn create_character(state: &mut LuaState) -> LuaResult<u32> {
    let name = text_argument(state, 1)?;
    #[cfg(feature = "client-wowforever")]
    let (surname, use_npe_index) = (Some(text_argument(state, 2)?), 3);
    #[cfg(feature = "client-retail")]
    let (surname, use_npe_index) = (None, 2);
    let use_npe = !matches!(stack_val(state, use_npe_index), Val::Nil | Val::Bool(false));
    let faction_index = use_npe_index + 1;
    let race_faction = match stack_val(state, faction_index) {
        Val::Nil => None,
        Val::Str(_) => Some(FactionArgument::String(text_argument(
            state,
            faction_index,
        )?)),
        Val::Num(number) => Some(FactionArgument::Number(number)),
        _ => return Err(bad_argument(faction_index, "CreateCharacter", CREATE_USAGE)),
    };
    borrow_state_mut(state)?.character_creation.last_call = Some(CreateCharacterCall {
        name,
        surname,
        use_npe,
        race_faction,
    });
    // UNKNOWN native return count; simulator returns nothing (coordinator #208).
    // No outcome, roster/name mutation, event, or network request is inferred.
    Ok(0)
}

fn on_player_interaction(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?
        .character_creation
        .interaction_state = 1;
    Ok(0)
}

fn stop_all_spell_visual_kits(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    sim.character_creation.visual_kit_active = false;
    sim.character_creation.visual_kit_value = 0;
    // No modeled stop/finish helpers: their Lua-visible effects are UNKNOWN.
    Ok(0)
}
