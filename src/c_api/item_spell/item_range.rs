//! ItemHasRange: range index selection for the item use spell (WM43).

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use sim_data::item_spell_ranges::{
    DIFFICULTY_FALLBACKS, SPELL_EFFECT_TARGETS, SPELL_MISC_RANGES, SPELL_RANGES,
    SPELL_TARGET_RESTRICTIONS,
};

/// Implicit targets that give harmful evidence (WM43 follow-up, complete set).
const HARMFUL_TARGETS: [u32; 12] = [2, 6, 15, 16, 24, 28, 53, 54, 93, 104, 129, 134];
/// Implicit targets that give helpful evidence (WM43 follow-up, complete set).
const HELPFUL_TARGETS: [u32; 25] = [
    1, 3, 4, 5, 20, 21, 27, 29, 30, 31, 33, 34, 35, 45, 56, 57, 58, 59, 118, 119, 121, 128, 132,
    133, 136,
];
const FORCE_HARMFUL_ATTRIBUTE: i32 = 4096;
/// Attributes[0] bits 4 and 1024 return minimum 0, maximum 100 before RangeIndex lookup.
const UNLIMITED_RANGE_ATTRIBUTES: i32 = 4 | 1024;
const UNLIMITED_RANGE: (f32, f32) = (0.0, 100.0);
const MINIMUM_RANGE: f32 = 1.0 / 4_194_304.0;

/// Effect-scan input: EffectAttributes and both implicit targets.
pub type EffectTargets = (i32, u32, u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reach {
    Neither,
    Helpful,
    Harmful,
}

/// Targets mask of the first restriction row matching `difficulty`, following
/// Difficulty fallbacks only while the missed difficulty has a Difficulty row.
pub fn restriction_targets(
    rows: &[(i32, u32)],
    fallbacks: &[(i32, i32)],
    difficulty: i32,
) -> Option<u32> {
    let mut current = difficulty;
    loop {
        if let Some(&(_, targets)) = rows.iter().find(|row| row.0 == current) {
            return Some(targets);
        }
        let index = fallbacks.binary_search_by_key(&current, |row| row.0).ok()?;
        current = fallbacks[index].1;
    }
}

/// Restriction overrides first, then the harmful scan, then the helpful scan.
pub fn classify_reach(targets: Option<u32>, effects: &[EffectTargets]) -> Reach {
    match targets {
        Some(mask) if mask & 256 != 0 => return Reach::Helpful,
        Some(mask) if mask & 128 != 0 => return Reach::Harmful,
        Some(mask) if mask & 2 != 0 => return Reach::Neither,
        _ => {}
    }
    let any_target = |(_, first, second): &EffectTargets, set: &[u32]| {
        set.contains(first) || set.contains(second)
    };
    if effects.iter().any(|effect| {
        effect.0 & FORCE_HARMFUL_ATTRIBUTE != 0 || any_target(effect, &HARMFUL_TARGETS)
    }) {
        return Reach::Harmful;
    }
    if effects
        .iter()
        .any(|effect| any_target(effect, &HELPFUL_TARGETS))
    {
        return Reach::Helpful;
    }
    Reach::Neither
}

fn spell_reach(spell: u32, difficulty: i32) -> Reach {
    let start = SPELL_TARGET_RESTRICTIONS.partition_point(|row| row.0 < spell);
    let end = SPELL_TARGET_RESTRICTIONS.partition_point(|row| row.0 <= spell);
    let restrictions: Vec<(i32, u32)> = SPELL_TARGET_RESTRICTIONS[start..end]
        .iter()
        .map(|&(_, difficulty, targets)| (difficulty, targets))
        .collect();
    let start = SPELL_EFFECT_TARGETS.partition_point(|row| row.0 < spell);
    let end = SPELL_EFFECT_TARGETS.partition_point(|row| row.0 <= spell);
    let effects: Vec<EffectTargets> = SPELL_EFFECT_TARGETS[start..end]
        .iter()
        .map(|&(_, attributes, first, second)| (attributes, first, second))
        .collect();
    let targets = restriction_targets(&restrictions, DIFFICULTY_FALLBACKS, difficulty);
    classify_reach(targets, &effects)
}

/// Selected (minimum, maximum) for the spell; None without its SpellMisc or SpellRange record.
fn spell_range(spell: u32, difficulty: i32) -> Option<(f32, f32)> {
    let misc = SPELL_MISC_RANGES
        .binary_search_by_key(&spell, |row| row.0)
        .ok()
        .map(|index| SPELL_MISC_RANGES[index])?;
    let (_, attributes, range_index) = misc;
    if attributes & UNLIMITED_RANGE_ATTRIBUTES != 0 {
        return Some(UNLIMITED_RANGE);
    }
    let range = SPELL_RANGES
        .binary_search_by_key(&range_index, |row| row.0)
        .ok()
        .map(|index| SPELL_RANGES[index])?;
    let index = usize::from(spell_reach(spell, difficulty) == Reach::Helpful);
    Some((range.1[index], range.2[index]))
}

pub(super) fn item_has_range(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let item = super::parse_item_id_key(state, value)
        .ok_or_else(|| runtime_error("C_Item.ItemHasRange: expected itemInfo"))?;
    let (present, difficulty) = {
        let sim = borrow_state(state)?;
        (sim.player.present, sim.world.instance_difficulty)
    };
    let spell = if present {
        super::item_effects::selected_item_spell(state, item)?.map(|(spell, _)| spell)
    } else {
        None
    };
    let result = spell
        .and_then(|spell| spell_range(spell, difficulty))
        .is_some_and(|(min, max)| min.abs() > MINIMUM_RANGE || max.abs() > MINIMUM_RANGE);
    state.push(Val::Bool(result));
    Ok(1)
}
