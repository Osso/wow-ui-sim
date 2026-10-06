#![cfg(feature = "retail-12-1-0")]

use wow_ui_sim::lua_api::{MajorFactionData, WowLuaEnv};

#[test]
fn patch_12_0_1_creature_id_follows_identity_visibility() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("A_Admin.SetTarget('Hogger', 11, 1, true)").unwrap();
    env.state().borrow_mut().current_target.as_mut().unwrap().guid =
        "Creature-0-1-2-3-448-000001".into();
    env.exec("assert(UnitCreatureID('target') == 448)").unwrap();
    env.state().borrow_mut().instance_identity.on_instanced_map = true;
    env.exec(r#"
        assert(C_Secrets.ShouldUnitIdentityBeSecret('target'))
        assert(UnitCreatureID('target') == nil)
        local function addon()
            assert(UnitCreatureID('target') == nil)
            assert(debug.getstacktaint() == 'CreatureProbe')
        end
        debug.setobjecttaint(addon, 'CreatureProbe')
        addon()
    "#).unwrap();
    env.state().borrow_mut().instance_identity.on_instanced_map = false;
    env.exec("assert(UnitCreatureID('target') == 448); assert(UnitCreatureID('player') == nil)").unwrap();
}

#[test]
fn patch_12_0_1_major_faction_flags_are_host_state_snapshots() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().major_factions.insert(9801, MajorFactionData {
        faction_id: 9801,
        name: "Proof faction".into(),
        max_level: 27,
        use_journey_unlock_toast: true,
        ..Default::default()
    });
    env.exec(r#"
        HeldFaction = C_MajorFactions.GetMajorFactionData(9801)
        assert(HeldFaction.name == 'Proof faction')
        assert(type(HeldFaction.maxLevel) == 'number' and HeldFaction.maxLevel == 27)
        assert(type(HeldFaction.useJourneyUnlockToast) == 'boolean' and HeldFaction.useJourneyUnlockToast)
        HeldFaction.maxLevel = -1
        assert(C_MajorFactions.GetMajorFactionData(9801).maxLevel == 27)
    "#).unwrap();
    {
        let mut state = env.state().borrow_mut();
        let faction = state.major_factions.get_mut(&9801).unwrap();
        faction.max_level = 31;
        faction.use_journey_unlock_toast = false;
    }
    env.exec(r#"
        local fresh = C_MajorFactions.GetMajorFactionData(9801)
        assert(fresh.maxLevel == 31 and fresh.useJourneyUnlockToast == false)
        assert(HeldFaction.maxLevel == -1 and HeldFaction.useJourneyUnlockToast)
        assert(C_MajorFactions.GetMajorFactionData(9802) == nil)
    "#).unwrap();
}

#[test]
fn patch_12_0_1_secret_precision_retains_jar_jar_payload() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local secret = secretwrap('Jar Jar Binks')
        local text = string.format('%.1s', secret)
        assert(issecretvalue(text) and secretunwrap(text) == 'Jar Jar Binks')
        assert(string.format('%.1s', 'Jar Jar Binks') == 'J')
        local function addon()
            local result = string.format('%.1s', secret)
            assert(issecretvalue(result))
            assert(not pcall(secretunwrap, result))
            assert(debug.getstacktaint() == 'PrecisionProbe')
            return result
        end
        debug.setobjecttaint(addon, 'PrecisionProbe')
        assert(secretunwrap(addon()) == 'Jar Jar Binks')
    "#).unwrap();
}

#[test]
fn patch_12_0_1_long_buff_exemptions_survive_restricted_aura_context() {
    let env = WowLuaEnv::new().unwrap();
    let spells = [
        1126, 1459, 6673, 21562, 369459, 462854, 474754, 381732, 381741, 381746,
        381748, 381749, 381750, 381751, 381752, 381753, 381754, 381756, 381757,
        381758, 433568, 433583, 2823, 8679, 3408, 5761, 315584, 381637, 381664,
        319773, 319778, 382021, 382022, 457496, 457481, 462757, 462742, 205473, 260286,
    ];
    for spell in spells {
        {
            let mut state = env.state().borrow_mut();
            let mut aura = state.player.buffs[0].clone();
            aura.spell_id = spell;
            aura.aura_instance_id = 710;
            aura.name = "Exempt buff".into();
            aura.duration = 3600.0;
            aura.expiration_time = 7200.0;
            state.player.buffs = vec![aura];
            state.unit_auras_restricted = true;
        }
        env.exec(&format!(r#"
            local spell = {spell}
            assert(C_Secrets.GetSpellAuraSecrecy(spell) == Enum.SecrecyLevel.NeverSecret)
            assert(not C_Secrets.ShouldSpellAuraBeSecret(spell))
            local function addon()
                assert(not issecure())
                local aura = C_UnitAuras.GetPlayerAuraBySpellID(spell)
                assert(aura and not issecretvalue(aura))
                assert(aura.spellId == spell and aura.auraInstanceID == 710)
                assert(aura.name == 'Exempt buff' and aura.duration == 3600)
                assert(aura.expirationTime == 7200)
                assert(not issecretvalue(aura.spellId) and not issecretvalue(aura.duration))
                assert(debug.getstacktaint() == 'ExtractBuffProbe')
            end
            debug.setobjecttaint(addon, 'ExtractBuffProbe')
            addon()
        "#)).unwrap_or_else(|err| panic!("spell {spell}: {err}"));
    }
    env.state().borrow_mut().player.buffs[0].spell_id = 35395;
    env.exec(r#"
        assert(C_Secrets.ShouldSpellAuraBeSecret(35395))
        local function addon()
            assert(C_UnitAuras.GetPlayerAuraBySpellID(35395) == nil)
            assert(not issecure())
        end
        debug.setobjecttaint(addon, 'ExtractBuffControl')
        addon()
    "#).unwrap();
}
