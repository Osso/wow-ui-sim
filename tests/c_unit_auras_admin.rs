//! Tests pinning the SimState-backed `C_UnitAuras` surface:
//! admin-added buffs must be findable by index, aura instance id, and
//! spell name.
//!
//! After the rewrite of `globals/auras.rs`, these five methods read
//! from `SimState.player.buffs` (populated by
//! `admin::add_buff` / `admin_buffs::add_buff`) instead of a
//! hard-coded fixture:
//!
//! - `GetAuraDataByIndex`
//! - `GetAuraDataByAuraInstanceID`
//! - `GetAuraDataBySpellName`
//! - `GetBuffDataByIndex`
//! - `GetDebuffDataByIndex`

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::AuraInfo;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

fn clear_buffs_and_insert(env: &WowLuaEnv, auras: Vec<AuraInfo>) {
    let mut state = env.state().borrow_mut();
    state.player.buffs.clear();
    state.player.buffs.extend(auras);
}

fn admin_aura(name: &str, spell_id: i32, aura_instance_id: i32, is_helpful: bool) -> AuraInfo {
    AuraInfo {
        name: name.into(),
        spell_id,
        icon: 135841,
        duration: 60.0,
        expiration_time: 60.0,
        applications: 1,
        source_unit: "player".into(),
        is_helpful,
        is_raid: is_helpful,
        is_nameplate_only: false,
        is_stealable: false,
        can_apply_aura: true,
        is_from_player_or_player_pet: true,
        dispel_type: None,
        aura_instance_id,
    }
}

fn admin_buff(name: &str, spell_id: i32, aura_instance_id: i32, is_helpful: bool) -> AuraInfo {
    admin_aura(name, spell_id, aura_instance_id, is_helpful)
}

fn dispellable_debuff(name: &str, spell_id: i32, aura_instance_id: i32, dispel_type: &str) -> AuraInfo {
    AuraInfo {
        dispel_type: Some(dispel_type.to_string()),
        ..admin_aura(name, spell_id, aura_instance_id, false)
    }
}

#[test]
fn admin_buff_is_findable_by_index_instance_id_and_name() {
    let env = env();
    clear_buffs_and_insert(&env, vec![admin_buff("Admin Buff", 11111, 555, true)]);

    let (by_index, by_instance, by_name): (String, String, String) = env
        .eval(
            r#"
            local a = C_UnitAuras.GetBuffDataByIndex("player", 1)
            local b = C_UnitAuras.GetAuraDataByAuraInstanceID("player", 555)
            local c = C_UnitAuras.GetAuraDataBySpellName("player", "Admin Buff")
            return a.name, b.name, c.name
            "#,
        )
        .unwrap();
    assert_eq!(by_index, "Admin Buff");
    assert_eq!(by_instance, "Admin Buff");
    assert_eq!(by_name, "Admin Buff");
}

#[test]
fn get_aura_data_by_index_helpful_walks_only_helpful_auras() {
    let env = env();
    clear_buffs_and_insert(
        &env,
        vec![
            admin_buff("Buff A", 1, 1, true),
            admin_buff("Debuff X", 2, 2, false),
            admin_buff("Buff B", 3, 3, true),
        ],
    );

    let (first, second): (String, String) = env
        .eval(
            r#"
            return C_UnitAuras.GetAuraDataByIndex("player", 1, "HELPFUL").name,
                   C_UnitAuras.GetAuraDataByIndex("player", 2, "HELPFUL").name
            "#,
        )
        .unwrap();
    assert_eq!(first, "Buff A", "helpful index skips debuffs");
    assert_eq!(second, "Buff B");
}

#[test]
fn get_aura_data_by_index_harmful_walks_only_debuffs() {
    let env = env();
    clear_buffs_and_insert(
        &env,
        vec![
            admin_buff("Buff A", 1, 1, true),
            admin_buff("Debuff X", 2, 2, false),
            admin_buff("Debuff Y", 3, 3, false),
        ],
    );

    let (first, second): (String, String) = env
        .eval(
            r#"
            return C_UnitAuras.GetAuraDataByIndex("player", 1, "HARMFUL").name,
                   C_UnitAuras.GetAuraDataByIndex("player", 2, "HARMFUL").name
            "#,
        )
        .unwrap();
    assert_eq!(first, "Debuff X");
    assert_eq!(second, "Debuff Y");
}

#[test]
fn get_buff_data_by_index_past_end_returns_nil() {
    let env = env();
    clear_buffs_and_insert(&env, vec![admin_buff("Only Buff", 1, 1, true)]);
    let is_nil: bool = env
        .eval(r#"return C_UnitAuras.GetBuffDataByIndex("player", 99) == nil"#)
        .unwrap();
    assert!(is_nil);
}

#[test]
fn get_debuff_data_by_index_returns_nil_when_no_debuffs() {
    let env = env();
    clear_buffs_and_insert(&env, vec![admin_buff("Buff Only", 1, 1, true)]);
    let is_nil: bool = env
        .eval(r#"return C_UnitAuras.GetDebuffDataByIndex("player", 1) == nil"#)
        .unwrap();
    assert!(is_nil);
}

#[test]
fn get_aura_dispel_type_color_is_transparent_for_non_dispellable_aura() {
    let env = env();
    clear_buffs_and_insert(&env, vec![admin_buff("Plain Debuff", 77, 777, false)]);

    let alpha: f64 = env
        .eval(
            r#"
            local aura = C_UnitAuras.GetAuraDataByIndex("player", 1, "HARMFUL")
            local color = C_UnitAuras.GetAuraDispelTypeColor("player", aura.auraInstanceID, CreateColor(1, 1, 1, 1))
            local _, _, _, a = color:GetRGBA()
            return a
            "#,
        )
        .unwrap();

    assert!(
        alpha.abs() < 0.001,
        "non-dispellable auras should produce an alpha-0 color"
    );
}

#[test]
fn dispellable_debuff_surfaces_type_icon_and_color() {
    let env = env();
    clear_buffs_and_insert(&env, vec![dispellable_debuff("Arcane Shock", 88, 888, "Magic")]);

    let (dispel_name, icon, r, g, b, a): (String, i64, f64, f64, f64, f64) = env
        .eval(
            r#"
            local aura = C_UnitAuras.GetAuraDataByIndex("player", 1, "HARMFUL")
            local color = C_UnitAuras.GetAuraDispelTypeColor("player", aura.auraInstanceID, CreateColor(1, 1, 1, 1))
            local r, g, b, a = color:GetRGBA()
            return aura.dispelName, aura.icon, r, g, b, a
            "#,
        )
        .unwrap();

    assert_eq!(dispel_name, "Magic");
    assert_eq!(icon, 135841);
    assert!((r - 0.2).abs() < 0.001);
    assert!((g - 0.6).abs() < 0.001);
    assert!((b - 1.0).abs() < 0.001);
    assert!((a - 1.0).abs() < 0.001);
}

#[test]
fn get_aura_data_by_aura_instance_id_returns_nil_for_unknown_id() {
    let env = env();
    clear_buffs_and_insert(&env, vec![admin_buff("Buff", 1, 111, true)]);
    let is_nil: bool = env
        .eval(r#"return C_UnitAuras.GetAuraDataByAuraInstanceID("player", 222) == nil"#)
        .unwrap();
    assert!(is_nil);
}

#[test]
fn get_aura_data_by_spell_name_returns_nil_for_unknown_name() {
    let env = env();
    clear_buffs_and_insert(&env, vec![admin_buff("Buff", 1, 111, true)]);
    let is_nil: bool = env
        .eval(r#"return C_UnitAuras.GetAuraDataBySpellName("player", "Nope") == nil"#)
        .unwrap();
    assert!(is_nil);
}

#[test]
fn get_aura_data_by_index_maw_returns_nil_for_ordinary_buffs() {
    // Blizzard_MawBuffs calls GetAuraDataByIndex("player", 1, "MAW"). Without
    // this guard, the filter fell through to HELPFUL and returned the first
    // ordinary player buff, causing ShouldShowMawBuffs() to display the
    // Torghast anima-power button outside Torghast.
    let env = env();
    clear_buffs_and_insert(
        &env,
        vec![admin_buff("Power Word: Fortitude", 21562, 1, true)],
    );
    let is_nil: bool = env
        .eval(r#"return C_UnitAuras.GetAuraDataByIndex("player", 1, "MAW") == nil"#)
        .unwrap();
    assert!(is_nil, "MAW filter must not return ordinary helpful buffs");
}

#[test]
fn aura_data_table_carries_retail_fields() {
    let env = env();
    clear_buffs_and_insert(&env, vec![admin_buff("Shield", 21562, 999, true)]);

    let (name, spell_id, instance_id, is_helpful, is_harmful): (String, i32, i32, bool, bool) = env
        .eval(
            r#"
            local a = C_UnitAuras.GetAuraDataByIndex("player", 1, "HELPFUL")
            return a.name, a.spellId, a.auraInstanceID, a.isHelpful, a.isHarmful
            "#,
        )
        .unwrap();
    assert_eq!(name, "Shield");
    assert_eq!(spell_id, 21562);
    assert_eq!(instance_id, 999);
    assert!(is_helpful);
    assert!(!is_harmful);
}

#[test]
fn admin_add_debuff_models_dispellable_player_debuff_and_fires_unit_aura() {
    let env = env();
    clear_buffs_and_insert(&env, vec![]);

    let (name, dispel, is_harmful, fired): (String, String, bool, bool) = env
        .eval(
            r#"
            local fired = false
            local listener = CreateFrame("Frame")
            listener:RegisterEvent("UNIT_AURA")
            listener:SetScript("OnEvent", function(_, event, unit, updateInfo)
                if unit == "player" and updateInfo and updateInfo.isFullUpdate then
                    fired = true
                end
            end)
            A_Admin.AddDebuff(589, "Shadow Word: Pain", "136207", 30, 1, "Magic")
            local a = C_UnitAuras.GetAuraDataByIndex("player", 1, "HARMFUL")
            return a.name, tostring(a.dispelName), a.isHarmful, fired
            "#,
        )
        .unwrap();
    assert_eq!(name, "Shadow Word: Pain");
    assert_eq!(dispel, "Magic");
    assert!(is_harmful);
    assert!(
        fired,
        "AddDebuff must fire UNIT_AURA with isFullUpdate so BuffFrame/DebuffFrame listeners refresh"
    );
}

#[test]
fn admin_add_debuff_without_dispel_type_has_nil_dispel_name() {
    let env = env();
    clear_buffs_and_insert(&env, vec![]);

    let dispel_is_nil: bool = env
        .eval(
            r#"
            A_Admin.AddDebuff(770, "Faerie Fire", "136033", 40, 1)
            local a = C_UnitAuras.GetAuraDataByIndex("player", 1, "HARMFUL")
            return a ~= nil and a.dispelName == nil
            "#,
        )
        .unwrap();
    assert!(dispel_is_nil, "omitted dispelType must surface as nil dispelName");
}

#[test]
fn admin_add_buff_and_debuff_accept_numeric_icon() {
    let env = env();
    clear_buffs_and_insert(&env, vec![]);

    let (buff_icon, debuff_icon): (f64, f64) = env
        .eval(
            r#"
            A_Admin.AddBuff(21562, "Power Word: Fortitude", 135987, 0, 0)
            A_Admin.AddDebuff(589, "Shadow Word: Pain", 136207, 30, 1, "Magic")
            local b = C_UnitAuras.GetAuraDataByIndex("player", 1, "HELPFUL")
            local d = C_UnitAuras.GetAuraDataByIndex("player", 1, "HARMFUL")
            return b.icon, d.icon
            "#,
        )
        .unwrap();
    assert_eq!(buff_icon, 135987.0, "documented numeric icon form must work for AddBuff");
    assert_eq!(debuff_icon, 136207.0, "documented numeric icon form must work for AddDebuff");
}
