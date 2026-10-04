use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::globals::inventory_slot;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const MAX_ARMOR_EFFECTIVENESS: f64 = 0.85;
const LOW_LEVEL_ARMOR_BASE: f64 = 400.0;
const LOW_LEVEL_ARMOR_PER_LEVEL: f64 = 85.0;
const HIGH_LEVEL_ARMOR_PER_LEVEL: f64 = 467.5;
const HIGH_LEVEL_ARMOR_OFFSET: f64 = 22_167.5;
const HIGH_LEVEL_ARMOR_FORMULA_MIN_LEVEL: f64 = 60.0;

pub(crate) fn register_c_paper_doll_info_surface(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_PaperDollInfo")?;
    table_set_rust_fn_static(state, ns, "GetArmorEffectiveness", get_armor_effectiveness)?;
    if cfg!(any(
        feature = "retail-12-1-0",
        feature = "client-wowforever"
    )) {
        register_inventory_slot_methods(state, ns)?;
    }
    table_set_rust_fn_static(
        state,
        ns,
        "GetArmorEffectivenessAgainstTarget",
        get_armor_effectiveness_against_target,
    )?;
    table_set_rust_fn_static(state, ns, "GetStaggerPercentage", get_stagger_percentage)?;
    table_set_rust_fn_static(
        state,
        ns,
        "IsInventorySlotEnabled",
        is_inventory_slot_enabled,
    )?;
    table_set_rust_fn_static(state, ns, "GetInspectGuildInfo", get_inspect_guild_info)?;
    #[cfg(feature = "retail-12-1-0")]
    table_set_rust_fn_static(
        state,
        ns,
        "CancelTemporaryEnchantment",
        cancel_temporary_enchantment,
    )?;
    #[cfg(feature = "retail-12-1-0")]
    table_set_rust_fn_static(
        state,
        ns,
        "GetTemporaryEnchantmentInfo",
        get_temporary_enchantment_info,
    )?;
    Ok(())
}

/// Weapon enchant index for the main-hand (16), off-hand (17), or ranged (18)
/// inventory slot.
#[cfg(feature = "retail-12-1-0")]
fn weapon_enchant_slot(inventory_slot: i32) -> Option<usize> {
    match inventory_slot {
        16 => Some(0),
        17 => Some(1),
        18 => Some(2),
        _ => None,
    }
}

/// Removes temporary enchants from a weapon slot; permanent enchants and other
/// slots are untouched. INFERRED: a removal reports WEAPON_ENCHANT_CHANGED.
#[cfg(feature = "retail-12-1-0")]
fn cancel_temporary_enchantment(state: &mut LuaState) -> LuaResult<u32> {
    let Some(weapon_slot) = weapon_enchant_slot(i32::from_stack(state, 1)?) else {
        return Ok(0);
    };
    let removed = {
        let mut sim = crate::lua_api::methods::borrow_state_mut(state)?;
        let enchants = &mut sim.weapon_enchants[weapon_slot];
        let before = enchants.len();
        enchants.retain(|enchant| {
            enchant.enchant_type != super::weapon_enchants::TEMPORARY_ENCHANT_TYPE
        });
        enchants.len() != before
    };
    if removed {
        crate::lua_api::script_helpers::fire_named_event_state(
            state,
            "WEAPON_ENCHANT_CHANGED",
            &[],
        );
    }
    Ok(0)
}

/// `TemporaryItemEnchantInfo` for the slot's temporary enchant, or nothing.
/// INFERRED: enchants without remaining time have no expiration time.
#[cfg(feature = "retail-12-1-0")]
fn get_temporary_enchantment_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(weapon_slot) = weapon_enchant_slot(i32::from_stack(state, 1)?) else {
        return Ok(0);
    };
    let enchant = borrow_state(state)?.weapon_enchants[weapon_slot]
        .iter()
        .find(|enchant| enchant.enchant_type == super::weapon_enchants::TEMPORARY_ENCHANT_TYPE)
        .cloned();
    let Some(enchant) = enchant else {
        return Ok(0);
    };
    let info = crate::lua_api::methods::create_table(state);
    for (key, value) in [
        ("enchantID", f64::from(enchant.enchant_id)),
        ("remainingTimeMs", enchant.time_left),
        ("chargesRemaining", f64::from(enchant.charges)),
    ] {
        crate::lua_api::methods::table_set(state, info, key, Val::Num(value));
    }
    crate::lua_api::methods::table_set(
        state,
        info,
        "hasExpirationTime",
        Val::Bool(enchant.time_left > 0.0),
    );
    state.push(info);
    Ok(1)
}

fn register_inventory_slot_methods(
    state: &mut LuaState,
    ns: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        ns,
        "GetInventorySlotInfo",
        inventory_slot::get_inventory_slot_info,
    )?;
    table_set_rust_fn_static(
        state,
        ns,
        "GetInventorySlotInfoForInvSlot",
        inventory_slot::get_inventory_slot_info,
    )
}

fn get_armor_effectiveness(state: &mut LuaState) -> LuaResult<u32> {
    let armor = f64::from_stack(state, 1)?;
    let attacker_level = f64::from_stack(state, 2)?;
    state.push(Val::Num(armor_effectiveness(armor, attacker_level)));
    Ok(1)
}

fn get_armor_effectiveness_against_target(state: &mut LuaState) -> LuaResult<u32> {
    let armor = f64::from_stack(state, 1)?;
    let target_level = borrow_state(state)?
        .current_target
        .as_ref()
        .map(|target| target.level as f64);
    let Some(target_level) = target_level else {
        return Ok(0);
    };

    state.push(Val::Num(armor_effectiveness(armor, target_level)));
    Ok(1)
}

fn get_stagger_percentage(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

fn is_inventory_slot_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let name = match stack_val(state, 1) {
        Val::Str(s) => state
            .gc
            .string_arena
            .get(s)
            .and_then(|lua_str| std::str::from_utf8(lua_str.data()).ok())
            .map(str::to_owned),
        _ => None,
    };

    let enabled = name
        .as_deref()
        .and_then(inventory_slot::lookup_slot)
        .is_some();
    state.push(Val::Bool(enabled));
    Ok(1)
}

fn get_inspect_guild_info(state: &mut LuaState) -> LuaResult<u32> {
    let guild = {
        let sim = borrow_state(state)?;
        sim.world
            .guild_name
            .as_ref()
            .map(|name| (name.clone(), sim.world.guild_num_members))
    };

    let Some((name, members)) = guild else {
        return Ok(0);
    };

    state.push(Val::Num(0.0));
    state.push(Val::Num(members as f64));
    let name = crate::lua_api::methods::create_string(state, &name);
    state.push(name);
    Ok(3)
}

fn armor_effectiveness(armor: f64, attacker_level: f64) -> f64 {
    if armor <= 0.0 {
        return 0.0;
    }

    let mitigation_constant = armor_mitigation_constant(attacker_level);
    (armor / (armor + mitigation_constant)).clamp(0.0, MAX_ARMOR_EFFECTIVENESS)
}

fn armor_mitigation_constant(attacker_level: f64) -> f64 {
    let level = attacker_level.max(1.0);
    if level < HIGH_LEVEL_ARMOR_FORMULA_MIN_LEVEL {
        return LOW_LEVEL_ARMOR_BASE + LOW_LEVEL_ARMOR_PER_LEVEL * level;
    }

    (HIGH_LEVEL_ARMOR_PER_LEVEL * level - HIGH_LEVEL_ARMOR_OFFSET).max(1.0)
}
