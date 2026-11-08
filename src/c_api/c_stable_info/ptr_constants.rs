//! PTR 70493 PetConstantsDocumentation.lua:6–17, selected Standard StableUI inputs.

use crate::c_api::helpers::ensure_global_table;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{table_get, table_set};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE != ClientProfile::Ptr {
        return Ok(());
    }
    let constants = ensure_global_table(state, "Constants");
    let pets = table_get(state, constants, "PetConsts");
    if !matches!(pets, Val::Table(_)) {
        return Err(runtime_error(
            "PTR PetConsts requires the initialized Constants.PetConsts table",
        ));
    }
    for (key, value) in resolve_declared_values() {
        table_set(state, pets, key, Val::Num(f64::from(value)));
    }
    Ok(())
}

fn resolve_declared_values() -> [(&'static str, u32); 6] {
    // :11 is literal5; :12 aliases it; :14 aliases :12; :15 aliases :14.
    let learned_slots = 5_u32;
    let summonable_pets = learned_slots;
    let extra_slot = summonable_pets;
    let first_stabled_slot = extra_slot;
    // :13 is literal200; :16 adds :13 and :11.
    let stable_slots = 200_u32;
    let hunter_slots = stable_slots + learned_slots;
    [
        ("NUM_PET_SLOTS_THAT_NEED_LEARNED_SPELL", learned_slots),
        ("MAX_SUMMONABLE_HUNTER_PETS", summonable_pets),
        ("EXTRA_PET_STABLE_SLOT", extra_slot),
        ("STABLED_PETS_FIRST_SLOT_INDEX", first_stabled_slot),
        ("NUM_PET_SLOTS_HUNTER", hunter_slots),
        ("MAX_STABLE_SLOTS", stable_slots),
    ]
}
