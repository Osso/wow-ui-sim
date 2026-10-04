//! `C_Spell` probes backed by spell data and `SimState`.
//!
//! Migrates 10 entries off the namespace stub tables:
//!
//! - `GetSpellInfo(spellID)` → `SpellInfo` table from `spells::get_spell`, or nil.
//! - `GetSpellCooldown(spellID)` → `SpellCooldownInfo` table from
//!   `SimState.spell_cooldowns` (start/duration/isEnabled/isActive/modRate).
//! - `GetSpellDescription(spellID)` → localized spell description, or empty.
//! - `GetSpellTexture(spellID)` → `(fallbackTexturePath, fileDataID, conditionalIconID)`.
//! - `GetSpellPowerCost(spellID)` → `SpellPowerCostInfo[]` table or nil.
//! - `GetSchoolString(mask)` → localized school name for a bitmask.
//! - `PickupSpell(spellID)` → sets cursor to a spell and fires `CURSOR_CHANGED`.
//! - `GetSpellLink(spellID)` → spell hyperlink string or nil.
//! - `GetSpellName(spellID)` → localized name or `"Unknown"`.
//! - `IsAutoAttackSpell(spellID)` → true for spell 6603.
//! - `IsSpellHelpful(spellID)` → true for self-targeted spells.
//! - `IsSpellHarmful(spellID)` → true for known non-self-targeted spells.
//! - `GetMountFromSpell(spellID)` → scans `world.mounts` for matching spell
//!   id, returns mount_id or nil.
//! - `IsSelfBuff(spellID)` → true when `implicit_target == 1` (Self), else false.
//! - `IsSpellUsable(spellID)` → `(true, false)` when spell is known;
//!   `(false, false)` otherwise.
//! - `GetSpellTradeSkillLink(spellID)` → recipe link from
//!   `state.spell_trade_skill_links`, or nil.
//! - `GetSpellIDForSpellIdentifier(identifier)` → resolved spell id from
//!   `state.spell_id_aliases`. Numeric input passes through unchanged when
//!   no alias is registered; string input requires an entry.
//! - `IsCurrentSpell(spellID)` → matches `state.casting.spell_id`.
//! - `GetSpellLossOfControlCooldownInfo(spellID)` → LoC cooldown table from
//!   `state.spell_loss_of_control`, or nil.

use super::helpers::ensure_namespace;
use crate::c_api::item_spell::spell_link_for_id;
use crate::lua_api::globals::action_bar_api::spell_cooldown_times;
use crate::lua_api::globals::spellbook_data;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_string_static, create_table,
    create_table_with_capacity, frame_ref, table_set_num, table_set_static, val_to_string,
};
use crate::lua_api::script_helpers::{
    call_error_handler_state, get_dispatch_script, get_event_listeners, protected_lua_pcall_state,
};
use crate::lua_api::state_types::CursorInfo;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use crate::spell_lookup as spells;
use crate::spells::SPELL_DB;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

type SpellScriptFn = fn(&mut LuaState) -> LuaResult<u32>;

const SPELL_INFO_HASH_FIELDS: usize = 7;
const SPELL_COOLDOWN_HASH_FIELDS: usize = 5;

pub(crate) fn register_c_spell_surface(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_Spell")?;
    register_spell_methods(state, ns, SPELL_QUERY_METHODS)?;
    register_spell_methods(state, ns, SPELL_BOOLEAN_METHODS)?;
    if cfg!(feature = "client-mists") {
        register_legacy_spell_globals(state)?;
    }
    Ok(())
}

const SPELL_QUERY_METHODS: &[(&str, SpellScriptFn)] = &[
    #[cfg(feature = "base-spell-relationships")]
    ("GetBaseSpell", super::spell_base::get_base_spell),
    ("GetSpellDescription", get_spell_description),
    #[cfg(feature = "retail-12-1-0")]
    (
        "GetSpellDescriptionForItemLocation",
        get_spell_description_for_item_location,
    ),
    #[cfg(feature = "retail-12-1-0")]
    (
        "GetLastCategoryCooldownSource",
        get_last_category_cooldown_source,
    ),
    ("GetSpellQueueWindow", get_spell_queue_window),
    ("GetSpellInfo", get_spell_info),
    ("GetSpellTexture", get_spell_texture),
    ("GetSpellPowerCost", get_spell_power_cost),
    ("GetSchoolString", get_school_string),
    ("PickupSpell", pickup_spell),
    ("GetSpellLink", get_spell_link),
    ("GetSpellName", get_spell_name),
    ("GetSpellCooldown", get_spell_cooldown),
    ("GetSpellCharges", get_spell_charges),
    #[cfg(feature = "retail-12-0-5")]
    (
        "GetSpellCastCount",
        super::c_spell_counts::get_spell_cast_count,
    ),
    #[cfg(feature = "retail-12-0-5")]
    (
        "GetSpellDisplayCount",
        super::c_spell_counts::get_spell_display_count,
    ),
    #[cfg(feature = "retail-12-0-5")]
    (
        "GetSpellMaxCumulativeAuraApplications",
        super::c_spell_counts::get_spell_max_cumulative_aura_applications,
    ),
    #[cfg(feature = "retail-12-0-5")]
    ("GetSpellChargeDuration", get_spell_charge_duration),
    #[cfg(feature = "retail-12-0-0")]
    ("GetSpellCooldownDuration", get_spell_cooldown_duration),
    ("GetMountFromSpell", get_mount_from_spell),
    ("GetSpellTradeSkillLink", get_spell_trade_skill_link),
    (
        "GetSpellIDForSpellIdentifier",
        get_spell_id_for_spell_identifier,
    ),
    ("IsCurrentSpell", is_current_spell),
    (
        "GetSpellLossOfControlCooldownInfo",
        get_spell_loss_of_control_cooldown_info,
    ),
];

const SPELL_BOOLEAN_METHODS: &[(&str, SpellScriptFn)] = &[
    ("DoesSpellExist", does_spell_exist),
    ("IsSpellDataCached", is_spell_data_cached),
    ("IsSelfBuff", is_self_buff),
    ("IsAutoAttackSpell", is_auto_attack_spell),
    ("IsSpellHelpful", is_spell_helpful),
    ("IsSpellHarmful", is_spell_harmful),
    ("IsSpellUsable", is_spell_usable),
    #[cfg(feature = "retail-12-1-0")]
    (
        "TargetSpellChecksItemCondition",
        target_spell_checks_item_condition,
    ),
];

#[cfg(feature = "retail-12-1-0")]
fn target_spell_checks_item_condition(state: &mut LuaState) -> LuaResult<u32> {
    // Spell-item condition metadata is not modeled; default to no match.
    state.push(Val::Bool(false));
    Ok(1)
}

fn register_spell_methods(
    state: &mut LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
    methods: &[(&'static str, SpellScriptFn)],
) -> LuaResult<()> {
    for &(name, func) in methods {
        table_set_rust_fn_static(state, table_ref, name, func)?;
    }
    Ok(())
}

fn register_legacy_spell_globals(state: &mut LuaState) -> LuaResult<()> {
    table_set_rust_fn_static(state, state.global, "GetSpellInfo", legacy_get_spell_info)?;
    table_set_rust_fn_static(
        state,
        state.global,
        "GetSpellTexture",
        legacy_get_spell_texture,
    )?;
    Ok(())
}

fn get_spell_description(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = u32::from_stack(state, 1)?;
    let description_text = {
        let sim = borrow_state(state)?;
        crate::spell_description_resolver::resolve_spell_description_or_empty(&sim, spell_id)
    };
    let description = create_string(state, &description_text);
    state.push(description);
    Ok(1)
}

/// Item-scaled spell text is unmodeled, so the item location selects the plain
/// description; unknown spells return nothing.
#[cfg(feature = "retail-12-1-0")]
fn get_spell_description_for_item_location(state: &mut LuaState) -> LuaResult<u32> {
    let Some(spell_id) = numeric_spell_id(state, 1).filter(|id| spells::get_spell(*id).is_some())
    else {
        return Ok(0);
    };
    let description_text = {
        let sim = borrow_state(state)?;
        crate::spell_description_resolver::resolve_spell_description_or_empty(&sim, spell_id)
    };
    let description = create_string(state, &description_text);
    state.push(description);
    Ok(1)
}

/// Most recent cooldown start among spells in `spellCategory`, with the item
/// that triggered it when one did.
#[cfg(feature = "retail-12-1-0")]
fn get_last_category_cooldown_source(state: &mut LuaState) -> LuaResult<u32> {
    let category = i32::from_stack(state, 1)?;
    let source = {
        let sim = borrow_state(state)?;
        sim.spell_cooldowns
            .iter()
            .filter(|(spell_id, _)| sim.spell_cooldown_categories.get(spell_id) == Some(&category))
            .max_by(|(_, left), (_, right)| left.start.total_cmp(&right.start))
            .map(|(spell_id, _)| {
                let item_id = sim.spell_cooldown_item_sources.get(spell_id).copied();
                (*spell_id, item_id)
            })
    };
    let Some((spell_id, item_id)) = source else {
        return Ok(0);
    };
    state.push(Val::Num(f64::from(spell_id)));
    state.push(item_id.map_or(Val::Nil, |id| Val::Num(f64::from(id))));
    Ok(2)
}

fn get_spell_queue_window(state: &mut LuaState) -> LuaResult<u32> {
    let queue_window = borrow_state(state)?
        .cvars
        .get("SpellQueueWindow")
        .and_then(|value| value.parse::<f64>().ok());
    state.push(queue_window.map_or(Val::Nil, Val::Num));
    Ok(1)
}

/// `C_Spell.GetSpellInfo(spellID)` → `SpellInfo` table or nil.
///
/// Retail fields: `name, iconID, originalIconID, castTime, minRange, maxRange, spellID`.
fn get_spell_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(spell_id) = numeric_spell_id(state, 1) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let Some(spell) = spells::get_spell(spell_id) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let info = create_table_with_capacity(state, SPELL_INFO_HASH_FIELDS);
    let name = create_string(state, spell.name);
    table_set_static(state, info, "name", name);
    table_set_static(
        state,
        info,
        "iconID",
        Val::Num(spell.icon_file_data_id as f64),
    );
    table_set_static(
        state,
        info,
        "originalIconID",
        Val::Num(spell.icon_file_data_id as f64),
    );
    table_set_static(state, info, "castTime", Val::Num(0.0));
    table_set_static(state, info, "minRange", Val::Num(0.0));
    table_set_static(state, info, "maxRange", Val::Num(0.0));
    table_set_static(state, info, "spellID", Val::Num(spell_id as f64));
    state.push(info);
    Ok(1)
}

fn get_spell_texture(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = u32::from_stack(state, 1)?;
    let icon_id = spells::get_spell(spell_id)
        .map(|spell| spell.icon_file_data_id)
        .unwrap_or(136243);
    let texture = create_string(state, "Interface\\ICONS\\INV_Misc_QuestionMark");
    state.push(texture);
    state.push(Val::Num(icon_id as f64));
    #[cfg(feature = "retail-12-1-0")]
    {
        state.push(Val::Nil);
        return Ok(3);
    }
    #[cfg(not(feature = "retail-12-1-0"))]
    Ok(2)
}

fn legacy_get_spell_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(spell_id_arg) = numeric_spell_id(state, 1) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let Some(spell) = spells::get_spell(spell_id_arg) else {
        state.push(Val::Nil);
        return Ok(1);
    };

    let name = create_string(state, spell.name);
    let sub_name = create_string(state, "");
    state.push(name);
    state.push(sub_name);
    state.push(Val::Num(spell.icon_file_data_id as f64));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(Val::Num(spell_id_arg as f64));
    state.push(Val::Num(spell.icon_file_data_id as f64));
    Ok(8)
}

fn legacy_get_spell_texture(state: &mut LuaState) -> LuaResult<u32> {
    let first_arg = i32::from_stack(state, 1)?;
    let Some(spell_id) = legacy_spell_id_from_arg(first_arg, has_book_type_arg(state)) else {
        state.push(Val::Nil);
        return Ok(1);
    };

    let icon_id = spells::get_spell(spell_id)
        .map(|spell| spell.icon_file_data_id)
        .unwrap_or(136243);
    let texture_path = create_string(state, "Interface\\ICONS\\INV_Misc_QuestionMark");
    state.push(texture_path);
    state.push(Val::Num(icon_id as f64));
    Ok(2)
}

fn legacy_spell_id_from_arg(first_arg: i32, has_book_type_arg: bool) -> Option<u32> {
    if has_book_type_arg {
        return spellbook_data::get_spell_at_slot(first_arg).map(|(_, entry, _)| entry.spell_id);
    }
    u32::try_from(first_arg).ok()
}

fn has_book_type_arg(state: &LuaState) -> bool {
    !matches!(stack_val(state, 2), Val::Nil)
}

fn spell_power_min_cost(player_power_max: f32, cost: &crate::spell_power::SpellPowerCost) -> i32 {
    if cost.mana_cost > 0 {
        return cost.mana_cost;
    }
    if cost.cost_pct > 0.0 {
        return (player_power_max * (cost.cost_pct / 100.0)).round() as i32;
    }
    0
}

fn build_spell_power_cost_info(
    state: &mut LuaState,
    cost: &crate::spell_power::SpellPowerCost,
    player_power_max: f32,
) -> Option<Val> {
    let Val::Table(info) = create_table(state) else {
        return None;
    };

    let min_cost = spell_power_min_cost(player_power_max, cost);
    let total_cost = min_cost + cost.optional_cost.max(0);
    let power_name = create_string(state, crate::spell_power::power_type_name(cost.power_type));

    table_set_static(
        state,
        Val::Table(info),
        "type",
        Val::Num(cost.power_type as f64),
    );
    table_set_static(state, Val::Table(info), "name", power_name);
    table_set_static(state, Val::Table(info), "cost", Val::Num(total_cost as f64));
    for &(name, value) in &[
        ("minCost", min_cost as f64),
        ("costPercent", cost.cost_pct as f64),
        ("costPerSec", cost.cost_per_sec as f64),
        ("requiredAuraID", cost.required_aura_id as f64),
    ] {
        table_set_static(state, Val::Table(info), name, Val::Num(value));
    }
    table_set_static(
        state,
        Val::Table(info),
        "hasRequiredAura",
        Val::Bool(cost.required_aura_id == 0),
    );

    Some(Val::Table(info))
}

fn spell_power_costs_table(state: &mut LuaState, spell_id: u32) -> Option<Val> {
    let costs = crate::spell_power::get_spell_power(spell_id)?;
    if costs.is_empty() {
        return None;
    }

    let player_power_max = borrow_state(state).ok()?.player.power_max.max(0) as f32;
    let Val::Table(power_costs) = create_table(state) else {
        return None;
    };

    for (index, cost) in costs.iter().enumerate() {
        let Some(info) = build_spell_power_cost_info(state, cost, player_power_max) else {
            continue;
        };
        table_set_num(state, power_costs, (index + 1) as f64, info);
    }

    Some(Val::Table(power_costs))
}

fn get_spell_power_cost(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = u32::from_stack(state, 1)?;
    match spell_power_costs_table(state, spell_id) {
        Some(power_costs) => state.push(power_costs),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_school_string(state: &mut LuaState) -> LuaResult<u32> {
    let school_mask = u32::from_stack(state, 1)?;
    let school = match school_mask {
        1 => "Physical",
        2 => "Holy",
        4 => "Fire",
        8 => "Nature",
        16 => "Frost",
        32 => "Shadow",
        64 => "Arcane",
        _ => "Physical",
    };
    let school_string = create_string(state, school);
    state.push(school_string);
    Ok(1)
}

fn fire_cursor_changed(state: &mut LuaState) {
    for widget_id in get_event_listeners(state, "CURSOR_CHANGED") {
        let Some(handler) = get_dispatch_script(state, widget_id, "OnEvent") else {
            continue;
        };
        if !matches!(handler, Val::Function(_)) {
            continue;
        }
        let Ok(frame) = frame_ref(state, widget_id) else {
            continue;
        };
        let event_name = create_string(state, "CURSOR_CHANGED");
        if let Err(error) = protected_lua_pcall_state(state, handler, &[frame, event_name]) {
            call_error_handler_state(state, &error);
        }
    }
}

fn pickup_spell(state: &mut LuaState) -> LuaResult<u32> {
    let Some(spell_id) = Option::<u32>::from_stack(state, 1)? else {
        return Ok(0);
    };
    borrow_state_mut(state)?.cursor_item = Some(CursorInfo::Spell { spell_id });
    fire_cursor_changed(state);
    Ok(0)
}

fn get_spell_link(state: &mut LuaState) -> LuaResult<u32> {
    if cfg!(feature = "retail-12-0-5")
        && rilua::table_security::is_secret_value(state, stack_val(state, 2))
    {
        return Err(rilua::runtime_error(
            "C_Spell.GetSpellLink: argument 2 rejects secret values under NeverSecret policy",
        ));
    }
    let Some(spell_id) = numeric_spell_id(state, 1) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    match spell_link_for_id(spell_id) {
        Some(link) => {
            let link_string = create_string(state, &link);
            state.push(link_string);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_spell_name(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = u32::from_stack(state, 1)?;
    let name = spells::get_spell(spell_id)
        .map(|spell| spell.name)
        .unwrap_or("Unknown");
    let name_string = create_string_static(state, name);
    state.push(name_string);
    Ok(1)
}

/// `C_Spell.GetSpellCooldown(spellID)` → `SpellCooldownInfo` table.
///
/// Retail fields: `startTime, duration, isEnabled, isActive, modRate`.
fn get_spell_cooldown(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = u32::from_stack(state, 1)?;
    push_spell_cooldown_info(state, spell_id)
}

fn read_spell_cooldown_snapshot(
    state: &mut LuaState,
    spell_id: u32,
) -> LuaResult<(f64, f64, bool)> {
    let sim = borrow_state(state)?;
    let now = sim.start_time.elapsed().as_secs_f64();
    let (start, duration) = spell_cooldown_times(&sim, spell_id, now);
    Ok((
        start,
        duration,
        super::charge_state::cooldowns_are_restricted(&sim),
    ))
}

pub(crate) fn push_spell_cooldown_info(state: &mut LuaState, spell_id: u32) -> LuaResult<u32> {
    let (start, duration, restricted) = read_spell_cooldown_snapshot(state, spell_id)?;
    let info = create_table_with_capacity(state, SPELL_COOLDOWN_HASH_FIELDS);
    // Root the public DTO before keys or secret numeric wrappers allocate.
    state.push(info);
    for (name, number) in [
        ("startTime", start),
        ("duration", duration),
        ("modRate", 1.0),
    ] {
        let value = if restricted {
            rilua::table_security::wrap_host_secret_number(state, number)
        } else {
            Val::Num(number)
        };
        table_set_static(state, info, name, value);
    }
    table_set_static(state, info, "isEnabled", Val::Bool(true));
    table_set_static(state, info, "isActive", Val::Bool(duration > 0.0));
    Ok(1)
}

fn get_spell_charges(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = read_spell_identifier(state)?;
    super::charge_state::push_charge_info(state, spell_id)
}

#[cfg(feature = "retail-12-0-5")]
fn get_spell_charge_duration(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = read_spell_identifier(state)?;
    super::charge_state::push_charge_duration(state, spell_id)
}

#[cfg(feature = "retail-12-0-0")]
fn get_spell_cooldown_duration(state: &mut LuaState) -> LuaResult<u32> {
    let Some(spell_id) = read_spell_identifier(state)? else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let ignore_gcd = super::cooldown_duration::read_ignore_gcd(state, 2);
    let (start, seconds) = {
        let sim = borrow_state(state)?;
        let now = sim.start_time.elapsed().as_secs_f64();
        super::cooldown_duration::select_cooldown_duration_times(&sim, spell_id, now, ignore_gcd)
    };
    crate::lua_api::globals::lua_duration_object::push_timed_duration_object(state, start, seconds)
}

/// `C_Spell.GetMountFromSpell(spellID)` → mountID or nil.
///
/// Scans `world.mounts` for a matching spell_id. Returns the mount_id or nil.
fn get_mount_from_spell(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = u32::from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let mount_id = sim
        .world
        .mounts
        .iter()
        .find(|m| m.spell_id == spell_id)
        .map(|m| m.mount_id);
    drop(sim);
    match mount_id {
        Some(id) => state.push(Val::Num(id as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

/// `C_Spell.DoesSpellExist(spellID)` -> `bool`.
///
/// Permissive for addon/UI probes: any non-zero spell ID is treated as existing.
fn does_spell_exist(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = u32::from_stack(state, 1)?;
    state.push(Val::Bool(spell_id != 0));
    Ok(1)
}

/// `C_Spell.IsSpellDataCached(spellID)` -> `true` for non-zero spell IDs.
fn is_spell_data_cached(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = u32::from_stack(state, 1)?;
    state.push(Val::Bool(spell_id != 0));
    Ok(1)
}

/// `C_Spell.IsSelfBuff(spellID)` → `bool`.
///
/// Returns true when `implicit_target == 1` (TARGET_UNIT_CASTER / Self).
fn is_self_buff(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = u32::from_stack(state, 1)?;
    // implicit_target == 1 is TARGET_UNIT_CASTER (self-only cast target).
    let is_self = spells::get_spell(spell_id)
        .map(|s| s.implicit_target == 1)
        .unwrap_or(false);
    state.push(Val::Bool(is_self));
    Ok(1)
}

fn is_auto_attack_spell(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = u32::from_stack(state, 1)?;
    state.push(Val::Bool(spell_id == 6603));
    Ok(1)
}

fn spell_is_helpful(spell_id: u32) -> bool {
    spells::get_spell(spell_id)
        .map(|spell| spell.implicit_target == 1)
        .unwrap_or(false)
}

fn is_spell_helpful(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = u32::from_stack(state, 1)?;
    state.push(Val::Bool(spell_is_helpful(spell_id)));
    Ok(1)
}

fn is_spell_harmful(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = u32::from_stack(state, 1)?;
    let harmful = spells::get_spell(spell_id)
        .map(|_| !spell_is_helpful(spell_id))
        .unwrap_or(false);
    state.push(Val::Bool(harmful));
    Ok(1)
}

/// `C_Spell.IsSpellUsable(spellID)` → `(isUsable, insufficientPower)`.
///
/// Returns `(true, false)` for known spells; `(false, false)` otherwise.
fn is_spell_usable(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = u32::from_stack(state, 1)?;
    let known = is_known_spell_for_usability(state, spell_id);
    state.push(Val::Bool(known));
    state.push(Val::Bool(false));
    Ok(2)
}

fn is_known_spell_for_usability(state: &mut LuaState, spell_id: u32) -> bool {
    borrow_state(state)
        .map(|sim| sim.known_spells.contains(&spell_id))
        .unwrap_or(false)
}

pub(super) fn numeric_spell_id(state: &LuaState, index: i32) -> Option<u32> {
    let value = stack_val(state, index);
    match value {
        Val::Num(n) if n.is_finite() && n >= 0.0 => Some(n as u32),
        Val::Str(_) => {
            let input = val_to_string(state, value)?;
            input
                .parse::<u32>()
                .ok()
                .or_else(|| spell_id_by_name(&input))
        }
        _ => None,
    }
}

fn spell_id_by_name(name: &str) -> Option<u32> {
    let needle = name.to_ascii_lowercase();
    SPELL_DB
        .entries()
        .find_map(|(id, spell)| (spell.name.to_ascii_lowercase() == needle).then_some(*id))
}

/// `C_Spell.GetSpellTradeSkillLink(spellID)` → trade-skill recipe link, or
/// nil. `SpellFlyoutPopupButtonMixin:OnClick` uses this in the chat-link
/// modifier branch to insert a recipe link instead of the spell link when
/// the flyout entry corresponds to a profession recipe.
fn get_spell_trade_skill_link(state: &mut LuaState) -> LuaResult<u32> {
    let Some(spell_id) = numeric_spell_id(state, 1) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let link_text = borrow_state(state)?
        .spell_trade_skill_links
        .get(&spell_id)
        .cloned();
    match link_text {
        Some(link) => {
            let link_string = create_string(state, &link);
            state.push(link_string);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn alias_key_from_input(state: &LuaState, value: Val) -> Option<String> {
    match value {
        Val::Num(n) if n.is_finite() => Some((n as u32).to_string()),
        Val::Str(_) => val_to_string(state, value).map(|s| s.to_lowercase()),
        _ => None,
    }
}

/// `C_Spell.GetSpellIDForSpellIdentifier(identifier)` → spell id or nil.
///
/// `SpellFlyoutPopupButtonMixin:OnClick` uses this to resolve overrides
/// before calling `CastSpellByID`. Numeric input falls back to the same id
/// when no alias is registered (matches retail's identity behavior for
/// non-overridden spells); string input requires an alias entry.
fn get_spell_id_for_spell_identifier(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = read_spell_identifier(state)?;
    state.push(spell_id.map_or(Val::Nil, |id| Val::Num(id as f64)));
    Ok(1)
}

fn read_spell_identifier(state: &LuaState) -> LuaResult<Option<u32>> {
    read_spell_identifier_at(state, 1)
}

/// Strict public boundary shared by cooldown associations and spell classifications.
pub(crate) fn read_public_spell_identifier_at(
    state: &LuaState,
    arg_index: i32,
    api_name: &str,
) -> LuaResult<Option<u32>> {
    let value = stack_val(state, arg_index);
    // INFERRED conservative rejection, not native AllowedWhenTainted permissions.
    // Never unwrap secrets, inspect private payloads, or change caller taint.
    if rilua::table_security::is_secret_value(state, value) {
        return Err(rilua::runtime_error(format!(
            "{api_name}: secret spell identifier access is not modeled"
        )));
    }
    // INFERRED strict public representations; validate before alias resolution.
    match value {
        Val::Num(number)
            if number.is_finite()
                && number.fract() == 0.0
                && number >= 0.0
                && number <= u32::MAX as f64 => {}
        Val::Str(_) if val_to_string(state, value).is_some() => {}
        _ => {
            return Err(rilua::runtime_error(format!(
                "{api_name}: argument {arg_index} must be a public UTF-8 string or finite integral u32 number"
            )));
        }
    }
    read_spell_identifier_at(state, arg_index)
}

/// Shared alias-first resolution; callers own argument and secret validation.
pub(crate) fn read_spell_identifier_at(state: &LuaState, arg_index: i32) -> LuaResult<Option<u32>> {
    read_spell_identifier_value(state, stack_val(state, arg_index))
}

/// Alias-first resolution of a value; callers own authentication and validation.
pub(crate) fn read_spell_identifier_value(state: &LuaState, raw: Val) -> LuaResult<Option<u32>> {
    let Some(key) = alias_key_from_input(state, raw) else {
        return Ok(None);
    };
    let numeric_id = match raw {
        Val::Num(number) => Some(number as u32),
        _ => None,
    };
    let alias = borrow_state(state)?.spell_id_aliases.get(&key).copied();
    Ok(alias.or(numeric_id))
}

/// `C_Spell.IsCurrentSpell(spellID)` → `true` when the active cast matches.
///
/// Mirrors the global `IsCurrentSpell` probe so flyout / stance / possess
/// `UpdateState` paths stay aligned regardless of which surface they reach
/// for. Nil cast slot returns false.
fn is_current_spell(state: &mut LuaState) -> LuaResult<u32> {
    let Some(spell_id) = numeric_spell_id(state, 1) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    let match_cast = borrow_state(state)?
        .casting
        .as_ref()
        .is_some_and(|c| c.spell_id == spell_id);
    state.push(Val::Bool(match_cast));
    Ok(1)
}

/// `C_Spell.GetSpellLossOfControlCooldownInfo(spellID)` → LoC cooldown table
/// or nil. `ActionButton_UpdateCooldown` only renders the LoC overlay when
/// the table's `isActive` is true; absence (nil) falls back to the inert
/// `defaultLossOfControlInfo` baseline. Returning nil here mirrors that path
/// for spells with no registered loss-of-control entry.
fn get_spell_loss_of_control_cooldown_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(spell_id) = numeric_spell_id(state, 1) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    push_spell_loss_of_control_snapshot(state, spell_id)
}

pub(crate) fn push_spell_loss_of_control_snapshot(
    state: &mut LuaState,
    spell_id: u32,
) -> LuaResult<u32> {
    let (info, restricted) = {
        let sim = borrow_state(state)?;
        let info = sim.spell_loss_of_control.get(&spell_id).cloned();
        (info, super::charge_state::cooldowns_are_restricted(&sim))
    };
    let Some(info) = info else {
        state.push(Val::Nil);
        return Ok(1);
    };
    super::loss_of_control::push_loss_of_control_info(state, &info, restricted)
}
