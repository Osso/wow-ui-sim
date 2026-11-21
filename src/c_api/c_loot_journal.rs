//! INFERRED R88cm host-supplied item-set catalog for the Loot Journal.
//! Pinned Retail/Forever declarations: Blizzard_APIDocumentationGenerated/LootJournalDocumentation.lua:11-42, 58-75.

use crate::c_api::crafting_input::read_array;
use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, create_table_with_fields,
    table_set_num,
};
use crate::lua_bridge::{FromStack, TableBuilder, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

#[derive(Clone, Debug, Default)]
pub struct ItemSetCatalog {
    pub sets: Vec<ItemSet>,
}

#[derive(Clone, Debug)]
pub struct ItemSet {
    pub set_id: u32,
    pub name: String,
    pub item_level: u32,
    pub class_id: Option<u32>,
    pub spec_id: Option<u32>,
    pub items: Vec<ItemSetItem>,
}

#[derive(Clone, Debug)]
pub struct ItemSetItem {
    pub item_id: u32,
    pub icon: u32,
    pub slot: u32,
    pub inv_type: u32,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_LootJournal")?;
    table_set_rust_fn_static(state, namespace, "GetItemSets", get_item_sets)?;
    table_set_rust_fn_static(state, namespace, "GetItemSetItems", get_item_set_items)
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetLootJournalItemSets", set_item_sets)
}

fn get_item_sets(state: &mut LuaState) -> LuaResult<u32> {
    let class_id = Option::<u32>::from_stack(state, 1)?;
    let spec_id = Option::<u32>::from_stack(state, 2)?;
    let catalog = borrow_state(state)?
        .encounter_journal
        .item_set_catalog
        .clone();
    let rows = catalog
        .sets
        .iter()
        .filter(|set| matches_filter(set, class_id, spec_id))
        .map(|set| item_set_table(state, set))
        .collect::<Vec<_>>();
    push_sequence(state, rows)
}

fn matches_filter(set: &ItemSet, class_id: Option<u32>, spec_id: Option<u32>) -> bool {
    class_id.is_none_or(|value| set.class_id == Some(value))
        && spec_id.is_none_or(|value| set.spec_id == Some(value))
}

fn get_item_set_items(state: &mut LuaState) -> LuaResult<u32> {
    let set_id = u32::from_stack(state, 1)?;
    let catalog = borrow_state(state)?
        .encounter_journal
        .item_set_catalog
        .clone();
    let items = catalog
        .sets
        .iter()
        .find(|set| set.set_id == set_id)
        .map_or_else(Vec::new, |set| {
            set.items
                .iter()
                .map(|item| item_table(state, item))
                .collect()
        });
    push_sequence(state, items)
}

fn item_set_table(state: &mut LuaState, set: &ItemSet) -> Val {
    let name = create_string(state, &set.name);
    create_table_with_fields(
        state,
        &[
            ("setID", number(set.set_id)),
            ("name", name),
            ("itemLevel", number(set.item_level)),
        ],
    )
}

fn item_table(state: &mut LuaState, item: &ItemSetItem) -> Val {
    create_table_with_fields(
        state,
        &[
            ("itemID", number(item.item_id)),
            ("icon", number(item.icon)),
            ("invType", number(item.inv_type)),
        ],
    )
}

fn push_sequence(state: &mut LuaState, rows: Vec<Val>) -> LuaResult<u32> {
    let sequence = create_table(state);
    let Val::Table(table) = sequence else {
        unreachable!("create_table must return a table");
    };
    for (index, row) in rows.into_iter().enumerate() {
        table_set_num(state, table, (index + 1) as f64, row);
    }
    state.push(sequence);
    Ok(1)
}

fn set_item_sets(state: &mut LuaState) -> LuaResult<u32> {
    let rows = read_array(state, crate::lua_bridge::stack_val(state, 1))?;
    let sets = rows
        .into_iter()
        .map(|row| read_item_set(state, row))
        .collect::<LuaResult<Vec<_>>>()?;
    borrow_state_mut(state)?.encounter_journal.item_set_catalog = ItemSetCatalog { sets };
    Ok(0)
}

fn read_item_set(state: &mut LuaState, row: Val) -> LuaResult<ItemSet> {
    Ok(ItemSet {
        set_id: read_integer(state, row, "setID")?,
        name: read_text(state, row, "name")?,
        item_level: read_integer(state, row, "itemLevel")?,
        class_id: read_optional_integer(state, row, "classID")?,
        spec_id: read_optional_integer(state, row, "specID")?,
        items: read_items(state, row)?,
    })
}

fn read_items(state: &mut LuaState, row: Val) -> LuaResult<Vec<ItemSetItem>> {
    let value = crate::lua_api::methods::table_get(state, row, "items");
    read_array(state, value)?
        .into_iter()
        .map(|item| read_item(state, item))
        .collect()
}

fn read_item(state: &mut LuaState, row: Val) -> LuaResult<ItemSetItem> {
    Ok(ItemSetItem {
        item_id: read_integer(state, row, "itemID")?,
        icon: read_integer(state, row, "icon")?,
        slot: read_integer(state, row, "slot")?,
        inv_type: read_integer(state, row, "invType")?,
    })
}

fn read_integer(state: &mut LuaState, row: Val, key: &str) -> LuaResult<u32> {
    read_u32(crate::lua_api::methods::table_get(state, row, key), key)
}

fn read_optional_integer(state: &mut LuaState, row: Val, key: &str) -> LuaResult<Option<u32>> {
    match crate::lua_api::methods::table_get(state, row, key) {
        Val::Nil => Ok(None),
        value => read_u32(value, key).map(Some),
    }
}

fn read_u32(value: Val, key: &str) -> LuaResult<u32> {
    match value {
        Val::Num(value)
            if value.is_finite()
                && value.fract() == 0.0
                && (0.0..=f64::from(u32::MAX)).contains(&value) =>
        {
            Ok(value as u32)
        }
        _ => Err(runtime_error(format!("Loot Journal {key} must be a u32"))),
    }
}

fn read_text(state: &mut LuaState, row: Val, key: &str) -> LuaResult<String> {
    let value = crate::lua_api::methods::table_get(state, row, key);
    crate::lua_api::methods::val_to_string(state, value)
        .ok_or_else(|| runtime_error(format!("Loot Journal {key} must be a string")))
}

fn number(value: u32) -> Val {
    Val::Num(f64::from(value))
}
