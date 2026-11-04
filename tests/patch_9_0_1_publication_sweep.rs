//! Publication/absence only: no signature, output, security, or behavior parity claim.
#![cfg(feature = "client-retail")]

#[path = "common/publication_sweep.rs"]
mod sweep;

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn lfg_expand_prompt_accept_clears_supplied_state(env: &WowLuaEnv) {
    for (key, data) in [(19, vec![1, 2]), (42, vec![3, 4, 5])] {
        env.state().borrow_mut().lfg_expand_prompt = Some((key, data));
        env.exec("assert(type(StaticPopupDialogs.LFG_QUEUE_EXPAND.OnAccept) == 'function'); StaticPopupDialogs.LFG_QUEUE_EXPAND.OnAccept()")
            .expect("original LFG expand popup accept callback");
        assert_eq!(env.state().borrow().lfg_expand_prompt, None);
    }
}
}

#[cfg(feature = "optional-tests")]
prefork_full_ui_case! {
fn patch_9_0_1_publication_sweep(env: &WowLuaEnv) {
    sweep::run_publication_sweep(env, &sweep::SweepSpec {
        register: include_str!("../data/patch-api/sources/9.0.1-wikitext-register.json"),
        known_gaps: include_str!("data/patch_9_0_1_sweep_known_gaps.json"),
        row_count: 755,
        register_env: "P901_SWEEP_REGISTER",
        out_env: "P901_SWEEP_OUT",
        later_registers: &[
            include_str!("../data/patch-api/sources/9.0.2-wikitext-register.json"),
            include_str!("../data/patch-api/sources/9.0.5-wikitext-register.json"),
            include_str!("../data/patch-api/sources/9.1.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/9.1.5-wikitext-register.json"),
            include_str!("../data/patch-api/sources/9.2.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/9.2.5-wikitext-register.json"),
            include_str!("../data/patch-api/sources/9.2.7-wikitext-register.json"),
            include_str!("../data/patch-api/sources/10.0.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/10.0.2-wikitext-register.json"),
            include_str!("../data/patch-api/sources/10.0.5-wikitext-register.json"),
            include_str!("../data/patch-api/sources/10.0.7-wikitext-register.json"),
            include_str!("../data/patch-api/sources/10.1.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/10.1.5-wikitext-register.json"),
            include_str!("../data/patch-api/sources/10.1.7-wikitext-register.json"),
            include_str!("../data/patch-api/sources/10.2.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/10.2.5-wikitext-register.json"),
            include_str!("../data/patch-api/sources/10.2.6-wikitext-register.json"),
            include_str!("../data/patch-api/sources/10.2.7-wikitext-register.json"),
            include_str!("../data/patch-api/sources/11.0.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/11.0.2-wikitext-register.json"),
            include_str!("../data/patch-api/sources/11.0.5-wikitext-register.json"),
            include_str!("../data/patch-api/sources/11.0.7-wikitext-register.json"),
            include_str!("../data/patch-api/sources/11.1.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/11.1.5-wikitext-register.json"),
            include_str!("../data/patch-api/sources/11.1.7-wikitext-register.json"),
            include_str!("../data/patch-api/sources/11.2.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/11.2.5-wikitext-register.json"),
            include_str!("../data/patch-api/sources/11.2.7-wikitext-register.json"),
            include_str!("../data/patch-api/sources/12.0.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/12.0.1-wikitext-register.json"),
            include_str!("../data/patch-api/sources/12.0.5-wikitext-register.json"),
            include_str!("../data/patch-api/sources/12.0.7-wikitext-register.json"),
            include_str!("../data/patch-api/sources/12.1.0-wikitext-register.json"),
        ],
    });
}
}
