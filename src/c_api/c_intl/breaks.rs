//! ICU default segmentation; zero-based byte endpoints are simulator conventions.
use icu_segmenter::{GraphemeClusterSegmenter, LineSegmenter, SentenceSegmenter, WordSegmenter};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val, runtime_error};

use crate::lua_api::methods::{create_table, table_set_num};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "FindBreaks", |state| {
        find_breaks(state, 1)
    })
}

pub(super) fn register_context(state: &mut LuaState, metatable: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, metatable, "FindBreaks", |state| {
        super::storage::context_locale_bytes(state)?;
        find_breaks(state, 2)
    })
}

fn segment(text: &str, mode: Val) -> LuaResult<Vec<usize>> {
    let offsets = match mode {
        Val::Num(0.0) => GraphemeClusterSegmenter::new().segment_str(text).collect(),
        Val::Num(1.0) => WordSegmenter::new_auto(Default::default())
            .segment_str(text)
            .collect(),
        Val::Num(2.0) => SentenceSegmenter::new(Default::default())
            .segment_str(text)
            .collect(),
        Val::Num(3.0) => LineSegmenter::new_auto(Default::default())
            .segment_str(text)
            .collect(),
        _ => return Err(runtime_error("break type must be an integer from 0 to 3")),
    };
    Ok(offsets)
}

fn find_breaks(state: &mut LuaState, text_index: i32) -> LuaResult<u32> {
    let text = super::text::read_text(state, text_index, "segmentation")?;
    let offsets = segment(&text, stack_val(state, text_index + 1))?;
    let result = create_table(state);
    for (index, offset) in offsets.into_iter().enumerate() {
        table_set_num(state, result, (index + 1) as f64, Val::Num(offset as f64));
    }
    state.push(result);
    Ok(1)
}
