//! INFERRED sound registration/removal model with tick-diffed aura-change playback.

mod inputs;
pub(crate) mod playback;
pub use inputs::{AuraSoundRegistration, PrivateAuraSoundRegistrations};
pub use playback::AuraSoundPlayback;

use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

mod add;

const API_NAME: &str = "C_UnitAuras.RemovePrivateAuraAppliedSound";

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_UnitAuras")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "RemovePrivateAuraAppliedSound",
        remove_private_aura_applied_sound,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "AddPrivateAuraAppliedSound",
        add::add_private,
    )?;
    #[cfg(feature = "retail-12-1-0")]
    table_set_rust_fn_static(state, namespace, "AddAuraSound", add::add_modern)?;
    // The cached 12.1 deprecated chunk aliases legacy to modern after bootstrap.
    #[cfg(feature = "retail-12-1-0")]
    table_set_rust_fn_static(
        state,
        namespace,
        "RemoveAuraSound",
        remove_private_aura_applied_sound,
    )?;
    Ok(())
}

fn read_public_sound_id(state: &LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    // INFERRED conservative policy, including secure callers; never unwrap payloads.
    if rilua::table_security::is_secret_value(state, value) {
        return Err(rilua::runtime_error(format!(
            "{API_NAME}: secret sound identifier access is not modeled"
        )));
    }
    match value {
        Val::Num(number)
            if number.is_finite()
                && number.fract() == 0.0
                && number >= 0.0
                && number <= u32::MAX as f64 =>
        {
            Ok(number as u32)
        }
        _ => Err(rilua::runtime_error(format!(
            "{API_NAME}: argument 1 must be a public finite integral u32 number"
        ))),
    }
}

fn remove_private_aura_applied_sound(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_public_sound_id(state)?;
    let mut sim = borrow_state_mut(state)?;
    let sounds = &mut sim.private_aura_sound_registrations;
    sounds.live_ids.remove(&id);
    sounds.registrations.remove(&id);
    Ok(0)
}
