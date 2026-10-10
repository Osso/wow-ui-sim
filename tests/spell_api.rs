//! Tests for spell_api.rs: C_SpellBook, C_Spell, C_Traits.

use wow_ui_sim::c_api::charge_state::SpellChargeState;
use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[path = "spell_api/traits.rs"]
mod traits;

// ============================================================================
// C_SpellBook
// ============================================================================

#[test]
fn test_spellbook_get_spell_name_slot1() {
    let env = env();
    let name: String = env
        .eval("return C_SpellBook.GetSpellBookItemName(1)")
        .unwrap();
    assert!(!name.is_empty(), "Slot 1 should have a spell name");
}

#[test]
fn test_spellbook_get_spell_name_nil_invalid() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_SpellBook.GetSpellBookItemName(9999) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_spellbook_get_num_skill_lines() {
    let env = env();
    let count: i32 = env
        .eval("return C_SpellBook.GetNumSpellBookSkillLines()")
        .unwrap();
    assert!(count > 0, "Static spellbook data should have skill lines");
}

#[test]
fn test_spellbook_get_skill_line_info_valid() {
    let env = env();
    let is_table: bool = env
        .eval("return type(C_SpellBook.GetSpellBookSkillLineInfo(1)) == 'table'")
        .unwrap();
    assert!(is_table, "Skill line 1 should return a table");
}

#[test]
fn test_spellbook_get_skill_line_info_has_expected_fields() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local info = C_SpellBook.GetSpellBookSkillLineInfo(1)
            if type(info) ~= "table" then
                return "expected_table"
            end
            if type(info.name) ~= "string" then
                return "missing_name"
            end
            if type(info.itemIndexOffset) ~= "number" then
                return "missing_item_index_offset"
            end
            if type(info.numSpellBookItems) ~= "number" then
                return "missing_num_spell_book_items"
            end
            if type(info.iconID) ~= "number" then
                return "missing_icon_id"
            end
            if info.specID ~= nil and type(info.specID) ~= "number" then
                return "bad_spec_id"
            end
            if info.offSpecID ~= nil and type(info.offSpecID) ~= "number" then
                return "bad_off_spec_id"
            end
            if info.shouldHide ~= false then
                return "bad_should_hide"
            end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn test_spellbook_get_skill_line_info_nil_invalid() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_SpellBook.GetSpellBookSkillLineInfo(9999) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_spellbook_get_item_info_valid() {
    let env = env();
    let is_table: bool = env
        .eval("return type(C_SpellBook.GetSpellBookItemInfo(1)) == 'table'")
        .unwrap();
    assert!(is_table, "Slot 1 should return a table");
}

#[test]
fn test_spellbook_get_item_info_nil_invalid() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_SpellBook.GetSpellBookItemInfo(9999) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_c_spell_get_spell_info_accepts_spell_name() {
    let env = env();
    let spell_id: i32 = env
        .eval(
            r#"
            local info = C_Spell.GetSpellInfo("Auto Attack")
            return info and info.spellID or 0
            "#,
        )
        .unwrap();
    assert_eq!(spell_id, 6603);
}

#[test]
fn test_spellbook_get_item_texture_valid() {
    let env = env();
    let is_number: bool = env
        .eval("return type(C_SpellBook.GetSpellBookItemTexture(1, Enum.SpellBookSpellBank.Player)) == 'number'")
        .unwrap();
    assert!(
        is_number,
        "Slot 1 texture should return an icon file data id"
    );
}

#[test]
fn test_spellbook_get_item_power_cost_has_expected_fields() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local slotCount = 256
            for slot = 1, slotCount do
                local costs = C_SpellBook.GetSpellBookItemPowerCost(slot, Enum.SpellBookSpellBank.Player)
                if type(costs) == "table" and type(costs[1]) == "table" then
                    local info = costs[1]
                    if type(info.type) ~= "number" then
                        return "missing_type"
                    end
                    if type(info.name) ~= "string" then
                        return "missing_name"
                    end
                    if type(info.cost) ~= "number" then
                        return "missing_cost"
                    end
                    if type(info.minCost) ~= "number" then
                        return "missing_min_cost"
                    end
                    if type(info.costPercent) ~= "number" then
                        return "missing_cost_percent"
                    end
                    if type(info.costPerSec) ~= "number" then
                        return "missing_cost_per_sec"
                    end
                    if type(info.requiredAuraID) ~= "number" then
                        return "missing_required_aura_id"
                    end
                    if type(info.hasRequiredAura) ~= "boolean" then
                        return "missing_has_required_aura"
                    end
                    return "ok"
                end
            end
            return "no_spell_power_cost_found"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn test_spellbook_has_pet_spells() {
    let env = env();
    let has: bool = env.eval("return C_SpellBook.HasPetSpells()").unwrap();
    assert!(!has);
}

#[test]
fn test_get_call_pet_spell_info_defaults_to_no_pet_spell() {
    let env = env();
    let (is_function, first_is_nil, second_is_nil): (bool, bool, bool) = env
        .eval(
            r#"
            local spellID, texture = GetCallPetSpellInfo(1)
            return type(GetCallPetSpellInfo) == "function", spellID == nil, texture == nil
            "#,
        )
        .unwrap();

    assert!(is_function);
    assert!(first_is_nil);
    assert!(second_is_nil);
}

#[test]
fn test_spellbook_namespace_functions_survive_gc() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            collectgarbage("collect")

            if type(C_SpellBook.GetNumSpellBookSkillLines) ~= "function" then
                return "missing_count_fn"
            end

            if type(C_SpellBook.GetSpellBookSkillLineInfo) ~= "function" then
                return "missing_info_fn"
            end

            local count = C_SpellBook.GetNumSpellBookSkillLines()
            if count < 1 then
                return "bad_count"
            end

            local info = C_SpellBook.GetSpellBookSkillLineInfo(1)
            if type(info) ~= "table" then
                return "bad_info"
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn test_spellbook_get_override_spell() {
    let env = env();
    let id: i32 = env.eval("return C_SpellBook.GetOverrideSpell(42)").unwrap();
    assert_eq!(id, 42, "GetOverrideSpell should return the same ID");
}

#[test]
fn test_spellbook_get_item_auto_cast_defaults_disabled() {
    let env = env();
    let (enabled, auto_cast_allowed): (bool, bool) = env
        .eval("return C_SpellBook.GetSpellBookItemAutoCast(1, Enum.SpellBookSpellBank.Player)")
        .unwrap();

    assert!(!enabled);
    assert!(!auto_cast_allowed);
}

#[test]
fn test_spellbook_is_spell_known() {
    let env = env();
    let known: bool = env.eval("return C_SpellBook.IsSpellKnown(1)").unwrap();
    assert!(!known);
}

#[test]
fn test_spellbook_pickup_item_fires_cursor_changed() {
    let env = env();
    let changed_count: i32 = env
        .eval(
            r#"
            local f = CreateFrame("Frame")
            local count = 0
            f:SetScript("OnEvent", function(_, event)
                if event == "CURSOR_CHANGED" then
                    count = count + 1
                end
            end)
            f:RegisterEvent("CURSOR_CHANGED")

            C_SpellBook.PickupSpellBookItem(1)

            return count
            "#,
        )
        .unwrap();
    assert_eq!(
        changed_count, 1,
        "PickupSpellBookItem should fire CURSOR_CHANGED"
    );
}

#[test]
fn test_spellbook_get_loss_of_control_cooldown_info() {
    let env = env();
    let spell_id: i32 = env
        .eval("return C_SpellBook.GetSpellBookItemInfo(5, 0).spellID")
        .expect("actual player-bank slot 5 should have a spell");
    assert_eq!(spell_id, 19750);
    env.state().borrow_mut().spell_loss_of_control.insert(
        spell_id.try_into().expect("positive fixture spell ID"),
        wow_ui_sim::lua_api::LossOfControlInfo {
            start_time: 0.0,
            duration: 0.0,
            mod_rate: 1.0,
            is_active: false,
            should_replace_normal_cooldown: false,
        },
    );
    let result: String = env
        .eval(
            r#"
            local info = C_SpellBook.GetSpellBookItemLossOfControlCooldownInfo(5, Enum.SpellBookSpellBank.Player)
            if type(info) ~= "table" then
                return "expected_loc_info_table"
            end
            if info.isActive ~= false then
                return "expected_inactive_loc_info"
            end
            if info.startTime ~= 0 or info.duration ~= 0 then
                return "expected_zero_loc_cooldown"
            end
            if info.modRate ~= 1 then
                return "expected_default_loc_mod_rate"
            end
            if info.shouldReplaceNormalCooldown ~= false then
                return "expected_loc_not_to_replace_normal_cooldown"
            end
            if C_SpellBook.GetSpellBookItemLossOfControlCooldownInfo(9999, Enum.SpellBookSpellBank.Player) ~= nil then
                return "invalid_spellbook_slot_should_not_have_loc_info"
            end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

// ============================================================================
// C_Spell
// ============================================================================

#[test]
fn test_spell_get_spell_info() {
    let env = env();
    let is_table: bool = env
        .eval("return type(C_Spell.GetSpellInfo(100)) == 'table'")
        .unwrap();
    assert!(is_table);
}

#[test]
fn test_spell_get_spell_info_has_name() {
    let env = env();
    let has_name: bool = env
        .eval("return C_Spell.GetSpellInfo(100).name ~= nil")
        .unwrap();
    assert!(has_name);
}

#[test]
fn test_spell_get_spell_charges() {
    let env = env();
    env.state().borrow_mut().spell_charges.insert(
        19750,
        SpellChargeState {
            current_charges: 1,
            max_charges: 2,
            recharge_start: 12.0,
            recharge_duration: 40.0,
            charge_mod_rate: 2.0,
        },
    );
    let is_table: bool = env
        .eval("return type(C_Spell.GetSpellCharges(19750)) == 'table'")
        .unwrap();
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

// ============================================================================
// C_AssistedCombat
// ============================================================================

#[test]
fn test_assisted_combat_rotation_spells_returns_table() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local spells = C_AssistedCombat.GetRotationSpells()
            if type(spells) ~= "table" then
                return "not_table:" .. tostring(type(spells))
            end
            local count = 0
            for _ in ipairs(spells) do
                count = count + 1
            end
            return "ok:" .. tostring(count)
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok:0");
}

#[test]
fn test_assisted_combat_is_available_shape() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local available, reason = C_AssistedCombat.IsAvailable()
            return tostring(available) .. ":" .. tostring(reason)
            "#,
        )
        .unwrap();
    assert_eq!(result, "false:Not available");
}

#[test]
fn test_spell_get_spell_texture() {
    let env = env();
    let (icon, original): (i64, i64) = env.eval("return C_Spell.GetSpellTexture(100)").unwrap();
    assert_eq!(icon, i64::from(wow_ui_sim::spells::get_spell(100).unwrap().icon_file_data_id));
    assert_eq!(original, icon);
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
    // Spell 100 = "Charge"
    let name: String = env.eval("return C_Spell.GetSpellName(100)").unwrap();
    assert_eq!(name, "Charge");
}

#[test]
fn test_spell_get_spell_name_unknown() {
    let env = env();
    let name: String = env.eval("return C_Spell.GetSpellName(999999999)").unwrap();
    assert_eq!(name, "Unknown");
}

#[test]
fn test_spell_get_maw_power_border_atlas_by_spell_id_is_stubbed() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_Spell.GetMawPowerBorderAtlasBySpellID(12345) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_spell_pickup_fires_cursor_changed() {
    let env = env();
    let changed_count: i32 = env
        .eval(
            r#"
            local f = CreateFrame("Frame")
            local count = 0
            f:SetScript("OnEvent", function(_, event)
                if event == "CURSOR_CHANGED" then
                    count = count + 1
                end
            end)
            f:RegisterEvent("CURSOR_CHANGED")

            C_Spell.PickupSpell(100)

            return count
            "#,
        )
        .unwrap();
    assert_eq!(changed_count, 1, "PickupSpell should fire CURSOR_CHANGED");
}

#[test]
fn test_spell_get_spell_cooldown() {
    let env = env();
    let is_table: bool = env
        .eval("return type(C_Spell.GetSpellCooldown(100)) == 'table'")
        .unwrap();
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
