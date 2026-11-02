//! Retail deprecated alias consumer for localized recap links.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn death_recap_link_alias_selects_localized_supplied_recaps(env: &WowLuaEnv) {
    use wow_ui_sim::lua_api::state::DeathRecapEntry;
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.death_recaps.clear();
        for (id, eligible) in [(11, true), (22, false)] {
            sim.death_recaps.push(DeathRecapEntry {
                recap_id: id,
                default_eligible: eligible,
                events: vec![],
                max_health: 100,
                zone_name: "fixture".into(),
                killing_blows: vec![],
            });
        }
    }
    env.exec(r#"
        assert(GetDeathRecapLink == C_DeathRecap.GetRecapLink)
        assert(select('#', GetDeathRecapLink()) == 1)
        assert(GetDeathRecapLink() == string.format(DEATH_RECAP_LINK, 11))
        assert(GetDeathRecapLink(nil) == string.format(DEATH_RECAP_LINK, 11))
        assert(GetDeathRecapLink('22.9') == string.format(DEATH_RECAP_LINK, 22))
        assert(GetDeathRecapLink(999) == DEATH_RECAP_TEXT)
        assert(GetDeathRecapLink(-1) == DEATH_RECAP_TEXT)
        for _, value in ipairs({false, {}, 'not a number', 2147483648, -2147483649}) do
            assert(not pcall(GetDeathRecapLink, value))
        end
    "#).unwrap();
    {
        let state = env.state();
        state.borrow_mut().death_recaps[0].default_eligible = false;
    }
    env.exec(r#"
        assert(GetDeathRecapLink() == DEATH_RECAP_TEXT)
        assert(GetDeathRecapLink(11) == string.format(DEATH_RECAP_LINK, 11))
    "#).unwrap();
}
}
