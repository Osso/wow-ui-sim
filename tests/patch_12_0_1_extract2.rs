#![cfg(feature = "retail-12-1-0")]

use wow_ui_sim::lua_api::WowLuaEnv;

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
