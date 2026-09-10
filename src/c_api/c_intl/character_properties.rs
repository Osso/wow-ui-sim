//! First-scalar Unicode properties; naming and native input semantics are unverified.
use icu_properties::props::{Alphabetic, GeneralCategory, Script, WhiteSpace};
use icu_properties::{CodePointMapData, CodePointSetData, PropertyNamesShort};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val, runtime_error};

use crate::lua_api::methods::table_set_static;
use crate::lua_bridge::table_set_rust_fn_static;

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetCharacterProperties", get_properties)
}

fn get_properties(state: &mut LuaState) -> LuaResult<u32> {
    let text = super::text::read_text(state, 1, "character properties")?;
    let Some(character) = text.chars().next() else {
        return Ok(0);
    };
    let category = CodePointMapData::<GeneralCategory>::new().get(character);
    let result = state.gc.alloc_table(Table::with_sizes(0, 7));
    publish_scalar_properties(state, result, character, category);
    publish_property_names(state, result, character, category)?;
    state.push(Val::Table(result));
    Ok(1)
}

fn publish_scalar_properties(
    state: &mut LuaState,
    result: GcRef<Table>,
    character: char,
    category: GeneralCategory,
) {
    let alphabetic = CodePointSetData::new::<Alphabetic>().contains(character);
    let whitespace = CodePointSetData::new::<WhiteSpace>().contains(character);
    let values = [
        ("codePoint", Val::Num(u32::from(character) as f64)),
        ("isAlphabetic", Val::Bool(alphabetic)),
        (
            "isDigit",
            Val::Bool(category == GeneralCategory::DecimalNumber),
        ),
        ("isWhitespace", Val::Bool(whitespace)),
    ];
    for (name, value) in values {
        table_set_static(state, Val::Table(result), name, value);
    }
}

fn publish_property_names(
    state: &mut LuaState,
    result: GcRef<Table>,
    character: char,
    category: GeneralCategory,
) -> LuaResult<()> {
    let category_name = PropertyNamesShort::<GeneralCategory>::new()
        .get(category)
        .ok_or_else(|| runtime_error("ICU general category has no short name"))?;
    let script = CodePointMapData::<Script>::new().get(character);
    let script_name = PropertyNamesShort::<Script>::new()
        .get(script)
        .ok_or_else(|| runtime_error("ICU script has no short name"))?;
    let block_name = match unicode_blocks::find_unicode_block(character) {
        Some(block) => block.name(),
        None => "No_Block",
    };
    for (name, text) in [
        ("generalCategory", category_name),
        ("scriptCode", script_name),
        ("blockCode", block_name),
    ] {
        let value = state.gc.intern_string(text.as_bytes());
        table_set_static(state, Val::Table(result), name, Val::Str(value));
    }
    Ok(())
}
