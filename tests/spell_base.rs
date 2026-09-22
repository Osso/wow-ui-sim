#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn default_action_spells_keep_identity_without_modeled_overrides() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, spell in ipairs({19750, 31935, 275779, 26573, 53600, 85673,
                               62124, 853, 375576, 31850, 86659, 642}) do
            local base = C_Spell.GetBaseSpell(spell)
            assert(base == spell, 'missing base identity for '..spell)
            local included = {[base] = true}
            assert(included[spell])
        end
        assert(C_Spell.GetBaseSpell('Flash of Light') == 19750)
        assert(C_Spell.GetBaseSpell('flash of light') == 19750)
        assert(C_Spell.GetBaseSpell('19750') == 19750)
        "#,
    )
    .unwrap();
}

#[test]
fn configured_relations_follow_explicit_and_current_specialization() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.player.class_index = 2;
        state.player.active_spec_index = 2;
        // Test-owned relationships, not claims about live Paladin overrides.
        state.base_spell_relationships.set(66, 19750, 642);
        state.base_spell_relationships.set(70, 19750, 853);
        state.spell_id_aliases.insert("19750".into(), 26573);
    }
    env.exec(
        r#"
        assert(C_Spell.GetBaseSpell(19750) == 642)
        assert(C_Spell.GetBaseSpell('Flash of Light', 0) == 642)
        assert(C_Spell.GetBaseSpell(19750, 66) == 642)
        assert(C_Spell.GetBaseSpell(19750, 70) == 853)
        assert(C_Spell.GetBaseSpell(19750, 65) == 19750)
        assert(C_Spell.GetBaseSpell(642, 66) == 642)
        "#,
    )
    .unwrap();
    env.state().borrow_mut().player.active_spec_index = 3;
    assert_eq!(
        env.eval::<i64>("return C_Spell.GetBaseSpell(19750)")
            .unwrap(),
        853
    );
}

#[test]
fn rejects_invalid_identifiers_and_secret_arguments_explicitly() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, value in ipairs({0, -1, 1.5, math.huge, 4294967296, false, {}, 'no such spell'}) do
            assert(not pcall(C_Spell.GetBaseSpell, value))
        end
        assert(not pcall(C_Spell.GetBaseSpell, nil))
        for _, spec in ipairs({-1, 1.5, math.huge, false, {}, '66'}) do
            assert(not pcall(C_Spell.GetBaseSpell, 19750, spec))
        end
        local secret = secretwrap(19750)
        assert(issecretvalue(secret))
        assert(not pcall(C_Spell.GetBaseSpell, secret))
        assert(not pcall(C_Spell.GetBaseSpell, 19750, secretwrap(66)))
        local function addon_call()
            assert(C_Spell.GetBaseSpell(19750) == 19750)
            local ok, err = pcall(C_Spell.GetBaseSpell, secret)
            assert(not ok and err:find('secret arguments are not modeled', 1, true))
        end
        debug.setobjecttaint(addon_call, 'BaseSpellProbe')
        addon_call()
        "#,
    )
    .unwrap();
}
