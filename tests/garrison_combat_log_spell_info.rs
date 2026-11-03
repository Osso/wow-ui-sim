#![cfg(any(feature = "client-retail", feature = "client-ptr"))]
use wow_ui_sim::c_api::c_garrison_combat_log_spell::SpellPresentation;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn garrison_combat_log_spell_combines_catalog_and_supplied_fields() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(C_Garrison.GetCombatLogSpellInfo(4) == nil)").unwrap();
    env.state().borrow_mut().garrison_spell_presentation.insert(4, SpellPresentation {
        preview_mask: 1, spell_tutorial_flag: 7, has_thorns_effect: true,
    });
    env.state().borrow_mut().garrison_spell_presentation.insert(5, SpellPresentation {
        preview_mask: 2, spell_tutorial_flag: 3, has_thorns_effect: false,
    });
    env.exec(r#"
        local a = C_Garrison.GetCombatLogSpellInfo(4)
        assert(a.autoCombatSpellID == 4 and a.name == "Double Strike")
        assert(a.description == "Nadjia strikes the closest enemy twice, dealing a $s1 and then $s2 Physical damage.")
        assert(a.cooldown == 3 and a.duration == 0 and a.schoolMask == 1 and a.icon == 333546)
        assert(a.previewMask == 1 and a.spellTutorialFlag == 7 and a.hasThornsEffect == true)
        local b = C_Garrison.GetCombatLogSpellInfo(5)
        assert(b.name == "Wing Sweep" and b.previewMask == 2 and not b.hasThornsEffect)
        assert(C_Garrison.GetCombatLogSpellInfo(999999) == nil)
    "#).unwrap();
}
