use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use crate::specializations;
use rilua::vm::state::LuaState;
use rilua::{LuaError, LuaResult, RuntimeError, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Wrath {
        return Ok(());
    }
    table_set_rust_fn_static(state, state.global, "GetLootSpecialization", get)?;
    table_set_rust_fn_static(state, state.global, "SetLootSpecialization", set)?;
    table_set_rust_fn_static(state, state.global, "HasLootSpecializations", has)?;
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
    let value = match stack_val(state, 1) {
        Val::Num(n) => n as i64,
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
