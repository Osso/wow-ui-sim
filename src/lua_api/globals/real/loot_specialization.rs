use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use crate::specializations;
use rilua::vm::state::LuaState;
use rilua::{LuaError, LuaResult, RuntimeError, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE == ClientProfile::Wrath {
        return Ok(());
    }
    table_set_rust_fn_static(state, state.global, "GetLootSpecialization", get)?;
    table_set_rust_fn_static(state, state.global, "SetLootSpecialization", set)?;
    if ACTIVE != ClientProfile::Retail {
        table_set_rust_fn_static(state, state.global, "HasLootSpecializations", has)?;
    }
    Ok(())
}

fn get(state: &mut LuaState) -> LuaResult<u32> {
    let id = borrow_state(state)?.loot_specialization;
    state.push(Val::Num(id as f64));
    Ok(1)
}

fn has(state: &mut LuaState) -> LuaResult<u32> {
    let class = borrow_state(state)?.player.class_index.max(1) as u32;
    state.push(Val::Bool(
        specializations::specs_for_class(class).next().is_some(),
    ));
    Ok(1)
}

fn set(state: &mut LuaState) -> LuaResult<u32> {
    if ACTIVE == ClientProfile::Retail {
        return set_retail(state);
    }
    set_simulator(state)
}

/// No server transport is modeled; keep the verified argument boundary without
/// inventing a local update or a `PLAYER_LOOT_SPEC_UPDATED` response.
fn set_retail(state: &mut LuaState) -> LuaResult<u32> {
    if !matches!(stack_val(state, 1), Val::Num(_)) {
        return Err(LuaError::Runtime(RuntimeError {
            message: "Usage: SetLootSpecialization(specializationID)".into(),
            level: 1,
            traceback: vec![],
        }));
    }
    Ok(0)
}

fn set_simulator(state: &mut LuaState) -> LuaResult<u32> {
    let value = match stack_val(state, 1) {
        Val::Num(number) => number as i64,
        _ => {
            return Err(LuaError::Runtime(RuntimeError {
                message: "Usage: SetLootSpecialization(specializationID)".into(),
                level: 1,
                traceback: vec![],
            }));
        }
    };
    let mut sim = borrow_state_mut(state)?;
    let now = sim.clock_now();
    if sim
        .loot_spec_window
        .is_none_or(|(start, _)| now.duration_since(start).as_secs() >= 10)
    {
        sim.loot_spec_window = Some((now, 0));
    }
    let (_, count) = sim.loot_spec_window.as_mut().expect("window initialized");
    if *count < 5 {
        *count += 1;
        sim.pending_loot_specs.push(value);
    }
    Ok(0)
}

pub(crate) fn apply_pending(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE == ClientProfile::Retail {
        return Ok(());
    }
    let requests = std::mem::take(&mut borrow_state_mut(state)?.pending_loot_specs);
    for id in requests {
        let accepted = {
            let mut sim = borrow_state_mut(state)?;
            let class = sim.player.class_index.max(1) as u32;
            let valid = id == 0
                || specializations::specs_for_class(class).any(|spec| i64::from(spec.id) == id);
            if valid {
                sim.loot_specialization = id as u32;
            }
            valid
        };
        if accepted {
            fire_named_event_state(state, "PLAYER_LOOT_SPEC_UPDATED", &[]);
        }
    }
    Ok(())
}
