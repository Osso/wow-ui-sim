//! Readable item text (books, letters, plaques).
//!
//! Consumer: cached Retail `Blizzard_UIPanels_Game/Mainline/ItemTextFrame.lua`
//! (`ITEM_TEXT_BEGIN` reads item/material, `ITEM_TEXT_READY` reads text, creator,
//! page and next-page state; the page buttons call `ItemTextPrevPage` /
//! `ItemTextNextPage` and `OnHide` calls `CloseItemText`).
//! INFERRED (R-ag10-1): a page turn is answered by one `ITEM_TEXT_READY` for the
//! new page and `CloseItemText` by one `ITEM_TEXT_CLOSED`.

use super::{Arg, fire, push_optional_string};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, val_to_string};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

const OPEN_USAGE: &str =
    "Usage: A_Admin.OpenItemText(item, material|nil, creator|nil, pages, isFullPage)";

/// The text the server sent for the item being read; `page` is one-based.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemText {
    pub item: String,
    pub material: Option<String>,
    pub creator: Option<String>,
    pub pages: Vec<String>,
    pub page: usize,
    pub is_full_page: bool,
}

fn read_pages(state: &LuaState, index: i32) -> LuaResult<Vec<String>> {
    let Val::Table(table) = stack_val(state, index) else {
        return Err(runtime_error(OPEN_USAGE));
    };
    let values: Vec<Val> = state
        .gc
        .tables
        .get(table)
        .map(|table| table.array_slice().to_vec())
        .unwrap_or_default();
    let pages: Option<Vec<String>> = values
        .into_iter()
        .map(|value| val_to_string(state, value))
        .collect();
    pages
        .filter(|pages| !pages.is_empty())
        .ok_or_else(|| runtime_error(OPEN_USAGE))
}

fn open_item_text(state: &mut LuaState) -> LuaResult<u32> {
    let item = String::from_stack(state, 1)?;
    let material = Option::<String>::from_stack(state, 2)?;
    let creator = Option::<String>::from_stack(state, 3)?;
    let pages = read_pages(state, 4)?;
    let is_full_page = bool::from_stack(state, 5)?;
    if item.is_empty() || material.as_deref() == Some("") {
        return Err(runtime_error(OPEN_USAGE));
    }
    borrow_state_mut(state)?.interaction_dialogs.item_text = Some(ItemText {
        item,
        material,
        creator,
        pages,
        page: 1,
        is_full_page,
    });
    fire(state, "ITEM_TEXT_BEGIN", Vec::new())?;
    fire(state, "ITEM_TEXT_READY", Vec::new())?;
    Ok(0)
}

fn read_text<T>(state: &LuaState, read: impl FnOnce(&ItemText) -> T) -> LuaResult<Option<T>> {
    Ok(borrow_state(state)?
        .interaction_dialogs
        .item_text
        .as_ref()
        .map(read))
}

fn item_text_get_item(state: &mut LuaState) -> LuaResult<u32> {
    let item = read_text(state, |text| text.item.clone())?;
    push_optional_string(state, item.as_deref());
    Ok(1)
}

fn item_text_get_material(state: &mut LuaState) -> LuaResult<u32> {
    let material = read_text(state, |text| text.material.clone())?.flatten();
    push_optional_string(state, material.as_deref());
    Ok(1)
}

fn item_text_get_creator(state: &mut LuaState) -> LuaResult<u32> {
    let creator = read_text(state, |text| text.creator.clone())?.flatten();
    push_optional_string(state, creator.as_deref());
    Ok(1)
}

fn item_text_get_text(state: &mut LuaState) -> LuaResult<u32> {
    let text = read_text(state, |text| text.pages[text.page - 1].clone())?;
    push_optional_string(state, text.as_deref());
    Ok(1)
}

fn item_text_get_page(state: &mut LuaState) -> LuaResult<u32> {
    let page = read_text(state, |text| text.page)?;
    state.push(page.map_or(Val::Nil, |page| Val::Num(page as f64)));
    Ok(1)
}

fn item_text_has_next_page(state: &mut LuaState) -> LuaResult<u32> {
    let has_next = read_text(state, |text| text.page < text.pages.len())?;
    state.push(Val::Bool(has_next.unwrap_or(false)));
    Ok(1)
}

fn item_text_is_full_page(state: &mut LuaState) -> LuaResult<u32> {
    let full = read_text(state, |text| text.is_full_page)?;
    state.push(Val::Bool(full.unwrap_or(false)));
    Ok(1)
}

/// Move to `page + delta` when that page exists; returns whether it moved.
fn turn_page(state: &mut LuaState, forward: bool) -> LuaResult<u32> {
    let turned = {
        let mut sim = borrow_state_mut(state)?;
        let Some(text) = sim.interaction_dialogs.item_text.as_mut() else {
            return Ok(0);
        };
        let target = if forward {
            text.page + 1
        } else {
            text.page - 1
        };
        let valid = (1..=text.pages.len()).contains(&target);
        if valid {
            text.page = target;
        }
        valid
    };
    if turned {
        fire(state, "ITEM_TEXT_READY", Vec::new())?;
    }
    Ok(0)
}

fn item_text_next_page(state: &mut LuaState) -> LuaResult<u32> {
    turn_page(state, true)
}

fn item_text_prev_page(state: &mut LuaState) -> LuaResult<u32> {
    turn_page(state, false)
}

fn close_item_text(state: &mut LuaState) -> LuaResult<u32> {
    let closed = borrow_state_mut(state)?
        .interaction_dialogs
        .item_text
        .take()
        .is_some();
    if closed {
        fire(state, "ITEM_TEXT_CLOSED", Vec::<Arg>::new())?;
    }
    Ok(0)
}

pub(super) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    let functions: [(&str, rilua::RustFn); 10] = [
        ("ItemTextGetItem", item_text_get_item),
        ("ItemTextGetMaterial", item_text_get_material),
        ("ItemTextGetCreator", item_text_get_creator),
        ("ItemTextGetText", item_text_get_text),
        ("ItemTextGetPage", item_text_get_page),
        ("ItemTextHasNextPage", item_text_has_next_page),
        ("ItemTextIsFullPage", item_text_is_full_page),
        ("ItemTextNextPage", item_text_next_page),
        ("ItemTextPrevPage", item_text_prev_page),
        ("CloseItemText", close_item_text),
    ];
    for (name, function) in functions {
        LuaApiMut::register_function(lua, name, function)?;
    }
    Ok(())
}

pub(super) fn register_admin(b: TableBuilder) -> LuaResult<TableBuilder> {
    b.set_function("OpenItemText", open_item_text)
}
