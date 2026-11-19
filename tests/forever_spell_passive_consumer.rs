#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn forever_action_button_spell_status_uses_passive_and_bar_inputs() {
    let env = WowLuaEnv::new().expect("create Forever Lua environment");
    crate::common::load_game_startup_inventory(&env);
    env.sync_addon_names_to_lua();
    env.apply_post_load_workarounds();

    assert_eq!(
        wow_ui_sim::spells::get_spell(131).map(|spell| spell.name),
        Some("Water Breathing")
    );
    assert_eq!(
        wow_ui_sim::spells::get_spell(467).map(|spell| spell.name),
        Some("Thorns")
    );
    assert_eq!(
        wow_ui_sim::spells::get_spell(782).map(|spell| spell.name),
        Some("Thorns")
    );

    {
        let mut state = env.state().borrow_mut();
        state.spell_passive_attribute_masks.clear();
        state.spell_passive_attribute_masks.insert(131, 64);
        state.action_bars.clear();
        state.action_bars.insert(1, 467);
        state.ranged_auto_attack_spell_id = 467;
    }

    env.exec(
        r#"
        local status = ActionButtonUtil.GetActionBarStatusForSpell
        local actionStatus = ActionButtonUtil.ActionBarActionStatus
        assert(status(131) == actionStatus.NotMissing,
            'seeded passive spell must not be missing')
        assert(status(467) == actionStatus.NotMissing,
            'spell on an action bar must not be missing')
        assert(status(782) == actionStatus.MissingFromAllBars,
            'known non-passive spell off all bars must be missing')
        assert(C_Spell.IsSpellPassive(131), 'seeded passive mask is honored')
        assert(not C_Spell.IsSpellPassive(0), 'ID 0 is not passive')
        assert(C_Spell.IsRangedAutoAttackSpell(467))
        assert(not C_Spell.IsRangedAutoAttackSpell(782))
        "#,
    )
    .expect("original Forever ActionButtonUtil consumer returns spell status");
}
