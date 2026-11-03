#![cfg(any(feature = "client-retail", feature = "client-ptr"))]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn garrison_combat_log_spell_reads_pinned_catalog() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local a = C_Garrison.GetCombatLogSpellInfo(4)
        assert(a.autoCombatSpellID == 4 and a.name == "Double Strike")
        assert(a.description == "")
        assert(a.cooldown == 3 and a.duration == 0 and a.schoolMask == 1 and a.icon == 333546)
        assert(a.previewMask == 0 and a.spellTutorialFlag == 1 and a.hasThornsEffect == false)
        local b = C_Garrison.GetCombatLogSpellInfo(5)
        assert(b.name == "Wing Sweep" and b.spellTutorialFlag == 4 and not b.hasThornsEffect)
        local c = C_Garrison.GetCombatLogSpellInfo(81)
        assert(c.name == "Reconfiguration: Reflect" and c.previewMask == 4 and c.hasThornsEffect)
        assert(C_Garrison.GetCombatLogSpellInfo(999999) == nil)
    "#).unwrap();
}
