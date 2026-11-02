//! Runeforge power queries for the currently empty simulator catalogue.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::create_table;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::LuaResult;
use rilua::vm::state::LuaState;

/// Supplied shared-interaction state read by `CloseRuneforgeInteraction`.
///
/// The simulator has no runeforge open path; hosts supply the current target.
#[derive(Debug, Default)]
pub struct RuneforgeInteraction {
    /// Current `Enum.PlayerInteractionType`; zero means none.
    pub interaction_type: i32,
    /// Primary target identity; empty means none.
    pub target_guid: String,
    /// Whether the runeforge close callback is installed on the current target.
    pub close_callback_installed: bool,
    pub transport_available: bool,
    /// Close requests handed to transport; not server acceptance.
    pub submitted_close_requests: u32,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_LegendaryCrafting")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetRuneforgePowersByClassSpecAndCovenant",
        get_runeforge_powers,
    )
}

fn get_runeforge_powers(state: &mut LuaState) -> LuaResult<u32> {
    for index in 1..=4 {
        Option::<f64>::from_stack(state, index)?;
    }
    let result = create_table(state);
    state.push(result);
    Ok(1)
}
