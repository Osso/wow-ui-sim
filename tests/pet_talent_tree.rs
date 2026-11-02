//! API-level: no Blizzard UI file calls `C_PetInfo.GetPetTalentTree`.
//! Selection follows WM49 part B; assignments are ChrSpecialization 70391.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const PALADIN: i32 = 2;
const HUNTER: i32 = 3;
const UNIT_SEX_FEMALE: i32 = 3;

fn pet_env(class_index: i32, family: Option<u32>) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create pet environment");
    {
        let mut state = env.state().borrow_mut();
        state.player.class_index = class_index;
        state.pet.icon = Some("Interface\\Icons\\Ability_Hunter_Pet_Wolf".into());
        state.pet.creature_family_id = family;
    }
    env
}

fn talent_tree(env: &WowLuaEnv) -> (i32, Option<String>) {
    env.eval("return select('#', C_PetInfo.GetPetTalentTree()), C_PetInfo.GetPetTalentTree()")
        .expect("GetPetTalentTree is callable")
}

#[test]
fn hunter_pet_family_selects_its_pet_specialization_name() {
    // CreatureFamily 70391: Wolf 1 -> type 0, Bear 4 -> type 1, Boar 5 -> type 2.
    for (family, name) in [(1, "Ferocity"), (4, "Tenacity"), (5, "Cunning")] {
        let env = pet_env(HUNTER, Some(family));
        assert_eq!(talent_tree(&env), (1, Some(name.into())), "family {family}");
    }
}

#[test]
fn female_owner_reads_the_female_name_column() {
    let env = pet_env(HUNTER, Some(4));
    env.state().borrow_mut().player.sex = UNIT_SEX_FEMALE;
    // Both 70391 name columns of row 81 read "Tenacity".
    assert_eq!(talent_tree(&env), (1, Some("Tenacity".into())));
}

#[test]
fn failed_selection_returns_zero_values() {
    let no_pet = WowLuaEnv::new().expect("create default environment");
    no_pet.state().borrow_mut().player.class_index = HUNTER;
    assert_eq!(talent_tree(&no_pet), (0, None), "no summoned pet");

    // Felhunter 15 has PetTalentType -1; family 10 has no CreatureFamily row.
    for family in [None, Some(15), Some(10)] {
        let env = pet_env(HUNTER, family);
        assert_eq!(talent_tree(&env), (0, None), "family {family:?}");
    }

    let paladin = pet_env(PALADIN, Some(1));
    assert_eq!(talent_tree(&paladin), (0, None), "owner is not class 3");
}
