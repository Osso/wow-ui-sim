//! Host-supplied recipe source text. Native data provenance is unknown.
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    if matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        let ns = super::ensure_namespace(state, "C_TradeSkillUI")?;
        table_set_rust_fn_static(state, ns, "GetRecipeSourceText", get_recipe_source_text)?;
    }
    Ok(())
}

fn get_recipe_source_text(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let Val::Num(id) = value else {
        return Err(rilua::runtime_error("recipeID must be a number"));
    };
    if !id.is_finite() || id.fract() != 0.0 || id < 0.0 || id > u32::MAX as f64 {
        return Err(rilua::runtime_error("recipeID must be an unsigned integer"));
    }
    let text = borrow_state(state)?
        .recipe_source_text
        .get(&(id as u32))
        .cloned();
    match text {
        Some(text) => {
            let text = create_string(state, &text);
            state.push(text);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}
