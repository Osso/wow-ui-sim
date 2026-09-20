//! Private helpers shared between party/target admin setters in
//! `admin.rs`. Split out of `admin.rs` to keep that file under the
//! 750-line cap without disturbing the public `A_Admin.*` surface.
//!
//! NOTE: `src/lua_api/globals/admin_api/units.rs` has its own separate
//! copies of these helpers (pre-dating the rilua migration). Those are
//! untouched here — consolidating the two copies is a separate cleanup.

use crate::lua_api::game_data::{PartyMember, TargetInfo};
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::state::SimState;
use crate::lua_bridge::FromStack;
use rilua::LuaResult;
use rilua::vm::state::LuaState;

/// Build a `TargetInfo` with default stats from admin-supplied unit
/// parameters. GUID is stamped with the subsecond clock so repeated
/// admin calls produce distinguishable GUIDs without requiring the
/// caller to supply one.
pub(super) fn make_target_info(
    unit_id: &str,
    name: &str,
    level: i32,
    class_index: i32,
    is_enemy: bool,
) -> TargetInfo {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let guid = if is_enemy {
        format!("Creature-0-0-0-0-0-{}", nanos % 1_000_000)
    } else {
        format!("Player-0000-{:08}", nanos % 100_000_000)
    };
    TargetInfo {
        unit_id: unit_id.to_string(),
        name: name.to_string(),
        class_index,
        level,
        health: 100_000,
        health_max: 100_000,
        power: 50_000,
        power_max: 100_000,
        power_type: 0,
        power_type_name: "MANA".to_string(),
        is_player: !is_enemy,
        is_enemy,
        guid,
        classification: "normal".to_string(),
        creature_type: "Humanoid".to_string(),
        reaction: if is_enemy { 2 } else { 5 },
        interaction: Default::default(),
    }
}

/// Build a default `PartyMember` for padding when admin grows the
/// party beyond the seeded roster.
pub(super) fn default_party_member() -> PartyMember {
    PartyMember {
        name: "Unknown".to_string(),
        class_index: 1,
        level: 80,
        health: 100_000,
        health_max: 100_000,
        power: 0,
        power_max: 100,
        power_type: 1,
        power_type_name: "RAGE".to_string(),
        is_leader: false,
        dead_since: None,
        buffs: vec![],
        debuffs: vec![],
    }
}

// ── Targeting ────────────────────────────────────────────────────────────────

pub(super) fn set_target(state: &mut LuaState) -> LuaResult<u32> {
    set_selected_unit(state, "target", |st, unit| {
        st.current_target = Some(unit);
    })
}

pub(super) fn clear_target(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.current_target = None;
    Ok(0)
}

pub(super) fn set_focus(state: &mut LuaState) -> LuaResult<u32> {
    set_selected_unit(state, "focus", |st, unit| {
        st.current_focus = Some(unit);
    })
}

fn set_selected_unit(
    state: &mut LuaState,
    unit_id: &'static str,
    assign_unit: impl FnOnce(&mut SimState, TargetInfo),
) -> LuaResult<u32> {
    let name = String::from_stack(state, 1)?;
    let level = i32::from_stack(state, 2)?;
    let class_index = i32::from_stack(state, 3)?;
    let is_enemy = bool::from_stack(state, 4)?;
    let unit = make_target_info(unit_id, &name, level, class_index, is_enemy);
    let mut st = borrow_state_mut(state)?;
    assign_unit(&mut st, unit);
    Ok(0)
}

pub(super) fn clear_focus(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.current_focus = None;
    Ok(0)
}

pub(super) fn set_target_power(state: &mut LuaState) -> LuaResult<u32> {
    set_selected_unit_power(state, |st| st.current_target.as_mut())
}

pub(super) fn set_focus_power(state: &mut LuaState) -> LuaResult<u32> {
    set_selected_unit_power(state, |st| st.current_focus.as_mut())
}

fn set_selected_unit_power(
    state: &mut LuaState,
    select_unit: impl for<'a> FnOnce(&'a mut SimState) -> Option<&'a mut TargetInfo>,
) -> LuaResult<u32> {
    let cur = i32::from_stack(state, 1)?;
    let max = i32::from_stack(state, 2)?;
    let power_type = Option::<i32>::from_stack(state, 3)?;
    let mut st = borrow_state_mut(state)?;
    if let Some(unit) = select_unit(&mut st) {
        unit.power = cur;
        unit.power_max = max;
        if let Some(pt) = power_type {
            unit.power_type = pt;
        }
    }
    Ok(0)
}

pub(super) fn set_target_type(state: &mut LuaState) -> LuaResult<u32> {
    set_selected_unit_type(state, |st| st.current_target.as_mut())
}

pub(super) fn set_focus_type(state: &mut LuaState) -> LuaResult<u32> {
    set_selected_unit_type(state, |st| st.current_focus.as_mut())
}

fn set_selected_unit_type(
    state: &mut LuaState,
    select_unit: impl for<'a> FnOnce(&'a mut SimState) -> Option<&'a mut TargetInfo>,
) -> LuaResult<u32> {
    let classification = Option::<String>::from_stack(state, 1)?;
    let creature_type = Option::<String>::from_stack(state, 2)?;
    let reaction = Option::<i32>::from_stack(state, 3)?;
    let mut st = borrow_state_mut(state)?;
    if let Some(unit) = select_unit(&mut st) {
        if let Some(c) = classification {
            unit.classification = c;
        }
        if let Some(ct) = creature_type {
            unit.creature_type = ct;
        }
        if let Some(r) = reaction {
            unit.reaction = r;
        }
    }
    Ok(0)
}

pub(super) fn set_focus_health(state: &mut LuaState) -> LuaResult<u32> {
    let cur = i32::from_stack(state, 1)?;
    let max = i32::from_stack(state, 2)?;
    let mut st = borrow_state_mut(state)?;
    if let Some(unit) = st.current_focus.as_mut() {
        unit.health = cur;
        unit.health_max = max;
    }
    Ok(0)
}

// ── Party ─────────────────────────────────────────────────────────────────────

pub(super) fn set_party_size(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_api::globals::state_backed_queries::dispatch_event_now;

    let n = i32::from_stack(state, 1)?;
    let size = n.max(0) as usize;
    let should_refresh = {
        let mut st = borrow_state_mut(state)?;
        let size_changed = st.party_members.len() != size;
        while st.party_members.len() < size {
            st.party_members.push(default_party_member());
        }
        st.party_members.truncate(size);
        st.party_group_active = size > 0;
        let next_leader = None;
        let leader_changed = st.party_leader_index != next_leader;
        st.party_leader_index = next_leader;
        size_changed || leader_changed
    };
    if should_refresh {
        dispatch_event_now(state, "GROUP_ROSTER_UPDATE", &[])?;
    }
    Ok(0)
}

pub(super) fn set_party_leader(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_api::globals::state_backed_queries::dispatch_event_now;

    let n = i32::from_stack(state, 1)?;
    let changed = {
        let mut st = borrow_state_mut(state)?;
        let next_leader = if n <= 0 {
            Some(None)
        } else {
            let idx = (n - 1) as usize;
            (idx < st.party_members.len()).then_some(Some(idx))
        };
        let Some(next_leader) = next_leader else {
            return Ok(0);
        };
        if st.party_leader_index == next_leader {
            false
        } else {
            st.party_leader_index = next_leader;
            true
        }
    };
    if changed {
        dispatch_event_now(state, "GROUP_ROSTER_UPDATE", &[])?;
    }
    Ok(0)
}

pub(super) fn set_party_member(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_api::globals::state_backed_queries::dispatch_event_now;

    let idx = i32::from_stack(state, 1)?;
    let name = String::from_stack(state, 2)?;
    let class_index = i32::from_stack(state, 3)?;
    let level = i32::from_stack(state, 4)?;
    let changed = {
        let mut st = borrow_state_mut(state)?;
        let mut changed = false;
        if let Some(member) = st.party_members.get_mut((idx - 1) as usize) {
            changed =
                member.name != name || member.class_index != class_index || member.level != level;
            member.name = name;
            member.class_index = class_index;
            member.level = level;
        }
        changed
    };
    if changed {
        dispatch_event_now(state, "GROUP_ROSTER_UPDATE", &[])?;
    }
    Ok(0)
}

pub(super) fn set_party_member_health(state: &mut LuaState) -> LuaResult<u32> {
    let idx = i32::from_stack(state, 1)?;
    let cur = i32::from_stack(state, 2)?;
    let max = i32::from_stack(state, 3)?;
    let mut st = borrow_state_mut(state)?;
    if let Some(member) = st.party_members.get_mut((idx - 1) as usize) {
        member.health = cur;
        member.health_max = max;
    }
    Ok(0)
}

pub(super) fn kill_party_member(state: &mut LuaState) -> LuaResult<u32> {
    let idx = i32::from_stack(state, 1)?;
    let mut st = borrow_state_mut(state)?;
    if let Some(member) = st.party_members.get_mut((idx - 1) as usize) {
        member.dead_since = Some(std::time::Instant::now());
    }
    Ok(0)
}

pub(super) fn res_party_member(state: &mut LuaState) -> LuaResult<u32> {
    let idx = i32::from_stack(state, 1)?;
    let mut st = borrow_state_mut(state)?;
    if let Some(member) = st.party_members.get_mut((idx - 1) as usize) {
        member.dead_since = None;
    }
    Ok(0)
}

pub(super) fn set_rot_damage(state: &mut LuaState) -> LuaResult<u32> {
    let level = i32::from_stack(state, 1)?;
    borrow_state_mut(state)?.rot_damage_level = level as usize;
    Ok(0)
}
