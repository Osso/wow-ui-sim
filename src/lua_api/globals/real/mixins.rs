//! Retail native mixin globals.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::globals::security::register_native_function_with_flags;
use crate::lua_api::methods::{create_table, registry_get, table_get_static};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

/// VERIFIED (R88k.7): Retail `Mixin` and `CreateFromMixins` use flags value 7.
const MIXIN_FLAGS: u8 = 7;

/// Register the verified Retail native entries. Bootstrap fallbacks stay ordinary-only.
pub(crate) fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
    if ACTIVE == ClientProfile::Retail {
        register_native_function_with_flags(lua, "Mixin", mixin, MIXIN_FLAGS)?;
        register_native_function_with_flags(
            lua,
            "CreateFromMixins",
            create_from_mixins,
            MIXIN_FLAGS,
        )?;
    }
    Ok(())
}

fn mixin(state: &mut LuaState) -> LuaResult<u32> {
    let argument_count = state.top.saturating_sub(state.base);
    let object = stack_val(state, 1);
    apply_mixins(state, object, 2, argument_count)?;
    state.push(object);
    Ok(1)
}

fn create_from_mixins(state: &mut LuaState) -> LuaResult<u32> {
    let argument_count = state.top.saturating_sub(state.base);
    let object = create_table(state);
    state.push(object);
    apply_mixins(state, object, 1, argument_count)?;
    Ok(1)
}

fn apply_mixins(
    state: &mut LuaState,
    object: Val,
    first_mixin_index: usize,
    argument_count: usize,
) -> LuaResult<()> {
    for index in first_mixin_index..=argument_count {
        let mixin = stack_val(state, index as i32);
        apply_mixin(state, object, mixin)?;
    }
    Ok(())
}

fn apply_mixin(state: &mut LuaState, object: Val, mixin: Val) -> LuaResult<()> {
    let Val::Table(methods_ref) = resolve_mixin_methods(state, mixin) else {
        return Ok(());
    };
    let Some(methods) = state.gc.tables.get(methods_ref) else {
        return Ok(());
    };

    for (key, value) in collect_mixin_entries(methods) {
        state.settable(object, key, value)?;
    }
    Ok(())
}

fn collect_mixin_entries(methods: &Table) -> Vec<(Val, Val)> {
    let array_entries = methods
        .array_slice()
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(index, value)| {
            (!matches!(value, Val::Nil)).then_some((Val::Num((index + 1) as f64), value))
        });
    let hash_entries = methods
        .hash_entries()
        .into_iter()
        .filter(|(_, value)| !matches!(value, Val::Nil));
    array_entries.chain(hash_entries).collect()
}

fn resolve_mixin_methods(state: &mut LuaState, mixin: Val) -> Val {
    if !matches!(mixin, Val::Table(_)) {
        return mixin;
    }

    let secure_env = registry_get(state, "__secureenv");
    let secure_registry = table_get_static(state, secure_env, "__secureMixinMethods");
    let secure_methods = raw_table_get(state, secure_registry, mixin);
    if matches!(secure_methods, Val::Table(_)) {
        return secure_methods;
    }

    let global = Val::Table(state.global);
    let public_registry = table_get_static(state, global, "__secureMixinMethods");
    let public_methods = raw_table_get(state, public_registry, mixin);
    if matches!(public_methods, Val::Table(_)) {
        return public_methods;
    }

    mixin
}

fn raw_table_get(state: &LuaState, table: Val, key: Val) -> Val {
    let Val::Table(table_ref) = table else {
        return Val::Nil;
    };
    state
        .gc
        .tables
        .get(table_ref)
        .map(|table| table.get(key, &state.gc.string_arena))
        .unwrap_or(Val::Nil)
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[cfg(feature = "client-retail")]
    #[test]
    fn secure_mixins_use_the_secure_registry_and_initialize_the_result() {
        let env = WowLuaEnv::new().expect("lua env");
        env.exec_rilua_secure(
            r#"
            __secureMixinMethods = setmetatable({}, { __mode = "k" })
            local mixin = { publicField = true }
            __secureMixinMethods[mixin] = {
                secureField = true,
                Init = function(self, value) self.initialized = value end,
            }

            local mixed = {}
            Mixin(mixed, mixin)
            secureMixinProbeMixin = mixed.secureField == true and mixed.publicField == nil

            local created = CreateFromMixins(mixin)
            secureMixinProbeCreate = created.secureField == true and created.publicField == nil

            "#,
        )
        .expect("execute secure mixin probe");

        let result: (bool, bool, String, String, bool) = env
            .eval(
                r#"
                local initialized = CreateAndInitFromMixin({
                    Init = function(self, value) self.value = value end,
                }, 17)
                return __secureenv.secureMixinProbeMixin,
                       __secureenv.secureMixinProbeCreate,
                       type(CreateAndInitFromMixin),
                       type(rawget(__secureenv, "CreateAndInitFromMixin")),
                       initialized.value == 17
                "#,
            )
            .expect("read secure mixin probe");
        assert_eq!(
            result,
            (true, true, "function".to_owned(), "nil".to_owned(), true)
        );
    }
}
