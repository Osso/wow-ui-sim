//! Sound-kit requests share one model across C_Sound and the legacy global.

#[cfg(feature = "retail-12-1-0")]
mod options;
#[cfg(feature = "retail-12-1-0")]
pub use options::PlaySoundRequest;

use super::helpers::ensure_namespace;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::LuaResult;
use rilua::vm::state::LuaState;

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_Sound")?;
    table_set_rust_fn_static(state, namespace, "PlaySound", play_sound)?;
    #[cfg(feature = "retail-12-1-0")]
    options::register(state)?;
    Ok(())
}

pub(crate) fn play_sound(state: &mut LuaState) -> LuaResult<u32> {
    let sound_kit_id = u32::from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    sim.last_sound_kit_requested = Some(sound_kit_id);
    if let Some(manager) = sim.sound_manager.as_mut() {
        let _ = manager.play_sound(sound_kit_id);
    }
    Ok(0)
}
