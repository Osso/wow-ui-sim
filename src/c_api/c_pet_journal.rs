//! State-backed C_PetJournal queries used by legacy pet-journal consumers.
// C_PetJournal.GetOwnedPetIDs: PetJournalInfoDocumentation.lua:107.

use crate::lua_api::methods::{borrow_state, val_to_string};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::LuaResult;
use rilua::vm::state::LuaState;

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_PetJournal")?;
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        table_set_rust_fn_static(state, namespace, "GetOwnedPetIDs", get_owned_pet_ids)?;
    }
    table_set_rust_fn_static(
        state,
        namespace,
        "IsCurrentlySummoned",
        is_currently_summoned,
    )
}

fn is_currently_summoned(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let pet_guid = val_to_string(state, value)
        .ok_or_else(|| rilua::runtime_error("C_PetJournal.IsCurrentlySummoned requires a petID"))?;
    let is_summoned =
        borrow_state(state)?.pet.summoned_battle_pet_guid.as_deref() == Some(pet_guid.as_str());
    state.push(rilua::Val::Bool(is_summoned));
    Ok(1)
}

fn get_owned_pet_ids(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_api::methods::{create_string, create_table};
    let ids = borrow_state(state)?
        .world
        .pets
        .iter()
        .filter(|pet| pet.is_collected)
        .map(|pet| pet.pet_id.clone())
        .collect::<Vec<_>>();
    let result = create_table(state);
    for (index, id) in ids.iter().enumerate() {
        let guid = create_string(state, id);
        super::helpers::set_table_array(state, result, index as i64 + 1, guid);
    }
    state.push(result);
    Ok(1)
}

#[cfg(all(test, any(feature = "client-retail", feature = "client-wowforever")))]
#[test]
fn owned_pet_ids_follow_collection_membership() {
    crate::lua_api::WowLuaEnv::new()
        .unwrap()
        .exec(
            r#"
        local total = C_PetJournal.GetNumPets()
        local wanted
        for i=1,total do
            local id, species = C_PetJournal.GetPetInfoByIndex(i)
            if species == 2403 then wanted = id end
        end
        assert(wanted)
        local function hasId(ids)
            for _, id in ipairs(ids) do if id == wanted then return true end end
            return false
        end
        A_Admin.UncollectPet(2403)
        local initial = #C_PetJournal.GetOwnedPetIDs()
        A_Admin.CollectPet(2403)
        local ids = C_PetJournal.GetOwnedPetIDs()
        assert(#ids == initial+1 and hasId(ids))
        A_Admin.UncollectPet(2403)
        ids = C_PetJournal.GetOwnedPetIDs()
        assert(#ids == initial and not hasId(ids))
    "#,
        )
        .unwrap();
}
