//! Bounded pet-type state reads, not a pet-battle engine or native 2012 parity.
use wow_ui_sim::lua_api::WowLuaEnv;

fn assert_pet_type_reads(env: &WowLuaEnv) {
    {
        let mut sim = env.state().borrow_mut();
        sim.pet_battles.player_pets[0].pet_type = 7;
        sim.pet_battles.enemy_pets[0].pet_type = 9;
    }
    let types: (Option<i32>, Option<i32>) = env
        .eval("return C_PetBattles.GetPetType(1, 1), C_PetBattles.GetPetType(2, 1)")
        .unwrap();
    assert_eq!(types, (Some(7), Some(9)));
    env.state().borrow_mut().pet_battles.player_pets[0].pet_type = 8;
    let changed: Option<i32> = env.eval("return C_PetBattles.GetPetType(1, 1)").unwrap();
    assert_eq!(changed, Some(8));
    env.state().borrow_mut().pet_battles.player_pets.clear();
    let missing: (Option<i32>, Option<i32>, Option<i32>) = env
        .eval("return C_PetBattles.GetPetType(1, 1), C_PetBattles.GetPetType(2, 0), C_PetBattles.GetPetType(3, 1)")
        .unwrap();
    assert_eq!(missing, (None, None, None));
}

#[test]
fn patch_5_0_4_pet_type_reads_bare_state() {
    let env = WowLuaEnv::new().unwrap();
    assert_pet_type_reads(&env);
}

prefork_full_ui_case! {
fn patch_5_0_4_pet_type_reads_cached_state(env: &WowLuaEnv) {
    assert_pet_type_reads(env);
}
}
