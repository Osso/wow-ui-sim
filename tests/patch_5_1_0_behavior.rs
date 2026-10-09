//! Current retail backing and consumer-free absence; not 2012 gameplay parity.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::LossOfControlInfo;

fn assert_pet_id_absence(env: &WowLuaEnv) {
    env.exec(
        r#"
        for _, name in ipairs({'GetSummonedPetID', 'SummonPetByID'}) do
            assert(rawget(C_PetJournal, name) == nil)
            assert(C_PetJournal[name] == nil)
            assert(C_PetJournal[name] == nil)
        end
        assert(type(C_PetJournal.GetSummonedPetGUID) == 'function')
        assert(type(C_PetJournal.SummonPetByGUID) == 'function')
        assert(type(C_PetJournal.GetNumPets) == 'function')
        "#,
    )
    .expect("retired ID lookups cannot synthesize functions; GUID successors stay published");
}

#[test]
fn patch_5_1_0_bare_pet_id_absence() {
    let env = WowLuaEnv::new().unwrap();
    assert_pet_id_absence(&env);
}

prefork_full_ui_case! {
fn patch_5_1_0_loaded_pet_id_absence(env: &WowLuaEnv) {
    assert_pet_id_absence(env);
}
}

prefork_full_ui_case! {
fn patch_5_1_0_existing_loss_control_backing(env: &WowLuaEnv) {
    {
        let mut state = env.state().borrow_mut();
        state.cooldowns_restricted = false;
        state.action_bars.insert(17, 19750);
        state.spell_loss_of_control.insert(19750, LossOfControlInfo {
            start_time: 12.5,
            duration: 6.25,
            mod_rate: 1.0,
            is_active: true,
            should_replace_normal_cooldown: true,
        });
    }
    let timing: (f64, f64) = env.eval("return GetActionLossOfControlCooldown(17)")
        .expect("cached legacy wrapper reads assigned spell interval");
    assert_eq!(timing, (12.5, 6.25));
    env.state().borrow_mut().spell_loss_of_control.get_mut(&19750).unwrap().duration = 3.75;
    let updated: (f64, f64) = env.eval("return GetActionLossOfControlCooldown(17)")
        .expect("replacement interval is visible through legacy wrapper");
    assert_eq!(updated, (12.5, 3.75));
    env.state().borrow_mut().spell_loss_of_control.remove(&19750);
    let cleared: (f64, f64) = env.eval("return GetActionLossOfControlCooldown(17)")
        .expect("cleared spell uses inactive interval");
    assert_eq!(cleared, (0.0, 0.0));
}
}
