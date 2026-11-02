//! `C_PetInfo.GetPetTalentTree` (WM49 part B, Retail 12.1.5 build 70688).
//!
//! The image selects the ChrSpecialization route (selector 1); the alternate
//! TalentTab route is not taken. Assignments come from 70391 data.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::SimState;
use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
use rilua::vm::state::LuaState;
use sim_data::pet_talent_trees::{FAMILY_TALENT_TYPES, PET_SPECIALIZATIONS, PetSpecialization};

/// INFERRED (WM49-B): class discriminator 3 is the Hunter ChrClasses ID.
const HUNTER_CLASS_ID: i32 = 3;
/// ChrSpecialization flag that removes a row from selection.
const EXCLUDED_SPECIALIZATION_FLAG: u32 = 32;
/// OrderIndex whose selection publishes an empty name.
const NAMELESS_ORDER_INDEX: i32 = 4;
/// `UnitSex` value of native sex 1 (female).
const FEMALE_UNIT_SEX: i32 = 3;

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr) {
        return Ok(());
    }
    // PetInfoDocumentation.lua:11 in Retail and PTR.
    let namespace = super::ensure_namespace(state, "C_PetInfo")?;
    table_set_rust_fn_static(state, namespace, "GetPetTalentTree", get_pet_talent_tree)
}

/// EMULATED (WM49-B): failed selection publishes zero values.
fn get_pet_talent_tree(state: &mut LuaState) -> LuaResult<u32> {
    let Some(name) = select_pet_talent_tree(&*borrow_state(state)?) else {
        return Ok(0);
    };
    let name = create_string(state, name);
    state.push(name);
    Ok(1)
}

/// The summoned pet stands for the first stored pet identity; its owner is
/// the player (INFERRED, WM49-B).
fn select_pet_talent_tree(sim: &SimState) -> Option<&'static str> {
    if sim.pet.icon.is_none() || sim.player.class_index != HUNTER_CLASS_ID {
        return None;
    }
    let talent_type = family_talent_type(sim.pet.creature_family_id?)?;
    let specialization = find_specialization(PET_SPECIALIZATIONS, talent_type)?;
    // INFERRED (WM49-B): the sex is the owner player's.
    Some(specialization_name(specialization, sim.player.sex))
}

/// A missing family or a negative talent index fails selection.
fn family_talent_type(family_id: u32) -> Option<i8> {
    let index = FAMILY_TALENT_TYPES
        .binary_search_by_key(&family_id, |(id, _)| *id)
        .ok()?;
    let talent_type = FAMILY_TALENT_TYPES[index].1;
    (talent_type >= 0).then_some(talent_type)
}

/// First keyed-range-0 row of the talent type that lacks flag 32.
fn find_specialization(
    rows: &'static [PetSpecialization],
    talent_type: i8,
) -> Option<&'static PetSpecialization> {
    rows.iter().find(|row| {
        row.pet_talent_type == talent_type && row.flags & EXCLUDED_SPECIALIZATION_FLAG == 0
    })
}

fn specialization_name(specialization: &'static PetSpecialization, unit_sex: i32) -> &'static str {
    if specialization.order_index == NAMELESS_ORDER_INDEX {
        ""
    } else if unit_sex == FEMALE_UNIT_SEX {
        specialization.female_name
    } else {
        specialization.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn row(id: u32, order_index: i32, flags: u32) -> PetSpecialization {
        PetSpecialization {
            id,
            pet_talent_type: 1,
            order_index,
            flags,
            name: "Tenacity",
            female_name: "Tenacite",
        }
    }

    static FLAGGED_FIRST: [PetSpecialization; 2] = [row(537, 1, 32), row(81, 1, 0)];
    static NAMELESS: PetSpecialization = row(90, 4, 0);

    #[test]
    fn flag_32_rows_are_skipped_even_when_listed_first() {
        let selected = find_specialization(&FLAGGED_FIRST, 1).map(|row| row.id);
        assert_eq!(selected, Some(81));
        assert!(find_specialization(&FLAGGED_FIRST, 2).is_none());
    }

    #[test]
    fn sex_selects_the_female_name_and_order_index_4_publishes_empty_text() {
        assert_eq!(specialization_name(&FLAGGED_FIRST[1], 2), "Tenacity");
        assert_eq!(specialization_name(&FLAGGED_FIRST[1], 3), "Tenacite");
        assert_eq!(specialization_name(&NAMELESS, 3), "");
    }
}
