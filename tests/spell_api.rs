//! Tests for spell_api.rs: C_SpellBook, C_Spell, C_Traits.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ============================================================================
// C_SpellBook
// ============================================================================

#[test]
fn test_spellbook_get_spell_name_nil() {
    let env = env();
    let is_nil: bool = env.eval("return C_SpellBook.GetSpellBookItemName(1) == nil").unwrap();
    assert!(is_nil);
}

#[test]
fn test_spellbook_get_num_skill_lines() {
    let env = env();
    let count: i32 = env.eval("return C_SpellBook.GetNumSpellBookSkillLines()").unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_spellbook_get_skill_line_info_nil() {
    let env = env();
    let is_nil: bool = env.eval("return C_SpellBook.GetSpellBookSkillLineInfo(1) == nil").unwrap();
    assert!(is_nil);
}

#[test]
fn test_spellbook_get_item_info_nil() {
    let env = env();
    let is_nil: bool = env.eval("return C_SpellBook.GetSpellBookItemInfo(1) == nil").unwrap();
    assert!(is_nil);
}

#[test]
fn test_spellbook_has_pet_spells() {
    let env = env();
    let has: bool = env.eval("return C_SpellBook.HasPetSpells()").unwrap();
    assert!(!has);
}

#[test]
fn test_spellbook_get_override_spell() {
    let env = env();
    let id: i32 = env.eval("return C_SpellBook.GetOverrideSpell(42)").unwrap();
    assert_eq!(id, 42, "GetOverrideSpell should return the same ID");
}

#[test]
fn test_spellbook_is_spell_known() {
    let env = env();
    let known: bool = env.eval("return C_SpellBook.IsSpellKnown(1)").unwrap();
    assert!(!known);
}

// ============================================================================
// C_Spell
// ============================================================================

#[test]
fn test_spell_get_spell_info() {
    let env = env();
    let is_table: bool = env.eval("return type(C_Spell.GetSpellInfo(100)) == 'table'").unwrap();
    assert!(is_table);
}

#[test]
fn test_spell_get_spell_info_has_name() {
    let env = env();
    let has_name: bool = env.eval("return C_Spell.GetSpellInfo(100).name ~= nil").unwrap();
    assert!(has_name);
}

#[test]
fn test_spell_get_spell_charges() {
    let env = env();
    let is_table: bool = env.eval("return type(C_Spell.GetSpellCharges(100)) == 'table'").unwrap();
    assert!(is_table);
}

#[test]
fn test_spell_is_spell_passive() {
    let env = env();
    let passive: bool = env.eval("return C_Spell.IsSpellPassive(100)").unwrap();
    assert!(!passive);
}

#[test]
fn test_spell_get_override_spell() {
    let env = env();
    let id: i32 = env.eval("return C_Spell.GetOverrideSpell(55)").unwrap();
    assert_eq!(id, 55);
}

#[test]
fn test_spell_get_school_string() {
    let env = env();
    // Bitmask 1 = Physical, 2 = Holy, etc.
    let school: String = env.eval("return C_Spell.GetSchoolString(1)").unwrap();
    assert!(!school.is_empty());
}

#[test]
fn test_spell_get_spell_texture() {
    let env = env();
    let tex: i32 = env.eval("return C_Spell.GetSpellTexture(100)").unwrap();
    assert!(tex > 0);
}

#[test]
fn test_spell_get_spell_link() {
    let env = env();
    let link: String = env.eval("return C_Spell.GetSpellLink(100)").unwrap();
    assert!(link.contains("100"), "Link should contain spell ID");
}

#[test]
fn test_spell_get_spell_name() {
    let env = env();
    let name: String = env.eval("return C_Spell.GetSpellName(100)").unwrap();
    assert!(name.contains("100"), "Name should contain spell ID");
}

#[test]
fn test_spell_get_spell_cooldown() {
    let env = env();
    let is_table: bool = env.eval("return type(C_Spell.GetSpellCooldown(100)) == 'table'").unwrap();
    assert!(is_table);
}

#[test]
fn test_spell_does_spell_exist() {
    let env = env();
    let exists: bool = env.eval("return C_Spell.DoesSpellExist(100)").unwrap();
    assert!(exists);
    let no_exist: bool = env.eval("return C_Spell.DoesSpellExist(0)").unwrap();
    assert!(!no_exist);
}

// ============================================================================
// C_Traits
// ============================================================================

#[test]
fn test_traits_generate_import_string() {
    let env = env();
    let s: String = env.eval("return C_Traits.GenerateImportString(1)").unwrap();
    assert!(!s.is_empty());
}

#[test]
fn test_traits_get_config_id_by_system_id() {
    let env = env();
    let id: i32 = env.eval("return C_Traits.GetConfigIDBySystemID(1)").unwrap();
    assert_eq!(id, 0);
}

#[test]
fn test_traits_get_config_id_by_tree_id() {
    let env = env();
    let id: i32 = env.eval("return C_Traits.GetConfigIDByTreeID(1)").unwrap();
    assert_eq!(id, 0);
}

#[test]
fn test_traits_get_config_info() {
    let env = env();
    let is_table: bool = env.eval("return type(C_Traits.GetConfigInfo(1)) == 'table'").unwrap();
    assert!(is_table);
}

#[test]
fn test_traits_get_node_info_nil() {
    let env = env();
    let is_nil: bool = env.eval("return C_Traits.GetNodeInfo(1, 1) == nil").unwrap();
    assert!(is_nil);
}

#[test]
fn test_traits_get_entry_info_nil() {
    let env = env();
    let is_nil: bool = env.eval("return C_Traits.GetEntryInfo(1, 1) == nil").unwrap();
    assert!(is_nil);
}

#[test]
fn test_traits_initialize_view_loadout() {
    let env = env();
    let ok: bool = env.eval("return C_Traits.InitializeViewLoadout(1, 1)").unwrap();
    assert!(ok);
}

#[test]
fn test_traits_get_tree_info_nil() {
    let env = env();
    let is_nil: bool = env.eval("return C_Traits.GetTreeInfo(1, 1) == nil").unwrap();
    assert!(is_nil);
}

#[test]
fn test_traits_get_tree_nodes_empty() {
    let env = env();
    let is_table: bool = env.eval("return type(C_Traits.GetTreeNodes(1, 1)) == 'table'").unwrap();
    assert!(is_table);
}

#[test]
fn test_traits_get_all_tree_ids_empty() {
    let env = env();
    let is_table: bool = env.eval("return type(C_Traits.GetAllTreeIDs()) == 'table'").unwrap();
    assert!(is_table);
}

#[test]
fn test_traits_get_trait_system_flags() {
    let env = env();
    let flags: i32 = env.eval("return C_Traits.GetTraitSystemFlags(1)").unwrap();
    assert_eq!(flags, 0);
}

#[test]
fn test_traits_can_purchase_rank() {
    let env = env();
    let can: bool = env.eval("return C_Traits.CanPurchaseRank(1, 1, 1)").unwrap();
    assert!(!can);
}

#[test]
fn test_traits_get_loadout_serialization_version() {
    let env = env();
    let ver: i32 = env.eval("return C_Traits.GetLoadoutSerializationVersion()").unwrap();
    assert_eq!(ver, 2);
}
