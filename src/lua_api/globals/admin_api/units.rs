use crate::lua_api::game_data::{PartyMember, TargetInfo};
use crate::lua_api::state::SimState;
use mlua::{Lua, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

pub(super) fn register_targeting_api(
    lua: &Lua,
    t: &mlua::Table,
    state: Rc<RefCell<SimState>>,
) -> Result<()> {
    add_target_unit_setter(lua, t, "SetTarget", Rc::clone(&state), TargetSlot::Target)?;
    add_target_unit_clearer(lua, t, "ClearTarget", Rc::clone(&state), TargetSlot::Target)?;
    add_target_unit_setter(lua, t, "SetFocus", Rc::clone(&state), TargetSlot::Focus)?;
    add_target_unit_clearer(lua, t, "ClearFocus", Rc::clone(&state), TargetSlot::Focus)?;
    add_target_power_setter(
        lua,
        t,
        "SetTargetPower",
        Rc::clone(&state),
        TargetSlot::Target,
    )?;
    add_target_power_setter(
        lua,
        t,
        "SetFocusPower",
        Rc::clone(&state),
        TargetSlot::Focus,
    )?;
    add_target_type_setter(
        lua,
        t,
        "SetTargetType",
        Rc::clone(&state),
        TargetSlot::Target,
    )?;
    add_target_type_setter(lua, t, "SetFocusType", Rc::clone(&state), TargetSlot::Focus)?;
    add_target_health_setter(lua, t, "SetFocusHealth", state, TargetSlot::Focus)?;
    Ok(())
}

pub(super) fn register_party_api(
    lua: &Lua,
    t: &mlua::Table,
    state: Rc<RefCell<SimState>>,
) -> Result<()> {
    add_party_size_setter(lua, t, Rc::clone(&state))?;
    add_party_member_setter(
        lua,
        t,
        "SetPartyMember",
        Rc::clone(&state),
        |member, args| {
            let (name, class_index, level) = args;
            member.name = name;
            member.class_index = class_index;
            member.level = level;
        },
    )?;
    add_party_member_setter(
        lua,
        t,
        "SetPartyMemberHealth",
        Rc::clone(&state),
        |member, args| {
            let (cur, max) = args;
            member.health = cur;
            member.health_max = max;
        },
    )?;
    add_party_member_index_action(lua, t, "KillPartyMember", Rc::clone(&state), |member| {
        member.dead_since = Some(Instant::now());
    })?;
    add_party_member_index_action(lua, t, "ResPartyMember", Rc::clone(&state), |member| {
        member.dead_since = None;
    })?;
    super::add_rot_damage_setter(lua, t, state)?;
    Ok(())
}

pub(super) fn register_movement_api(
    lua: &Lua,
    t: &mlua::Table,
    state: Rc<RefCell<SimState>>,
) -> Result<()> {
    add_movement_bool_setter(lua, t, "SetMoving", Rc::clone(&state), |movement, value| {
        movement.moving = value;
    })?;
    add_movement_bool_setter(
        lua,
        t,
        "SetMounted",
        Rc::clone(&state),
        |movement, value| {
            movement.mounted = value;
        },
    )?;
    add_movement_bool_setter(lua, t, "SetFlying", Rc::clone(&state), |movement, value| {
        movement.flying = value;
    })?;
    add_movement_bool_setter(
        lua,
        t,
        "SetFalling",
        Rc::clone(&state),
        |movement, value| {
            movement.falling = value;
        },
    )?;
    add_movement_bool_setter(lua, t, "SetSwimming", state, |movement, value| {
        movement.swimming = value;
    })?;
    Ok(())
}

#[derive(Clone, Copy)]
enum TargetSlot {
    Target,
    Focus,
}

fn add_movement_bool_setter<F>(
    lua: &Lua,
    t: &mlua::Table,
    name: &str,
    state: Rc<RefCell<SimState>>,
    apply: F,
) -> Result<()>
where
    F: Fn(&mut crate::lua_api::state::MovementState, bool) + 'static,
{
    super::set_fn(lua, t, name, move |_, value: bool| {
        apply(&mut state.borrow_mut().player.movement, value);
        Ok(())
    })
}

fn add_target_unit_setter(
    lua: &Lua,
    t: &mlua::Table,
    name: &str,
    state: Rc<RefCell<SimState>>,
    slot: TargetSlot,
) -> Result<()> {
    super::set_fn(
        lua,
        t,
        name,
        move |_, (name, level, class_index, is_enemy): (String, i32, i32, bool)| {
            set_target_slot(
                &mut state.borrow_mut(),
                slot,
                Some(make_target_info(
                    unit_id_for_slot(slot),
                    &name,
                    level,
                    class_index,
                    is_enemy,
                )),
            );
            Ok(())
        },
    )
}

fn add_target_unit_clearer(
    lua: &Lua,
    t: &mlua::Table,
    name: &str,
    state: Rc<RefCell<SimState>>,
    slot: TargetSlot,
) -> Result<()> {
    super::set_fn(lua, t, name, move |_, ()| {
        set_target_slot(&mut state.borrow_mut(), slot, None);
        Ok(())
    })
}

fn add_target_power_setter(
    lua: &Lua,
    t: &mlua::Table,
    name: &str,
    state: Rc<RefCell<SimState>>,
    slot: TargetSlot,
) -> Result<()> {
    super::set_fn(
        lua,
        t,
        name,
        move |_, (cur, max, power_type): (i32, i32, Option<i32>)| {
            if let Some(unit) = target_slot_mut(&mut state.borrow_mut(), slot) {
                unit.power = cur;
                unit.power_max = max;
                if let Some(power_type) = power_type {
                    unit.power_type = power_type;
                }
            }
            Ok(())
        },
    )
}

fn add_target_type_setter(
    lua: &Lua,
    t: &mlua::Table,
    name: &str,
    state: Rc<RefCell<SimState>>,
    slot: TargetSlot,
) -> Result<()> {
    super::set_fn(
        lua,
        t,
        name,
        move |_,
              (classification, creature_type, reaction): (
            Option<String>,
            Option<String>,
            Option<i32>,
        )| {
            if let Some(unit) = target_slot_mut(&mut state.borrow_mut(), slot) {
                if let Some(classification) = classification {
                    unit.classification = classification;
                }
                if let Some(creature_type) = creature_type {
                    unit.creature_type = creature_type;
                }
                if let Some(reaction) = reaction {
                    unit.reaction = reaction;
                }
            }
            Ok(())
        },
    )
}

fn add_target_health_setter(
    lua: &Lua,
    t: &mlua::Table,
    name: &str,
    state: Rc<RefCell<SimState>>,
    slot: TargetSlot,
) -> Result<()> {
    super::set_fn(lua, t, name, move |_, (cur, max): (i32, i32)| {
        if let Some(unit) = target_slot_mut(&mut state.borrow_mut(), slot) {
            unit.health = cur;
            unit.health_max = max;
        }
        Ok(())
    })
}

fn set_target_slot(state: &mut SimState, slot: TargetSlot, target: Option<TargetInfo>) {
    match slot {
        TargetSlot::Target => state.current_target = target,
        TargetSlot::Focus => state.current_focus = target,
    }
}

fn target_slot_mut(state: &mut SimState, slot: TargetSlot) -> Option<&mut TargetInfo> {
    match slot {
        TargetSlot::Target => state.current_target.as_mut(),
        TargetSlot::Focus => state.current_focus.as_mut(),
    }
}

fn unit_id_for_slot(slot: TargetSlot) -> &'static str {
    match slot {
        TargetSlot::Target => "target",
        TargetSlot::Focus => "focus",
    }
}

fn make_target_info(
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
        guid: target_guid(is_enemy, nanos),
        classification: target_classification().to_string(),
        creature_type: target_creature_type().to_string(),
        reaction: if is_enemy { 2 } else { 5 },
        interaction: Default::default(),
    }
}

fn target_guid(is_enemy: bool, nanos: u32) -> String {
    if is_enemy {
        format!("Creature-0-0-0-0-0-{}", nanos % 1_000_000)
    } else {
        format!("Player-0000-{:08}", nanos % 100_000_000)
    }
}

fn target_classification() -> &'static str {
    "normal"
}

fn target_creature_type() -> &'static str {
    "Humanoid"
}

fn add_party_member_setter<F, A>(
    lua: &Lua,
    t: &mlua::Table,
    name: &str,
    state: Rc<RefCell<SimState>>,
    apply: F,
) -> Result<()>
where
    F: Fn(&mut PartyMember, A) + 'static,
    A: mlua::FromLuaMulti,
{
    super::set_fn(lua, t, name, move |_, (idx, args): (i32, A)| {
        with_party_member(&mut state.borrow_mut(), idx, |member| apply(member, args));
        Ok(())
    })
}

fn add_party_member_index_action<F>(
    lua: &Lua,
    t: &mlua::Table,
    name: &str,
    state: Rc<RefCell<SimState>>,
    apply: F,
) -> Result<()>
where
    F: Fn(&mut PartyMember) + 'static,
{
    super::set_fn(lua, t, name, move |_, idx: i32| {
        with_party_member(&mut state.borrow_mut(), idx, &apply);
        Ok(())
    })
}

fn add_party_size_setter(lua: &Lua, t: &mlua::Table, state: Rc<RefCell<SimState>>) -> Result<()> {
    super::set_fn(lua, t, "SetPartySize", move |lua, n: i32| {
        let size = n.max(0) as usize;
        let changed = {
            let mut state = state.borrow_mut();
            let changed = state.party_members.len() != size;
            resize_party_members(&mut state, size);
            state.party_group_active = size > 0;
            changed
        };
        if changed {
            fire_wow_event(lua, "GROUP_ROSTER_UPDATE")?;
        }
        Ok(())
    })
}

fn with_party_member<F>(state: &mut SimState, idx: i32, apply: F)
where
    F: FnOnce(&mut PartyMember),
{
    if let Some(member) = party_member_mut(state, idx) {
        apply(member);
    }
}

fn resize_party_members(state: &mut SimState, size: usize) {
    while state.party_members.len() < size {
        state.party_members.push(default_party_member());
    }
    state.party_members.truncate(size);
}

fn default_party_member() -> PartyMember {
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

fn party_member_mut(state: &mut SimState, idx: i32) -> Option<&mut PartyMember> {
    state.party_members.get_mut((idx - 1) as usize)
}

fn fire_wow_event(lua: &Lua, event_name: &str) -> Result<()> {
    let fire: mlua::Function = lua.globals().get("FireEvent")?;
    fire.call(mlua::MultiValue::from_vec(vec![Value::String(
        lua.create_string(event_name)?,
    )]))
}
