//! Retail/Forever queries over explicitly supplied pet-battle inputs.
//!
//! INFERRED host projection unless a bounded native observation is named.
//! R88f.D.49–63 does not establish complete absent tuples or argument parity.
use super::pet_battle_runtime_model::{PetBattleAbility, PetBattleAbilityState, PetBattleAura};
use crate::lua_api::methods::{borrow_state, create_string, table_set};
use crate::lua_api::state::PetBattlePet;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register_queries(state: &mut LuaState) -> LuaResult<()> {
    register_identity_and_abilities(state)?;
    register_stats_and_selection(state)
}

fn register_identity_and_abilities(state: &mut LuaState) -> LuaResult<()> {
    let ns = super::ensure_namespace(state, "C_PetBattles")?;
    table_set_rust_fn_static(state, ns, "IsInBattle", is_in_battle)?;
    table_set_rust_fn_static(state, ns, "IsWildBattle", is_wild_battle)?;
    table_set_rust_fn_static(state, ns, "GetAbilityInfo", get_ability_info)?;
    table_set_rust_fn_static(state, ns, "GetAbilityInfoByID", get_ability_info_by_id)?;
    table_set_rust_fn_static(state, ns, "GetAbilityState", get_ability_state)?;
    table_set_rust_fn_static(state, ns, "GetAuraInfo", get_aura_info)?;
    table_set_rust_fn_static(state, ns, "GetNumAuras", get_num_auras)
}

fn register_stats_and_selection(state: &mut LuaState) -> LuaResult<()> {
    let ns = super::ensure_namespace(state, "C_PetBattles")?;
    table_set_rust_fn_static(state, ns, "GetHealth", get_health)?;
    table_set_rust_fn_static(state, ns, "GetMaxHealth", get_max_health)?;
    table_set_rust_fn_static(state, ns, "GetPower", get_power)?;
    table_set_rust_fn_static(state, ns, "GetSpeed", get_speed)?;
    table_set_rust_fn_static(state, ns, "GetLevel", get_level)?;
    table_set_rust_fn_static(state, ns, "GetBreedQuality", get_breed_quality)?;
    table_set_rust_fn_static(state, ns, "GetAttackModifier", get_attack_modifier)?;
    table_set_rust_fn_static(state, ns, "GetAllStates", get_all_states)?;
    table_set_rust_fn_static(state, ns, "GetSelectedAction", get_selected_action)
}

fn pet_key(state: &mut LuaState) -> LuaResult<(i32, i32)> {
    Ok((i32::from_stack(state, 1)?, i32::from_stack(state, 2)?))
}

fn pet_at(state: &LuaState, owner: i32, slot: i32) -> LuaResult<Option<PetBattlePet>> {
    let index = slot
        .checked_sub(1)
        .and_then(|index| usize::try_from(index).ok());
    let sim = borrow_state(state)?;
    let pets = match owner {
        1 => Some(&sim.pet_battles.player_pets),
        2 => Some(&sim.pet_battles.enemy_pets),
        _ => None,
    };
    Ok(pets
        .and_then(|pets| index.and_then(|index| pets.get(index)))
        .cloned())
}

fn in_battle(state: &LuaState) -> LuaResult<bool> {
    // INFERRED legacy phase model, not R88f.D.47's native context-pointer test.
    // Finished=6: both pinned PetBattleConstantsDocumentation.lua:64.
    let phase = borrow_state(state)?.pet_battles.battle_state;
    Ok(phase != 0 && phase != 6)
}

fn is_in_battle(state: &mut LuaState) -> LuaResult<u32> {
    let active = in_battle(state)?;
    state.push(Val::Bool(active));
    Ok(1)
}

fn is_wild_battle(state: &mut LuaState) -> LuaResult<u32> {
    let wild = in_battle(state)? && borrow_state(state)?.pet_battles.runtime.is_wild;
    state.push(Val::Bool(wild));
    Ok(1)
}

fn get_ability_info(state: &mut LuaState) -> LuaResult<u32> {
    let (owner, slot) = pet_key(state)?;
    let ability_slot = i32::from_stack(state, 3)?;
    let index = ability_slot
        .checked_sub(1)
        .and_then(|index| usize::try_from(index).ok());
    let id = pet_at(state, owner, slot)?
        .and_then(|pet| index.and_then(|index| pet.ability_ids.get(index).copied()));
    push_ability(state, id)
}

fn get_ability_info_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    push_ability(state, Some(id))
}

fn push_ability(state: &mut LuaState, id: Option<i32>) -> LuaResult<u32> {
    let ability = {
        let sim = borrow_state(state)?;
        id.and_then(|id| sim.pet_battles.runtime.abilities.get(&id).cloned())
    };
    match (id, ability) {
        (Some(id), Some(ability)) => push_ability_fields(state, id, ability),
        _ => {
            // INFERRED: retain the compatibility nil result for absent metadata.
            state.push(Val::Nil);
            Ok(1)
        }
    }
}

fn push_ability_fields(state: &mut LuaState, id: i32, ability: PetBattleAbility) -> LuaResult<u32> {
    // INFERRED tuple from original PetBattleUI consumers; native tail is unresolved.
    state.push(Val::Num(f64::from(id)));
    let name = create_string(state, &ability.name);
    state.push(name);
    state.push(Val::Num(f64::from(ability.icon)));
    state.push(Val::Num(f64::from(ability.max_cooldown)));
    let description = create_string(state, &ability.description);
    state.push(description);
    state.push(Val::Num(f64::from(ability.num_turns)));
    state.push(Val::Num(f64::from(ability.pet_type)));
    state.push(Val::Bool(ability.no_strong_weak_hints));
    Ok(8)
}

fn ability_state_at(
    state: &LuaState,
    owner: i32,
    pet: i32,
    ability: i32,
) -> LuaResult<PetBattleAbilityState> {
    let index = ability
        .checked_sub(1)
        .and_then(|index| usize::try_from(index).ok());
    let id = pet_at(state, owner, pet)?
        .and_then(|pet| index.and_then(|index| pet.ability_ids.get(index).copied()));
    let sim = borrow_state(state)?;
    let runtime = &sim.pet_battles.runtime;
    Ok(id
        .filter(|id| runtime.abilities.contains_key(id))
        .and_then(|_| runtime.ability_states.get(&(owner, pet, ability)))
        .cloned()
        .unwrap_or_default())
}

fn get_ability_state(state: &mut LuaState) -> LuaResult<u32> {
    let (owner, pet) = pet_key(state)?;
    let ability = i32::from_stack(state, 3)?;
    let supplied = ability_state_at(state, owner, pet, ability)?;
    // INFERRED absent compatibility tuple; R88f.D.51's final publication is UNKNOWN.
    state.push(Val::Bool(supplied.usable));
    state.push(Val::Num(f64::from(supplied.cooldown)));
    state.push(Val::Num(f64::from(supplied.lockdown)));
    Ok(3)
}

fn aura_at(state: &LuaState, key: (i32, i32), slot: i32) -> LuaResult<Option<PetBattleAura>> {
    if pet_at(state, key.0, key.1)?.is_none() {
        return Ok(None);
    }
    let index = slot
        .checked_sub(1)
        .and_then(|index| usize::try_from(index).ok());
    Ok(borrow_state(state)?
        .pet_battles
        .runtime
        .auras
        .get(&key)
        .and_then(|auras| index.and_then(|index| auras.get(index)))
        .cloned())
}

fn get_aura_info(state: &mut LuaState) -> LuaResult<u32> {
    let key = pet_key(state)?;
    let slot = i32::from_stack(state, 3)?;
    let aura = aura_at(state, key, slot)?;
    let Some(aura) = aura else {
        state.push(Val::Nil);
        return Ok(1);
    };
    // INFERRED consumer tuple; the native final publication remains UNKNOWN.
    state.push(Val::Num(f64::from(aura.ability_id)));
    state.push(Val::Num(f64::from(aura.instance_id)));
    state.push(Val::Num(f64::from(aura.turns_remaining)));
    state.push(Val::Bool(aura.is_buff));
    Ok(4)
}

fn get_num_auras(state: &mut LuaState) -> LuaResult<u32> {
    let key = pet_key(state)?;
    let count = if pet_at(state, key.0, key.1)?.is_some() {
        borrow_state(state)?
            .pet_battles
            .runtime
            .auras
            .get(&key)
            .map_or(0, Vec::len)
    } else {
        0
    };
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn push_pet_stat(
    state: &mut LuaState,
    absent: i32,
    field: fn(&PetBattlePet) -> i32,
) -> LuaResult<u32> {
    let (owner, slot) = pet_key(state)?;
    let value = pet_at(state, owner, slot)?.as_ref().map_or(absent, field);
    state.push(Val::Num(f64::from(value)));
    Ok(1)
}

fn get_health(state: &mut LuaState) -> LuaResult<u32> {
    // VERIFIED bounded R88f.D.54 push: absent pet => 100; complete native tail UNKNOWN.
    push_pet_stat(state, 100, |pet| pet.current_health)
}

fn get_max_health(state: &mut LuaState) -> LuaResult<u32> {
    // VERIFIED bounded R88f.D.55 push: absent pet => 100; complete native tail UNKNOWN.
    push_pet_stat(state, 100, |pet| pet.max_health)
}

fn get_power(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED nominal R88f.D.56 projection, not complete native parity.
    push_pet_stat(state, 0, |pet| pet.power)
}

fn get_speed(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED nominal R88f.D.57 projection, not complete native parity.
    push_pet_stat(state, 0, |pet| pet.speed)
}

fn get_level(state: &mut LuaState) -> LuaResult<u32> {
    // VERIFIED bounded R88f.D.58 absent-pet push is 1; complete native tail UNKNOWN.
    push_pet_stat(state, 1, |pet| pet.level)
}

fn get_breed_quality(state: &mut LuaState) -> LuaResult<u32> {
    let key = pet_key(state)?;
    // INFERRED nominal R88f.D.59 projection; no synthesized rare-quality seed.
    let quality = if pet_at(state, key.0, key.1)?.is_some() {
        borrow_state(state)?
            .pet_battles
            .runtime
            .qualities
            .get(&key)
            .copied()
            .unwrap_or(0)
    } else {
        0
    };
    state.push(Val::Num(f64::from(quality)));
    Ok(1)
}

fn get_attack_modifier(state: &mut LuaState) -> LuaResult<u32> {
    let attacker = i32::from_stack(state, 1)?;
    let defender = i32::from_stack(state, 2)?;
    let modifier = super::pet_battle_type_chart::attack_modifier(attacker, defender);
    state.push(Val::Num(modifier));
    Ok(1)
}

fn get_all_states(state: &mut LuaState) -> LuaResult<u32> {
    let target = stack_val(state, 1);
    if !matches!(target, Val::Table(_)) {
        return Ok(0);
    }
    let entries = borrow_state(state)?
        .pet_battles
        .runtime
        .parser_states
        .clone();
    for (name, value) in entries {
        table_set(
            state,
            target,
            &format!("STATE_{name}"),
            Val::Num(f64::from(value)),
        );
    }
    // INFERRED existing mutation-only contract; R88f.D.62 return tail UNKNOWN.
    Ok(0)
}

fn get_selected_action(state: &mut LuaState) -> LuaResult<u32> {
    let action = borrow_state(state)?.pet_battles.runtime.selected_action;
    let (kind, index) = action.map_or((Val::Nil, Val::Nil), |(kind, index)| {
        (
            Val::Num(f64::from(kind)),
            index.map_or(Val::Nil, |index| Val::Num(f64::from(index))),
        )
    });
    // INFERRED empty/selectorless tuple; R88f.D.67 leaves zero-action results UNKNOWN.
    state.push(kind);
    state.push(index);
    Ok(2)
}
