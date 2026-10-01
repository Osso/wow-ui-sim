//! Tests for aura/buff API functions (aura_api.rs, system_api.rs C_UnitAuras).

use wow_ui_sim::lua_api::{WowLuaEnv, state::AuraInfo};

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ============================================================================
// UnitBuff
// ============================================================================

#[test]
fn test_unit_buff_returns_data_for_player() {
    let env = env();
    let name: String = env.eval("return UnitBuff('player', 1)").unwrap();
    assert!(!name.is_empty(), "First buff name should be non-empty");
}

#[test]
fn test_unit_buff_returns_all_fields() {
    let env = env();
    let (has_name, has_spell_id, has_duration): (bool, bool, bool) = env
        .eval(
            r#"
            local name, icon, count, dispelName, duration, expirationTime,
                  source, isStealable, nspp, spellId = UnitBuff('player', 1)
            return name ~= nil, spellId ~= nil, duration ~= nil
        "#,
        )
        .unwrap();
    assert!(has_name, "UnitBuff should return name");
    assert!(has_spell_id, "UnitBuff should return spellId");
    assert!(has_duration, "UnitBuff should return duration");
}

#[test]
fn test_unit_buff_past_end_returns_nil() {
    let env = env();
    let is_nil: bool = env.eval("return UnitBuff('player', 100) == nil").unwrap();
    assert!(is_nil);
}

#[test]
fn test_unit_buff_non_player_returns_nil() {
    let env = env();
    let is_nil: bool = env.eval("return UnitBuff('target', 1) == nil").unwrap();
    assert!(is_nil);
}

#[test]
fn test_unit_buff_count_between_4_and_8() {
    let env = env();
    let count: i32 = env
        .eval(
            r#"
            local n = 0
            for i = 1, 20 do
                if UnitBuff('player', i) then n = n + 1 else break end
            end
            return n
        "#,
        )
        .unwrap();
    assert!(count >= 4, "Should have at least 4 buffs, got {}", count);
    assert!(count <= 8, "Should have at most 8 buffs, got {}", count);
}

// ============================================================================
// UnitAura
// ============================================================================

#[test]
fn test_unit_aura_helpful_returns_data() {
    let env = env();
    let name: String = env.eval("return UnitAura('player', 1, 'HELPFUL')").unwrap();
    assert!(!name.is_empty(), "UnitAura HELPFUL should return buff name");
}

#[test]
fn test_unit_aura_harmful_returns_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return UnitAura('player', 1, 'HARMFUL') == nil")
        .unwrap();
    assert!(is_nil);
}

// ============================================================================
// UnitDebuff
// ============================================================================

#[test]
fn test_unit_debuff_returns_nil() {
    let env = env();
    let is_nil: bool = env.eval("return UnitDebuff('player', 1) == nil").unwrap();
    assert!(is_nil);
}

// ============================================================================
// GetPlayerAuraBySpellID
// ============================================================================

#[test]
fn test_get_player_aura_by_spell_id_unknown() {
    let env = env();
    let is_nil: bool = env
        .eval("return GetPlayerAuraBySpellID(99999) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_get_player_aura_by_spell_id_finds_buff() {
    let env = env();
    let found: bool = env
        .eval(
            r#"
            local name, icon, count, dispelName, duration, expirationTime,
                  source, isStealable, nspp, spellId = UnitBuff('player', 1)
            if not spellId then return false end
            local data = GetPlayerAuraBySpellID(spellId)
            return data ~= nil and data.name == name
        "#,
        )
        .unwrap();
    assert!(found, "GetPlayerAuraBySpellID should find an active buff");
}

#[test]
fn test_get_player_aura_by_spell_id_table_fields() {
    let env = env();
    let ok: bool = env
        .eval(
            r#"
            local _, _, _, _, _, _, _, _, _, spellId = UnitBuff('player', 1)
            if not spellId then return false end
            local d = GetPlayerAuraBySpellID(spellId)
            return d.name ~= nil
                and d.icon ~= nil
                and d.spellId == spellId
                and d.isHelpful == true
                and d.auraInstanceID ~= nil
                and type(d.points) == "table"
        "#,
        )
        .unwrap();
    assert!(ok, "AuraData table should have expected fields");
}

// ============================================================================
// AuraUtil namespace
// ============================================================================

#[test]
fn test_aura_util_exists() {
    let env = env();
    let is_table: bool = env.eval("return type(AuraUtil) == 'table'").unwrap();
    assert!(is_table);
}

#[test]
fn test_aura_registration_surfaces_exist() {
    let env = env();
    let (
        get_aura_slots,
        get_player_aura_by_spell_id,
        unit_aura,
        aura_util_for_each,
        aura_util_find_by_name,
    ): (String, String, String, String, String) = env
        .eval(
            r#"
            return type(C_UnitAuras.GetAuraSlots),
                   type(GetPlayerAuraBySpellID),
                   type(UnitAura),
                   type(AuraUtil.ForEachAura),
                   type(AuraUtil.FindAuraByName)
        "#,
        )
        .unwrap();
    assert_eq!(get_aura_slots, "function");
    assert_eq!(get_player_aura_by_spell_id, "function");
    assert_eq!(unit_aura, "function");
    assert_eq!(aura_util_for_each, "function");
    assert_eq!(aura_util_find_by_name, "function");
}

#[test]
fn test_aura_util_for_each_aura() {
    let env = env();
    env.eval::<()>("AuraUtil.ForEachAura('player', 'HELPFUL', nil, function() end)")
        .unwrap();
}

#[test]
fn test_aura_util_find_aura_returns_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return AuraUtil.FindAura(function() end, 'player', 'HELPFUL') == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_aura_util_unpack_aura_data_returns_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return AuraUtil.UnpackAuraData(nil) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_aura_util_find_aura_by_name_returns_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return AuraUtil.FindAuraByName('Test', 'player', 'HELPFUL') == nil")
        .unwrap();
    assert!(is_nil);
}

// ============================================================================
// C_UnitAuras
// ============================================================================

#[test]
fn test_c_unit_auras_get_buff_data_by_index() {
    let env = env();
    let ok: bool = env
        .eval(
            r#"
            local data = C_UnitAuras.GetBuffDataByIndex("player", 1)
            return data ~= nil
                and data.name ~= nil
                and data.spellId ~= nil
                and data.isHelpful == true
        "#,
        )
        .unwrap();
    assert!(ok, "GetBuffDataByIndex(1) should return AuraData");
}

#[test]
fn test_c_unit_auras_get_buff_data_by_index_past_end() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_UnitAuras.GetBuffDataByIndex('player', 100) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_c_unit_auras_get_aura_data_by_index() {
    let env = env();
    let ok: bool = env
        .eval(
            r#"
            local data = C_UnitAuras.GetAuraDataByIndex("player", 1, "HELPFUL")
            return data ~= nil and data.name ~= nil and data.isHelpful == true
        "#,
        )
        .unwrap();
    assert!(ok, "GetAuraDataByIndex should return AuraData for HELPFUL");
}

#[test]
fn test_c_unit_auras_get_unit_auras_returns_iterable_table() {
    let env = env();
    let result: (bool, bool, bool) = env
        .eval(
            r#"
            local helpful = C_UnitAuras.GetUnitAuras("player", "HELPFUL")
            local harmful = C_UnitAuras.GetUnitAuras("player", "HARMFUL")
            return type(helpful) == "table",
                helpful[1] ~= nil and helpful[1].auraInstanceID ~= nil,
                type(harmful) == "table" and harmful[1] == nil
        "#,
        )
        .unwrap();

    assert!(result.0, "GetUnitAuras should return a table");
    assert!(result.1, "HELPFUL table should contain aura data");
    assert!(
        result.2,
        "empty filters should return an empty table, not nil"
    );
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_c_unit_auras_filters_aura_instances_by_polarity_and_player_source() {
    let env = env();
    env.state().borrow_mut().player.buffs = vec![
        AuraInfo {
            name: "Player Helpful Aura".into(),
            spell_id: 1001,
            icon: 1,
            duration: 30.0,
            expiration_time: 30.0,
            applications: 1,
            source_unit: "player".into(),
            is_helpful: true,
            is_raid: true,
            is_nameplate_only: false,
            is_stealable: false,
            can_apply_aura: true,
            is_from_player_or_player_pet: true,
            dispel_type: None,
            aura_instance_id: 101,
        },
        AuraInfo {
            name: "Party Helpful Aura".into(),
            spell_id: 1002,
            icon: 2,
            duration: 30.0,
            expiration_time: 30.0,
            applications: 1,
            source_unit: "party1".into(),
            is_helpful: true,
            is_raid: true,
            is_nameplate_only: false,
            is_stealable: false,
            can_apply_aura: true,
            is_from_player_or_player_pet: false,
            dispel_type: None,
            aura_instance_id: 102,
        },
    ];

    let result: (bool, bool, bool, bool, bool) = env
        .eval(
            r#"
            return C_UnitAuras.IsAuraFilteredOutByInstanceID("player", 101, "HELPFUL"),
                C_UnitAuras.IsAuraFilteredOutByInstanceID("player", 101, "HARMFUL"),
                C_UnitAuras.IsAuraFilteredOutByInstanceID("player", 101, "HELPFUL|PLAYER"),
                C_UnitAuras.IsAuraFilteredOutByInstanceID("player", 102, "HELPFUL|PLAYER"),
                C_UnitAuras.IsAuraFilteredOutByInstanceID("player", 999, "HELPFUL")
            "#,
        )
        .unwrap();

    assert_eq!(result, (false, true, false, true, true));
}

#[test]
fn test_c_unit_auras_get_aura_data_by_index_harmful() {
    let env = env();
    let is_nil: bool = env
        .eval(
            r#"
            return C_UnitAuras.GetAuraDataByIndex("player", 1, "HARMFUL") == nil
        "#,
        )
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_c_unit_auras_get_aura_slots() {
    let env = env();
    let (has_slots, token_nil): (bool, bool) = env
        .eval(
            r#"
            local token, s1 = C_UnitAuras.GetAuraSlots("player", "HELPFUL")
            return s1 ~= nil, token == nil
        "#,
        )
        .unwrap();
    assert!(has_slots, "GetAuraSlots should return at least one slot ID");
    assert!(token_nil, "Continuation token should be nil (all returned)");
}

#[test]
fn test_c_unit_auras_get_aura_data_by_slot() {
    let env = env();
    let ok: bool = env
        .eval(
            r#"
            local token, s1 = C_UnitAuras.GetAuraSlots("player", "HELPFUL")
            if not s1 then return false end
            local data = C_UnitAuras.GetAuraDataBySlot("player", s1)
            return data ~= nil and data.name ~= nil and data.auraInstanceID == s1
        "#,
        )
        .unwrap();
    assert!(ok, "GetAuraDataBySlot should return data for a valid slot");
}

#[test]
fn test_c_unit_auras_get_player_aura_by_spell_id() {
    let env = env();
    let ok: bool = env
        .eval(
            r#"
            local data = C_UnitAuras.GetBuffDataByIndex("player", 1)
            if not data then return false end
            local found = C_UnitAuras.GetPlayerAuraBySpellID(data.spellId)
            return found ~= nil and found.name == data.name
        "#,
        )
        .unwrap();
    assert!(ok, "C_UnitAuras.GetPlayerAuraBySpellID should find buff");
}

#[test]
fn test_c_unit_auras_get_aura_data_by_spell_name() {
    let env = env();
    let ok: bool = env
        .eval(
            r#"
            local data = C_UnitAuras.GetBuffDataByIndex("player", 1)
            if not data then return false end
            local found = C_UnitAuras.GetAuraDataBySpellName("player", data.name)
            return found ~= nil and found.spellId == data.spellId
        "#,
        )
        .unwrap();
    assert!(ok, "GetAuraDataBySpellName should find buff by name");
}

#[test]
fn test_c_unit_auras_slots_harmful_empty() {
    let env = env();
    let is_nil: bool = env
        .eval(
            r#"
            local token, s1 = C_UnitAuras.GetAuraSlots("player", "HARMFUL")
            return s1 == nil
        "#,
        )
        .unwrap();
    assert!(is_nil, "HARMFUL GetAuraSlots should return no slots");
}

#[test]
fn test_c_unit_auras_blocking_and_provider_switch_affect_visible_aura_queries() {
    let env = env();
    let result: (bool, bool, bool) = env
        .eval(
            r#"
            local helpfulToken, helpfulSlot = C_UnitAuras.GetAuraSlots("player", "HELPFUL")
            assert(helpfulSlot ~= nil, "expected a default helpful aura slot")
            local helpfulAura = C_UnitAuras.GetAuraDataBySlot("player", helpfulSlot)
            assert(helpfulAura ~= nil, "expected to resolve the default helpful aura")

            C_UnitAuras.AddBlockedAura("player", helpfulAura.auraInstanceID)
            local _, blockedFirstSlot = C_UnitAuras.GetAuraSlots("player", "HELPFUL")
            local blockedAura = C_UnitAuras.GetAuraDataByIndex("player", 1, "HELPFUL")
            local blockedHidden = blockedFirstSlot ~= helpfulAura.auraInstanceID
                and blockedAura ~= nil
                and blockedAura.auraInstanceID ~= helpfulAura.auraInstanceID

            local beforeSwitch = AuraUtil.GetAuraDataByAuraInstanceID("player", blockedAura.auraInstanceID)
            C_UnitAuras.SwitchAuraDataProvider()
            local switched = AuraUtil.GetAuraDataByAuraInstanceID("player", blockedAura.auraInstanceID)
            C_UnitAuras.ResetAuraDataProvider()
            local reset = AuraUtil.GetAuraDataByAuraInstanceID("player", blockedAura.auraInstanceID)

            return blockedHidden,
                beforeSwitch ~= nil and switched == nil,
                reset ~= nil and reset.auraInstanceID == blockedAura.auraInstanceID
            "#,
        )
        .unwrap();

    assert!(
        result.0,
        "AddBlockedAura should hide the blocked aura from slot/index iteration"
    );
    assert!(
        result.1,
        "SwitchAuraDataProvider should move AuraUtil off the default C_UnitAuras provider"
    );
    assert!(
        result.2,
        "ResetAuraDataProvider should restore AuraUtil to the default C_UnitAuras provider"
    );
}
