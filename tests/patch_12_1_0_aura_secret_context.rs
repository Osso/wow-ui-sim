//! 12.1.0 aura-secret context: while auras are secret, addon (tainted) callers
//! cannot reach aura data by index, slot or instance ID, spell-keyed queries
//! stay callable, and AuraData and UNIT_AURA payloads are secret.
#![cfg(feature = "retail-12-1-0")]

use wow_ui_sim::lua_api::WowLuaEnv;

/// Mark of the Wild is flagged never-secret in the 12.1.0 spell data.
const NEVER_SECRET_SPELL: i32 = 1126;
const CONTEXTUAL_SPELL: i32 = 9072;

fn seeded_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        let template = state.player.buffs[0].clone();
        state.player.buffs = [
            (71, NEVER_SECRET_SPELL, "Wild Mark", true),
            (72, CONTEXTUAL_SPELL, "Hidden Hex", false),
        ]
        .into_iter()
        .map(|(id, spell, name, helpful)| {
            let mut aura = template.clone();
            aura.aura_instance_id = id;
            aura.spell_id = spell;
            aura.name = name.into();
            aura.is_helpful = helpful;
            aura.is_from_player_or_player_pet = helpful;
            aura.dispel_type = Some("Magic".into());
            aura
        })
        .collect();
    }
    env.exec(
        r#"
        DispelCurve = C_CurveUtil.CreateColorCurve()
        DispelCurve:AddPoint(0, CreateColor(0, 0, 0, 1))
        DispelCurve:AddPoint(1, CreateColor(0.2, 0.6, 1, 1))
        AccessCalls = {
            function() return C_UnitAuras.GetAuraDataByAuraInstanceID('player', 71) end,
            function() return C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL') end,
            function() return C_UnitAuras.GetBuffDataByIndex('player', 1) end,
            function() return C_UnitAuras.GetDebuffDataByIndex('player', 1) end,
            function() return C_UnitAuras.GetAuraDataBySlot('player', 71) end,
            function() return C_UnitAuras.GetAuraSlots('player', 'HELPFUL') end,
            function() return C_UnitAuras.GetUnitAuraInstanceIDs('player', 'HELPFUL') end,
            function() return C_UnitAuras.GetUnitAuras('player', 'HELPFUL') end,
            function() return C_UnitAuras.IsAuraFilteredOutByInstanceID('player', 71, 'HELPFUL') end,
            function() return C_UnitAuras.GetAuraDuration('player', 71) end,
            function() return C_UnitAuras.DoesAuraHaveExpirationTime('player', 71) end,
            function() return C_UnitAuras.GetAuraBaseDuration('player', 71) end,
            function() return C_UnitAuras.GetRefreshExtendedDuration('player', 71) end,
            function() return C_UnitAuras.GetAuraApplicationDisplayCount('player', 71) end,
            function() return C_UnitAuras.GetAuraDispelTypeColor('player', 71, DispelCurve) end,
            function() return C_UnitAuras.CancelAuraByInstanceID('player', 71) end,
            function() return C_TooltipInfo.GetUnitAura('player', 1, 'HELPFUL') end,
            function() return C_TooltipInfo.GetUnitBuff('player', 1) end,
            function() return C_TooltipInfo.GetUnitDebuff('player', 1) end,
            function() return C_TooltipInfo.GetUnitAuraByAuraInstanceID('player', 71) end,
            function() return C_TooltipInfo.GetUnitBuffByAuraInstanceID('player', 71) end,
            function() return C_TooltipInfo.GetUnitDebuffByAuraInstanceID('player', 72) end,
        }
        function AddonProbe(body)
            local function probe()
                body()
                assert(debug.getstacktaint() == 'AuraSecretProbe', 'caller keeps its taint')
            end
            debug.setobjecttaint(probe, 'AuraSecretProbe')
            probe()
        end
        "#,
    )
    .unwrap();
    env
}

fn set_restricted(env: &WowLuaEnv, restricted: bool) {
    env.state().borrow_mut().unit_auras_restricted = restricted;
}

#[test]
fn addon_index_slot_and_instance_access_errors_only_while_auras_are_secret() {
    let env = seeded_env();
    env.exec(
        r#"
        AddonProbe(function()
            for index, call in ipairs(AccessCalls) do
                if index ~= 16 then
                    local ok, err = pcall(call)
                    assert(ok, 'public before restriction: ' .. index .. ' ' .. tostring(err))
                end
            end
            local data = C_UnitAuras.GetAuraDataByAuraInstanceID('player', 72)
            assert(not issecretvalue(data) and data.name == 'Hidden Hex')
        end)
        "#,
    )
    .unwrap();
    set_restricted(&env, true);
    env.exec(
        r#"
        AddonProbe(function()
            for index, call in ipairs(AccessCalls) do
                local ok, err = pcall(call)
                assert(not ok, 'restricted call succeeded: ' .. index)
                assert(err:find('cannot be accessed by addons while auras are secret', 1, true), err)
            end
        end)
        assert(C_UnitAuras.GetAuraDataByAuraInstanceID('player', 71) ~= nil, 'secure code keeps access')
        assert(#C_UnitAuras.GetUnitAuraInstanceIDs('player', 'HELPFUL') == 1)
        "#,
    )
    .unwrap();
    assert_eq!(
        env.state().borrow().player.buffs.len(),
        2,
        "rejected cancel left the aura in place"
    );
}

#[test]
fn spell_keyed_queries_stay_callable_and_never_secret_spells_stay_plain() {
    let env = seeded_env();
    set_restricted(&env, true);
    env.exec(
        r#"
        AddonProbe(function()
            local mark = C_UnitAuras.GetPlayerAuraBySpellID(1126)
            assert(not issecretvalue(mark) and mark.auraInstanceID == 71)
            local byUnit = C_UnitAuras.GetUnitAuraBySpellID('player', 1126)
            assert(not issecretvalue(byUnit) and byUnit.name == 'Wild Mark')
            local byName = C_UnitAuras.GetAuraDataBySpellName('player', 'Wild Mark')
            assert(not issecretvalue(byName) and byName.spellId == 1126)
            assert(C_UnitAuras.GetUnitAuraBySpellID('player', 9072) == nil)
            assert(C_UnitAuras.GetAuraDataBySpellName('player', 'Hidden Hex') == nil)
        end)
        local hex = C_UnitAuras.GetUnitAuraBySpellID('player', 9072)
        assert(issecretvalue(hex) and hex.auraInstanceID == 72, 'secure caller gets secret data')
        assert(not issecretvalue(C_UnitAuras.GetPlayerAuraBySpellID(1126)))
        "#,
    )
    .unwrap();
}

#[test]
fn aura_data_is_fully_secret_while_auras_are_secret() {
    let env = seeded_env();
    set_restricted(&env, true);
    env.exec(
        r#"
        local byInstance = C_UnitAuras.GetAuraDataByAuraInstanceID('player', 71)
        local bySlot = C_UnitAuras.GetAuraDataBySlot('player', 72)
        local byIndex = C_UnitAuras.GetAuraDataByIndex('player', 1, 'HARMFUL')
        local list = C_UnitAuras.GetUnitAuras('player', 'HELPFUL')
        assert(not issecretvalue(list) and #list == 1)
        for _, data in ipairs({ byInstance, bySlot, byIndex, list[1] }) do
            assert(issecretvalue(data), 'AuraData must be secret')
        end
        assert(byInstance.name == 'Wild Mark' and bySlot.spellId == 9072)
        assert(byIndex.auraInstanceID == 72 and list[1].auraInstanceID == 71)
        Held = byInstance
        AddonProbe(function()
            assert(not pcall(function() return Held.name end), 'addon cannot read secret AuraData')
            assert(not canaccessvalue(Held))
        end)
        "#,
    )
    .unwrap();
    set_restricted(&env, false);
    env.exec(
        r#"
        local data = C_UnitAuras.GetAuraDataByAuraInstanceID('player', 71)
        assert(not issecretvalue(data) and data.name == 'Wild Mark')
        assert(issecretvalue(Held), 'earlier secret data is not declassified')
        "#,
    )
    .unwrap();
}

#[test]
fn unit_aura_payload_is_secret_while_auras_are_secret() {
    let env = seeded_env();
    env.exec(
        r#"
        Payloads = {}
        local frame = CreateFrame('Frame')
        frame:RegisterUnitEvent('UNIT_AURA', 'player')
        frame:SetScript('OnEvent', function(_, _, unit, info)
            Payloads[#Payloads + 1] = { unit = unit, info = info }
        end)
        C_UnitAuras.CancelAuraByInstanceID('player', 71)
        "#,
    )
    .unwrap();
    set_restricted(&env, true);
    env.state().borrow_mut().player.buffs[0].is_helpful = true;
    env.exec(
        r#"
        C_UnitAuras.CancelAuraByInstanceID('player', 72)
        assert(#Payloads == 2)
        local plain, secret = Payloads[1], Payloads[2]
        assert(not issecretvalue(plain.info) and plain.info.removedAuraInstanceIDs[1] == 71)
        assert(secret.unit == 'player', 'unit token stays routable')
        assert(issecretvalue(secret.info))
        assert(secret.info.removedAuraInstanceIDs[1] == 72, 'secure handlers read the payload')
        Held = secret.info
        AddonProbe(function()
            assert(not pcall(function() return Held.removedAuraInstanceIDs end))
        end)
        "#,
    )
    .unwrap();
}
