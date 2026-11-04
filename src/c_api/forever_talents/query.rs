//! WM90 query parsing and publication. Forever declaration:
//! Blizzard_APIDocumentationGenerated/SpecializationInfoDocumentation.lua:316.

use super::{ForeverTalentState, TalentGroup, TalentRow};
use crate::lua_api::globals::unit_misc::existing_guid_for_unit;
use crate::lua_api::methods::{
    borrow_state, create_string, create_table, table_get, table_set, val_to_string,
};
use crate::lua_api::state::SimState;
use crate::lua_bridge::stack_val;
use rilua::{LuaResult, Val, vm::state::LuaState};

struct Query {
    group: Option<i64>,
    tier: i64,
    column: i64,
    inspect: bool,
    target: Option<String>,
}

pub(crate) fn get_talent_info(state: &mut LuaState) -> LuaResult<u32> {
    let query = parse_query(state)?;
    let result = {
        let sim = borrow_state(state)?;
        build_result(&sim, &query)
    };
    let value = match result {
        Some(result) => publish_result(state, result),
        None => Val::Nil,
    };
    state.push(value);
    Ok(1)
}

fn read_field(state: &mut LuaState, query: Val, name: &str) -> LuaResult<Val> {
    let value = table_get(state, query, name);
    rilua::table_security::unwrap_secret(state, value)
}

fn read_index(state: &mut LuaState, query: Val, name: &str) -> LuaResult<Option<i64>> {
    match read_field(state, query, name)? {
        Val::Nil => Ok(None),
        Val::Num(n) if n.is_finite() && (0.0..=f64::from(u32::MAX)).contains(&n) => {
            // The shared native reader subtracts before integer conversion.
            Ok(Some((n - 1.0).trunc() as i64))
        }
        Val::Num(_) => Err(rilua::runtime_error(format!(
            "Current Field: [{name}], outside of expected range 0 to 4294967295"
        ))),
        _ => Err(rilua::runtime_error(format!(
            "query.{name} requires an index"
        ))),
    }
}

fn require_coordinate(value: Option<i64>, name: &str) -> LuaResult<i64> {
    value.ok_or_else(|| {
        rilua::runtime_error(format!(
            "C_SpecializationInfo.GetTalent: query.{name} must be specified."
        ))
    })
}

fn parse_query(state: &mut LuaState) -> LuaResult<Query> {
    let query = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    if !matches!(query, Val::Table(_)) {
        return Err(rilua::runtime_error(
            "Usage: local result = C_SpecializationInfo.GetTalentInfo(query)",
        ));
    }
    let group = read_index(state, query, "groupIndex")?;
    let inspect = !matches!(
        read_field(state, query, "isInspect")?,
        Val::Nil | Val::Bool(false)
    );
    let tier = read_index(state, query, "tier")?;
    let column = read_index(state, query, "column")?;
    let target = read_field(state, query, "target")?;
    if !matches!(target, Val::Nil | Val::Num(_) | Val::Str(_)) {
        return Err(rilua::runtime_error("query.target requires a unit token"));
    }
    let target = val_to_string(state, target);
    read_index(state, query, "specializationIndex")?;
    read_index(state, query, "talentIndex")?;
    read_field(state, query, "isPet")?;
    Ok(Query {
        group,
        tier: require_coordinate(tier, "tier")?,
        column: require_coordinate(column, "column")?,
        inspect,
        target,
    })
}

struct ResultRecord {
    row: TalentRow,
    name: String,
    icon: u32,
    selected: bool,
    known: bool,
    available: bool,
    granted: bool,
}

fn group_at(groups: &[TalentGroup], index: i64) -> Option<&TalentGroup> {
    groups.get(usize::try_from(index).ok()?)
}

fn select_row(
    grid: &ForeverTalentState,
    class_id: i32,
    spec_id: u32,
    q: &Query,
) -> Option<TalentRow> {
    let mut neutral = None;
    for row in &grid.rows {
        if row.class_id != class_id
            || i64::from(row.tier_id) != q.tier
            || i64::from(row.column_index) != q.column
            || row.id > 32767
            || row.spec_id > 32767
        {
            continue;
        }
        if row.spec_id == spec_id {
            return Some(row.clone());
        }
        if row.spec_id == 0 {
            neutral = Some(row.clone());
        }
    }
    neutral
}

fn allowance(grid: &ForeverTalentState, class_id: i32) -> u32 {
    if let Some(allowance) = grid.allowance {
        return allowance;
    }
    let Some(effective) = grid.level_allowances.get(&grid.effective_level) else {
        return 0;
    };
    let Some(base) = grid.level_allowances.get(&grid.base_level) else {
        return 0;
    };
    let cap = match class_id {
        6 => base.death_knight,
        12 => base.demon_hunter,
        _ => base.num_talents,
    };
    effective.num_talents.min(cap)
}

fn build_result(sim: &SimState, q: &Query) -> Option<ResultRecord> {
    if !(0..7).contains(&q.tier) || !(0..3).contains(&q.column) {
        return None;
    }
    let grid = &sim.talents.forever;
    let (class_id, group, grants) = if q.inspect {
        let inspection = grid.inspection.as_ref()?;
        let target = q.target.as_deref()?;
        let guid = existing_guid_for_unit(sim, target).unwrap_or_else(|| target.to_owned());
        if guid != inspection.guid || guid.is_empty() {
            return None;
        }
        let group = group_at(
            &inspection.groups,
            q.group.unwrap_or(inspection.current_group as i64),
        )?;
        (inspection.class_id, group, Some(&inspection.aura_grants))
    } else {
        if !sim.player.present {
            return None;
        }
        let group = group_at(&grid.groups, q.group.unwrap_or(grid.current_group as i64))?;
        let grants = match q.target.as_deref() {
            None | Some("player") => Some(&grid.aura_grants),
            Some(target) => {
                let guid = existing_guid_for_unit(sim, target).unwrap_or_else(|| target.to_owned());
                if existing_guid_for_unit(sim, "player").as_deref() == Some(&guid) {
                    Some(&grid.aura_grants)
                } else {
                    grid.subject_aura_grants.get(&guid)
                }
            }
        };
        (sim.player.class_index as i32, group, grants)
    };
    let row = select_row(grid, class_id, group.spec_id, q)?;
    if row.id == 0 {
        return None;
    }
    let name = grid.spell_names.get(&row.spell_id)?.clone();
    let icon = grid.spell_icons.get(&row.spell_id).map_or(0, |icons| {
        if icons.visual_icon != 0 {
            icons.visual_icon
        } else {
            icons.misc_icon
        }
    });
    let selected = group.selected_ids.contains(&row.id);
    let granted = grants.is_some_and(|grants| grants.contains(&row.id));
    let current = grid.groups.get(grid.current_group);
    let known = current.is_some_and(|g| g.selected_ids.contains(&row.id));
    let available = selected
        || (sim.player.present
            && !granted
            && current.is_some_and(|g| {
                g.tier_mask & (1 << row.tier_id) == 0
                    && row.tier_id < allowance(grid, sim.player.class_index as i32)
            }));
    Some(ResultRecord {
        row,
        name,
        icon,
        selected,
        known,
        available,
        granted,
    })
}

fn signed_number(value: u32) -> Val {
    Val::Num(f64::from(value as i32))
}

fn publish_result(state: &mut LuaState, r: ResultRecord) -> Val {
    let result = create_table(state);
    let name = create_string(state, &r.name);
    table_set(state, result, "name", name);
    for (key, value) in [
        ("talentID", signed_number(r.row.id)),
        ("spellID", signed_number(r.row.spell_id)),
        (
            "icon",
            if r.icon == 0 {
                Val::Nil
            } else {
                signed_number(r.icon)
            },
        ),
        ("tier", Val::Num(f64::from(r.row.tier_id + 1))),
        ("column", Val::Num(f64::from(r.row.column_index + 1))),
        ("selected", Val::Bool(r.selected)),
        ("available", Val::Bool(r.available)),
        ("known", Val::Bool(r.known)),
        ("grantedByAura", Val::Bool(r.granted)),
        ("isPVPTalentUnlocked", Val::Bool(false)),
        ("rank", Val::Num(1.0)),
        ("maxRank", Val::Num(1.0)),
        ("previewRank", Val::Num(1.0)),
        ("meetsPrereq", Val::Bool(false)),
        ("meetsPreviewPrereq", Val::Bool(false)),
        ("isExceptional", Val::Bool(false)),
        ("hasGoldBorder", Val::Bool(false)),
    ] {
        table_set(state, result, key, value);
    }
    result
}
