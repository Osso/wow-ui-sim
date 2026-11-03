//! `SecureCmdOptionParse(options)` returns the first option with a matching
//! condition group and the explicit unit selector from that group.

use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

use crate::lua_api::globals::group_queries::visible_party_member;
use crate::lua_api::globals::unit_api::is_raid_group;
use crate::lua_api::methods::borrow_state;

pub(super) fn secure_cmd_option_parse(state: &mut LuaState) -> LuaResult<u32> {
    let arg = state.stack_get(state.base);
    let Val::Str(s_ref) = arg else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let text = {
        let lua_str = state
            .gc
            .string_arena
            .get(s_ref)
            .ok_or_else(|| runtime_error("SecureCmdOptionParse: invalid string"))?;
        std::str::from_utf8(lua_str.data())
            .map_err(|_| runtime_error("SecureCmdOptionParse: non-UTF8 string"))?
            .to_owned()
    };
    let selected = {
        let sim = borrow_state(state)?;
        resolve_cmd_option_with_explicit_unit(&text, &sim)
            .map(|(value, unit)| (value.to_owned(), unit.map(str::to_owned)))
    };
    Ok(push_cmd_option_results(state, selected))
}

fn push_cmd_option_results(
    state: &mut LuaState,
    selected: Option<(String, Option<String>)>,
) -> u32 {
    let Some((value, unit)) = selected else {
        state.push(Val::Nil);
        return 1;
    };
    let value = Val::Str(state.gc.intern_string(value.as_bytes()));
    state.push(value);
    let unit = unit.map_or(Val::Nil, |unit| {
        Val::Str(state.gc.intern_string(unit.as_bytes()))
    });
    state.push(unit);
    2
}

pub(crate) fn resolve_cmd_option<'a>(
    text: &'a str,
    sim: &crate::lua_api::SimState,
) -> Option<&'a str> {
    resolve_cmd_option_with_unit(text, sim).map(|(value, _)| value)
}

/// Select one clause using the existing condition grammar and unit selector.
pub(crate) fn resolve_cmd_option_with_unit<'a>(
    text: &'a str,
    sim: &crate::lua_api::SimState,
) -> Option<(&'a str, &'a str)> {
    resolve_cmd_option_with_explicit_unit(text, sim)
        .map(|(value, unit)| (value, unit.unwrap_or("target")))
}

fn resolve_cmd_option_with_explicit_unit<'a>(
    text: &'a str,
    sim: &crate::lua_api::SimState,
) -> Option<(&'a str, Option<&'a str>)> {
    text.split(';')
        .find_map(|clause| match_cmd_option_clause(clause, sim))
}

fn match_cmd_option_clause<'a>(
    clause: &'a str,
    sim: &crate::lua_api::SimState,
) -> Option<(&'a str, Option<&'a str>)> {
    let clause = clause.trim();
    let mut value = clause;
    let mut matched_group = None;
    while let Some((conditions, rest)) = value
        .strip_prefix('[')
        .and_then(|group| group.split_once(']'))
    {
        if matched_group.is_none() && condition_list_matches(conditions, sim) {
            matched_group = Some(conditions);
        }
        value = rest.trim_start();
    }

    if value.is_empty() {
        return None;
    }
    if value.len() == clause.len() {
        return Some((value, None));
    }
    matched_group.map(|conditions| (value, explicit_condition_unit(conditions)))
}

fn condition_list_matches(conditions: &str, sim: &crate::lua_api::SimState) -> bool {
    let unit = condition_unit(conditions);
    conditions
        .split(',')
        .map(str::trim)
        .all(|condition| condition_matches(condition, unit, sim))
}

fn condition_unit(conditions: &str) -> &str {
    explicit_condition_unit(conditions).unwrap_or("target")
}

fn explicit_condition_unit(conditions: &str) -> Option<&str> {
    // INFERRED: preserve the existing parser's last-selector-wins policy.
    conditions
        .rsplit(',')
        .find_map(|condition| parse_unit_override(condition.trim()))
}

fn parse_unit_override(condition: &str) -> Option<&str> {
    condition
        .strip_prefix('@')
        .or_else(|| condition.strip_prefix("target="))
        .or_else(|| condition.strip_prefix("unit="))
        .map(str::trim)
        .filter(|unit| !unit.is_empty())
}

fn condition_matches(condition: &str, unit: &str, sim: &crate::lua_api::SimState) -> bool {
    if condition.is_empty() || parse_unit_override(condition).is_some() {
        return true;
    }
    let condition = condition.to_ascii_lowercase();
    let (name, argument) = condition
        .split_once(':')
        .map_or((condition.as_str(), ""), |(name, argument)| {
            (name, argument)
        });
    match name {
        "combat" => sim.player.in_combat,
        "nocombat" => !sim.player.in_combat,
        "vehicleui" => sim.player.has_vehicle_ui,
        "unithasvehicleui" => match unit {
            "player" => sim.player.has_vehicle_ui,
            _ => unit_target_info(unit, sim).is_some_and(|target| target.has_vehicle_ui),
        },
        "mod" => modifier_condition_matches(argument, sim, true),
        "nomod" => modifier_condition_matches(argument, sim, false),
        "harm" => unit_is_harmful(unit, sim),
        "noharm" => !unit_is_harmful(unit, sim),
        "help" => unit_is_helpful(unit, sim),
        "nohelp" => !unit_is_helpful(unit, sim),
        "exists" => unit_exists_for_option(unit, sim),
        "noexists" => !unit_exists_for_option(unit, sim),
        "dead" => unit_is_dead(unit, sim),
        "nodead" => !unit_is_dead(unit, sim),
        "group" => group_condition_matches(argument, sim),
        "nogroup" => !group_condition_matches(argument, sim),
        _ => false,
    }
}

fn modifier_condition_matches(
    argument: &str,
    sim: &crate::lua_api::SimState,
    expected: bool,
) -> bool {
    let actual = if argument.is_empty() {
        sim.modifier_keys.any_modifier()
    } else {
        argument.split('/').any(|key| modifier_key_down(key, sim))
    };
    actual == expected
}

fn modifier_key_down(key: &str, sim: &crate::lua_api::SimState) -> bool {
    match key.trim() {
        "shift" => sim.modifier_keys.shift,
        "ctrl" | "control" => sim.modifier_keys.control,
        "alt" => sim.modifier_keys.alt,
        "meta" => sim.modifier_keys.meta,
        _ => false,
    }
}

fn group_condition_matches(argument: &str, sim: &crate::lua_api::SimState) -> bool {
    match argument {
        "" => sim.party_group_active,
        "party" => sim.party_group_active && !is_raid_group(sim),
        "raid" => is_raid_group(sim),
        _ => false,
    }
}

fn unit_exists_for_option(unit: &str, sim: &crate::lua_api::SimState) -> bool {
    match unit {
        "player" | "pet" | "vehicle" => true,
        "target" => sim.current_target.is_some(),
        "focus" => sim.current_focus.is_some(),
        other => visible_party_member(sim, other).is_some(),
    }
}

fn unit_is_harmful(unit: &str, sim: &crate::lua_api::SimState) -> bool {
    match unit_target_info(unit, sim) {
        Some(target) => target.is_enemy || target.reaction < 4,
        None => false,
    }
}

fn unit_is_helpful(unit: &str, sim: &crate::lua_api::SimState) -> bool {
    match unit_target_info(unit, sim) {
        Some(target) => !target.is_enemy || target.reaction >= 4,
        None => {
            visible_party_member(sim, unit).is_some()
                || matches!(unit, "player" | "pet" | "vehicle")
        }
    }
}

fn unit_is_dead(unit: &str, sim: &crate::lua_api::SimState) -> bool {
    match unit_target_info(unit, sim) {
        Some(target) => target.health <= 0,
        None => visible_party_member(sim, unit).is_some_and(|member| member.dead_since.is_some()),
    }
}

fn unit_target_info<'a>(
    unit: &str,
    sim: &'a crate::lua_api::SimState,
) -> Option<&'a crate::lua_api::game_data::TargetInfo> {
    match unit {
        "target" => sim.current_target.as_ref(),
        "focus" => sim.current_focus.as_ref(),
        _ => None,
    }
}
