//! C_ToyBox namespace.

use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_string_static,
};
use crate::lua_bridge::{FromStack, IntoStack, TableBuilder};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

use super::set_global_val;

fn push_toy_info(state: &mut LuaState, tid: f64, name: &str, icon: f64, fanfare: bool) -> u32 {
    let name_val = create_string(state, name);
    state.push(Val::Num(tid));
    state.push(name_val);
    state.push(Val::Num(icon));
    state.push(Val::Bool(false));
    state.push(Val::Bool(fanfare));
    state.push(Val::Num(1.0));
    6
}

fn toy_get_total_displayed(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.world.toys.len() as i32;
    count.into_stack(state)
}

fn toy_get_learned_displayed(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?
        .world
        .toys
        .iter()
        .filter(|toy| toy.is_collected)
        .count() as i32;
    count.into_stack(state)
}

fn toy_get_num_toys(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.world.toys.len() as i32;
    count.into_stack(state)
}

fn toy_get_num_filtered(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.world.toys.len() as i32;
    count.into_stack(state)
}

fn toy_get_from_index(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let item_id = {
        let st = borrow_state(state)?;
        let toy_index = (index - 1) as usize;
        st.world.toys.get(toy_index).map(|toy| toy.item_id as i32)
    };
    match item_id {
        Some(item_id) => item_id.into_stack(state),
        None => (-1i32).into_stack(state),
    }
}

fn toy_get_info(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = u32::from_stack(state, 1)?;
    let info = {
        let st = borrow_state(state)?;
        st.world
            .toys
            .iter()
            .find(|t| t.item_id == item_id)
            .map(|toy| {
                (
                    toy.item_id as f64,
                    toy.name.clone(),
                    toy.icon as f64,
                    st.world.toy_fanfare.needs_fanfare(item_id),
                )
            })
    };
    let Some((tid, name, icon, fanfare)) = info else {
        let empty_name = create_string_static(state, "");
        state.push(Val::Num(0.0));
        state.push(empty_name);
        state.push(Val::Num(0.0));
        state.push(Val::Bool(false));
        state.push(Val::Bool(false));
        state.push(Val::Num(0.0));
        return Ok(6);
    };
    Ok(push_toy_info(state, tid, &name, icon, fanfare))
}

fn toy_is_usable(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = i32::from_stack(state, 1)?;
    let usable = borrow_state(state)?
        .world
        .toys
        .iter()
        .find(|t| t.item_id == item_id as u32)
        .map(|t| t.is_usable)
        .unwrap_or(false);
    usable.into_stack(state)
}

fn player_has_toy(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = i32::from_stack(state, 1)?;
    let has_toy = borrow_state(state)?
        .world
        .toys
        .iter()
        .find(|t| t.item_id == item_id as u32)
        .map(|t| t.is_collected)
        .unwrap_or(false);
    has_toy.into_stack(state)
}

fn use_toy(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = i32::from_stack(state, 1)?;
    let can_use = borrow_state(state)?
        .world
        .toys
        .iter()
        .find(|t| t.item_id == item_id as u32)
        .map(|t| t.is_collected && t.is_usable)
        .unwrap_or(false);
    can_use.into_stack(state)
}

fn toy_get_link(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = i32::from_stack(state, 1)?;
    let link = {
        let st = borrow_state(state)?;
        st.world
            .toys
            .iter()
            .find(|t| t.item_id == item_id as u32)
            .map(|toy| {
                format!(
                    "|cff0070dd|Hitem:{}::::::::1:0|h[{}]|h|r",
                    toy.item_id, toy.name
                )
            })
    };
    match link {
        Some(s) => create_string(state, &s).into_stack(state),
        None => Val::Nil.into_stack(state),
    }
}

fn register_toy_queries(b: TableBuilder) -> LuaResult<TableBuilder> {
    b.set_function("GetNumTotalDisplayedToys", toy_get_total_displayed)?
        .set_function("GetNumLearnedDisplayedToys", toy_get_learned_displayed)?
        .set_function("GetNumToys", toy_get_num_toys)?
        .set_function("GetNumFilteredToys", toy_get_num_filtered)?
        .set_function("GetToyFromIndex", toy_get_from_index)?
        .set_function("GetToyInfo", toy_get_info)?
        .set_function("IsToyUsable", toy_is_usable)?
        .set_function("GetToyLink", toy_get_link)
}

fn register_toy_favorites(b: TableBuilder) -> LuaResult<TableBuilder> {
    b.set_function("GetIsFavorite", |state| {
        let item_id = i32::from_stack(state, 1)?;
        let st = borrow_state(state)?;
        let fav = st.world.favorite_toys.contains(&(item_id as u32));
        drop(st);
        fav.into_stack(state)
    })?
    .set_function("HasFavorites", |state| {
        let st = borrow_state(state)?;
        let has = !st.world.favorite_toys.is_empty();
        drop(st);
        has.into_stack(state)
    })?
    .set_function("SetIsFavorite", |state| {
        let item_id = i32::from_stack(state, 1)?;
        let is_fav = bool::from_stack(state, 2)?;
        let mut st = borrow_state_mut(state)?;
        if is_fav {
            st.world.favorite_toys.insert(item_id as u32);
        } else {
            st.world.favorite_toys.remove(&(item_id as u32));
        }
        Ok(0)
    })
}

fn register_toy_filter_stubs(b: TableBuilder) -> LuaResult<TableBuilder> {
    let b = register_toy_visibility_filter_stubs(b)?;
    register_toy_type_filter_stubs(b)
}

fn register_toy_visibility_filter_stubs(b: TableBuilder) -> LuaResult<TableBuilder> {
    b.set_function("GetCollectedShown", |state| true.into_stack(state))?
        .set_function("GetUncollectedShown", |state| true.into_stack(state))?
        .set_function("GetUnusableShown", |state| true.into_stack(state))?
        .set_function("SetCollectedShown", |_state| Ok(0))?
        .set_function("SetUncollectedShown", |_state| Ok(0))?
        .set_function("SetUnusableShown", |_state| Ok(0))?
        .set_function("ForceToyRefilter", |_state| Ok(0))
}

fn register_toy_type_filter_stubs(b: TableBuilder) -> LuaResult<TableBuilder> {
    b.set_function("IsExpansionTypeFilterChecked", |state| {
        let _filter_index = i32::from_stack(state, 1)?;
        true.into_stack(state)
    })?
    .set_function("IsSourceTypeFilterChecked", |state| {
        let _filter_index = i32::from_stack(state, 1)?;
        true.into_stack(state)
    })?
    .set_function("SetExpansionTypeFilter", |_state| Ok(0))?
    .set_function("SetSourceTypeFilter", |_state| Ok(0))
}

fn register_toy_box_info(lua: &mut rilua::Lua) -> LuaResult<()> {
    let table = TableBuilder::new(lua.state_mut())
        .set_function("IsUsingDefaultFilters", |state| true.into_stack(state))?
        .set_function("SetDefaultFilters", |_state| Ok(0))?
        .set_function("IsToySourceValid", |state| {
            let source_index = i32::from_stack(state, 1)?;
            (source_index > 0).into_stack(state)
        })?;
    let table = crate::c_api::c_toy_box_info::register_fanfare(table)?.build();
    set_global_val(lua.state_mut(), "C_ToyBoxInfo", table);
    Ok(())
}

pub fn register_rilua_toy_box(lua: &mut rilua::Lua) -> LuaResult<()> {
    let b = TableBuilder::new(lua.state_mut());
    let b = register_toy_queries(b)?;
    let b = register_toy_favorites(b)?;
    let t = register_toy_filter_stubs(b)?.build();
    set_global_val(lua.state_mut(), "C_ToyBox", t);
    register_toy_box_info(lua)?;
    LuaApiMut::register_function(lua, "PlayerHasToy", player_has_toy)?;
    LuaApiMut::register_function(lua, "UseToy", use_toy)?;
    Ok(())
}
