use crate::lua_api::methods::{registry_get, table_get_static};
use rilua::Val;
use rilua::vm::state::LuaState;

/// INFERRED (simulator model from `shared_bootstrap.lua`): resolve methods from
/// the secure registry first, then the public registry, then the source mixin.
/// R88k.7 verifies the registration flags, not this lookup order. Evidence: pr46-49-decomp-answers.md, sections R88k.7.
pub(super) fn resolve_mixin_methods(state: &mut LuaState, mixin: Val) -> Val {
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
