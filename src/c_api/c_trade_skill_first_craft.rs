//! `C_TradeSkillUI.IsRecipeFirstCraft` (WM88, Retail 12.1.5 build 70688).

use crate::lua_api::globals::profession_data;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const USAGE: &str = "Usage: local result = C_TradeSkillUI.IsRecipeFirstCraft(recipeID)";

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail
        || !cfg!(feature = "retail-12-1-5")
    {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_TradeSkillUI")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "IsRecipeFirstCraft",
        is_recipe_first_craft,
    )
}

/// EMULATED (WM88): the simulator has no SkillLineAbility, SpellEffect or
/// CraftingData export, so a catalog recipe stands for a resolved ability with
/// first-craft metadata. No ability relation is native false.
pub(crate) fn is_first_craft(recipe_id: i32) -> bool {
    profession_data::get_recipe(recipe_id).is_some() && !completion_recorded(recipe_id)
}

/// UNKNOWN (WM88): completion bookkeeping, its post-craft writer and events are
/// not modeled. Unavailable completion state reads clear, as on the native
/// missing-state paths, so true does not prove a recipe was never crafted.
fn completion_recorded(_recipe_id: i32) -> bool {
    false
}

fn is_recipe_first_craft(state: &mut LuaState) -> LuaResult<u32> {
    let recipe_id =
        super::helpers::read_untainted_signed32_argument(state, 1, "IsRecipeFirstCraft", USAGE)?;
    state.push(Val::Bool(is_first_craft(recipe_id)));
    Ok(1)
}
