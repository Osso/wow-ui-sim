use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn buff_caster_is_visible_to_aura_queries() {
    WowLuaEnv::new().unwrap().eval::<()>(r#"
        A_Admin.SetPartySize(1)
        A_Admin.SetPartyMember(1, 'Caster', 1, 80)
        A_Admin.ClearBuffs()
        A_Admin.AddBuff(21562, 'Test buff', 135987, 60, 1, 'party1')
        local aura = C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL')
        assert(aura.sourceUnit == 'party1', 'source unit')
        assert(aura.isFromPlayerOrPlayerPet == false, 'player ownership')
        local _, _, _, _, _, _, caster = UnitBuff('player', 1)
        assert(caster == 'party1', 'legacy caster')
        assert(C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL|PLAYER') == nil)
        A_Admin.ClearBuffs()
        A_Admin.AddBuff(21562, 'Test buff', 135987, 60, 1)
        aura = C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL')
        assert(aura.sourceUnit == 'player' and aura.isFromPlayerOrPlayerPet)
    "#).unwrap();
}

#[test]
fn replacing_seeded_party_member_clears_auras() {
    WowLuaEnv::new().unwrap().eval::<()>(r#"
        A_Admin.SetPartySize(2)
        local seen = false
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('UNIT_AURA')
        listener:SetScript('OnEvent', function(_, _, unit)
            if unit == 'party1' then
                seen = C_UnitAuras.GetAuraDataByIndex('party1', 1, 'HELPFUL') == nil
            end
        end)
        A_Admin.SetPartyMember(1, 'Replacement', 1, 80)
        assert(seen, 'replacement must dispatch UNIT_AURA after clearing')
        assert(C_UnitAuras.GetAuraDataByIndex('party1', 1, 'HELPFUL') == nil, 'old buff remains')
        assert(C_UnitAuras.GetAuraDataByIndex('party1', 1, 'HARMFUL') == nil, 'old debuff remains')
    "#).unwrap();
}

#[test]
fn admin_spell_membership_is_consistent() {
    WowLuaEnv::new().unwrap().eval::<()>(r#"
        local id = 9876543
        assert(not IsSpellKnown(id) and not IsPlayerSpell(id))
        assert(not C_SpellBook.IsSpellKnown(id))
        A_Admin.LearnSpell(id)
        assert(IsSpellKnown(id) and IsPlayerSpell(id), 'legacy spell queries')
        assert(C_SpellBook.IsSpellKnown(id), 'spellbook known')
        assert(C_SpellBook.IsSpellKnownOrInSpellBook(id), 'spellbook membership')
        A_Admin.UnlearnSpell(id)
        assert(not IsSpellKnown(id) and not IsPlayerSpell(id))
        assert(not C_SpellBook.IsSpellKnown(id))
        assert(not C_SpellBook.IsSpellKnownOrInSpellBook(id))
    "#).unwrap();
}

#[test]
fn admin_events_expose_new_state() {
    WowLuaEnv::new().unwrap().eval::<()>(r#"
        local events = {}
        local frame = CreateFrame('Frame')
        for _, e in ipairs({'PLAYER_REGEN_DISABLED','PLAYER_REGEN_ENABLED','PLAYER_TARGET_CHANGED','UNIT_MAXHEALTH','UNIT_HEALTH'}) do frame:RegisterEvent(e) end
        frame:SetScript('OnEvent', function(_, e, unit)
            events[#events+1] = {e, unit, unit and UnitHealth(unit), unit and UnitHealthMax(unit), UnitExists('target'), UnitAffectingCombat('player')}
        end)
        A_Admin.SetInCombat(true)
        assert(events[1][1] == 'PLAYER_REGEN_DISABLED' and events[1][6], 'combat event')
        A_Admin.SetInCombat(false)
        assert(events[2][1] == 'PLAYER_REGEN_ENABLED' and not events[2][6], 'combat exit')
        A_Admin.SetTarget('Enemy', 80, 1, true)
        assert(events[3][1] == 'PLAYER_TARGET_CHANGED' and events[3][5], 'target event')
        A_Admin.SetPlayerHealth(12, 123)
        assert(events[4][1] == 'UNIT_HEALTH' and events[4][2] == 'player' and events[4][3] == 12 and events[4][4] == 123)
        assert(events[5][1] == 'UNIT_MAXHEALTH' and events[5][2] == 'player')
        A_Admin.SetTargetHealth(23, 234)
        assert(events[6][1] == 'UNIT_HEALTH' and events[6][2] == 'target' and events[6][3] == 23)
        assert(events[7][1] == 'UNIT_MAXHEALTH' and events[7][2] == 'target')
        A_Admin.SetPartySize(1)
        A_Admin.SetPartyMember(1, 'Member', 1, 80)
        A_Admin.SetPartyMemberHealth(1, 34, 345)
        assert(events[8][1] == 'UNIT_HEALTH' and events[8][2] == 'party1' and events[8][3] == 34)
        assert(events[9][1] == 'UNIT_MAXHEALTH' and events[9][2] == 'party1')
        A_Admin.KillPartyMember(1)
        assert(events[10][1] == 'UNIT_HEALTH' and events[10][2] == 'party1')
        A_Admin.ResPartyMember(1)
        assert(events[11][1] == 'UNIT_HEALTH' and events[11][2] == 'party1')
        A_Admin.ClearTarget()
        assert(events[12][1] == 'PLAYER_TARGET_CHANGED' and not events[12][5])
    "#).unwrap();
}
