#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::c_api::helpers::ensure_namespace;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::lua_bridge::{FromStack, TableBuilder, table_set_rust_fn_static};
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use rilua::vm::state::LuaState;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use rilua::{LuaResult, Val};

#[derive(Clone, Copy, Debug, Default)]
pub struct RangedSlotState {
    /// INFERRED R88dv (native evidence pending): the seeded Paladin hides this slot.
    pub shown: bool,
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_PaperDollInfo")?;
    table_set_rust_fn_static(state, namespace, "IsRangedSlotShown", is_ranged_slot_shown)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn is_ranged_slot_shown(state: &mut LuaState) -> LuaResult<u32> {
    let shown = borrow_state(state)?.ranged_slot.shown;
    state.push(Val::Bool(shown));
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetRangedSlotShown", set_ranged_slot_shown)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn set_ranged_slot_shown(state: &mut LuaState) -> LuaResult<u32> {
    let shown = bool::from_stack(state, 1)?;
    borrow_state_mut(state)?.ranged_slot.shown = shown;
    Ok(0)
}
