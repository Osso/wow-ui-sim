//! Runeforge power queries for the currently empty simulator catalogue, and
//! the runeforge interaction close request.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_table};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// `Enum.PlayerInteractionType.LegendaryCrafting`,
/// PlayerInteractionManagerConstantsDocumentation.lua:61.
const LEGENDARY_CRAFTING_INTERACTION: i32 = 48;

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
    // LegendaryCraftingDocumentation.lua:11 in Retail, PTR and Forever.
    table_set_rust_fn_static(
        state,
        namespace,
        "CloseRuneforgeInteraction",
        close_runeforge_interaction,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetRuneforgePowersByClassSpecAndCovenant",
        get_runeforge_powers,
    )
}

/// WM33-A: close the legendary-crafting target, then submit a close request
/// whenever transport exists, whether or not a target closed.
fn close_runeforge_interaction(state: &mut LuaState) -> LuaResult<u32> {
    close_runeforge_target(state)?;
    submit_runeforge_close_request(state)?;
    Ok(0)
}

fn close_runeforge_target(state: &mut LuaState) -> LuaResult<()> {
    let close_callback_installed = {
        let interaction = &borrow_state(state)?.runeforge_interaction;
        let is_runeforge_target = interaction.interaction_type == LEGENDARY_CRAFTING_INTERACTION
            && !interaction.target_guid.is_empty();
        if !is_runeforge_target {
            return Ok(());
        }
        interaction.close_callback_installed
    };
    if close_callback_installed {
        // State is not cleared first, so a Lua close re-entering from this
        // event repeats the whole sequence before the outer close resumes.
        dispatch_event_now(state, "RUNEFORGE_LEGENDARY_CRAFTING_CLOSED", &[])?;
    }
    // The hide carries the type current at dispatch: zero after a nested reset.
    let hide_type = borrow_state(state)?.runeforge_interaction.interaction_type;
    dispatch_event_now(
        state,
        "PLAYER_INTERACTION_MANAGER_FRAME_HIDE",
        &[Val::Num(f64::from(hide_type))],
    )?;
    reset_runeforge_target(&mut borrow_state_mut(state)?.runeforge_interaction);
    Ok(())
}

fn reset_runeforge_target(interaction: &mut RuneforgeInteraction) {
    interaction.interaction_type = 0;
    interaction.target_guid.clear();
    interaction.close_callback_installed = false;
}

fn submit_runeforge_close_request(state: &mut LuaState) -> LuaResult<()> {
    let interaction = &mut borrow_state_mut(state)?.runeforge_interaction;
    if interaction.transport_available {
        interaction.submitted_close_requests += 1;
    }
    Ok(())
}

fn get_runeforge_powers(state: &mut LuaState) -> LuaResult<u32> {
    for index in 1..=4 {
        Option::<f64>::from_stack(state, index)?;
    }
    let result = create_table(state);
    state.push(result);
    Ok(1)
}
