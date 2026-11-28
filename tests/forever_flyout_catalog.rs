//! Forever 70707 documents these C_Flyout structures; no pinned Lua caller uses them.
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

const FLYOUTS: &str = r#"
    A_Admin.SetSpellFlyouts({
        { flyoutID = 90, name = 'First', description = 'Travel', isKnown = true,
          slots = { { spellID = 122, overrideSpellID = 122, isKnown = true, name = 'Frost Nova' } } },
        { flyoutID = 17, name = 'Second', description = 'Support', isKnown = false,
          slots = { { spellID = 19750, overrideSpellID = 19750, isKnown = false, specID = 65 },
                    { spellID = 853, overrideSpellID = 853, isKnown = true, name = 'Hammer of Justice' } } },
    })
"#;

#[test]
fn flyout_index_and_count_follow_supplied_order() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(C_Flyout.GetNumFlyouts() == 0); assert(select('#', C_Flyout.GetFlyoutID(1)) == 0)").unwrap();
    env.exec(FLYOUTS).unwrap();
    env.exec(r#"
        assert(C_Flyout.GetNumFlyouts() == 2)
        assert(C_Flyout.GetFlyoutID(1) == 90 and C_Flyout.GetFlyoutID(2) == 17)
        assert(select('#', C_Flyout.GetFlyoutID(3)) == 0)
        A_Admin.SetSpellFlyouts({})
        assert(C_Flyout.GetNumFlyouts() == 0 and C_Flyout.GetFlyoutID(1) == nil)
    "#).unwrap();
}

#[test]
fn flyout_info_returns_distinct_structures() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(select('#', C_Flyout.GetFlyoutInfo(90)) == 0)").unwrap();
    env.exec(FLYOUTS).unwrap();
    env.exec(r#"
        local a, b = C_Flyout.GetFlyoutInfo(90), C_Flyout.GetFlyoutInfo(17)
        assert(a.name == 'First' and a.description == 'Travel' and a.numSlots == 1 and a.isKnown == true)
        assert(b.name == 'Second' and b.description == 'Support' and b.numSlots == 2 and b.isKnown == false)
        a.name = 'Changed'
        assert(C_Flyout.GetFlyoutInfo(90).name == 'First')
    "#).unwrap();
}

#[test]
fn flyout_slot_info_preserves_supplied_optional_fields() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(select('#', C_Flyout.GetFlyoutSlotInfo(90, 1)) == 0)").unwrap();
    env.exec(FLYOUTS).unwrap();
    env.exec(r#"
        local a, b = C_Flyout.GetFlyoutSlotInfo(90, 1), C_Flyout.GetFlyoutSlotInfo(17, 1)
        assert(a.spellID == 122 and a.overrideSpellID == 122 and a.isKnown == true)
        assert(a.name == 'Frost Nova' and a.specID == nil)
        assert(b.spellID == 19750 and b.overrideSpellID == 19750 and b.isKnown == false)
        assert(b.name == nil and b.specID == 65)
        assert(C_Flyout.GetFlyoutSlotInfo(17, 2).spellID == 853)
        assert(select('#', C_Flyout.GetFlyoutSlotInfo(17, 3)) == 0)
    "#).unwrap();
}

#[test]
fn flyout_has_spell_checks_membership_of_selected_flyout() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(select('#', C_Flyout.FlyoutHasSpell(90, 122)) == 0)").unwrap();
    env.exec(FLYOUTS).unwrap();
    env.exec(r#"
        assert(C_Flyout.FlyoutHasSpell(90, 122) == true)
        assert(C_Flyout.FlyoutHasSpell(17, 122) == false)
        assert(C_Flyout.FlyoutHasSpell(17, 19750) == true)
        assert(C_Flyout.FlyoutHasSpell(90, 853) == false)
    "#).unwrap();
}
