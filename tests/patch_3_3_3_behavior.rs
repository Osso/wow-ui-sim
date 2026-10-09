//! Existing pet scalar backing state at the full cached-retail load boundary.
//! No historical native or event-production parity claim.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn patch_3_3_3_pet_bonus_scalar_after_cached_ui(env: &WowLuaEnv) {
    env.state().borrow_mut().pet.spell_bonus_damage = Some(137.25);
    let (arity, bonus): (i32, f64) = env
        .eval("return select('#', GetPetSpellBonusDamage()), GetPetSpellBonusDamage()")
        .unwrap();
    assert_eq!(arity, 1);
    assert_eq!(bonus, 137.25);
    env.state().borrow_mut().pet.spell_bonus_damage = Some(58.5);
    assert_eq!(env.eval::<f64>("return GetPetSpellBonusDamage()").unwrap(), 58.5);
}
}
