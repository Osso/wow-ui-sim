use wow_ui_sim::c_api::charge_state::SpellChargeState;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn test_spell_override_maw_epoch_and_explicit_charge_state() {
    let env = WowLuaEnv::new().expect("WowLuaEnv init");
    env.state().borrow_mut().spell_charges.insert(
        19750,
        SpellChargeState {
            current_charges: 1,
            max_charges: 2,
            recharge_start: 12.0,
            recharge_duration: 40.0,
            charge_mod_rate: 2.0,
        },
    );
    // data/patch-api/sources/12.0.7-wikitext-register.json records this removal.
    let maw_expression = if cfg!(feature = "retail-12-0-7") {
        "C_Spell.GetMawPowerBorderAtlasBySpellID == nil"
    } else {
        // Preserve the earlier-profile fixture baseline, not a native contract claim.
        "C_Spell.GetMawPowerBorderAtlasBySpellID(116) == nil"
    };
    let (override_id, maw_atlas_is_nil, current, max, start, duration, mod_rate): (
        i64,
        bool,
        i64,
        i64,
        i64,
        i64,
        f64,
    ) = env
        .eval(&format!(
            "local charges = C_Spell.GetSpellCharges(19750)
             return C_Spell.GetOverrideSpell(116),
                    {maw_expression},
                    charges.currentCharges,
                    charges.maxCharges,
                    charges.cooldownStartTime,
                    charges.cooldownDuration,
                    charges.chargeModRate",
        ))
        .unwrap();

    assert_eq!(override_id, 116);
    assert!(maw_atlas_is_nil);
    assert_eq!(current, 1);
    assert_eq!(max, 2);
    assert_eq!(start, 12);
    assert_eq!(duration, 40);
    assert!((mod_rate - 2.0).abs() < 0.001);
}
