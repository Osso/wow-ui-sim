//! R88m.4 bounded Retail alias retirement, not a universal native-absence claim.
//!
//! The native registration census omits these four names; script/outside-census
//! publication remains UNKNOWN. The pinned Retail cache audit verified all 4026
//! manifest entries and scanned all 3647 Lua/XML files, including Game and Glue.
//! It found zero Retail-selected calls or definitions for these exact names.
//! Twelve GetSpellInfo calls and one IsPassiveSpell call belong exclusively to
//! Classic/Cata/Mists sources, excluded by TOC flavor/include selection. The
//! deprecated transition guide is a long comment, not executable alias code;
//! namespaced C_Spell APIs and C_SpellBook.GetSpellBookItemInfo are distinct.
//! This does not rule out arbitrary dynamically constructed names or third-party
//! addons. These are bootstrap-publication retirement checks, not original-Lua
//! consumer REDs. GetSpellInfo is already absent and is preservation only.
#![cfg(all(feature = "optional-tests", feature = "client-retail"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn retail_spellbook_override_alias_is_not_published() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local spellBook = rawget(_G, 'C_SpellBook')
        assert(type(spellBook) == 'table', 'C_SpellBook namespace must remain available')
        assert(rawget(spellBook, 'GetOverrideSpell') == nil,
            'unsupported Retail C_SpellBook.GetOverrideSpell alias is published')
        "#,
    )
    .unwrap();
}

#[test]
fn retail_global_spellbook_item_info_alias_is_not_published() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(rawget(_G, 'GetSpellBookItemInfo') == nil,
            'unsupported Retail GetSpellBookItemInfo alias is published')
        "#,
    )
    .unwrap();
}

#[test]
fn retail_global_passive_spell_alias_is_not_published() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(rawget(_G, 'IsPassiveSpell') == nil,
            'unsupported Retail IsPassiveSpell alias is published')
        "#,
    )
    .unwrap();
}

#[test]
fn retail_global_spell_info_existing_absence_is_preserved() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(rawget(_G, 'GetSpellInfo') == nil,
            'legacy GetSpellInfo must not be newly published in Retail')
        "#,
    )
    .unwrap();
}
