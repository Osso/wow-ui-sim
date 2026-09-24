//! Forever Camelot character-panel stat-row inputs, exercised through Lua.

#[cfg(feature = "client-wowforever")]
mod forever {
    use wow_ui_sim::lua_api::WowLuaEnv;
    use wow_ui_sim::lua_api::state::{EquippedItem, TargetInfo};

    fn env() -> WowLuaEnv {
        WowLuaEnv::new().expect("Forever environment")
    }

    fn item(item_id: u32) -> EquippedItem {
        EquippedItem {
            item_id,
            enchant_id: 0,
            gem_ids: [0; 3],
        }
    }

    #[test]
    fn spirit_constant_and_required_globals_have_forever_shapes() {
        let env = env();
        let (spirit, types, ranged_arity, defense_arity): (i32, bool, i32, i32) = env
            .eval(
                "local function count(...) return select('#', ...) end; \
                 return LE_UNIT_STAT_SPIRIT, \
                 type(IsDualWielding) == 'function' and type(IsRangedWeapon) == 'function' \
                 and type(GetOverrideAPBySpellPower) == 'function' \
                 and type(GetOverrideSpellPowerByAP) == 'function' \
                 and type(GetRangedHitModifier) == 'function' \
                 and type(GetArmorPenetration) == 'function' \
                 and type(GetSpellPenetration) == 'function' \
                 and type(UnitDefenseSkill) == 'function', \
                 count(GetRangedHaste()), count(UnitDefenseSkill('player'))",
            )
            .unwrap();
        assert_eq!(spirit, 5);
        assert!(types);
        assert_eq!((ranged_arity, defense_arity), (2, 2));
    }

    #[test]
    fn configured_modifiers_and_quiver_haste_are_returned_in_row_units() {
        let env = env();
        let baseline_haste: f64 = env.eval("return GetHaste()").unwrap();
        {
            let state = env.state();
            let mut sim = state.borrow_mut();
            let stats = &mut sim.player.stats;
            stats.ranged_hit_modifier_pct = 1.25;
            stats.armor_penetration = 250.0;
            stats.spell_penetration = 42.0;
            stats.spell_power_to_attack_power = 0.45;
            stats.attack_power_to_spell_power = 0.75;
            stats.quiver_haste_pct = 3.5;
        }
        let (hit, armor, spell, ap, sp, haste, quiver, melee): (
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
        ) = env
            .eval(
                "local haste, quiver = GetRangedHaste(); return GetRangedHitModifier(), \
             GetArmorPenetration(), GetSpellPenetration(), GetOverrideAPBySpellPower(), \
             GetOverrideSpellPowerByAP(), haste, quiver, GetHaste()",
            )
            .unwrap();
        assert_eq!((hit, armor, spell, ap, sp), (1.25, 250.0, 42.0, 0.45, 0.75));
        assert_eq!(
            (haste, quiver, melee),
            (baseline_haste, 3.5, baseline_haste)
        );
        let fresh = WowLuaEnv::new().expect("independent Forever environment");
        let (hit, armor, spell, ap, sp, quiver): (f64, f64, f64, f64, f64, f64) = fresh
            .eval("local _, q = GetRangedHaste(); return GetRangedHitModifier(), GetArmorPenetration(), GetSpellPenetration(), GetOverrideAPBySpellPower(), GetOverrideSpellPowerByAP(), q")
            .unwrap();
        assert_eq!(
            (hit, armor, spell, ap, sp, quiver),
            (0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
        );
    }

    #[test]
    fn defense_skill_resolves_player_target_and_missing_unit_without_changing_unit_defense() {
        let env = env();
        env.state().borrow_mut().player.level = 12;
        let (base, modifier, legacy): (i32, i32, i32) = env
            .eval("local base, mod = UnitDefenseSkill('player'); return base, mod, UnitDefense('player')")
            .unwrap();
        assert_eq!((base, modifier, legacy), (60, 0, 60));
        env.state().borrow_mut().current_target = Some(TargetInfo {
            unit_id: "target".into(),
            name: "Target".into(),
            health: 100,
            health_max: 100,
            power: 0,
            power_max: 0,
            power_type: 0,
            power_type_name: "MANA".into(),
            is_player: false,
            is_enemy: true,
            guid: "Creature-0-0".into(),
            level: 23,
            class_index: 1,
            classification: "normal".into(),
            creature_type: "Humanoid".into(),
            reaction: 2,
            interaction: Default::default(),
        });
        let (target, target_mod, missing, missing_mod): (i32, i32, i32, i32) = env
            .eval("local a, b = UnitDefenseSkill('target'); local c, d = UnitDefenseSkill('mouseover'); return a, b, c, d")
            .unwrap();
        assert_eq!((target, target_mod, missing, missing_mod), (115, 0, 0, 0));
    }

    #[test]
    fn weapon_predicates_follow_equipment_metadata_and_removal() {
        let env = env();
        let (dual, ranged): (bool, bool) = env
            .eval("return IsDualWielding(), IsRangedWeapon()")
            .unwrap();
        assert_eq!((dual, ranged), (false, false));
        {
            let state = env.state();
            let mut sim = state.borrow_mut();
            sim.player.equipped_items.insert(16, item(221165)); // one-hand weapon, invtype 13
            sim.player.equipped_items.insert(17, item(133503)); // shield, invtype 14
            sim.player.equipped_items.insert(18, item(237732)); // bow, invtype 15
        }
        let (dual, ranged): (bool, bool) = env
            .eval("return IsDualWielding(), IsRangedWeapon()")
            .unwrap();
        assert_eq!((dual, ranged), (false, true));
        env.state()
            .borrow_mut()
            .player
            .equipped_items
            .insert(17, item(258525)); // one-hand weapon, invtype 13
        let (dual, ranged): (bool, bool) = env
            .eval("return IsDualWielding(), IsRangedWeapon()")
            .unwrap();
        assert_eq!((dual, ranged), (true, true));
        env.state().borrow_mut().player.equipped_items.remove(&17);
        env.state().borrow_mut().player.equipped_items.remove(&18);
        let (dual, ranged): (bool, bool) = env
            .eval("return IsDualWielding(), IsRangedWeapon()")
            .unwrap();
        assert_eq!((dual, ranged), (false, false));
        env.state()
            .borrow_mut()
            .player
            .equipped_items
            .insert(18, item(234492)); // ranged-right, invtype 26
        assert!(env.eval::<bool>("return IsRangedWeapon()").unwrap());
    }
}
