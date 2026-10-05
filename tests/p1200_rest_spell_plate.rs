#![cfg(feature = "retail-12-0-0")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn p1200_rest_spell_classification() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().aura_filter_facts.important_spell_ids.insert(19750);
    env.exec(r#"
        assert(C_Spell.IsSpellImportant(19750) == true)
        assert(C_Spell.IsSpellImportant('Flash of Light') == true)
        assert(C_Spell.IsSpellImportant(999999) == false)
        assert(C_Spell.IsSpellCrowdControl(19750) == false)
        assert(C_Spell.IsExternalDefensive(19750) == false)
        assert(C_Spell.IsConsumableSpell(19750) == false)
    "#).unwrap();
    env.state().borrow_mut().spell_classifications.insert(19750, wow_ui_sim::c_api::c_spell_classification::SpellClassification {
        consumable: true, crowd_control: true, external_defensive: true,
    });
    env.exec("assert(C_Spell.IsSpellCrowdControl(19750)); assert(C_Spell.IsExternalDefensive(19750)); assert(C_Spell.IsConsumableSpell(19750))").unwrap();
    env.state().borrow_mut().aura_filter_facts.important_spell_ids.clear();
    env.exec("assert(not C_Spell.IsSpellImportant(19750))").unwrap();
}

#[test]
fn p1200_rest_nameplate_size_roundtrip() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        C_NamePlate.SetNamePlateSize(140, 32)
        local w,h = C_NamePlate.GetNamePlateSize()
        assert(w == 140 and h == 32)
        assert(C_NamePlateManager.IsNamePlateUnitBehindCamera('nameplate1') == false)
        C_NamePlateManager.SetNamePlateSimplified('nameplate1', true)
        assert(not pcall(C_NamePlate.SetNamePlateSize, -1, 10))
        local w,h = C_NamePlate.GetNamePlateSize()
        assert(w == 140 and h == 32)
    "#).unwrap();
    assert!(env.state().borrow().nameplate_configuration.simplified.contains("nameplate1"));
    env.state().borrow_mut().nameplate_configuration.behind_camera.insert("nameplate1".into());
    env.exec("assert(C_NamePlateManager.IsNamePlateUnitBehindCamera('nameplate1')); C_NamePlateManager.SetNamePlateSimplified('nameplate1', false)").unwrap();
    assert!(env.state().borrow().nameplate_configuration.simplified.is_empty());
}

#[test]
fn p1200_rest_cooldown_alert_types_empty() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(#C_CooldownViewer.GetValidAlertTypes(123) == 0)").unwrap();
    env.state().borrow_mut().cooldown_viewer_cooldowns.insert(123, wow_ui_sim::c_api::c_cooldown_viewer::CooldownViewerCooldown {
        cooldown_id: 123, valid_alert_types: vec![0, 3, 5], ..Default::default()
    });
    env.exec(r#"
        local alerts = C_CooldownViewer.GetValidAlertTypes(123)
        assert(#alerts == 3 and alerts[1] == 0 and alerts[2] == 3 and alerts[3] == 5)
        alerts[1] = 9
        assert(C_CooldownViewer.GetValidAlertTypes(123)[1] == 0)
    "#).unwrap();
}
