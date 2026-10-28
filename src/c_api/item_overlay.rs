//! Scenario-supplied classifications used by the item overlay.
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ItemOverlayFlag {
    Corrupted,
    Cosmetic,
    Conduit,
    Curio,
    Relic,
    Decor,
}

fn supports_overlay() -> bool {
    matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr)
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !supports_overlay() {
        return Ok(());
    }
    let item = super::ensure_namespace(state, "C_Item")?;
    for (name, func) in [
        (
            "IsCorruptedItem",
            corrupted as fn(&mut LuaState) -> LuaResult<u32>,
        ),
        ("IsCosmeticItem", cosmetic),
        ("IsCurioItem", curio),
        ("IsRelicItem", relic),
        ("IsDecorItem", decor),
    ] {
        table_set_rust_fn_static(state, item, name, func)?;
    }
    let soulbinds = super::ensure_namespace(state, "C_Soulbinds")?;
    table_set_rust_fn_static(state, soulbinds, "IsItemConduitByItemInfo", conduit)
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if supports_overlay() {
        builder.set_function("SetItemOverlayFlag", set_flag)
    } else {
        Ok(builder)
    }
}

fn set_flag(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = u32::from_stack(state, 1)?;
    let name = String::from_stack(state, 2)?;
    let flag = match name.as_str() {
        "corrupted" => ItemOverlayFlag::Corrupted,
        "cosmetic" => ItemOverlayFlag::Cosmetic,
        "conduit" => ItemOverlayFlag::Conduit,
        "curio" => ItemOverlayFlag::Curio,
        "relic" => ItemOverlayFlag::Relic,
        "decor" => ItemOverlayFlag::Decor,
        _ => return Err(runtime_error("A_Admin.SetItemOverlayFlag: unknown flag")),
    };
    let Val::Bool(enabled) = stack_val(state, 3) else {
        return Err(runtime_error(
            "A_Admin.SetItemOverlayFlag: expected boolean",
        ));
    };
    let mut sim = borrow_state_mut(state)?;
    if enabled {
        sim.item_overlay_flags.insert((item_id, flag));
    } else {
        sim.item_overlay_flags.remove(&(item_id, flag));
    }
    Ok(0)
}

fn query(state: &mut LuaState, flag: ItemOverlayFlag) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    if !matches!(value, Val::Num(_) | Val::Str(_)) {
        return Err(runtime_error("itemInfo must be an item ID or link"));
    }
    let id = super::item_spell::parse_item_id_from_val(state, value);
    let result = {
        let sim = borrow_state(state)?;
        id.is_some_and(|id| sim.item_overlay_flags.contains(&(id, flag)))
    };
    state.push(Val::Bool(result));
    Ok(1)
}

macro_rules! query_flag {
    ($name:ident, $flag:ident) => {
        fn $name(state: &mut LuaState) -> LuaResult<u32> {
            query(state, ItemOverlayFlag::$flag)
        }
    };
}
query_flag!(corrupted, Corrupted);
query_flag!(cosmetic, Cosmetic);
query_flag!(conduit, Conduit);
query_flag!(curio, Curio);
query_flag!(relic, Relic);
query_flag!(decor, Decor);
