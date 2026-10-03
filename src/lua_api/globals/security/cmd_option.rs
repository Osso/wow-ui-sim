//! `SecureCmdOptionParse(options)` — returns the first option whose bracketed
//! condition list matches current simulator state.

use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

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
        resolve_cmd_option(&text, &sim).map(str::to_string)
    };
    match selected {
        Some(value) => {
            let result = Val::Str(state.gc.intern_string(value.as_bytes()));
            state.push(result);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
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
    text.split(';')
        .filter_map(parse_cmd_option_clause)
        .find_map(|clause| {
            clause.matches(sim).then(|| {
                let unit = clause.conditions.map_or("target", condition_unit);
                (clause.value, unit)
            })
        })
}

struct CmdOptionClause<'a> {
    conditions: Option<&'a str>,
    value: &'a str,
}

impl<'a> CmdOptionClause<'a> {
    fn matches(&self, sim: &crate::lua_api::SimState) -> bool {
        self.conditions
            .is_none_or(|conditions| condition_list_matches(conditions, sim))
    }
}

fn parse_cmd_option_clause(clause: &str) -> Option<CmdOptionClause<'_>> {
    let trimmed = clause.trim();
    if trimmed.is_empty() {
        return None;
    }
    let Some(rest) = trimmed.strip_prefix('[') else {
        return Some(CmdOptionClause {
            conditions: None,
            value: trimmed,
        });
    };
    let Some(close_index) = rest.find(']') else {
        return Some(CmdOptionClause {
            conditions: None,
            value: trimmed,
        });
    };
    let conditions = &rest[..close_index];
    let value = rest[close_index + 1..].trim();
    (!value.is_empty()).then_some(CmdOptionClause {
        conditions: Some(conditions),
        value,
    })
}

fn condition_list_matches(conditions: &str, sim: &crate::lua_api::SimState) -> bool {
    let unit = condition_unit(conditions);
    conditions
        .split(',')
        .map(str::trim)
        .all(|condition| condition_matches(condition, unit, sim))
}

fn condition_unit(conditions: &str) -> &str {
    // INFERRED: preserve the existing parser's last-selector-wins policy.
    conditions
        .rsplit(',')
        .find_map(|condition| parse_unit_override(condition.trim()))
        .unwrap_or("target")
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
        "" | "party" | "raid" => sim.party_group_active,
        _ => false,
    }
}

fn unit_exists_for_option(unit: &str, sim: &crate::lua_api::SimState) -> bool {
    match unit {
        "player" | "pet" | "vehicle" => true,
        "target" => sim.current_target.is_some(),
        "focus" => sim.current_focus.is_some(),
        other => visible_party_member_for_option(other, sim).is_some(),
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
            visible_party_member_for_option(unit, sim).is_some()
                || matches!(unit, "player" | "pet" | "vehicle")
        }
    }
}

fn unit_is_dead(unit: &str, sim: &crate::lua_api::SimState) -> bool {
    match unit_target_info(unit, sim) {
        Some(target) => target.health <= 0,
        None => visible_party_member_for_option(unit, sim)
            .is_some_and(|member| member.dead_since.is_some()),
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

fn visible_party_member_for_option<'a>(
    unit: &str,
    sim: &'a crate::lua_api::SimState,
) -> Option<&'a crate::lua_api::state::PartyMember> {
    if !sim.party_group_active {
        return None;
    }
    if let Some(idx) = crate::lua_api::globals::unit_api::parse_party_index(unit) {
        return sim.party_members.get(idx);
    }
    unit.strip_prefix("raid")
        .and_then(|rest| rest.parse::<usize>().ok())
        .and_then(|n| n.checked_sub(1))
        .and_then(|idx| sim.party_members.get(idx))
}
