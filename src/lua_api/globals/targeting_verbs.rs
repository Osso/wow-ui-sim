//! SimState-backed implementations of WoW player targeting globals.
//!
//! Migrated off `GLOBAL_NIL_STUBS` in `stubs/global_stubs.rs`. Each function
//! reads and writes `SimState::current_target`, `current_focus`,
//! `previous_target`, and `enemy_pool` then queues the matching WoW event.
//!
//! | Global               | Event fired                |
//! |----------------------|----------------------------|
//! | `TargetUnit`         | `PLAYER_TARGET_CHANGED`    |
//! | `FocusUnit`          | `PLAYER_FOCUS_CHANGED`     |
//! | `AssistUnit`         | `PLAYER_TARGET_CHANGED`    |
//! | `ClearTarget`        | `PLAYER_TARGET_CHANGED`    |
//! | `ClearFocus`         | `PLAYER_FOCUS_CHANGED`     |
//! | `IsTargetLoose`      | none (query only)          |
//! | `CanBeRaidTarget`    | none (query only)          |
//! | `GetRaidTargetIndex` | none (query only)          |
//! | `SetRaidTarget`      | `RAID_TARGET_UPDATE`       |
//! | `SetRaidTargetIcon`  | `RAID_TARGET_UPDATE`       |
//! | `TargetLastTarget`   | `PLAYER_TARGET_CHANGED`    |
//! | `TargetNearestEnemy` | `PLAYER_TARGET_CHANGED`    |
//! | `TargetNearestFriend`| `PLAYER_TARGET_CHANGED`    |
//!
//! `TargetUnit` resolves the supplied unit token against the current sim
//! state using the same token set as `UnitExists` / `UnitName`. Unrecognised
//! tokens are a silent no-op (matches real WoW: targeting an invalid unit
//! does nothing).
//!
//! `TargetNearestEnemy` / `TargetNearestFriend` pick the first entry of
//! `enemy_pool` / party members respectively. `enemy1` falls back to a seeded
//! default enemy when the pool is empty. Empty nearest-pool lookups are silent
//! no-ops.

use crate::event::Event;
use crate::lua_api::game_data::{PartyMember, TargetInfo};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string, frame_ref};
use crate::lua_api::script_helpers::{
    call_error_handler_state, get_event_listeners, get_script, protected_lua_pcall_state,
};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::LuaResult;
use rilua::vm::state::LuaState;

// ── Token resolution ──────────────────────────────────────────────────────────

/// Resolve a unit token to a `TargetInfo` snapshot from the current state.
/// Returns `None` for tokens that don't map to an existing unit.
fn resolve_token_to_target_info(
    state: &mut LuaState,
    token: &str,
) -> LuaResult<Option<TargetInfo>> {
    let st = borrow_state_mut(state)?;
    Ok(resolve_unit_snapshot(&st, token))
}

pub(crate) fn resolve_unit_snapshot(
    st: &crate::lua_api::state::SimState,
    token: &str,
) -> Option<TargetInfo> {
    let token = token.to_ascii_lowercase();
    let token = if token == "softinteract" {
        st.soft_interact_target.as_deref()?
    } else {
        &token
    };
    match token {
        "player" | "self" => Some(player_target_info(&st)),
        "target" => st.current_target.clone(),
        "focus" => st.current_focus.clone(),
        other => resolve_party_token(&st, other).or_else(|| resolve_enemy_token(&st, other)),
    }
}

fn player_target_info(st: &crate::lua_api::state::SimState) -> TargetInfo {
    TargetInfo {
        unit_id: "player".to_string(),
        name: st.player.name.clone(),
        class_index: st.player.class_index,
        level: st.player.level,
        health: st.player.health,
        health_max: st.player.health_max,
        power: st.player.power,
        power_max: st.player.power_max,
        power_type: st.player.power_type,
        power_type_name: "MANA".to_string(),
        is_player: true,
        is_enemy: false,
        guid: super::unit_misc::guid_for_unit(st, "player"),
        interaction: Default::default(),
        classification: "normal".to_string(),
        creature_type: "Humanoid".to_string(),
        reaction: 5,
    }
}

fn resolve_party_token(st: &crate::lua_api::state::SimState, token: &str) -> Option<TargetInfo> {
    let idx = parse_party_slot(token)?;
    st.party_members.get(idx).map(|member| {
        party_member_to_target_info(member, super::unit_misc::guid_for_unit(st, token))
    })
}

fn resolve_enemy_token(st: &crate::lua_api::state::SimState, token: &str) -> Option<TargetInfo> {
    let idx = token
        .strip_prefix("enemy")
        .and_then(|rest| rest.parse::<usize>().ok())
        .and_then(|n| n.checked_sub(1))?;
    if let Some(enemy) = st.enemy_pool.get(idx) {
        return Some(enemy.clone());
    }
    (idx == 0).then(default_enemy_target_info)
}

fn default_enemy_target_info() -> TargetInfo {
    TargetInfo {
        unit_id: "target".to_string(),
        name: "Hogger".to_string(),
        class_index: 1,
        level: 11,
        health: 85_000,
        health_max: 85_000,
        power: 0,
        power_max: 100,
        power_type: 1,
        power_type_name: "RAGE".to_string(),
        is_player: false,
        is_enemy: true,
        guid: "Creature-0-0-0-0-448-000001".to_string(),
        interaction: Default::default(),
        classification: "normal".to_string(),
        creature_type: "Humanoid".to_string(),
        reaction: 2,
    }
}

fn parse_party_slot(token: &str) -> Option<usize> {
    let b = token.as_bytes();
    if b.len() == 6 && b[..5].eq_ignore_ascii_case(b"party") {
        match b[5] {
            b'1' => Some(0),
            b'2' => Some(1),
            b'3' => Some(2),
            b'4' => Some(3),
            _ => None,
        }
    } else {
        None
    }
}

fn party_member_to_target_info(m: &PartyMember, guid: String) -> TargetInfo {
    TargetInfo {
        unit_id: "target".to_string(),
        name: m.name.clone(),
        class_index: m.class_index,
        level: m.level,
        health: m.health,
        health_max: m.health_max,
        power: m.power,
        power_max: m.power_max,
        power_type: m.power_type,
        power_type_name: m.power_type_name.clone(),
        is_player: true,
        is_enemy: false,
        guid,
        interaction: Default::default(),
        classification: "normal".to_string(),
        creature_type: "Humanoid".to_string(),
        reaction: 5,
    }
}

// ── Event helpers ─────────────────────────────────────────────────────────────

fn push_target_changed(state: &mut LuaState) -> LuaResult<()> {
    borrow_state_mut(state)?.events.push(Event {
        name: "PLAYER_TARGET_CHANGED".to_string(),
        args: Vec::new(),
    });
    fire_event_now(state, "PLAYER_TARGET_CHANGED", &[]);
    sync_target_frame_visibility(state)?;
    Ok(())
}

fn push_focus_changed(state: &mut LuaState) -> LuaResult<()> {
    borrow_state_mut(state)?.events.push(Event {
        name: "PLAYER_FOCUS_CHANGED".to_string(),
        args: Vec::new(),
    });
    fire_event_now(state, "PLAYER_FOCUS_CHANGED", &[]);
    Ok(())
}

fn fire_event_now(state: &mut LuaState, event_name: &str, args: &[rilua::Val]) {
    for widget_id in get_event_listeners(state, event_name) {
        let Some(handler) = get_script(state, widget_id, "OnEvent") else {
            continue;
        };
        if !matches!(handler, rilua::Val::Function(_)) {
            continue;
        }
        let Ok(frame) = frame_ref(state, widget_id) else {
            continue;
        };
        let event_name_val = create_string(state, event_name);
        let mut call_args = Vec::with_capacity(args.len() + 2);
        call_args.push(frame);
        call_args.push(event_name_val);
        call_args.extend_from_slice(args);
        if let Err(error) = protected_lua_pcall_state(state, handler, &call_args) {
            call_error_handler_state(state, &error);
        }
    }
}

fn sync_target_frame_visibility(state: &mut LuaState) -> LuaResult<()> {
    let show_target_frame = borrow_state(state)?.current_target.is_some();
    let mut sim = borrow_state_mut(state)?;
    let Some(target_frame_id) = sim.widgets.get_id_by_name("TargetFrame") else {
        return Ok(());
    };
    if let Some(frame) = sim.widgets.get_mut_visual(target_frame_id) {
        frame.visible = show_target_frame;
    }
    Ok(())
}

// ── Globals ───────────────────────────────────────────────────────────────────

/// `TargetUnit(unit)` — resolve token, set `current_target`, snapshot previous.
pub fn target_unit(state: &mut LuaState) -> LuaResult<u32> {
    let token = match Option::<String>::from_stack(state, 1)? {
        Some(t) => t,
        None => return Ok(0),
    };
    let new_target = resolve_token_to_target_info(state, &token)?;
    {
        let mut st = borrow_state_mut(state)?;
        let old = st.current_target.take();
        st.previous_target = old;
        st.current_target = new_target;
    }
    push_target_changed(state)?;
    Ok(0)
}

/// `FocusUnit(unit)` — resolve token, set `current_focus`.
pub fn focus_unit(state: &mut LuaState) -> LuaResult<u32> {
    let token = match Option::<String>::from_stack(state, 1)? {
        Some(t) => t,
        None => return Ok(0),
    };
    let new_focus = resolve_token_to_target_info(state, &token)?;
    {
        let mut st = borrow_state_mut(state)?;
        st.current_focus = new_focus;
    }
    push_focus_changed(state)?;
    Ok(0)
}

fn assisted_target_for_unit(state: &mut LuaState, token: &str) -> LuaResult<Option<TargetInfo>> {
    let Some(unit) = resolve_token_to_target_info(state, token)? else {
        return Ok(None);
    };
    if unit.is_enemy {
        let st = borrow_state(state)?;
        return Ok(Some(player_target_info(&st)));
    }
    Ok(Some(default_enemy_target_info()))
}

/// `AssistUnit(unit)` — target the selected unit's simulated target.
pub fn assist_unit(state: &mut LuaState) -> LuaResult<u32> {
    let token = match Option::<String>::from_stack(state, 1)? {
        Some(t) => t,
        None => return Ok(0),
    };
    let Some(new_target) = assisted_target_for_unit(state, &token)? else {
        return Ok(0);
    };
    {
        let mut st = borrow_state_mut(state)?;
        let old = st.current_target.take();
        st.previous_target = old;
        st.current_target = Some(new_target);
    }
    push_target_changed(state)?;
    Ok(0)
}

/// `ClearTarget()` — clear `current_target`, snapshot to `previous_target`.
pub fn clear_target(state: &mut LuaState) -> LuaResult<u32> {
    let cleared_target = {
        let mut st = borrow_state_mut(state)?;
        let old = st.current_target.take();
        let cleared_target = old.is_some();
        st.previous_target = old;
        cleared_target
    };
    push_target_changed(state)?;
    state.push(rilua::Val::Bool(cleared_target));
    Ok(1)
}

/// `ClearFocus()` — clear `current_focus`.
pub fn clear_focus(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.current_focus = None;
    push_focus_changed(state)?;
    Ok(0)
}

/// `IsTargetLoose()` — false until the sim models soft/loose targeting.
pub fn is_target_loose(state: &mut LuaState) -> LuaResult<u32> {
    state.push(rilua::Val::Bool(false));
    Ok(1)
}

/// `CanBeRaidTarget(unit)` — true when the token resolves to a live unit.
pub fn can_be_raid_target(state: &mut LuaState) -> LuaResult<u32> {
    let token = match Option::<String>::from_stack(state, 1)? {
        Some(token) => token,
        None => {
            state.push(rilua::Val::Bool(false));
            return Ok(1);
        }
    };
    let can_mark = resolve_token_to_target_info(state, &token)?.is_some();
    state.push(rilua::Val::Bool(can_mark));
    Ok(1)
}

/// `GetRaidTargetIndex(unit)` — assigned unit icon, or nil for an unmarked unit.
pub fn get_raid_target_index(state: &mut LuaState) -> LuaResult<u32> {
    let target = match Option::<String>::from_stack(state, 1)? {
        Some(token) => resolve_token_to_target_info(state, &token)?,
        None => None,
    };
    let icon = match target {
        Some(target) => borrow_state(state)?
            .unit_raid_target_icons
            .get(&target.guid)
            .copied(),
        None => None,
    };
    state.push(icon.map_or(rilua::Val::Nil, |icon| rilua::Val::Num(icon as f64)));
    Ok(1)
}

/// Search the eight user markers, including the starting index; zero means exhausted.
#[cfg(feature = "client-wowforever")]
fn get_next_available_raid_target_marker_index(state: &mut LuaState) -> LuaResult<u32> {
    let start = i32::from_stack(state, 1)?;
    if !(1..=8).contains(&start) {
        return Err(rilua::runtime_error("startIndex must be between 1 and 8"));
    }
    let reverse = Option::<bool>::from_stack(state, 2)?.unwrap_or(false);
    let wrap = Option::<bool>::from_stack(state, 3)?.unwrap_or(false);
    let allow_dead = Option::<bool>::from_stack(state, 4)?.unwrap_or(false);
    let occupied = occupied_raid_target_markers(state, allow_dead)?;
    let direction = if reverse { -1 } else { 1 };
    let next = (0..8)
        .map(|offset| start + offset * direction)
        .find_map(|index| {
            if !wrap && !(1..=8).contains(&index) {
                return None;
            }
            let index = (index - 1).rem_euclid(8) + 1;
            (!occupied[index as usize]).then_some(index)
        });
    state.push(rilua::Val::Num(next.unwrap_or(0) as f64));
    Ok(1)
}

#[cfg(feature = "client-wowforever")]
fn occupied_raid_target_markers(state: &LuaState, allow_dead: bool) -> LuaResult<[bool; 9]> {
    let sim = borrow_state(state)?;
    let mut occupied = [false; 9];
    for (guid, &marker) in &sim.unit_raid_target_icons {
        let unit = sim
            .current_target
            .iter()
            .chain(sim.current_focus.iter())
            .chain(sim.enemy_pool.iter())
            .find(|unit| &unit.guid == guid);
        let reusable = allow_dead && unit.is_some_and(|unit| unit.health <= 0 && unit.reaction < 5);
        if !reusable {
            occupied[marker as usize] = true;
        }
    }
    Ok(occupied)
}

fn read_raid_target_icon(state: &LuaState) -> LuaResult<u8> {
    match crate::lua_bridge::stack_val(state, 2) {
        rilua::Val::Num(index)
            if index.is_finite() && index.fract() == 0.0 && (0.0..=8.0).contains(&index) =>
        {
            Ok(index as u8)
        }
        _ => Err(rilua::runtime_error(
            "SetRaidTarget: index must be an integer from 0 to 8",
        )),
    }
}

/// Explicit simulator policy: assign/move one unit icon, or clear it with zero.
/// Notify synchronously after mutation, including repeated valid assignments.
pub fn set_raid_target(state: &mut LuaState) -> LuaResult<u32> {
    let Some(token) = Option::<String>::from_stack(state, 1)? else {
        return Ok(0);
    };
    let Some(target) = resolve_token_to_target_info(state, &token)? else {
        return Ok(0);
    };
    let icon = read_raid_target_icon(state)?;
    {
        let mut sim = borrow_state_mut(state)?;
        let icons = &mut sim.unit_raid_target_icons;
        icons.retain(|guid, assigned| guid != &target.guid && *assigned != icon);
        if icon != 0 {
            icons.insert(target.guid, icon);
        }
    }
    borrow_state_mut(state)?.events.push(Event {
        name: "RAID_TARGET_UPDATE".to_string(),
        args: Vec::new(),
    });
    fire_event_now(state, "RAID_TARGET_UPDATE", &[]);
    Ok(0)
}

/// `TargetLastTarget()` — swap `current_target` ↔ `previous_target`.
pub fn target_last_target(state: &mut LuaState) -> LuaResult<u32> {
    let has_previous = borrow_state_mut(state)?.previous_target.is_some();
    if !has_previous {
        return Ok(0);
    }
    {
        let mut st = borrow_state_mut(state)?;
        let old_current = st.current_target.take();
        let previous = st.previous_target.take();
        st.current_target = previous;
        st.previous_target = old_current;
    }
    push_target_changed(state)?;
    Ok(0)
}

/// `TargetNearestEnemy()` — pick first member of `enemy_pool`. No-op when empty.
pub fn target_nearest_enemy(state: &mut LuaState) -> LuaResult<u32> {
    let Some(new_target) = ({
        let st = borrow_state(state)?;
        st.enemy_pool.first().cloned()
    }) else {
        return Ok(0);
    };
    {
        let mut st = borrow_state_mut(state)?;
        let old = st.current_target.take();
        st.previous_target = old;
        st.current_target = Some(new_target);
    }
    push_target_changed(state)?;
    Ok(0)
}

/// `TargetNearestFriend()` — pick first party member. No-op when party is empty.
pub fn target_nearest_friend(state: &mut LuaState) -> LuaResult<u32> {
    let new_target = {
        let st = borrow_state(state)?;
        if st.party_group_active {
            resolve_party_token(&st, "party1")
        } else {
            None
        }
    };
    if new_target.is_none() {
        return Ok(0);
    }
    {
        let mut st = borrow_state_mut(state)?;
        let old = st.current_target.take();
        st.previous_target = old;
        st.current_target = new_target;
    }
    push_target_changed(state)?;
    Ok(0)
}

// ── Admin setter ──────────────────────────────────────────────────────────────

/// `A_Admin.SetEnemyPool({name, level, class_index, is_enemy=true}, ...)`.
///
/// Replaces `SimState::enemy_pool` entirely. Each entry in the Lua array must
/// be a table with at minimum a `name` key. `level` defaults to 63,
/// `class_index` to 1. `is_enemy` is always forced true regardless of what
/// the caller passes.
pub fn admin_set_enemy_pool(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_api::globals::admin_party_target_helpers::make_target_info;
    use crate::lua_api::methods::table_get;
    use rilua::Val;

    let nargs = state.top as i32 - state.base as i32;
    let mut pool: Vec<TargetInfo> = Vec::new();
    for i in 1..=nargs {
        let entry_val = crate::lua_bridge::stack_val(state, i);
        let Val::Table(_) = entry_val else {
            continue;
        };

        let name_val = table_get(state, entry_val, "name");
        let name = match name_val {
            Val::Str(s) => state
                .gc
                .string_arena
                .get(s)
                .map(|ls| String::from_utf8_lossy(ls.data()).into_owned())
                .unwrap_or_else(|| "Unknown".to_string()),
            _ => "Unknown".to_string(),
        };

        let level_val = table_get(state, entry_val, "level");
        let level = match level_val {
            Val::Num(n) => n as i32,
            _ => 63,
        };

        let class_val = table_get(state, entry_val, "class_index");
        let class_index = match class_val {
            Val::Num(n) => n as i32,
            _ => 1,
        };

        pool.push(make_target_info("target", &name, level, class_index, true));
    }
    borrow_state_mut(state)?.enemy_pool = pool;
    Ok(0)
}

// ── Registration ──────────────────────────────────────────────────────────────

pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    use rilua::LuaApiMut;
    let state = lua.state_mut();
    let g = state.global;
    table_set_rust_fn_static(state, g, "TargetUnit", target_unit)?;
    table_set_rust_fn_static(state, g, "FocusUnit", focus_unit)?;
    table_set_rust_fn_static(state, g, "AssistUnit", assist_unit)?;
    table_set_rust_fn_static(state, g, "ClearTarget", clear_target)?;
    table_set_rust_fn_static(state, g, "ClearFocus", clear_focus)?;
    table_set_rust_fn_static(state, g, "IsTargetLoose", is_target_loose)?;
    table_set_rust_fn_static(state, g, "CanBeRaidTarget", can_be_raid_target)?;
    table_set_rust_fn_static(state, g, "GetRaidTargetIndex", get_raid_target_index)?;
    #[cfg(feature = "client-wowforever")]
    table_set_rust_fn_static(
        state,
        g,
        "GetNextAvailableRaidTargetMarkerIndex",
        get_next_available_raid_target_marker_index,
    )?;
    table_set_rust_fn_static(state, g, "SetRaidTarget", set_raid_target)?;
    table_set_rust_fn_static(state, g, "SetRaidTargetIcon", set_raid_target)?;
    table_set_rust_fn_static(state, g, "TargetLastTarget", target_last_target)?;
    table_set_rust_fn_static(state, g, "TargetNearestEnemy", target_nearest_enemy)?;
    table_set_rust_fn_static(state, g, "TargetNearestFriend", target_nearest_friend)?;
    Ok(())
}
