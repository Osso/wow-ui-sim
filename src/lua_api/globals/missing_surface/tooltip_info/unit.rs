use super::super::{
    LINE_TYPE_UNIT_NAME, TOOLTIP_TYPE_UNIT, WORLD_CURSOR_GUID, WORLD_LOOT_TOOLTIP_INVENTORY_TYPE,
    WORLD_LOOT_TOOLTIP_SPELL_ID,
};
use super::builders::{empty_tooltip, push_plain_line, push_tooltip_line};
use super::spell::tooltip_for_spell_id;
use crate::lua_api::game_data::CLASS_LABELS;
use crate::lua_api::methods::{borrow_state, create_string, table_get, table_set};
use crate::lua_api::state::{
    PartyMember, PlayerState, RACE_DATA, SEEDED_LOCAL_CHARACTER_GUID, SimState, TargetInfo,
};
use rilua::Val;
use rilua::vm::state::LuaState;

fn class_color(class_index: i32) -> (f64, f64, f64) {
    match class_index {
        1 => (0.78, 0.61, 0.43),
        2 => (0.96, 0.55, 0.73),
        3 => (0.67, 0.83, 0.45),
        4 => (1.0, 0.96, 0.41),
        5 => (1.0, 1.0, 1.0),
        6 => (0.77, 0.12, 0.23),
        7 => (0.0, 0.44, 0.87),
        8 => (0.25, 0.78, 0.92),
        9 => (0.53, 0.53, 0.93),
        10 => (0.0, 1.0, 0.6),
        11 => (1.0, 0.49, 0.04),
        12 => (0.64, 0.19, 0.79),
        13 => (0.2, 0.58, 0.5),
        _ => (1.0, 1.0, 1.0),
    }
}

pub(super) struct UnitTooltipInfo {
    pub(super) name: String,
    pub(super) level: i32,
    pub(super) race: String,
    pub(super) class_name: String,
    pub(super) color: (f64, f64, f64),
}

fn class_label(class_index: i32) -> String {
    CLASS_LABELS
        .get((class_index - 1).max(0) as usize)
        .copied()
        .unwrap_or("Unknown")
        .to_string()
}

fn target_tooltip_info(target: &TargetInfo) -> UnitTooltipInfo {
    UnitTooltipInfo {
        name: target.name.clone(),
        level: target.level,
        race: target.creature_type.clone(),
        class_name: class_label(target.class_index),
        color: class_color(target.class_index),
    }
}

fn player_tooltip_info(player: &PlayerState) -> UnitTooltipInfo {
    let race = RACE_DATA
        .get(player.race_index)
        .map(|(name, _, _)| (*name).to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    UnitTooltipInfo {
        name: player.name.clone(),
        level: player.level,
        race,
        class_name: class_label(player.class_index),
        color: class_color(player.class_index),
    }
}

fn party_tooltip_info(member: &PartyMember) -> UnitTooltipInfo {
    UnitTooltipInfo {
        name: member.name.clone(),
        level: member.level,
        race: "Player".to_string(),
        class_name: class_label(member.class_index),
        color: class_color(member.class_index),
    }
}

fn active_party_member_index(sim: &SimState, unit: &str) -> Option<usize> {
    if !sim.party_group_active {
        return None;
    }
    let idx = crate::lua_api::globals::unit_api::parse_party_index(unit)?;
    (idx < sim.party_members.len()).then_some(idx)
}

fn active_party_member<'a>(sim: &'a SimState, unit: &str) -> Option<&'a PartyMember> {
    let idx = active_party_member_index(sim, unit)?;
    sim.party_members.get(idx)
}

pub(super) fn unit_tooltip_info(state: &LuaState, unit: &str) -> Option<UnitTooltipInfo> {
    let sim = borrow_state(state).ok()?;
    match unit {
        "target" => sim.current_target.as_ref().map(target_tooltip_info),
        "focus" => sim.current_focus.as_ref().map(target_tooltip_info),
        "player" => Some(player_tooltip_info(&sim.player)),
        other => active_party_member(&sim, other).map(party_tooltip_info),
    }
}

fn unit_guid(state: &LuaState, unit: &str) -> Option<String> {
    let sim = borrow_state(state).ok()?;
    match unit {
        "player" => Some(SEEDED_LOCAL_CHARACTER_GUID.to_string()),
        "target" => sim
            .current_target
            .as_ref()
            .map(|target| target.guid.clone()),
        "focus" => sim.current_focus.as_ref().map(|target| target.guid.clone()),
        other => active_party_member_index(&sim, other)
            .map(|idx| format!("Player-0000-000000{:02}", idx + 2)),
    }
}

fn find_unit_for_guid(sim: &SimState, guid: &str) -> Option<String> {
    if guid == SEEDED_LOCAL_CHARACTER_GUID {
        return Some("player".into());
    }
    if sim
        .current_target
        .as_ref()
        .is_some_and(|target| target.guid == guid)
    {
        return Some("target".into());
    }
    if sim
        .current_focus
        .as_ref()
        .is_some_and(|focus| focus.guid == guid)
    {
        return Some("focus".into());
    }
    if !sim.party_group_active {
        return None;
    }
    (0..sim.party_members.len())
        .find(|idx| guid == format!("Player-0000-000000{:02}", idx + 2))
        .map(|idx| format!("party{}", idx + 1))
}

pub(super) fn tooltip_for_unit_guid(state: &mut LuaState, guid: &str) -> Option<Val> {
    let unit = find_unit_for_guid(&borrow_state(state).ok()?, guid)?;
    let info = unit_tooltip_info(state, &unit)?;
    if info.name.is_empty() {
        return None;
    }
    Some(tooltip_for_unit(state, &unit))
}

fn push_unit_tooltip_lines(state: &mut LuaState, lines: Val, info: &UnitTooltipInfo) {
    push_tooltip_line(
        state,
        lines,
        1,
        LINE_TYPE_UNIT_NAME,
        &info.name,
        Some(info.color),
        false,
    );
    let level_text = format!("Level {}", info.level);
    push_plain_line(state, lines, 2, &level_text);
    push_plain_line(state, lines, 3, &info.race);
    push_plain_line(state, lines, 4, &info.class_name);
}

pub(super) fn tooltip_for_unit(state: &mut LuaState, unit: &str) -> Val {
    let tooltip = empty_tooltip(state, TOOLTIP_TYPE_UNIT);
    let lines = table_get(state, tooltip, "lines");
    if let Some(info) = unit_tooltip_info(state, unit) {
        push_unit_tooltip_lines(state, lines, &info);
        if let Some(guid) = unit_guid(state, unit) {
            let guid = create_string(state, &guid);
            table_set(state, tooltip, "guid", guid);
        }
    }
    tooltip
}

pub(super) fn tooltip_for_world_loot(state: &mut LuaState) -> Val {
    let tooltip = tooltip_for_spell_id(state, WORLD_LOOT_TOOLTIP_SPELL_ID);
    table_set(
        state,
        tooltip,
        "worldLootObjectInventoryType",
        Val::Num(WORLD_LOOT_TOOLTIP_INVENTORY_TYPE),
    );
    let guid = create_string(state, WORLD_CURSOR_GUID);
    table_set(state, tooltip, "worldLootObjectGUID", guid);
    tooltip
}
