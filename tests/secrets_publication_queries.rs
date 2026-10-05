//! C_Secrets queries describe the simulator's existing secrecy policies.
#![cfg(all(feature = "retail-12-0-0", feature = "profile-retail"))]

use rilua::LuaApiMut;
use wow_ui_sim::lua_api::WowLuaEnv;

fn probe_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        function AddonProbe(body)
            local function run()
                body()
                assert(debug.getstacktaint() == 'SecretsProbe')
            end
            debug.setobjecttaint(run, 'SecretsProbe')
            run()
        end
        "#,
    ).unwrap();
    env
}

#[test]
fn secrets_publication_cooldown_queries_agree_with_all_three_outputs() {
    let env = probe_env();
    env.state().borrow_mut().action_bars.insert(17, 19750);
    let restricted_epoch = cfg!(feature = "retail-12-0-5");
    for restricted in [false, true, false] {
        env.state().borrow_mut().cooldowns_restricted = restricted;
        let expected = restricted && restricted_epoch;
        env.exec(&format!(
            r#"
            local pairs = {{
                {{ function() return C_Secrets.ShouldSpellCooldownBeSecret(19750) end,
                   function() return C_Spell.GetSpellCooldown(19750).duration end }},
                {{ function() return C_Secrets.ShouldActionCooldownBeSecret(17) end,
                   function() return C_ActionBar.GetActionCooldown(17).duration end }},
                {{ function() return C_Secrets.ShouldSpellBookItemCooldownBeSecret(5, 0) end,
                   function() return C_SpellBook.GetSpellBookItemCooldown(5, 0).duration end }},
            }}
            AddonProbe(function()
                for _, pair in ipairs(pairs) do
                    local query = pair[1]()
                    assert(query == {expected})
                    assert(not issecretvalue(query))
                    assert(issecretvalue(pair[2]()) == query)
                end
            end)
            assert(not C_Secrets.ShouldSpellBookItemCooldownBeSecret(99999, 0))
            assert(not C_Secrets.ShouldSpellBookItemCooldownBeSecret(5, 1))
            assert(C_Secrets.GetSpellCooldownSecrecy(19750) == Enum.SecrecyLevel.ContextuallySecret)
            "#,
        )).unwrap();
    }
}

#[test]
#[cfg(feature = "retail-12-1-0")]
fn secrets_publication_aura_queries_agree_with_access_and_spell_exemptions() {
    let env = probe_env();
    {
        let mut state = env.state().borrow_mut();
        let mut aura = state.player.buffs[0].clone();
        aura.aura_instance_id = 71;
        aura.spell_id = 9072;
        aura.is_helpful = true;
        let mut public = aura.clone();
        public.aura_instance_id = 72;
        public.spell_id = 1126;
        state.player.buffs = vec![aura, public];
    }
    for restricted in [false, true, false] {
        env.state().borrow_mut().unit_auras_restricted = restricted;
        env.exec(&format!(
            r#"
            local pairs = {{
                {{ function() return C_Secrets.ShouldUnitAuraIndexBeSecret('player', 1, 'HELPFUL') end,
                   function() return C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL') end }},
                {{ function() return C_Secrets.ShouldUnitAuraInstanceBeSecret('player', 71) end,
                   function() return C_UnitAuras.GetAuraDataByAuraInstanceID('player', 71) end }},
                {{ function() return C_Secrets.ShouldUnitAuraSlotBeSecret('player', 71) end,
                   function() return C_UnitAuras.GetAuraDataBySlot('player', 71) end }},
            }}
            assert(C_Secrets.ShouldAurasBeSecret() == {restricted})
            for _, pair in ipairs(pairs) do
                assert(pair[1]() == {restricted})
                assert(issecretvalue(pair[2]()) == pair[1]())
            end
            AddonProbe(function()
                for _, pair in ipairs(pairs) do
                    assert(not issecretvalue(pair[1]()))
                    assert(pcall(pair[2]) == not pair[1]())
                end
                assert(C_Secrets.ShouldSpellAuraBeSecret(9072) == {restricted})
                assert(not C_Secrets.ShouldSpellAuraBeSecret(1126))
                local public = C_UnitAuras.GetPlayerAuraBySpellID(1126)
                assert(not issecretvalue(public))
                assert((C_UnitAuras.GetPlayerAuraBySpellID(9072) == nil) == {restricted})
            end)
            assert(issecretvalue(C_UnitAuras.GetPlayerAuraBySpellID(9072)) == {restricted})
            "#,
        )).unwrap();
    }
}

#[test]
#[cfg(feature = "retail-12-0-5")]
fn secrets_publication_identity_reads_live_classification_and_exemptions() {
    let env = probe_env();
    env.state().borrow_mut().party_group_active = true;
    let guid: String = env.eval("return UnitGUID('party1')").unwrap();
    for restricted in [false, true, false] {
        {
            let mut state = env.state().borrow_mut();
            state.instance_identity.on_instanced_map = true;
            if restricted {
                state.identity_secret_guids.insert(guid.clone());
            } else {
                state.identity_secret_guids.remove(&guid);
            }
        }
        env.exec(&format!(
            r#"
            AddonProbe(function()
                local query = C_Secrets.ShouldUnitIdentityBeSecret('party1')
                assert(query == {restricted} and not issecretvalue(query))
                assert(issecretvalue(UnitGUID('party1')) == query)
                assert(issecretvalue(UnitFullName('party1')) == query)
                assert(not C_Secrets.ShouldUnitIdentityBeSecret('player'))
                assert(not C_Secrets.ShouldUnitIdentityBeSecret('absent'))
            end)
            "#,
        )).unwrap();
    }
}

#[test]
fn secrets_publication_unmodeled_aspects_match_public_existing_outputs() {
    let env = probe_env();
    // Unrelated restriction inputs must not invent missing power, health,
    // comparison, foreign-cast or active-totem secrecy policies.
    for restricted in [false, true, false] {
        {
            let mut state = env.state().borrow_mut();
            state.unit_stats_restricted = restricted;
            state.cooldowns_restricted = restricted;
            state.unit_auras_restricted = restricted;
            state.player.in_combat = restricted;
        }
        env.exec(
            r#"
            assert(C_Secrets.HasSecretRestrictions())
            assert(not issecretvalue(C_Secrets.HasSecretRestrictions()))
            local pairs = {
                { function() return C_Secrets.ShouldUnitHealthMaxBeSecret('player') end,
                  function() return UnitHealthMax('player') end },
                { function() return C_Secrets.ShouldUnitPowerBeSecret('player', 0) end,
                  function() return UnitPower('player', 0) end },
                { function() return C_Secrets.ShouldUnitPowerMaxBeSecret('player', 0) end,
                  function() return UnitPowerMax('player', 0) end },
                { function() return C_Secrets.ShouldUnitComparisonBeSecret('player', 'player') end,
                  function() return UnitIsUnit('player', 'player') end },
                { function() return C_Secrets.ShouldTotemSlotBeSecret(1) end,
                  function() return GetTotemInfo(1) end },
                { function() return C_Secrets.ShouldUnitSpellCastBeSecret('player', 19750) end,
                  function() return UnitCastingInfo('player') end },
            }
            AddonProbe(function()
                for _, pair in ipairs(pairs) do
                    local query = pair[1]()
                    assert(query == false and not issecretvalue(query))
                    assert(not issecretvalue(pair[2]()))
                end
                assert(not C_Secrets.ShouldTotemSpellBeSecret(19750))
            end)
            assert(C_Secrets.GetPowerTypeSecrecy(0) == Enum.SecrecyLevel.NeverSecret)
            assert(C_Secrets.GetSpellCastSecrecy(19750) == Enum.SecrecyLevel.NeverSecret)
            "#,
        ).unwrap();
    }
}

#[test]
fn secrets_publication_selector_authentication_preserves_caller_taint() {
    let env = probe_env();
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        let unit = rilua::table_security::wrap_host_secret_string(lua.state_mut(), "player");
        lua.set_global_val("SecretUnit", unit).unwrap();
        let number = rilua::table_security::wrap_host_secret_number(lua.state_mut(), 19750.0);
        lua.set_global_val("SecretNumber", number).unwrap();
    }
    env.exec(
        r#"
        local secretUnit = SecretUnit
        local secretNumber = SecretNumber
        local calls = {
            function() return C_Secrets.GetPowerTypeSecrecy(secretNumber) end,
            function() return C_Secrets.GetSpellCastSecrecy(secretNumber) end,
            function() return C_Secrets.GetSpellCooldownSecrecy(secretNumber) end,
            function() return C_Secrets.ShouldActionCooldownBeSecret(secretNumber) end,
            function() return C_Secrets.ShouldSpellAuraBeSecret(secretNumber) end,
            function() return C_Secrets.ShouldSpellBookItemCooldownBeSecret(5, secretNumber) end,
            function() return C_Secrets.ShouldSpellCooldownBeSecret(secretNumber) end,
            function() return C_Secrets.ShouldTotemSlotBeSecret(secretNumber) end,
            function() return C_Secrets.ShouldTotemSpellBeSecret(secretNumber) end,
            function() return C_Secrets.ShouldUnitAuraIndexBeSecret(secretUnit, 1) end,
            function() return C_Secrets.ShouldUnitAuraInstanceBeSecret(secretUnit, 71) end,
            function() return C_Secrets.ShouldUnitAuraSlotBeSecret(secretUnit, 71) end,
            function() return C_Secrets.ShouldUnitComparisonBeSecret('player', secretUnit) end,
            function() return C_Secrets.ShouldUnitHealthMaxBeSecret(secretUnit) end,
            function() return C_Secrets.ShouldUnitIdentityBeSecret(secretUnit) end,
            function() return C_Secrets.ShouldUnitPowerBeSecret('player', secretNumber) end,
            function() return C_Secrets.ShouldUnitPowerMaxBeSecret('player', secretNumber) end,
            function() return C_Secrets.ShouldUnitSpellCastBeSecret('player', secretNumber) end,
        }
        for index, call in ipairs(calls) do
            assert(pcall(call), 'secure selector ' .. index)
        end
        AddonProbe(function()
            for index, call in ipairs(calls) do
                assert(not pcall(call), 'tainted selector ' .. index)
            end
        end)
        "#,
    ).unwrap();
}
