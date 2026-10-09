//! Publication/absence only: no signature, output, security, or behavior parity claim.
#![cfg(feature = "client-retail")]

#[path = "common/publication_sweep.rs"]
mod sweep;

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn patch_3_0_8_publication_sweep(env: &WowLuaEnv) {
    let register = include_str!("../data/patch-api/sources/3.0.8-wikitext-register.json");
    sweep::run_publication_sweep(env, &sweep::SweepSpec {
        register,
        known_gaps: include_str!("data/patch_3_0_8_sweep_known_gaps.json"),
        row_count: serde_json::from_str::<serde_json::Value>(register).unwrap()["entries"]
            .as_array().unwrap().len(),
        register_env: "P308_SWEEP_REGISTER",
        out_env: "P308_SWEEP_OUT",
        later_registers: &[
            // Pending retail 3.3.0 / 3.2.0 / 3.1.0: main adds oldest-first at integration.
            include_str!("../data/patch-api/sources/3.3.3-wikitext-register.json"),
            include_str!("../data/patch-api/sources/3.3.5-wikitext-register.json"),
            include_str!("../data/patch-api/sources/4.0.1-wikitext-register.json"),
            include_str!("../data/patch-api/sources/4.1.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/4.2.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/4.3.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/4.3.4-wikitext-register.json"),
            include_str!("../data/patch-api/sources/5.0.1-wikitext-register.json"),
            include_str!("../data/patch-api/sources/5.0.4-wikitext-register.json"),
            include_str!("../data/patch-api/sources/5.1.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/5.2.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/5.3.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/5.4.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/5.4.1-wikitext-register.json"),
            include_str!("../data/patch-api/sources/5.4.2-wikitext-register.json"),
            include_str!("../data/patch-api/sources/5.4.7-wikitext-register.json"),
            include_str!("../data/patch-api/sources/5.4.8-wikitext-register.json"),
            include_str!("../data/patch-api/sources/6.0.1-wikitext-register.json"),
            include_str!("../data/patch-api/sources/6.0.2-wikitext-register.json"),
            include_str!("../data/patch-api/sources/6.1.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/6.2.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/6.2.2-wikitext-register.json"),
            include_str!("../data/patch-api/sources/6.2.4-wikitext-register.json"),
            include_str!("../data/patch-api/sources/7.0.1-wikitext-register.json"),
            include_str!("../data/patch-api/sources/7.0.3-wikitext-register.json"),
            include_str!("../data/patch-api/sources/7.1.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/7.2.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/7.2.5-wikitext-register.json"),
            include_str!("../data/patch-api/sources/7.3.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/7.3.2-wikitext-register.json"),
            include_str!("../data/patch-api/sources/8.0.1-wikitext-register.json"),
            include_str!("../data/patch-api/sources/8.1.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/8.1.5-wikitext-register.json"),
            include_str!("../data/patch-api/sources/8.2.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/8.2.5-wikitext-register.json"),
            include_str!("../data/patch-api/sources/8.3.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/8.3.7-wikitext-register.json"),
            include_str!("../data/patch-api/sources/9.0.1-wikitext-register.json"),
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

prefork_full_ui_case! {
fn patch_3_0_8_hook_without_script(env: &WowLuaEnv) {
    env.eval::<()>(r#"
        local f = CreateFrame('Frame')
        assert(f:GetScript('OnShow') == nil)
        local seen = {}
        f:HookScript('OnShow', function(self) seen[#seen + 1] = self end)
        f:Hide()
        f:Show()
        assert(#seen == 1 and seen[1] == f)
        f:Hide()
        f:Show()
        assert(#seen == 2 and seen[2] == f)
    "#).unwrap();
}
}
