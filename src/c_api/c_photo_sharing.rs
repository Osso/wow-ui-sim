//! `C_PhotoSharing` account-link state — SimState-backed.
//!
//! The two flags are independent:
//! - `IsAuthorized` = has the user linked an account with the sharing
//!   service?
//! - `IsEnabled`    = has the user opted into uploads in the current
//!   session?
//!
//! A user can authorize once and enable/disable per session. Both default
//! to false (sim has no real service). `ClearAuthorization` unlinks the
//! account and `GetStatus` derives the documented status from both flags.
//! The service-facing members (OAuth flow, capture, upload) are permanent
//! shims in `permanent_shims::c_photo_sharing_service`.
//!
//! Admin: `A_Admin.SetPhotoSharingAuthorized(b?)` /
//! `A_Admin.SetPhotoSharingEnabled(b?)` — no-arg defaults to true.

#[cfg(feature = "retail-12-0-5")]
use crate::event::{Event, EventArg};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_table};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

// Enum.PhotoSharingStatus values (ImageSharingConstantsDocumentation.lua).
#[cfg(feature = "retail-12-0-5")]
const STATUS_DISABLED: f64 = 0.0;
#[cfg(feature = "retail-12-0-5")]
const STATUS_NOT_CONFIGURED: f64 = 1.0;
#[cfg(feature = "retail-12-0-5")]
const STATUS_CONFIGURED: f64 = 2.0;

pub fn is_authorized(state: &mut LuaState) -> LuaResult<u32> {
    let v = borrow_state(state)?.photo_sharing_authorized;
    state.push(Val::Bool(v));
    Ok(1)
}

pub fn is_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let v = borrow_state(state)?.photo_sharing_enabled;
    state.push(Val::Bool(v));
    Ok(1)
}

/// INFERRED: Expired, ParentalControl and the other restriction statuses need
/// account restrictions the simulator does not model.
#[cfg(feature = "retail-12-0-5")]
fn get_status(state: &mut LuaState) -> LuaResult<u32> {
    let sim = borrow_state(state)?;
    let status = match (sim.photo_sharing_enabled, sim.photo_sharing_authorized) {
        (false, _) => STATUS_DISABLED,
        (true, false) => STATUS_NOT_CONFIGURED,
        (true, true) => STATUS_CONFIGURED,
    };
    drop(sim);
    state.push(Val::Num(status));
    Ok(1)
}

/// Unlinks the account; INFERRED: the update event (without a notification)
/// fires only when an authorization was actually cleared.
#[cfg(feature = "retail-12-0-5")]
fn clear_authorization(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    if sim.photo_sharing_authorized {
        sim.photo_sharing_authorized = false;
        sim.events.push(Event {
            name: "PHOTO_SHARING_AUTHORIZATION_UPDATED".into(),
            args: vec![EventArg::Boolean(false)],
        });
    }
    Ok(0)
}

fn ensure_c_photo_sharing_table(state: &mut LuaState) -> GcRef<Table> {
    let key = state.gc.intern_string_static(b"C_PhotoSharing");
    let global = state.global;
    let existing = state
        .gc
        .tables
        .get(global)
        .map(|t| t.get_str(key, &state.gc.string_arena));
    if let Some(Val::Table(r)) = existing {
        return r;
    }
    let new_val = create_table(state);
    let Val::Table(new_ref) = new_val else {
        unreachable!("create_table must return a table");
    };
    if let Some(global_table) = state.gc.tables.get_mut(global) {
        let _ = global_table.raw_set(Val::Str(key), new_val, &state.gc.string_arena);
    }
    state.gc.barrier_back(global);
    new_ref
}

pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    use rilua::LuaApiMut;
    let state = lua.state_mut();
    let table_ref = ensure_c_photo_sharing_table(state);
    table_set_rust_fn_static(state, table_ref, "IsAuthorized", is_authorized)?;
    table_set_rust_fn_static(state, table_ref, "IsEnabled", is_enabled)?;
    #[cfg(feature = "retail-12-0-5")]
    {
        table_set_rust_fn_static(state, table_ref, "GetStatus", get_status)?;
        table_set_rust_fn_static(state, table_ref, "ClearAuthorization", clear_authorization)?;
        super::permanent_shims::c_photo_sharing_service::register(state, table_ref)?;
    }
    Ok(())
}

/// `A_Admin.SetPhotoSharingAuthorized(b?)` — no-arg defaults to true.
pub fn admin_set_photo_sharing_authorized(state: &mut LuaState) -> LuaResult<u32> {
    let v = Option::<bool>::from_stack(state, 1)?.unwrap_or(true);
    borrow_state_mut(state)?.photo_sharing_authorized = v;
    Ok(0)
}

/// `A_Admin.SetPhotoSharingEnabled(b?)` — no-arg defaults to true.
pub fn admin_set_photo_sharing_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let v = Option::<bool>::from_stack(state, 1)?.unwrap_or(true);
    borrow_state_mut(state)?.photo_sharing_enabled = v;
    Ok(0)
}
