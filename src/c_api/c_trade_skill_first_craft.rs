//! `C_TradeSkillUI.IsRecipeFirstCraft` (WM88, Retail 12.1.5 build 70688).

use crate::lua_api::globals::profession_data;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const USAGE: &str = "Usage: local result = C_TradeSkillUI.IsRecipeFirstCraft(recipeID)";

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !publishes_first_craft() {
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

/// PTR 70493 ProfessionsRecipeSchematicForm.lua:661 consumes this API; same
/// contract as Retail WM88, INFERRED for PTR pending native evidence. Forever
/// 70707 documents it (TradeSkillUIDocumentation.lua:971-983) and the same
/// selected SchematicForm line calls it; Forever parity is INFERRED.
fn publishes_first_craft() -> bool {
    use crate::client_profile::{ACTIVE, ClientProfile};
    let modern_retail = matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr)
        && cfg!(feature = "retail-12-1-5");
    modern_retail || ACTIVE == ClientProfile::WowForever
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
