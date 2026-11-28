//! Player movement/stance actions and the DEATH dialog's control predicates.
//! Retail/Forever PlayerScriptDocumentation and UnitDocumentation blocks are
//! byte-identical. Original consumers: Bindings_Standard.xml (TOGGLESHEATH,
//! FOLLOWTARGET, DISMOUNT), SlashCommands.lua (/follow, /dismount), ZoneText.lua
//! AutoFollowStatus, ModelSceneUtil.lua (GetSheathState) and GameDialogDefs.lua
//! (DEATH OnUpdate `IsOutOfBounds`, RESURRECT OnCancel `UnitIsControlling`).
//! Native follow matching, sheath events and controlled units are INFERRED (R9-7).

use crate::lua_api::SimState;
use crate::lua_api::globals::group_queries::{unit_exists_in_state, unit_name_for};
use crate::lua_api::globals::real::admin_fields as fields;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

/// `GetSheathState` values: 1 = weapons sheathed, 2 = melee weapons drawn.
const SHEATHED: u8 = 1;
const MELEE_DRAWN: u8 = 2;

#[derive(Debug)]
pub struct PlayerActionState {
    pub sheath_state: u8,
    pub allow_low_level_raid: bool,
    pub out_of_bounds: bool,
    pub controlling_unit: bool,
    pub following: Option<String>,
}

impl Default for PlayerActionState {
    fn default() -> Self {
        Self {
            sheath_state: SHEATHED,
            allow_low_level_raid: false,
            out_of_bounds: false,
            controlling_unit: false,
            following: None,
        }
    }
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    let functions: [(&str, rilua::vm::closure::RustFn); 8] = [
        ("Dismount", dismount),
        ("GetSheathState", get_sheath_state),
        ("ToggleSheath", toggle_sheath),
        ("FollowUnit", follow_unit),
        ("GetAllowLowLevelRaid", get_allow_low_level_raid),
        ("SetAllowLowLevelRaid", set_allow_low_level_raid),
        ("IsOutOfBounds", is_out_of_bounds),
        ("UnitIsControlling", unit_is_controlling),
    ];
    for (name, function) in functions {
        LuaApiMut::register_function(lua, name, function)?;
    }
    Ok(())
}

/// `A_Admin.SetOutOfBounds(bool)`, `A_Admin.SetControllingUnit(bool)` and
/// `A_Admin.EndAutoFollow()` (movement breaks the follow; raises AUTOFOLLOW_END).
pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder
        .set_function("SetOutOfBounds", |state| {
            let value = fields::boolean_arg(state, 1, "A_Admin.SetOutOfBounds")?;
            borrow_state_mut(state)?.player_actions.out_of_bounds = value;
            Ok(0)
        })?
        .set_function("SetControllingUnit", |state| {
            let value = fields::boolean_arg(state, 1, "A_Admin.SetControllingUnit")?;
            borrow_state_mut(state)?.player_actions.controlling_unit = value;
            Ok(0)
        })?
        .set_function("EndAutoFollow", end_auto_follow)
}

fn push_bool(state: &mut LuaState, read: fn(&SimState) -> bool) -> LuaResult<u32> {
    let value = read(&*borrow_state(state)?);
    state.push(Val::Bool(value));
    Ok(1)
}

/// INFERRED: leaving the mount changes the mount display the action bars watch.
fn dismount(state: &mut LuaState) -> LuaResult<u32> {
    {
        let mut sim = borrow_state_mut(state)?;
        if !sim.player.movement.mounted {
            return Ok(0);
        }
        sim.player.movement.mounted = false;
    }
    dispatch_event_now(state, "PLAYER_MOUNT_DISPLAY_CHANGED", &[])?;
    Ok(0)
}

fn get_sheath_state(state: &mut LuaState) -> LuaResult<u32> {
    let sheath = borrow_state(state)?.player_actions.sheath_state;
    state.push(Val::Num(f64::from(sheath)));
    Ok(1)
}

fn toggle_sheath(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    let actions = &mut sim.player_actions;
    actions.sheath_state = if actions.sheath_state == SHEATHED {
        MELEE_DRAWN
    } else {
        SHEATHED
    };
    Ok(0)
}

/// `FollowUnit(name, exactMatch)`: `name` is a unit token or a visible unit's name.
fn follow_unit(state: &mut LuaState) -> LuaResult<u32> {
    let name = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let exact = matches!(stack_val(state, 2), Val::Bool(true));
    let Some(leader) = resolve_follow_name(state, &name, exact)? else {
        return Ok(0);
    };
    borrow_state_mut(state)?.player_actions.following = Some(leader.clone());
    let payload = create_string(state, &leader);
    dispatch_event_now(state, "AUTOFOLLOW_BEGIN", &[payload])?;
    Ok(0)
}

const FOLLOW_CANDIDATES: [&str; 6] = ["target", "focus", "party1", "party2", "party3", "party4"];

/// The player cannot follow themselves; unknown names follow nobody.
fn resolve_follow_name(state: &mut LuaState, name: &str, exact: bool) -> LuaResult<Option<String>> {
    let token = name.to_ascii_lowercase();
    let player = unit_name_for(state, "player")?;
    if token != "player" && unit_exists_in_state(&*borrow_state(state)?, &token) {
        let leader = unit_name_for(state, &token)?;
        return Ok((leader != player).then_some(leader));
    }
    for unit in FOLLOW_CANDIDATES {
        if !unit_exists_in_state(&*borrow_state(state)?, unit) {
            continue;
        }
        let candidate = unit_name_for(state, unit)?;
        let matched = if exact {
            candidate == name
        } else {
            candidate.eq_ignore_ascii_case(name)
        };
        if matched {
            return Ok(Some(candidate));
        }
    }
    Ok(None)
}

fn end_auto_follow(state: &mut LuaState) -> LuaResult<u32> {
    if borrow_state_mut(state)?
        .player_actions
        .following
        .take()
        .is_some()
    {
        dispatch_event_now(state, "AUTOFOLLOW_END", &[])?;
    }
    Ok(0)
}

fn get_allow_low_level_raid(state: &mut LuaState) -> LuaResult<u32> {
    push_bool(state, |sim| sim.player_actions.allow_low_level_raid)
}

fn set_allow_low_level_raid(state: &mut LuaState) -> LuaResult<u32> {
    let allow = match stack_val(state, 1) {
        Val::Nil => false,
        Val::Bool(allow) => allow,
        _ => return Err(runtime_error("SetAllowLowLevelRaid requires a boolean")),
    };
    borrow_state_mut(state)?.player_actions.allow_low_level_raid = allow;
    Ok(0)
}

fn is_out_of_bounds(state: &mut LuaState) -> LuaResult<u32> {
    push_bool(state, |sim| sim.player_actions.out_of_bounds)
}

/// Only the player's control of another unit is modeled; other units report false.
fn unit_is_controlling(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?.to_ascii_lowercase();
    let controlling = unit == "player" && borrow_state(state)?.player_actions.controlling_unit;
    state.push(Val::Bool(controlling));
    Ok(1)
}
