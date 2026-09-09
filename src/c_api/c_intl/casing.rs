//! ICU casing with parsed locale identifiers; folding uses Unicode default mappings.
use icu_casemap::{CaseMapper, TitlecaseMapper};
use icu_locale_core::{LanguageIdentifier, Locale};
use icu_segmenter::WordSegmenter;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val, runtime_error};

use crate::lua_bridge::table_set_rust_fn_static;

#[derive(Clone, Copy)]
enum Operation {
    Lower,
    Upper,
    Fold,
    Title,
}

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "ToLower", |s| {
        apply_global(s, Operation::Lower)
    })?;
    table_set_rust_fn_static(state, namespace, "ToUpper", |s| {
        apply_global(s, Operation::Upper)
    })?;
    table_set_rust_fn_static(state, namespace, "ToTitle", |s| {
        apply_global(s, Operation::Title)
    })?;
    table_set_rust_fn_static(state, namespace, "FoldCase", |s| {
        apply_global(s, Operation::Fold)
    })
}

pub(super) fn register_context(state: &mut LuaState, metatable: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, metatable, "ToLower", |s| {
        apply_context(s, Operation::Lower)
    })?;
    table_set_rust_fn_static(state, metatable, "ToUpper", |s| {
        apply_context(s, Operation::Upper)
    })?;
    table_set_rust_fn_static(state, metatable, "ToTitle", |s| {
        apply_context(s, Operation::Title)
    })?;
    table_set_rust_fn_static(state, metatable, "FoldCase", |s| {
        apply_context(s, Operation::Fold)
    })
}

fn parse_locale(bytes: &[u8]) -> LuaResult<LanguageIdentifier> {
    let is_wow_tag = bytes.len() == 4 && bytes.iter().all(u8::is_ascii_alphabetic);
    let tag = if is_wow_tag {
        [bytes[..2].to_vec(), vec![b'-'], bytes[2..].to_vec()].concat()
    } else {
        bytes.to_vec()
    };
    Locale::try_from_utf8(&tag)
        .map(|locale| locale.id)
        .map_err(|_| runtime_error("casing locale must be a valid ICU locale identifier"))
}

fn apply_global(state: &mut LuaState, operation: Operation) -> LuaResult<u32> {
    let text = super::text::read_text(state, 1, "casing")?;
    let result = match operation {
        Operation::Fold => CaseMapper::new().fold_string(&text).into_owned(),
        _ => {
            let bytes = super::storage::current_locale_bytes(state)?;
            map_case(&text, operation, &parse_locale(&bytes)?)
        }
    };
    push_result(state, &result)
}

fn apply_context(state: &mut LuaState, operation: Operation) -> LuaResult<u32> {
    let bytes = super::storage::context_locale_bytes(state)?;
    let text = super::text::read_text(state, 2, "casing")?;
    let result = match operation {
        Operation::Fold => CaseMapper::new().fold_string(&text).into_owned(),
        _ => map_case(&text, operation, &parse_locale(&bytes)?),
    };
    push_result(state, &result)
}

fn map_case(text: &str, operation: Operation, locale: &LanguageIdentifier) -> String {
    let mapper = CaseMapper::new();
    match operation {
        Operation::Lower => mapper.lowercase_to_string(text, locale).into_owned(),
        Operation::Upper => mapper.uppercase_to_string(text, locale).into_owned(),
        Operation::Fold => mapper.fold_string(text).into_owned(),
        Operation::Title => titlecase_words(text, locale),
    }
}

fn titlecase_words(text: &str, locale: &LanguageIdentifier) -> String {
    let segmenter = WordSegmenter::new_auto(Default::default());
    let mapper = TitlecaseMapper::new();
    let mut result = String::with_capacity(text.len());
    let mut start = 0;
    for (end, word_type) in segmenter.segment_str(text).iter_with_word_type() {
        let segment = &text[start..end];
        if word_type.is_word_like() {
            result.push_str(&mapper.titlecase_segment_to_string(
                segment,
                locale,
                Default::default(),
            ));
        } else {
            result.push_str(segment);
        }
        start = end;
    }
    result
}

fn push_result(state: &mut LuaState, text: &str) -> LuaResult<u32> {
    let value = state.gc.intern_string(text.as_bytes());
    state.push(Val::Str(value));
    Ok(1)
}
