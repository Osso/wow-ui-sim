//! Publication/absence only: no signature, output, security, or behavior parity claim.
#![cfg(feature = "client-retail")]

#[path = "common/publication_sweep.rs"]
mod sweep;

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn inspected_spec_cache_is_read_by_player_spells(env: &WowLuaEnv) {
    env.exec("assert(C_AddOns.LoadAddOn('Blizzard_PlayerSpells'))").unwrap();
    let guid: String = env.eval("return UnitGUID('player')").unwrap();
    for spec in [65, 66] {
        env.state().borrow_mut().inspect_specialization_cache = Some((guid.clone(), spec));
        let actual: i32 = env.eval(r#"
            local observer = {
                IsInspecting = function() return true end,
                GetInspectUnit = function() return 'player' end,
            }
            return PlayerSpellsFrameMixin.GetSpecID(observer)
        "#).unwrap();
        assert_eq!(actual, spec as i32);
    }
    env.state().borrow_mut().inspect_specialization_cache = Some(("Player-9-9".into(), 65));
    let mismatch: i32 = env.eval(r#"
        local observer = {
            IsInspecting = function() return true end,
            GetInspectUnit = function() return 'player' end,
        }
        return PlayerSpellsFrameMixin.GetSpecID(observer)
    "#).unwrap();
    assert_eq!(mismatch, 0);
    env.exec("NotifyInspect('player')").unwrap();
    assert!(env.state().borrow().inspect_specialization_cache.is_none());
}
}

prefork_full_ui_case! {
fn inspect_specialization_deprecated_alias_matches_original_publisher(env: &WowLuaEnv) {
    env.exec(r#"
        assert(C_AddOns.IsAddOnLoaded('Blizzard_Deprecated'), 'publisher is not loaded')
        assert(GetCVarBool('loadDeprecationFallbacks'), 'deprecation fallback is disabled')
        assert(rawequal(GetInspectSpecialization, C_SpecializationInfo.GetInspectSpecialization),
            'Blizzard deprecated alias differs from C_SpecializationInfo.GetInspectSpecialization')
        restoreSentinels = {
            spec = C_SpecializationInfo.GetInspectSpecialization,
            token = C_WowTokenSecure.GetTokenCount,
            store = C_StoreSecure.GetProductGroups,
        }
    "#).unwrap();
    env.restore_post_cleanup_globals();
    env.exec(r#"
        assert(rawequal(restoreSentinels.spec, C_SpecializationInfo.GetInspectSpecialization))
        assert(rawequal(restoreSentinels.token, C_WowTokenSecure.GetTokenCount))
        assert(rawequal(restoreSentinels.store, C_StoreSecure.GetProductGroups))
        assert(rawequal(GetInspectSpecialization, C_SpecializationInfo.GetInspectSpecialization))
    "#).unwrap();
}
}

prefork_full_ui_case! {
fn patch_12_1_0_publication_sweep(env: &WowLuaEnv) {
    sweep::run_publication_sweep(env, &sweep::SweepSpec {
        register: include_str!("../data/patch-api/sources/12.1.0-wikitext-register.json"),
        known_gaps: include_str!("data/patch_12_1_0_sweep_known_gaps.json"),
        row_count: 778,
        register_env: "P1210_SWEEP_REGISTER",
        out_env: "P1210_SWEEP_OUT",
        later_registers: &[],
    });
}
}
