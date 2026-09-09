//! ICU normalization over validated UTF-8; native error/security semantics are unverified.
use icu_normalizer::{ComposingNormalizerBorrowed, DecomposingNormalizerBorrowed};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

use crate::lua_bridge::{stack_val, table_set_rust_fn_static};

enum Form {
    Nfc,
    Nfd,
    Nfkc,
    Nfkd,
}

pub(super) fn register(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "Normalize", normalize)?;
    table_set_rust_fn_static(state, namespace, "IsNormalized", is_normalized)
}

fn read_form(state: &LuaState) -> LuaResult<Form> {
    match stack_val(state, 2) {
        Val::Num(0.0) => Ok(Form::Nfc),
        Val::Num(1.0) => Ok(Form::Nfd),
        Val::Num(2.0) => Ok(Form::Nfkc),
        Val::Num(3.0) => Ok(Form::Nfkd),
        _ => Err(runtime_error(
            "normalization form must be an integer from 0 to 3",
        )),
    }
}

fn read_text(state: &LuaState) -> LuaResult<String> {
    let Val::Str(reference) = stack_val(state, 1) else {
        return Err(runtime_error("normalization text must be a string"));
    };
    let bytes = state
        .gc
        .string_arena
        .get(reference)
        .ok_or_else(|| runtime_error("normalization text has been collected"))?;
    std::str::from_utf8(bytes.data())
        .map(str::to_owned)
        .map_err(|_| runtime_error("normalization text must be valid UTF-8"))
}

fn normalize_text(text: &str, form: Form) -> String {
    match form {
        Form::Nfc => ComposingNormalizerBorrowed::new_nfc()
            .normalize(text)
            .into_owned(),
        Form::Nfd => DecomposingNormalizerBorrowed::new_nfd()
            .normalize(text)
            .into_owned(),
        Form::Nfkc => ComposingNormalizerBorrowed::new_nfkc()
            .normalize(text)
            .into_owned(),
        Form::Nfkd => DecomposingNormalizerBorrowed::new_nfkd()
            .normalize(text)
            .into_owned(),
    }
}

fn text_is_normalized(text: &str, form: Form) -> bool {
    match form {
        Form::Nfc => ComposingNormalizerBorrowed::new_nfc().is_normalized(text),
        Form::Nfd => DecomposingNormalizerBorrowed::new_nfd().is_normalized(text),
        Form::Nfkc => ComposingNormalizerBorrowed::new_nfkc().is_normalized(text),
        Form::Nfkd => DecomposingNormalizerBorrowed::new_nfkd().is_normalized(text),
    }
}

fn normalize(state: &mut LuaState) -> LuaResult<u32> {
    let text = read_text(state)?;
    let form = read_form(state)?;
    let result = normalize_text(&text, form);
    let value = state.gc.intern_string(result.as_bytes());
    state.push(Val::Str(value));
    Ok(1)
}

fn is_normalized(state: &mut LuaState) -> LuaResult<u32> {
    let text = read_text(state)?;
    let form = read_form(state)?;
    state.push(Val::Bool(text_is_normalized(&text, form)));
    Ok(1)
}
