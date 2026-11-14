//! Tests for profession spells exposed through C_Spell and C_SpellBook.

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[cfg(feature = "client-retail")]
#[test]
fn professions_book_selection_consumer_preserves_invalid_slot_state() {
    test_timeout! {
    with_blizzard_addon_startup_shape(&["Blizzard_ProfessionsBook"], &[], |env, loaded| {
        assert!(loaded.iter().any(|name| name == "Blizzard_ProfessionsBook"));
        let checked: bool = env
            .eval(
                r#"
                local parent = { spellOffset = 0 }
                local button = { checked = true }
                function button:GetID() return 0 end
                function button:GetParent() return parent end
                function button:SetChecked(value) self.checked = value end
                ProfessionSpellButtonMixin.UpdateSelection(button)
                return button.checked
                "#,
            )
            .expect("the original ProfessionSpellButtonMixin consumer should run");

        // Preservation only: the original consumer at Blizzard_ProfessionsBook.lua:285
        // uses the return as a condition, so zero results and false both leave it unchecked.
        assert!(!checked);
    });
    }
}

#[cfg(feature = "client-retail")]
#[test]
fn professions_book_selection_consumer_preserves_positive_out_of_range_slot() {
    test_timeout! {
    with_blizzard_addon_startup_shape(&["Blizzard_ProfessionsBook"], &[], |env, loaded| {
        assert!(loaded.iter().any(|name| name == "Blizzard_ProfessionsBook"));
        let checked: bool = env
            .eval(
                r##"
                local slot = 1
                while C_SpellBook.GetSpellBookItemInfo(slot, Enum.SpellBookSpellBank.Player) do
                    slot = slot + 1
                end
                assert(slot <= 1024)
                local parent = { spellOffset = 0 }
                local button = { checked = true, slot = slot }
                function button:GetID() return self.slot end
                function button:GetParent() return parent end
                function button:SetChecked(value) self.checked = value end
                ProfessionSpellButtonMixin.UpdateSelection(button)
                return button.checked
                "##,
            )
            .expect("the original ProfessionSpellButtonMixin consumer should run");

        // Preservation only: `UpdateSelection` uses the return as a condition.
        assert!(!checked);
    });
    }
}

#[test]
fn test_blacksmithing_spell_exists() {
    let env = env();
    let name: String = env.eval("return C_Spell.GetSpellInfo(2018).name").unwrap();
    assert_eq!(name, "Blacksmithing");
}

#[test]
fn test_smelt_copper_spell_exists() {
    let env = env();
    let name: String = env.eval("return C_Spell.GetSpellInfo(2657).name").unwrap();
    assert_eq!(name, "Smelt Copper");
}

#[test]
fn test_mining_spell_exists() {
    let env = env();
    let name: String = env.eval("return C_Spell.GetSpellInfo(2575).name").unwrap();
    assert_eq!(name, "Mining");
}

#[test]
fn test_mining_passive_spell_exists() {
    let env = env();
    let name: String = env.eval("return C_Spell.GetSpellInfo(2576).name").unwrap();
    assert_eq!(name, "Mining");
}

#[test]
fn test_blacksmithing_skill_line_in_spellbook() {
    let env = env();
    let name: String = env
        .eval(
            r#"
            local n = C_SpellBook.GetNumSpellBookSkillLines()
            for i = 1, n do
                local info = C_SpellBook.GetSpellBookSkillLineInfo(i)
                if info.name == "Blacksmithing" then return info.name end
            end
            return ""
            "#,
        )
        .unwrap();
    assert_eq!(name, "Blacksmithing");
}

#[test]
fn test_mining_skill_line_in_spellbook() {
    let env = env();
    let name: String = env
        .eval(
            r#"
            local n = C_SpellBook.GetNumSpellBookSkillLines()
            for i = 1, n do
                local info = C_SpellBook.GetSpellBookSkillLineInfo(i)
                if info.name == "Mining" then return info.name end
            end
            return ""
            "#,
        )
        .unwrap();
    assert_eq!(name, "Mining");
}

#[cfg(feature = "client-mists")]
#[test]
fn test_legacy_spell_tabs_expose_backing_skill_lines() {
    let env = env();
    let (tab_count, tab_name, item_offset, item_count): (i32, String, i32, i32) = env
        .eval(
            r#"
            local name, _, itemOffset, itemCount = GetSpellTabInfo(1)
            return GetNumSpellTabs(), name or "", itemOffset or -1, itemCount or -1
            "#,
        )
        .unwrap();

    assert!(tab_count > 1, "legacy spell tabs should expose skill lines");
    assert_eq!(tab_name, "General");
    assert_eq!(item_offset, 0);
    assert!(item_count > 0, "first spell tab should contain spells");
}

#[cfg(feature = "client-mists")]
#[test]
fn test_legacy_spellbook_item_globals_return_mists_shape() {
    let env = env();
    let (slot_type, action_id, name, spell_id, texture, spell_info_name, spell_texture, is_passive): (
        String,
        i32,
        String,
        i32,
        i32,
        String,
        i32,
        bool,
    ) = env
        .eval(
            r#"
            local slotType, actionID = GetSpellBookItemInfo(1, BOOKTYPE_SPELL)
            local name, _, spellID = GetSpellBookItemName(1, BOOKTYPE_SPELL)
            local texture = GetSpellBookItemTexture(1, BOOKTYPE_SPELL)
            local spellInfoName = GetSpellInfo(spellID)
            local _, spellTexture = GetSpellTexture(spellID)
            local isPassive = IsPassiveSpell(1, BOOKTYPE_SPELL)
            return slotType or "", actionID or 0, name or "", spellID or 0,
                texture or 0, spellInfoName or "", spellTexture or 0, isPassive
            "#,
        )
        .unwrap();

    assert_eq!(slot_type, "SPELL");
    assert_eq!(action_id, 6603);
    assert_eq!(name, "Auto Attack");
    assert_eq!(spell_id, 6603);
    assert!(texture > 0, "legacy texture should be a file data id");
    assert_eq!(spell_info_name, "Auto Attack");
    assert_eq!(
        spell_texture, texture,
        "legacy spell texture should match the spellbook item texture"
    );
    assert!(!is_passive);
}

#[cfg(feature = "client-mists")]
#[test]
fn legacy_spellbook_item_info_has_classic_shape() {
    let env = env();
    let has_retail_info: bool = env
        .eval(
            r#"
            local _, _, spellInfo = GetSpellBookItemInfo(1, BOOKTYPE_SPELL)
            return spellInfo ~= nil
            "#,
        )
        .unwrap();

    assert!(
        !has_retail_info,
        "legacy GetSpellBookItemInfo should not expose retail spellInfo"
    );
}

#[test]
fn test_spellbook_item_info_blacksmithing_slot() {
    let env = env();
    let (name, spell_id): (String, i32) = env
        .eval(
            r#"
            local n = C_SpellBook.GetNumSpellBookSkillLines()
            for i = 1, n do
                local line = C_SpellBook.GetSpellBookSkillLineInfo(i)
                if line.name == "Blacksmithing" then
                    local slot = line.itemIndexOffset + 1
                    local info = C_SpellBook.GetSpellBookItemInfo(slot)
                    return info.name, info.spellID
                end
            end
            return "", 0
            "#,
        )
        .unwrap();
    assert_eq!(name, "Blacksmithing");
    assert_eq!(spell_id, 2018);
}

#[test]
fn test_spellbook_item_info_mining_passive() {
    let env = env();
    let (name, is_passive): (String, bool) = env
        .eval(
            r#"
            local n = C_SpellBook.GetNumSpellBookSkillLines()
            for i = 1, n do
                local line = C_SpellBook.GetSpellBookSkillLineInfo(i)
                if line.name == "Mining" then
                    local slot = line.itemIndexOffset + 2
                    local info = C_SpellBook.GetSpellBookItemInfo(slot)
                    return info.name, info.isPassive
                end
            end
            return "", false
            "#,
        )
        .unwrap();
    assert_eq!(name, "Mining");
    assert!(is_passive);
}
