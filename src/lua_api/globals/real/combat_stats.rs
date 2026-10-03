//! State-backed character combat-rating globals backed by `PlayerState.stats`.

#[cfg(feature = "client-retail")]
mod retail_inputs;

use crate::c_api::c_secrets::push_stat_number;
#[cfg(feature = "client-wowforever")]
use crate::lua_api::globals::targeting_verbs::resolve_unit_snapshot;
use crate::lua_api::methods::borrow_state;
use crate::lua_api::state_types::CharacterStats;
use crate::lua_bridge::FromStack;
use rilua::vm::closure::RustFn;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

const CRIT_RATING_INDEX: i32 = 9;
const HASTE_RATING_INDEX: i32 = 6;
const MASTERY_RATING_INDEX: i32 = 26;
const VERSATILITY_DAMAGE_RATING_INDEX: i32 = 14;
const VERSATILITY_DAMAGE_TAKEN_RATING_INDEX: i32 = 29;
const SPEED_RATING_INDEX: i32 = 13;
const LIFESTEAL_RATING_INDEX: i32 = 17;
const AVOIDANCE_RATING_INDEX: i32 = 21;

const CRIT_RATING_PER_PERCENT: f64 = 180.0;
const HASTE_RATING_PER_PERCENT: f64 = 170.0;
const MASTERY_RATING_PER_PERCENT: f64 = 130.0;
const VERSATILITY_RATING_PER_PERCENT: f64 = 205.0;
// GUESS: existing common conversion, not native-verified or user-selected.
const TERTIARY_RATING_PER_PERCENT: f64 = 180.0;
const TERTIARY_RATING_INPUTS_ENABLED: bool = cfg!(feature = "retail-12-0-5");

fn combat_rating_for(state: &mut LuaState, rating_index: i32) -> i32 {
    let Ok(sim) = borrow_state(state) else {
        return 0;
    };
    match rating_index {
        SPEED_RATING_INDEX if TERTIARY_RATING_INPUTS_ENABLED => sim.player.stats.speed_rating,
        LIFESTEAL_RATING_INDEX if TERTIARY_RATING_INPUTS_ENABLED => sim.player.stats.leech_rating,
        AVOIDANCE_RATING_INDEX if TERTIARY_RATING_INPUTS_ENABLED => {
            sim.player.stats.avoidance_rating
        }
        CRIT_RATING_INDEX => sim.player.stats.crit_rating,
        HASTE_RATING_INDEX => sim.player.stats.haste_rating,
        MASTERY_RATING_INDEX => sim.player.stats.mastery_rating,
        VERSATILITY_DAMAGE_RATING_INDEX | VERSATILITY_DAMAGE_TAKEN_RATING_INDEX => {
            sim.player.stats.versatility_rating
        }
        _ => 0,
    }
}

fn rating_bonus_for_value(rating_index: i32, rating_value: f64) -> f64 {
    let divisor = match rating_index {
        SPEED_RATING_INDEX | LIFESTEAL_RATING_INDEX | AVOIDANCE_RATING_INDEX => {
            TERTIARY_RATING_PER_PERCENT
        }
        CRIT_RATING_INDEX => CRIT_RATING_PER_PERCENT,
        HASTE_RATING_INDEX => HASTE_RATING_PER_PERCENT,
        MASTERY_RATING_INDEX => MASTERY_RATING_PER_PERCENT,
        VERSATILITY_DAMAGE_RATING_INDEX | VERSATILITY_DAMAGE_TAKEN_RATING_INDEX => {
            VERSATILITY_RATING_PER_PERCENT
        }
        _ => 180.0,
    };
    (rating_value / divisor).max(0.0)
}

fn get_combat_rating(state: &mut LuaState) -> LuaResult<u32> {
    let rating_index = i32::from_stack(state, 1)?;
    let rating = combat_rating_for(state, rating_index);
    push_stat_number(state, rating as f64)?;
    Ok(1)
}

fn get_combat_rating_bonus(state: &mut LuaState) -> LuaResult<u32> {
    let rating_index = i32::from_stack(state, 1)?;
    let bonus = if TERTIARY_RATING_INPUTS_ENABLED
        && matches!(
            rating_index,
            SPEED_RATING_INDEX | LIFESTEAL_RATING_INDEX | AVOIDANCE_RATING_INDEX
        ) {
        let rating = combat_rating_for(state, rating_index);
        rating_bonus_for_value(rating_index, rating as f64)
    } else {
        let Ok(sim) = borrow_state(state) else {
            return Ok(0);
        };
        match rating_index {
            CRIT_RATING_INDEX => sim.player.stats.crit_pct(),
            HASTE_RATING_INDEX => sim.player.stats.haste_pct(),
            MASTERY_RATING_INDEX => sim.player.stats.mastery_pct(),
            VERSATILITY_DAMAGE_RATING_INDEX | VERSATILITY_DAMAGE_TAKEN_RATING_INDEX => {
                sim.player.stats.versatility_pct()
            }
            _ => 0.0,
        }
    };
    push_stat_number(state, bonus)?;
    Ok(1)
}

fn get_combat_rating_bonus_for_value(state: &mut LuaState) -> LuaResult<u32> {
    let rating_index = i32::from_stack(state, 1)?;
    let rating_value = f64::from_stack(state, 2)?;
    state.push(Val::Num(rating_bonus_for_value(rating_index, rating_value)));
    Ok(1)
}

fn get_crit_chance(state: &mut LuaState) -> LuaResult<u32> {
    let crit = borrow_state(state)?.player.stats.crit_pct() + 5.0;
    push_stat_number(state, crit)?;
    Ok(1)
}

fn get_spell_crit_chance(state: &mut LuaState) -> LuaResult<u32> {
    let _school = Option::<i32>::from_stack(state, 1)?;
    get_crit_chance(state)
}

fn get_ranged_crit_chance(state: &mut LuaState) -> LuaResult<u32> {
    get_crit_chance(state)
}

fn get_crit_chance_from_agility(state: &mut LuaState) -> LuaResult<u32> {
    let _unit = String::from_stack(state, 1).unwrap_or_default();
    let crit = borrow_state(state)?.player.stats.crit_pct();
    state.push(Val::Num(crit));
    Ok(1)
}

fn get_spell_crit_chance_from_intellect(state: &mut LuaState) -> LuaResult<u32> {
    let _unit = String::from_stack(state, 1).unwrap_or_default();
    let crit = borrow_state(state)?.player.stats.crit_pct();
    state.push(Val::Num(crit));
    Ok(1)
}

fn get_crit_chance_provides_parry_effect(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

fn get_haste(state: &mut LuaState) -> LuaResult<u32> {
    let haste = borrow_state(state)?.player.stats.haste_pct();
    push_stat_number(state, haste)?;
    Ok(1)
}

fn get_melee_haste(state: &mut LuaState) -> LuaResult<u32> {
    get_haste(state)
}

fn get_ranged_haste(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(feature = "client-wowforever")]
    {
        let sim = borrow_state(state)?;
        let haste = sim.player.stats.haste_pct();
        let quiver = sim.player.stats.quiver_haste_pct;
        drop(sim);
        state.push(Val::Num(haste));
        state.push(Val::Num(quiver));
        Ok(2)
    }
    #[cfg(not(feature = "client-wowforever"))]
    get_haste(state)
}

#[cfg(feature = "client-wowforever")]
fn get_ranged_hit_modifier(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?.player.stats.ranged_hit_modifier_pct;
    state.push(Val::Num(value));
    Ok(1)
}

#[cfg(feature = "client-wowforever")]
fn get_armor_penetration(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?.player.stats.armor_penetration;
    state.push(Val::Num(value));
    Ok(1)
}

#[cfg(feature = "client-wowforever")]
fn get_spell_penetration(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?.player.stats.spell_penetration;
    state.push(Val::Num(value));
    Ok(1)
}

#[cfg(feature = "client-wowforever")]
fn get_override_ap_by_spell_power(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?
        .player
        .stats
        .spell_power_to_attack_power;
    state.push(Val::Num(value));
    Ok(1)
}

#[cfg(feature = "client-wowforever")]
fn get_override_spell_power_by_ap(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?
        .player
        .stats
        .attack_power_to_spell_power;
    state.push(Val::Num(value));
    Ok(1)
}

#[cfg(feature = "client-wowforever")]
fn equipped_item_inv_type(state: &LuaState, slot: i32) -> LuaResult<Option<u8>> {
    let sim = borrow_state(state)?;
    Ok(sim
        .player
        .equipped_items
        .get(&slot)
        .and_then(|equipped| crate::items::get_item(equipped.item_id))
        .map(|item| item.inventory_type))
}

#[cfg(feature = "client-wowforever")]
fn is_dual_wielding(state: &mut LuaState) -> LuaResult<u32> {
    use crate::c_api::item_spell::helpers::inv_type_to_class_id;

    let main = equipped_item_inv_type(state, 16)?;
    let off = equipped_item_inv_type(state, 17)?;
    let weapon = |inv_type: Option<u8>| {
        inv_type.is_some_and(|kind| matches!(kind, 13 | 21 | 22) && inv_type_to_class_id(kind) == 2)
    };
    state.push(Val::Bool(weapon(main) && weapon(off)));
    Ok(1)
}

#[cfg(feature = "client-wowforever")]
fn is_ranged_weapon(state: &mut LuaState) -> LuaResult<u32> {
    use crate::c_api::item_spell::helpers::inv_type_to_class_id;

    let inv_type = equipped_item_inv_type(state, 18)?;
    let ranged = inv_type.is_some_and(|inv_type| {
        matches!(inv_type, 15 | 25 | 26) && inv_type_to_class_id(inv_type) == 2
    });
    state.push(Val::Bool(ranged));
    Ok(1)
}

#[cfg(feature = "client-wowforever")]
fn unit_has_relic_slot(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    // Classic Vanilla class capability inference; not native-verified on Forever.
    let has_relic_slot = resolve_unit_snapshot(&sim, &unit)
        .is_some_and(|unit| matches!(unit.class_index, 2 | 7 | 11));
    drop(sim);
    state.push(Val::Bool(has_relic_slot));
    Ok(1)
}

/// Wraps one explicit `CharacterStats` input; see docs/specs/explicit-stat-inputs.md.
fn push_player_stat(state: &mut LuaState, read: fn(&CharacterStats) -> f64) -> LuaResult<u32> {
    let value = read(&borrow_state(state)?.player.stats);
    push_stat_number(state, value)?;
    Ok(1)
}

fn push_player_stat_triple(
    state: &mut LuaState,
    read: fn(&CharacterStats) -> [f64; 3],
) -> LuaResult<u32> {
    let values = read(&borrow_state(state)?.player.stats);
    for value in values {
        push_stat_number(state, value)?;
    }
    Ok(3)
}

fn get_hit_modifier(state: &mut LuaState) -> LuaResult<u32> {
    push_player_stat(state, |stats| stats.hit_modifier)
}

fn get_spell_hit_modifier(state: &mut LuaState) -> LuaResult<u32> {
    push_player_stat(state, |stats| stats.spell_hit_modifier)
}

fn get_expertise(state: &mut LuaState) -> LuaResult<u32> {
    push_player_stat_triple(state, |stats| stats.expertise)
}

fn get_expertise_percent(state: &mut LuaState) -> LuaResult<u32> {
    push_player_stat_triple(state, |stats| stats.expertise_percent)
}

fn get_mod_resilience_damage_reduction(state: &mut LuaState) -> LuaResult<u32> {
    push_player_stat(state, |stats| stats.mod_resilience_damage_reduction)
}

fn get_pvp_power_damage(state: &mut LuaState) -> LuaResult<u32> {
    push_player_stat(state, |stats| stats.pvp_power_damage)
}

fn get_pvp_power_healing(state: &mut LuaState) -> LuaResult<u32> {
    push_player_stat(state, |stats| stats.pvp_power_healing)
}

fn get_mastery_effect(state: &mut LuaState) -> LuaResult<u32> {
    let mastery = borrow_state(state)?.player.stats.mastery_pct();
    push_stat_number(state, 8.0 + mastery)?;
    push_stat_number(state, mastery)?;
    Ok(2)
}

fn get_versatility_bonus(state: &mut LuaState) -> LuaResult<u32> {
    let vers = borrow_state(state)?.player.stats.versatility_pct();
    push_stat_number(state, vers)?;
    Ok(1)
}

fn get_zero_percent(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

fn get_speed(state: &mut LuaState) -> LuaResult<u32> {
    let rating = combat_rating_for(state, SPEED_RATING_INDEX) as f64;
    push_stat_number(state, rating_bonus_for_value(SPEED_RATING_INDEX, rating))?;
    Ok(1)
}

fn get_lifesteal(state: &mut LuaState) -> LuaResult<u32> {
    let rating = combat_rating_for(state, LIFESTEAL_RATING_INDEX) as f64;
    push_stat_number(
        state,
        rating_bonus_for_value(LIFESTEAL_RATING_INDEX, rating),
    )?;
    Ok(1)
}

fn get_avoidance(state: &mut LuaState) -> LuaResult<u32> {
    let rating = combat_rating_for(state, AVOIDANCE_RATING_INDEX) as f64;
    push_stat_number(
        state,
        rating_bonus_for_value(AVOIDANCE_RATING_INDEX, rating),
    )?;
    Ok(1)
}

fn get_mana_regen(state: &mut LuaState) -> LuaResult<u32> {
    let sim = borrow_state(state)?;
    let intellect = sim.player.stats.intellect.max(0.0);
    let base = 1.0 + intellect / 500.0;
    #[cfg(feature = "client-wowforever")]
    let total = base
        + super::forever_stat_contributions::mana_regen_from_spirit(
            sim.player.stats.spirit.max(0.0),
        );
    #[cfg(not(feature = "client-wowforever"))]
    let total = base;
    drop(sim);
    push_stat_number(state, total)?;
    push_stat_number(state, base * 0.5)?;
    Ok(2)
}

fn get_unit_mana_regen_rate_from_spirit(state: &mut LuaState) -> LuaResult<u32> {
    let _unit = String::from_stack(state, 1).unwrap_or_default();
    state.push(Val::Num(0.0));
    Ok(1)
}

const COMBAT_STAT_GLOBALS: &[(&str, RustFn)] = &[
    ("GetCombatRating", get_combat_rating),
    ("GetCombatRatingBonus", get_combat_rating_bonus),
    (
        "GetCombatRatingBonusForCombatRatingValue",
        get_combat_rating_bonus_for_value,
    ),
    ("GetCritChance", get_crit_chance),
    ("GetSpellCritChance", get_spell_crit_chance),
    ("GetRangedCritChance", get_ranged_crit_chance),
    ("GetCritChanceFromAgility", get_crit_chance_from_agility),
    (
        "GetSpellCritChanceFromIntellect",
        get_spell_crit_chance_from_intellect,
    ),
    (
        "GetCritChanceProvidesParryEffect",
        get_crit_chance_provides_parry_effect,
    ),
    ("GetHaste", get_haste),
    ("GetMeleeHaste", get_melee_haste),
    ("GetRangedHaste", get_ranged_haste),
    #[cfg(feature = "client-wowforever")]
    ("GetRangedHitModifier", get_ranged_hit_modifier),
    #[cfg(feature = "client-wowforever")]
    ("GetArmorPenetration", get_armor_penetration),
    #[cfg(feature = "client-wowforever")]
    ("GetSpellPenetration", get_spell_penetration),
    #[cfg(feature = "client-wowforever")]
    ("GetOverrideAPBySpellPower", get_override_ap_by_spell_power),
    #[cfg(feature = "client-wowforever")]
    ("GetOverrideSpellPowerByAP", get_override_spell_power_by_ap),
    #[cfg(feature = "client-wowforever")]
    ("IsDualWielding", is_dual_wielding),
    #[cfg(feature = "client-wowforever")]
    ("IsRangedWeapon", is_ranged_weapon),
    #[cfg(feature = "client-wowforever")]
    ("UnitHasRelicSlot", unit_has_relic_slot),
    ("GetHitModifier", get_hit_modifier),
    ("GetSpellHitModifier", get_spell_hit_modifier),
    ("GetExpertise", get_expertise),
    ("GetExpertisePercent", get_expertise_percent),
    ("GetMasteryEffect", get_mastery_effect),
    ("GetVersatilityBonus", get_versatility_bonus),
    (
        "GetModResilienceDamageReduction",
        get_mod_resilience_damage_reduction,
    ),
    ("GetPvpPowerDamage", get_pvp_power_damage),
    ("GetPvpPowerHealing", get_pvp_power_healing),
    ("GetMeleeMissChance", get_zero_percent),
    ("GetRangedMissChance", get_zero_percent),
    ("GetSpellMissChance", get_zero_percent),
    ("GetEnemyDodgeChance", get_zero_percent),
    ("GetEnemyParryChance", get_zero_percent),
    ("GetSpeed", get_speed),
    ("GetLifesteal", get_lifesteal),
    ("GetAvoidance", get_avoidance),
    ("GetManaRegen", get_mana_regen),
    (
        "GetUnitManaRegenRateFromSpirit",
        get_unit_mana_regen_rate_from_spirit,
    ),
];

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    #[cfg(feature = "client-retail")]
    retail_inputs::register_all(lua)?;
    for &(name, function) in COMBAT_STAT_GLOBALS {
        LuaApiMut::register_function(lua, name, function)?;
    }
    Ok(())
}
