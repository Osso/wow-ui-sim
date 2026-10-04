//! B18/B20/B21 live host queries. No catalog seeds or inferred housing service.

use crate::lua_api::methods::{borrow_state, create_string, val_to_string};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

use crate::c_api::patch_12_0_7_inputs::{authenticate_arguments, integer_selector};

pub(super) fn names(state: &mut LuaState) -> LuaResult<u32> {
    let args = authenticate_arguments(state)?;
    let Some(id) = integer_selector(args.first().copied().unwrap_or(Val::Nil)) else {
        return Ok(0);
    };
    // INFERRED: missing relations or labels return no results, never guessed labels.
    let labels = {
        let sim = borrow_state(state)?;
        let catalog = &sim.housing.catalog;
        catalog.subcategories.get(&id).and_then(|sub| {
            let category = catalog.categories.get(&sub.parent_category_id)?;
            Some((category.name.clone()?, sub.name.clone()?))
        })
    };
    let Some((category, subcategory)) = labels else {
        return Ok(0);
    };
    let category = create_string(state, &category);
    state.push(category);
    let subcategory = create_string(state, &subcategory);
    state.push(subcategory);
    Ok(2)
}

pub(super) fn supports_door(state: &mut LuaState) -> LuaResult<u32> {
    let args = authenticate_arguments(state)?;
    let room = args.first().copied().and_then(|value| match value {
        Val::Str(_) => val_to_string(state, value),
        _ => None,
    });
    let component = args.get(1).copied().and_then(integer_selector);
    let door_type = args.get(2).copied().and_then(integer_selector);
    // INFERRED: unknown/malformed room-component/type combinations deny support.
    let supported = match (room, component, door_type) {
        (Some(room), Some(component), Some(door_type)) => borrow_state(state)?
            .housing
            .room_connection_door_types
            .get(&(room, component))
            .is_some_and(|types| types.contains(&door_type)),
        _ => false,
    };
    state.push(Val::Bool(supported));
    Ok(1)
}

pub(super) fn can_view_floor(state: &mut LuaState) -> LuaResult<u32> {
    let args = authenticate_arguments(state)?;
    let floor = integer_selector(args.first().copied().unwrap_or(Val::Nil));
    // INFERRED: host permission is authoritative; unknown/malformed floors deny.
    let allowed = {
        let sim = borrow_state(state)?;
        floor.is_some_and(|floor| {
            sim.housing
                .viewed_floor_permissions
                .get(&floor)
                .copied()
                .unwrap_or(false)
        })
    };
    state.push(Val::Bool(allowed));
    Ok(1)
}
