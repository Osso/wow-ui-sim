#![cfg(feature = "retail-12-0-5")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_12_0_1_comparison_query_matches_unit_comparison_boundary() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(C_Secrets.CanCompareUnitTokens('player', 'target'))
        assert(not C_Secrets.CanCompareUnitTokens('nameplate1', 'nameplate2'))
        assert(select('#', UnitIsUnit('nameplate1', 'nameplate2')) == 0)
        assert(C_Secrets.ShouldUnitThreatStateBeSecret('player', 'target') == false)
    "#).unwrap();
    env.state().borrow_mut().plain_global_inputs.threat_state_restricted = true;
    env.exec("assert(C_Secrets.ShouldUnitThreatStateBeSecret('player', 'target'))").unwrap();
}

#[test]
fn patch_12_0_1_host_policy_inputs_have_no_fabricated_enabled_defaults() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(C_Housing.IsHousingMarketCartFullRemoveEnabled() == false)
        assert(C_LFGList.IsPlayerValidForEndgameFieldEdits() == false)
        assert(C_LFGList.ListingUsesEndgameEditRestrictions(71) == false)
        assert(C_TransmogCollection.IsSpellItemEnchantmentHiddenVisual(91) == false)
        assert(C_PvP.GetArenaCrowdControlDuration('player'):GetTotalDuration() == 0)
    "#).unwrap();
    let guid: String = env.eval("return UnitGUID('player')").unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.housing.market_cart_full_remove_enabled = true;
        sim.lfg_endgame_policy.player_valid = true;
        sim.lfg_endgame_policy.restricted_activities.insert(71);
        sim.hidden_spell_item_enchantments.insert(91);
        sim.arena_crowd_control.insert(guid.clone(), wow_ui_sim::c_api::c_pvp_crowd_control::CrowdControlWindow {
            start_time: 0.0, duration: 3600.0,
        });
    }
    env.exec(r#"
        assert(C_Housing.IsHousingMarketCartFullRemoveEnabled())
        assert(C_LFGList.IsPlayerValidForEndgameFieldEdits())
        assert(C_LFGList.ListingUsesEndgameEditRestrictions(71))
        assert(not C_LFGList.ListingUsesEndgameEditRestrictions(72))
        assert(C_TransmogCollection.IsSpellItemEnchantmentHiddenVisual(91))
        assert(not C_TransmogCollection.IsSpellItemEnchantmentHiddenVisual(92))
        assert(C_PvP.GetArenaCrowdControlDuration('player'):GetTotalDuration() == 3600)
        assert(C_PvP.GetArenaCrowdControlDuration('unknown'):GetTotalDuration() == 0)
    "#).unwrap();
    env.state().borrow_mut().arena_crowd_control.get_mut(&guid).unwrap().start_time = -4000.0;
    env.exec("assert(C_PvP.GetArenaCrowdControlDuration('player'):GetTotalDuration() == 0)").unwrap();
}
