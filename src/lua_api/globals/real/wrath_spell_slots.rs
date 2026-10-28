//! Explicit scenario inputs for Wrath spellbook slot presentation.

use std::collections::HashMap;

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Clone)]
pub struct SpellSlot {
    pub spell_id: u32,
    pub name: String,
    pub rank: String,
    pub icon: String,
    pub eligible_trade_skill_line: Option<u32>,
    pub shapeshift_form: Option<u32>,
}

pub type SpellSlots = HashMap<(String, u32), SpellSlot>;

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if ACTIVE != ClientProfile::Wrath {
        return Ok(builder);
    }
    builder
        .set_function("SetWrathSpellBookSlot", set_spell_slot)?
        .set_function("SetWrathSpellBookSelection", set_selection)
}

pub(crate) fn register_queries(state: &mut LuaState) -> LuaResult<()> {
    table_set_rust_fn_static(state, state.global, "GetSpellTexture", get_spell_texture)?;
    table_set_rust_fn_static(state, state.global, "GetSpellName", get_spell_name)?;
    table_set_rust_fn_static(state, state.global, "IsSelectedSpell", is_selected_spell)
}

enum SpellQuery {
    Slot(Option<SpellSlot>),
    Name(Option<SpellSlot>),
}

fn find_named_spell(slots: &SpellSlots, name: &str, rank: &str) -> Option<SpellSlot> {
    let name = name.to_lowercase();
    let rank = rank.to_lowercase();
    slots
        .iter()
        .filter(|(_, record)| {
            record.name.to_lowercase() == name
                && (rank.is_empty() || record.rank.to_lowercase() == rank)
        })
        .max_by(
            |((left_book, left_slot), _), ((right_book, right_slot), _)| {
                (left_slot, left_book).cmp(&(right_slot, right_book))
            },
        )
        .map(|(_, record)| record.clone())
}

fn find_spell_by_name(state: &mut LuaState) -> LuaResult<Option<SpellSlot>> {
    let name = String::from_stack(state, 1)?;
    let rank = match stack_val(state, 2) {
        Val::Str(_) => Some(String::from_stack(state, 2)?),
        _ => None,
    };
    let slots = &borrow_state(state)?.wrath_spell_slots;
    if let Some(rank) = rank {
        return Ok(find_named_spell(slots, &name, &rank));
    }
    if let Some(record) = find_named_spell(slots, &name, "") {
        return Ok(Some(record));
    }
    let Some((base, suffix)) = name.rsplit_once('(') else {
        return Ok(None);
    };
    let rank = suffix.split(')').next().unwrap_or(suffix);
    Ok(find_named_spell(slots, base, rank))
}

fn find_spell_slot(state: &mut LuaState, function: &str) -> LuaResult<SpellQuery> {
    if matches!(stack_val(state, 1), Val::Str(_)) {
        return Ok(SpellQuery::Name(find_spell_by_name(state)?));
    }
    Ok(SpellQuery::Slot(find_numeric_spell_slot(
        state, 1, function,
    )?))
}

pub(crate) fn find_numeric_spell_slot(
    state: &mut LuaState,
    index: i32,
    function: &str,
) -> LuaResult<Option<SpellSlot>> {
    let slot = match stack_val(state, index) {
        Val::Num(slot) if (1.0..=1024.0).contains(&slot) && slot.fract() == 0.0 => slot as u32,
        _ => {
            return Err(rilua::runtime_error(format!(
                "{function}(): Invalid spell slot"
            )));
        }
    };
    if !matches!(stack_val(state, index + 1), Val::Str(_)) {
        return Err(rilua::runtime_error(format!(
            "{function}(): Invalid spell slot"
        )));
    }
    let book = String::from_stack(state, index + 1)?;
    let book = if book.eq_ignore_ascii_case("pet") {
        "pet"
    } else {
        "spell"
    };
    Ok(borrow_state(state)?
        .wrath_spell_slots
        .get(&(book.to_string(), slot))
        .cloned())
}

fn resolve_spell_slot(
    state: &mut LuaState,
    function: &str,
) -> LuaResult<Option<Option<SpellSlot>>> {
    match find_spell_slot(state, function)? {
        SpellQuery::Name(None) => Ok(None),
        SpellQuery::Name(record) | SpellQuery::Slot(record) => Ok(Some(record)),
    }
}

fn get_spell_texture(state: &mut LuaState) -> LuaResult<u32> {
    let Some(record) = resolve_spell_slot(state, "GetSpellTexture")? else {
        return Ok(0);
    };
    let texture = record
        .filter(|record| !record.icon.is_empty())
        .map_or(Val::Nil, |record| create_string(state, &record.icon));
    state.push(texture);
    Ok(1)
}

fn get_spell_name(state: &mut LuaState) -> LuaResult<u32> {
    let Some(record) = resolve_spell_slot(state, "GetSpellName")? else {
        return Ok(0);
    };
    if let Some(record) = record {
        let name = create_string(state, &record.name);
        state.push(name);
        let rank = create_string(state, &record.rank);
        state.push(rank);
    } else {
        state.push(Val::Nil);
        state.push(Val::Nil);
    }
    Ok(2)
}

fn is_selected_spell(state: &mut LuaState) -> LuaResult<u32> {
    let Some(record) = resolve_spell_slot(state, "IsSelectedSpell")? else {
        return Ok(0);
    };
    let selected = if let Some(record) = record {
        let (trade_skill_line, form) = borrow_state(state)?.wrath_spell_selection;
        let trade_match = record.eligible_trade_skill_line.is_some()
            && record.eligible_trade_skill_line == trade_skill_line;
        let form_match = record
            .shapeshift_form
            .is_some_and(|id| id != 0 && Some(id) == form);
        trade_match || form_match
    } else {
        false
    };
    state.push(if selected { Val::Num(1.0) } else { Val::Nil });
    Ok(1)
}

fn set_spell_slot(state: &mut LuaState) -> LuaResult<u32> {
    let book = String::from_stack(state, 1)?;
    let book = if book.eq_ignore_ascii_case("pet") {
        "pet"
    } else {
        "spell"
    };
    let slot = u32::from_stack(state, 2)?;
    if !(1..=1024).contains(&slot) {
        return Err(rilua::runtime_error("SetWrathSpellBookSlot: invalid slot"));
    }
    let record = SpellSlot {
        spell_id: u32::from_stack(state, 3)?,
        name: String::from_stack(state, 4)?,
        rank: String::from_stack(state, 5)?,
        icon: String::from_stack(state, 6)?,
        eligible_trade_skill_line: Option::<u32>::from_stack(state, 7)?,
        shapeshift_form: Option::<u32>::from_stack(state, 8)?,
    };
    borrow_state_mut(state)?
        .wrath_spell_slots
        .insert((book.to_string(), slot), record);
    Ok(0)
}

fn set_selection(state: &mut LuaState) -> LuaResult<u32> {
    let trade_skill_line = Option::<u32>::from_stack(state, 1)?;
    let shapeshift_form = Option::<u32>::from_stack(state, 2)?;
    borrow_state_mut(state)?.wrath_spell_selection = (trade_skill_line, shapeshift_form);
    Ok(0)
}
