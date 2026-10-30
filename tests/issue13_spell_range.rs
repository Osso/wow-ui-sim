//! Issue 13 range behavior through the public Lua APIs.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn c_spell_range_valid_and_invalid_inputs() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(C_Spell.IsSpellInRange(100, 'player') == true)
        assert(C_Spell.IsSpellInRange(99999999, 'player') == nil)
        assert(C_Spell.IsSpellInRange(100, 'nonexistent') == nil)
        assert(C_Spell.IsSpellInRange(100) == nil)
        A_Admin.SetTarget('Range target', 80, 1, true)
        assert(C_Spell.IsSpellInRange(100) == true)
        assert(C_Spell.IsSpellInRange('Charge', 'target') == true)
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_reporter_buff_name_and_range() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local name = C_Spell.GetSpellName(10157)
        assert(name == 'Arcane Intellect', 'Forever rank name')
        assert(C_Spell.GetSpellInfo(10157).name == name, 'spell info name')
        A_Admin.LearnSpell(10157)
        assert(IsSpellKnown(10157), 'learned spell')
        A_Admin.SetTarget('Range target', 60, 1, true)
        assert(C_Spell.IsSpellInRange(name, 'target') == true, 'name range')
    "#,
    )
    .unwrap();
}

#[test]
fn legacy_name_and_spellbook_range() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().known_spells.insert(100);
    env.exec(
        r#"
        assert(IsSpellInRange('Charge', 'player') == 1)
        assert(IsSpellInRange('Charge', 'nonexistent') == nil)
        assert(IsSpellInRange('not a spell', 'player') == nil)
        assert(IsSpellInRange(99999, 'spell', 'player') == nil)
        local spellName = C_SpellBook.GetSpellBookItemName(1)
        assert(spellName ~= nil)
        assert(IsSpellInRange(1, 'spell', 'player') == 1)
        local spellID = C_SpellBook.GetSpellBookItemInfo(1).spellID
        if IsSpellKnown(spellID) then
            assert(IsSpellInRange(spellName, 'player') == 1)
        end
    "#,
    )
    .unwrap();
    env.state().borrow_mut().known_spells.clear();
    env.exec("assert(IsSpellInRange('Charge', 'player') == nil)")
        .unwrap();
}
