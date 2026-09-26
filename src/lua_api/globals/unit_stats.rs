//! Unit-stat probe globals.
//!
//! Implements modeled combat-stat probes:
//!
//! - `UnitArmor`, `UnitAttackPower`, `UnitCriticalStrike`, `UnitDamage`,
//!   `UnitDefense`, `UnitDodge`, `UnitParry`, `UnitSpellHaste`, `UnitStat`,
//!   `UnitResistance`, `UnitRangedAttackPower`, `UnitRangedCriticalStrike`,
//!   `UnitRangedDamage`, `UnitReaction`, `UnitXP`, `UnitXPMax`.
//! Health/power maxima are registered with the shared unit-vitals queries.
//!
//! Stats come from one of three sources depending on the unit token:
//!
//! - `"player"` / `"pet"`: `SimState.player.stats` + level + equipped
//!   item-level, wrapped into a `UnitStats` snapshot.
//! - `"target"` / `"focus"` / `"mouseover"`: the corresponding
//!   `SimState.current_target` / `focus_target` / `mouseover_target`
//!   entry, with stats derived from level (no stat model on
//!   `TargetInfo`).
//! - `"party1"`..`"party4"`: `PartyMember` entries, stats derived from
//!   level.
//!
//! Units the sim doesn't model return zero-valued stats (the retail
//! convention when `UnitExists(unit)` is false).

use crate::lua_api::game_data::{PartyMember, TargetInfo};
use crate::lua_api::globals::unit_api::parse_party_index;
use crate::lua_api::methods::{borrow_state, val_to_string};
use crate::lua_api::state::{PlayerState, SimState};
use crate::lua_bridge::stack_val;
use rilua::vm::closure::RustFn;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

/// Snapshot of the stats we expose for a unit. Every field is derivable
/// from `PlayerState` / `TargetInfo` / `PartyMember`; callers pick out
/// whichever subset each global needs.
#[derive(Clone, Copy)]
struct UnitStats {
    level: i32,
    health_max: i32,
    armor: i32,
    attack_power: i32,
    crit_rating: i32,
    haste_rating: i32,
    strength: f64,
    agility: f64,
    stamina: f64,
    intellect: f64,
    damage_min: f64,
    damage_max: f64,
    reaction: i32,
    xp: i64,
    xp_max: i64,
}

/// The Default impl is the "unit doesn't exist" fallback: zero for
/// numerics, friendly (5) for reaction. Retail returns 0 from stat
/// probes when the unit token is unknown.
impl Default for UnitStats {
    fn default() -> Self {
        Self {
            level: 0,
            health_max: 0,
            armor: 0,
            attack_power: 0,
            crit_rating: 0,
            haste_rating: 0,
            strength: 0.0,
            agility: 0.0,
            stamina: 0.0,
            intellect: 0.0,
            damage_min: 0.0,
            damage_max: 0.0,
            reaction: 4, // neutral
            xp: 0,
            xp_max: 0,
        }
    }
}

fn player_stats(player: &PlayerState) -> UnitStats {
    let level_f = player.level as f64;
    let ap = (player.stats.strength + player.stats.agility) + level_f * 10.0;
    let weapon_low = 50.0 + level_f * 5.0;
    let weapon_high = 80.0 + level_f * 7.0;
    UnitStats {
        level: player.level,
        health_max: player.health_max,
        armor: player.stats.armor,
        attack_power: ap as i32,
        crit_rating: player.stats.crit_rating,
        haste_rating: player.stats.haste_rating,
        strength: player.stats.strength,
        agility: player.stats.agility,
        stamina: player.stats.stamina,
        intellect: player.stats.intellect,
        damage_min: weapon_low + ap * 0.1,
        damage_max: weapon_high + ap * 0.1,
        reaction: 5, // friendly to self
        xp: player.xp,
        xp_max: player.xp_max,
    }
}

fn target_stats(target: &TargetInfo) -> UnitStats {
    let level_f = target.level.max(1) as f64;
    let ap = level_f * 80.0;
    UnitStats {
        level: target.level,
        health_max: target.health_max,
        armor: (level_f * 150.0) as i32,
        attack_power: ap as i32,
        crit_rating: (level_f * 4.0) as i32,
        haste_rating: (level_f * 3.0) as i32,
        strength: level_f * 8.0,
        agility: level_f * 8.0,
        stamina: level_f * 15.0,
        intellect: level_f * 8.0,
        damage_min: level_f * 20.0,
        damage_max: level_f * 35.0,
        reaction: target.reaction,
        xp: 0,
        xp_max: 0,
    }
}

fn party_stats(member: &PartyMember) -> UnitStats {
    let level_f = member.level.max(1) as f64;
    let ap = level_f * 120.0;
    UnitStats {
        level: member.level,
        health_max: member.health_max,
        armor: (level_f * 180.0) as i32,
        attack_power: ap as i32,
        crit_rating: (level_f * 5.0) as i32,
        haste_rating: (level_f * 4.0) as i32,
        strength: level_f * 10.0,
        agility: level_f * 10.0,
        stamina: level_f * 20.0,
        intellect: level_f * 10.0,
        damage_min: level_f * 25.0,
        damage_max: level_f * 45.0,
        reaction: 5, // friendly
        xp: 0,
        xp_max: 0,
    }
}

fn lookup_unit_stats(sim: &SimState, unit: &str) -> UnitStats {
    match unit {
        "player" | "pet" => player_stats(&sim.player),
        "target" => sim
            .current_target
            .as_ref()
            .map(target_stats)
            .unwrap_or_default(),
        "focus" => sim
            .current_focus
            .as_ref()
            .map(target_stats)
            .unwrap_or_default(),
        // The sim has no mouseover target model; fall back to zero stats
        // to match retail's "unit doesn't exist" behaviour.
        "mouseover" => UnitStats::default(),
        other => parse_party_index(other)
            .and_then(|index| sim.party_members.get(index))
            .map(party_stats)
            .unwrap_or_default(),
    }
}

fn unit_token_at(state: &LuaState, index: i32) -> String {
    val_to_string(state, stack_val(state, index)).unwrap_or_else(|| "player".to_string())
}

fn unit_token(state: &LuaState) -> String {
    unit_token_at(state, 1)
}

fn stats_for_unit(state: &LuaState, unit: &str) -> UnitStats {
    let sim = borrow_state(state).expect("sim state should exist");
    lookup_unit_stats(&sim, unit)
}

fn stats_for(state: &LuaState) -> UnitStats {
    let unit = unit_token(state);
    stats_for_unit(state, &unit)
}

fn stack_i32(state: &LuaState, index: i32) -> i32 {
    match stack_val(state, index) {
        Val::Num(n) => n as i32,
        _ => 0,
    }
}

fn stack_f64(state: &LuaState, index: i32) -> f64 {
    match stack_val(state, index) {
        Val::Num(n) => n,
        _ => 0.0,
    }
}

fn active_spec_primary_stat(state: &LuaState) -> Option<i32> {
    let (class_id, active_spec_index) = {
        let sim = borrow_state(state).ok()?;
        (
            sim.player.class_index.max(1) as u32,
            sim.player.active_spec_index.max(1),
        )
    };
    crate::specializations::specs_for_class(class_id)
        .nth((active_spec_index - 1) as usize)
        .map(|spec| spec.primary_stat as i32)
}

fn attack_power_for_stat_value(state: &LuaState, stat_index: i32, stat_value: f64) -> f64 {
    let value = stat_value.max(0.0);
    match active_spec_primary_stat(state) {
        Some(1) if stat_index == 1 => value, // strength specs
        Some(2) if stat_index == 2 => value, // agility specs
        // No current sim spec maps spell power back into attack power.
        Some(4) if stat_index == 4 => 0.0,
        // Fallback when spec metadata is unavailable.
        None if matches!(stat_index, 1 | 2) => value,
        _ => 0.0,
    }
}

fn has_ap_effects_spell_power_model(state: &LuaState) -> bool {
    matches!(active_spec_primary_stat(state), Some(1 | 2))
}

fn has_sp_effects_attack_power_model(_state: &LuaState) -> bool {
    false
}

fn baseline_hp_per_stamina(level: i32) -> f64 {
    let level = level.max(1) as f64;
    2.5 + level * 0.05
}

fn unit_hp_per_stamina_value(stats: UnitStats) -> f64 {
    if stats.stamina <= 0.0 {
        return 0.0;
    }
    baseline_hp_per_stamina(stats.level)
}

fn unit_max_health_modifier_value(stats: UnitStats) -> f64 {
    if stats.stamina <= 0.0 {
        return 1.0;
    }
    let base_health = stats.stamina * unit_hp_per_stamina_value(stats);
    if base_health <= 0.0 {
        return 1.0;
    }
    (stats.health_max as f64 / base_health).max(0.0)
}

pub(crate) fn secondary_power_max(power_type: i32) -> i32 {
    match power_type {
        4 => 7,
        5 => 6,
        9 => 5,
        16 => 4,
        _ => 5,
    }
}

// ── Stat probes ──────────────────────────────────────────────────────────────

/// `UnitArmor(unit)` — retail: `(base, effectiveArmor, armor, posBuff, negBuff)`.
/// We model armor as a single value and report it as base/effective/total,
/// with zero buffs.
fn unit_armor(state: &mut LuaState) -> LuaResult<u32> {
    let stats = stats_for(state);
    state.push(Val::Num(stats.armor as f64));
    state.push(Val::Num(stats.armor as f64));
    state.push(Val::Num(stats.armor as f64));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    Ok(5)
}

/// `UnitAttackPower(unit)` — retail: `(base, posBuff, negBuff)`.
fn unit_attack_power(state: &mut LuaState) -> LuaResult<u32> {
    let stats = stats_for(state);
    state.push(Val::Num(stats.attack_power as f64));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    Ok(3)
}

/// `UnitRangedAttackPower(unit)` — same retail shape as `UnitAttackPower`.
fn unit_ranged_attack_power(state: &mut LuaState) -> LuaResult<u32> {
    unit_attack_power(state)
}

/// `UnitCriticalStrike(unit)` — retail returns a single percent-chance
/// number. Approximates crit % from the rating.
fn unit_critical_strike(state: &mut LuaState) -> LuaResult<u32> {
    let stats = stats_for(state);
    state.push(Val::Num(rating_to_percent(stats.crit_rating)));
    Ok(1)
}

/// `UnitRangedCriticalStrike(unit)` — alias for melee crit in this sim.
fn unit_ranged_critical_strike(state: &mut LuaState) -> LuaResult<u32> {
    unit_critical_strike(state)
}

/// `UnitSpellHaste(unit)` — retail returns a percent-chance number from
/// the player's haste rating.
fn unit_spell_haste(state: &mut LuaState) -> LuaResult<u32> {
    let stats = stats_for(state);
    state.push(Val::Num(rating_to_percent(stats.haste_rating)));
    Ok(1)
}

/// `UnitDamage(unit)` — retail: `(minDmg, maxDmg, minOffHand, maxOffHand,
/// physicalBonusPos, physicalBonusNeg, percent)`. Off-hand and bonuses
/// are zero in this sim.
fn unit_damage(state: &mut LuaState) -> LuaResult<u32> {
    let stats = stats_for(state);
    state.push(Val::Num(stats.damage_min));
    state.push(Val::Num(stats.damage_max));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(Val::Num(1.0)); // percent (multiplier)
    Ok(7)
}

/// `UnitRangedDamage(unit)` — same shape as `UnitDamage`.
fn unit_ranged_damage(state: &mut LuaState) -> LuaResult<u32> {
    let stats = stats_for(state);
    state.push(Val::Num(2.0)); // ranged attack speed
    state.push(Val::Num(stats.damage_min));
    state.push(Val::Num(stats.damage_max));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(Val::Num(1.0));
    Ok(6)
}

/// `UnitAttackSpeed(unit)` — `(mainHandSpeed, offHandSpeed)`.
fn unit_attack_speed(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(2.0));
    state.push(Val::Num(2.0));
    Ok(2)
}

/// `UnitRangedAttack(unit)` — `(baseSkill, modifier)`.
fn unit_ranged_attack(state: &mut LuaState) -> LuaResult<u32> {
    let stats = stats_for(state);
    state.push(Val::Num((stats.level * 5).max(0) as f64));
    state.push(Val::Num(0.0));
    Ok(2)
}

/// `UnitDefense(unit)` — retail's old API, returns skill rating. We
/// approximate as `level * 5` (the pre-MoP formula).
fn unit_defense(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(defense_skill(stats_for(state))));
    Ok(1)
}

fn defense_skill(stats: UnitStats) -> f64 {
    (stats.level * 5).max(0) as f64
}

/// Forever's `UnitDefenseSkill(unit)` exposes base skill and modifier.
#[cfg(feature = "client-wowforever")]
fn unit_defense_skill(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(defense_skill(stats_for(state))));
    state.push(Val::Num(0.0)); // No defense bonus source is modeled.
    Ok(2)
}

/// `UnitDodge(unit)` — retail returns a percent-chance number.
fn unit_dodge(state: &mut LuaState) -> LuaResult<u32> {
    let stats = stats_for(state);
    state.push(Val::Num(base_avoidance_percent(stats.agility)));
    Ok(1)
}

/// `UnitParry(unit)` — retail returns a percent-chance number.
fn unit_parry(state: &mut LuaState) -> LuaResult<u32> {
    let stats = stats_for(state);
    state.push(Val::Num(base_avoidance_percent(stats.strength)));
    Ok(1)
}

/// `UnitReaction(unit, otherUnit)` — retail returns 1-8 reaction. We
/// always compare relative to the player, so `otherUnit` is informational
/// only (sim has no per-pair reaction table).
fn unit_reaction(state: &mut LuaState) -> LuaResult<u32> {
    let stats = stats_for(state);
    state.push(Val::Num(stats.reaction as f64));
    Ok(1)
}

/// `UnitXP(unit)` — player-only in retail; returns 0 for other units.
fn unit_xp(state: &mut LuaState) -> LuaResult<u32> {
    let stats = stats_for(state);
    state.push(Val::Num(stats.xp as f64));
    Ok(1)
}

/// `UnitXPMax(unit)` — player-only in retail; returns 0 for other units.
fn unit_xp_max(state: &mut LuaState) -> LuaResult<u32> {
    let stats = stats_for(state);
    state.push(Val::Num(stats.xp_max as f64));
    Ok(1)
}

/// `UnitStat(unit, statIndex)` — retail: `(stat, base, positive, negative)`.
/// `statIndex` 1 = Strength, 2 = Agility, 3 = Stamina, 4 = Intellect,
/// 5 = Spirit (Forever player state only; zero in other profiles).
fn unit_stat(state: &mut LuaState) -> LuaResult<u32> {
    let stat_index = match stack_val(state, 2) {
        Val::Num(n) => n as i32,
        _ => 0,
    };
    let stats = stats_for(state);
    let value = match stat_index {
        1 => stats.strength,
        2 => stats.agility,
        3 => stats.stamina,
        4 => stats.intellect,
        #[cfg(feature = "client-wowforever")]
        5 if unit_token(state) == "player" => borrow_state(state)?.player.stats.spirit,
        _ => 0.0,
    };
    state.push(Val::Num(value));
    state.push(Val::Num(value));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    Ok(4)
}

/// `UnitResistance(unit, resistanceIndex)` — retail: `(base, resistance,
/// positive, negative)`. The sim doesn't model resistances; always 0.
fn unit_resistance(state: &mut LuaState) -> LuaResult<u32> {
    let _ = state;
    for _ in 0..4 {
        state.push(Val::Num(0.0));
    }
    Ok(4)
}

// ── PaperDoll stat helpers ───────────────────────────────────────────────────

/// `GetAttackPowerForStat(statIndex, value)` — helper used by Blizzard's
/// PaperDoll tooltip generation.
fn get_attack_power_for_stat(state: &mut LuaState) -> LuaResult<u32> {
    let stat_index = stack_i32(state, 1);
    let stat_value = stack_f64(state, 2);
    let attack_power = attack_power_for_stat_value(state, stat_index, stat_value);
    state.push(Val::Num(attack_power));
    Ok(1)
}

/// `GetDodgeChanceFromAttribute()` — attribute-only dodge contribution.
fn get_dodge_chance_from_attribute(state: &mut LuaState) -> LuaResult<u32> {
    let stats = stats_for(state);
    state.push(Val::Num((stats.agility / 100.0).max(0.0)));
    Ok(1)
}

/// `GetParryChanceFromAttribute()` — attribute-only parry contribution.
fn get_parry_chance_from_attribute(state: &mut LuaState) -> LuaResult<u32> {
    let stats = stats_for(state);
    state.push(Val::Num((stats.strength / 100.0).max(0.0)));
    Ok(1)
}

fn get_dodge_chance(state: &mut LuaState) -> LuaResult<u32> {
    let stats = stats_for(state);
    state.push(Val::Num(base_avoidance_percent(stats.agility)));
    Ok(1)
}

fn get_parry_chance(state: &mut LuaState) -> LuaResult<u32> {
    let stats = stats_for(state);
    state.push(Val::Num(base_avoidance_percent(stats.strength)));
    Ok(1)
}

fn get_block_chance(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

fn get_shield_block(state: &mut LuaState) -> LuaResult<u32> {
    let stats = stats_for(state);
    state.push(Val::Num(stats.armor as f64));
    Ok(1)
}

/// `UnitHPPerStamina(unit)` — fixed multiplier used by PaperDoll stamina text.
fn unit_hp_per_stamina(state: &mut LuaState) -> LuaResult<u32> {
    let unit = unit_token_at(state, 1);
    let stats = stats_for_unit(state, &unit);
    state.push(Val::Num(unit_hp_per_stamina_value(stats)));
    Ok(1)
}

/// `GetUnitMaxHealthModifier(unit)` — percentage multiplier over the level-
/// scaled stamina baseline used by `UnitHPPerStamina`.
fn get_unit_max_health_modifier(state: &mut LuaState) -> LuaResult<u32> {
    let unit = unit_token_at(state, 1);
    let stats = stats_for_unit(state, &unit);
    state.push(Val::Num(unit_max_health_modifier_value(stats)));
    Ok(1)
}

/// `GetUnitTotalModifiedMaxHealthPercent(unit)` — percent change from the
/// unmodified max-health baseline. The current stat model has no bonus max
/// health modifiers, so the seeded default is 0.
fn get_unit_total_modified_max_health_percent(state: &mut LuaState) -> LuaResult<u32> {
    let _ = unit_token_at(state, 1);
    state.push(Val::Num(0.0));
    Ok(1)
}

/// `HasAPEffectsSpellPower()` and `HasSPEffectsAttackPower()` return class/spec
/// crossover flags in retail. The simulator currently models no crossover.
fn has_ap_effects_spell_power(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(has_ap_effects_spell_power_model(state)));
    Ok(1)
}

fn has_sp_effects_attack_power(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(has_sp_effects_attack_power_model(state)));
    Ok(1)
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Approximate rating-to-percent conversion used by the sim's stat
/// model. Retail uses per-expansion divisors; 180 is an acceptable
/// middle-ground for The War Within crit/haste without hard-coding a
/// DR curve.
fn rating_to_percent(rating: i32) -> f64 {
    (rating as f64) / 180.0
}

fn base_avoidance_percent(primary_stat: f64) -> f64 {
    5.0 + primary_stat / 100.0
}

fn register_paperdoll_helpers(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetAttackPowerForStat", get_attack_power_for_stat)?;
    LuaApiMut::register_function(
        lua,
        "GetDodgeChanceFromAttribute",
        get_dodge_chance_from_attribute,
    )?;
    LuaApiMut::register_function(
        lua,
        "GetParryChanceFromAttribute",
        get_parry_chance_from_attribute,
    )?;
    LuaApiMut::register_function(lua, "GetDodgeChance", get_dodge_chance)?;
    LuaApiMut::register_function(lua, "GetParryChance", get_parry_chance)?;
    LuaApiMut::register_function(lua, "GetBlockChance", get_block_chance)?;
    LuaApiMut::register_function(lua, "GetShieldBlock", get_shield_block)?;
    LuaApiMut::register_function(lua, "UnitHPPerStamina", unit_hp_per_stamina)?;
    LuaApiMut::register_function(
        lua,
        "GetUnitMaxHealthModifier",
        get_unit_max_health_modifier,
    )?;
    LuaApiMut::register_function(
        lua,
        "GetUnitTotalModifiedMaxHealthPercent",
        get_unit_total_modified_max_health_percent,
    )?;
    LuaApiMut::register_function(lua, "HasAPEffectsSpellPower", has_ap_effects_spell_power)?;
    LuaApiMut::register_function(lua, "HasSPEffectsAttackPower", has_sp_effects_attack_power)?;
    Ok(())
}

const UNIT_STAT_GLOBALS: &[(&str, RustFn)] = &[
    ("UnitArmor", unit_armor),
    ("UnitAttackPower", unit_attack_power),
    ("UnitRangedAttackPower", unit_ranged_attack_power),
    ("UnitCriticalStrike", unit_critical_strike),
    ("UnitRangedCriticalStrike", unit_ranged_critical_strike),
    ("UnitSpellHaste", unit_spell_haste),
    ("UnitDamage", unit_damage),
    ("UnitRangedDamage", unit_ranged_damage),
    ("UnitAttackSpeed", unit_attack_speed),
    ("UnitRangedAttack", unit_ranged_attack),
    ("UnitDefense", unit_defense),
    #[cfg(feature = "client-wowforever")]
    ("UnitDefenseSkill", unit_defense_skill),
    ("UnitDodge", unit_dodge),
    ("UnitParry", unit_parry),
    ("UnitReaction", unit_reaction),
    ("UnitXP", unit_xp),
    ("UnitXPMax", unit_xp_max),
    ("UnitStat", unit_stat),
    ("UnitResistance", unit_resistance),
];

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    for &(name, function) in UNIT_STAT_GLOBALS {
        LuaApiMut::register_function(lua, name, function)?;
    }
    register_paperdoll_helpers(lua)?;
    Ok(())
}
