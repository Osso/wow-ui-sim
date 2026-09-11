//! PTR locale storage, ICU4X text operations, and ICU4C number formatting.
#[cfg(feature = "retail-12-1-5")]
mod breaks;
#[cfg(feature = "retail-12-1-5")]
mod casing;
#[cfg(feature = "retail-12-1-5")]
mod character_properties;
#[cfg(feature = "retail-12-1-5")]
mod collation;
#[cfg(feature = "retail-12-1-5")]
mod currency_metadata;
#[cfg(feature = "retail-12-1-5")]
mod date_formatting;
#[cfg(feature = "retail-12-1-5")]
mod display_transliteration;
#[cfg(feature = "retail-12-1-5")]
mod normalization;
#[cfg(feature = "retail-12-1-5")]
mod number_formatting;
#[cfg(feature = "retail-12-1-5")]
mod plurals;
#[cfg(feature = "retail-12-1-5")]
mod string_matches;
#[cfg(feature = "retail-12-1-5")]
mod text;
#[cfg(feature = "retail-12-1-5")]
mod transform;
use rilua::LuaResult;
use rilua::vm::state::LuaState;

#[cfg(not(feature = "retail-12-1-5"))]
pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let absent = super::ensure_namespace(state, "__wow_absent_namespaces")?;
    crate::lua_api::methods::table_set_static(
        state,
        rilua::Val::Table(absent),
        "C_Intl",
        rilua::Val::Bool(true),
    );
    Ok(())
}

#[cfg(feature = "retail-12-1-5")]
pub(crate) use storage::register;

#[cfg(feature = "retail-12-1-5")]
fn parse_locale(bytes: &[u8], operation: &str) -> LuaResult<icu_locale_core::Locale> {
    let is_wow_tag = bytes.len() == 4 && bytes.iter().all(u8::is_ascii_alphabetic);
    let tag = if is_wow_tag {
        [bytes[..2].to_vec(), vec![b'-'], bytes[2..].to_vec()].concat()
    } else {
        bytes.to_vec()
    };
    icu_locale_core::Locale::try_from_utf8(&tag).map_err(|_| {
        rilua::runtime_error(format!(
            "{operation} locale must be a valid ICU locale identifier"
        ))
    })
}

#[cfg(feature = "retail-12-1-5")]
mod storage {
    use super::*;
    use crate::lua_api::methods::{call_function_state, table_set_static};
    use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
    use rilua::vm::value::Userdata;
    use rilua::{Val, runtime_error};

    struct LocaleContext {
        locale: Vec<u8>,
    }

    pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
        let namespace = crate::c_api::ensure_namespace(state, "C_Intl")?;
        table_set_rust_fn_static(state, namespace, "CreateLocaleContext", create)?;
        table_set_rust_fn_static(state, namespace, "GetCurrentLocale", current_locale)?;
        table_set_rust_fn_static(state, namespace, "Length", length)?;
        super::normalization::register(state, namespace)?;
        super::casing::register(state, namespace)?;
        super::breaks::register(state, namespace)?;
        super::transform::register(state, namespace)?;
        super::collation::register(state, namespace)?;
        super::plurals::register(state, namespace)?;
        super::character_properties::register(state, namespace)?;
        super::number_formatting::register(state, namespace)?;
        super::date_formatting::register(state, namespace)?;
        super::display_transliteration::register(state, namespace)?;
        super::currency_metadata::register(state, namespace)?;
        super::string_matches::register(state, namespace)
    }

    fn identifier(state: &LuaState, index: i32) -> LuaResult<Vec<u8>> {
        let Val::Str(reference) = stack_val(state, index) else {
            return Err(runtime_error(format!(
                "bad argument #{index} (string expected)"
            )));
        };
        state
            .gc
            .string_arena
            .get(reference)
            .map(|value| value.data().to_vec())
            .ok_or_else(|| runtime_error("locale string has been collected"))
    }

    fn valid_identifier(locale: &[u8]) -> bool {
        !locale.is_empty() && !locale.contains(&0)
    }

    fn create(state: &mut LuaState) -> LuaResult<u32> {
        let locale = identifier(state, 1)?;
        if !valid_identifier(&locale) {
            return Err(runtime_error("locale must be nonempty and NUL-free"));
        }
        let metatable = rilua::stdlib::new_metatable(state, "LuaLocaleContext")?;
        table_set_rust_fn_static(state, metatable, "GetLocale", get_locale)?;
        table_set_rust_fn_static(state, metatable, "SetLocale", set_locale)?;
        table_set_rust_fn_static(state, metatable, "Length", context_length)?;
        super::casing::register_context(state, metatable)?;
        super::breaks::register_context(state, metatable)?;
        super::transform::register_context(state, metatable)?;
        super::collation::register_context(state, metatable)?;
        super::plurals::register_context(state, metatable)?;
        super::number_formatting::register_context(state, metatable)?;
        super::date_formatting::register_context(state, metatable)?;
        super::display_transliteration::register_context(state, metatable)?;
        super::currency_metadata::register_context(state, metatable)?;
        super::string_matches::register_context(state, metatable)?;
        table_set_static(
            state,
            Val::Table(metatable),
            "__index",
            Val::Table(metatable),
        );
        let object = Userdata::with_metatable(Box::new(LocaleContext { locale }), metatable);
        let reference = state.gc.alloc_userdata(object);
        state.push(Val::Userdata(reference));
        Ok(1)
    }

    fn context(state: &mut LuaState) -> LuaResult<&mut LocaleContext> {
        let Val::Userdata(reference) = stack_val(state, 1) else {
            return Err(runtime_error(
                "locale context method requires userdata self",
            ));
        };
        state
            .gc
            .userdata
            .get_mut(reference)
            .and_then(|value| value.downcast_mut::<LocaleContext>())
            .ok_or_else(|| runtime_error("incompatible locale context receiver"))
    }

    pub(super) fn context_locale_bytes(state: &mut LuaState) -> LuaResult<Vec<u8>> {
        Ok(context(state)?.locale.clone())
    }

    fn get_locale(state: &mut LuaState) -> LuaResult<u32> {
        let locale = context_locale_bytes(state)?;
        let value = state.gc.intern_string(&locale);
        state.push(Val::Str(value));
        Ok(1)
    }

    fn set_locale(state: &mut LuaState) -> LuaResult<u32> {
        let locale = identifier(state, 2)?;
        let valid = valid_identifier(&locale);
        let context = context(state)?;
        if valid {
            context.locale = locale;
        }
        state.push(Val::Bool(valid));
        Ok(1)
    }

    fn length(state: &mut LuaState) -> LuaResult<u32> {
        super::text::push_length(state, 1)
    }

    fn context_length(state: &mut LuaState) -> LuaResult<u32> {
        context(state)?;
        super::text::push_length(state, 2)
    }

    pub(super) fn current_locale_bytes(state: &mut LuaState) -> LuaResult<Vec<u8>> {
        let getter = crate::c_api::global_val(state, "GetLocale");
        let result = call_function_state(state, getter, &[])?;
        let Val::Str(reference) = result else {
            return Err(runtime_error("GetLocale must return a string"));
        };
        state
            .gc
            .string_arena
            .get(reference)
            .map(|value| value.data().to_vec())
            .ok_or_else(|| runtime_error("current locale string has been collected"))
    }

    fn current_locale(state: &mut LuaState) -> LuaResult<u32> {
        let locale = current_locale_bytes(state)?;
        let value = state.gc.intern_string(&locale);
        state.push(Val::Str(value));
        Ok(1)
    }
}
