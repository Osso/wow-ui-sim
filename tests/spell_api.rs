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

#[cfg(feature = "client-retail")]
#[test]
fn get_call_pet_spell_info_non_numeric_input_uses_verified_usage() {
    let env = env();
    let (succeeded, error): (bool, String) = env
        .eval(
            r#"
            local succeeded, message = pcall(GetCallPetSpellInfo, "not-a-number")
            return succeeded, tostring(message)
            "#,
        )
        .unwrap();

    assert!(!succeeded);
    assert_eq!(error, "Usage: GetCallPetSpellInfo(spellID)");
}

#[cfg(feature = "client-retail")]
#[test]
fn selected_spellbook_item_invalid_numeric_slot_has_verified_zero_value_arity() {
    let env = env();
    let result_count: i32 = env
        .eval(r##"return select("#", IsSelectedSpellBookItem(0, "player"))"##)
        .unwrap();

    assert_eq!(result_count, 0);
}

#[cfg(feature = "client-retail")]
#[test]
fn selected_spellbook_item_past_player_fixture_has_verified_zero_value_arity() {
    let env = env();
    // The slot is chosen from the simulator's contiguous player-book fixture;
    // this does not claim that the same numeric slot is absent in native data.
    let result_count: i32 = env
        .eval(
            r##"
            local slot = 1
            while C_SpellBook.GetSpellBookItemInfo(slot, Enum.SpellBookSpellBank.Player) do
                slot = slot + 1
            end
            return select("#", IsSelectedSpellBookItem(slot, "player"))
            "##,
        )
        .unwrap();

    assert_eq!(result_count, 0);
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

#[cfg(feature = "client-retail")]
#[test]
fn spellbook_lookup_arguments_match_verified_conversion_and_errors() {
    let env = env();
    let result: bool = env
        .eval(
            r#"
            local methods = {
                {
                    C_SpellBook.FindSpellOverrideByID,
                    "Usage: local overrideSpellID = C_SpellBook.FindSpellOverrideByID(spellID)",
                },
                {
                    C_SpellBook.FindFlyoutSlotBySpellID,
                    "Usage: local flyoutSlot = C_SpellBook.FindFlyoutSlotBySpellID(spellID)",
                },
                {
                    C_SpellBook.FindBaseSpellByID,
                    "Usage: local baseSpellID = C_SpellBook.FindBaseSpellByID(spellID)",
                },
            }
            for _, method in ipairs(methods) do
                local missingOK, missingError = pcall(method[1])
                if missingOK or not string.find(tostring(missingError), method[2], 1, true) then
                    return false
                end
                local invalidOK, invalidError = pcall(method[1], true)
                if invalidOK
                    or not string.find(tostring(invalidError), "outside of expected range", 1, true)
                    or not string.find(tostring(invalidError), method[2], 1, true) then
                    return false
                end
                if not pcall(method[1], "116.9", {}, "ignored") then
                    return false
                end
            end
            return true
            "#,
        )
        .unwrap();

    assert!(result);
}

#[cfg(feature = "client-retail")]
#[test]
fn spellbook_item_arguments_match_verified_two_argument_conversion_and_errors() {
    let env = env();
    let result: bool = env
        .eval(
            r#"
            local methods = {
                {
                    C_SpellBook.GetSpellBookItemAutoCast,
                    "Usage: local autoCastAllowed, autoCastEnabled = C_SpellBook.GetSpellBookItemAutoCast(spellBookItem)",
                },
                {
                    C_SpellBook.GetSpellBookItemLossOfControlCooldownInfo,
                    "Usage: local lossOfControlInfo = C_SpellBook.GetSpellBookItemLossOfControlCooldownInfo(spellBookItem)",
                },
            }
            for _, method in ipairs(methods) do
                local missingSlotOK, missingSlotError = pcall(method[1])
                if missingSlotOK or not string.find(tostring(missingSlotError), method[2], 1, true) then
                    return false
                end
                local missingBankOK, missingBankError = pcall(method[1], 1)
                if missingBankOK or not string.find(tostring(missingBankError), method[2], 1, true) then
                    return false
                end
                local invalidBankOK, invalidBankError = pcall(method[1], 1, 2)
                if invalidBankOK or not string.find(tostring(invalidBankError), method[2], 1, true) then
                    return false
                end
                local invalidSlotOK, invalidSlotError = pcall(method[1], 4294967296, 0)
                if invalidSlotOK or not string.find(tostring(invalidSlotError), method[2], 1, true) then
                    return false
                end
                if not pcall(method[1], "5.9", "0.9", "ignored") then
                    return false
                end
                for _, slot in ipairs({0, 4294967295}) do
                    local ok, result = pcall(method[1], slot, 0)
                    if not ok and string.find(tostring(result), "outside of expected range", 1, true) then
                        return false
                    end
                end
            end
            return true
            "#,
        )
        .unwrap();

    assert!(result);
}

#[cfg(feature = "client-retail")]
#[test]
fn spellbook_flyout_lookup_miss_returns_one_numeric_zero() {
    let env = env();
    let result: (i32, String, i32) = env
        .eval(
            r##"
            local count = select("#", C_SpellBook.FindFlyoutSlotBySpellID(0))
            local value = C_SpellBook.FindFlyoutSlotBySpellID(0)
            return count, type(value), value or -1
            "##,
        )
        .expect("flyout lookup miss should be callable");

    assert_eq!(result, (1, "number".to_string(), 0));
}

#[cfg(feature = "client-retail")]
#[test]
fn spellbook_autocast_supported_item_returns_two_false_booleans_without_flag_data() {
    let env = env();
    let result: (bool, i32, bool, bool, bool, bool) = env
        .eval(
            r##"
            local item = C_SpellBook.GetSpellBookItemInfo(1, Enum.SpellBookSpellBank.Player)
            local count = select("#", C_SpellBook.GetSpellBookItemAutoCast(1, Enum.SpellBookSpellBank.Player))
            local allowed, enabled = C_SpellBook.GetSpellBookItemAutoCast(1, Enum.SpellBookSpellBank.Player)
            return type(item) == "table", count,
                   type(allowed) == "boolean", type(enabled) == "boolean",
                   allowed, enabled
            "##,
        )
        .expect("autocast query for a spellbook record should be callable");

    assert_eq!(result, (true, 2, true, true, false, false));
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
    assert_eq!(
        icon,
        i64::from(
            wow_ui_sim::spells::get_spell(100)
                .unwrap()
                .icon_file_data_id
        )
    );
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
    let result: bool = env
        .eval("return select('#', C_Spell.GetSpellName(999999999)) == 0 and C_Spell.GetSpellName(999999999) == nil")
        .unwrap();
    assert!(result);
}

#[test]
fn test_spell_get_maw_power_border_atlas_by_spell_id_follows_retirement_epoch() {
    let env = env();
    // The 12.0.7 source register removes this member from the native surface.
    let expression = if cfg!(feature = "retail-12-0-7") {
        "return C_Spell.GetMawPowerBorderAtlasBySpellID == nil"
    } else {
        "return C_Spell.GetMawPowerBorderAtlasBySpellID(12345) == nil"
    };
    let is_nil: bool = env.eval(expression).unwrap();
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
