//! Recipe favorite state and deferred notifications (WM83).

use std::collections::HashSet;

use crate::lua_api::globals::profession_data;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

/// An empty initial set is an emulated default, not a native preference claim.
#[derive(Debug, Clone, Default)]
pub(crate) struct RecipeFavorites {
    favorites: HashSet<i32>,
    pending: Vec<(bool, i32)>,
}

impl RecipeFavorites {
    pub(crate) fn contains(&self, recipe_id: i32) -> bool {
        self.favorites.contains(&recipe_id)
    }

    fn set(&mut self, recipe_id: i32, favorite: bool) {
        let changed = if favorite {
            self.favorites.insert(recipe_id)
        } else {
            self.favorites.remove(&recipe_id)
        };
        let event = (favorite, recipe_id);
        if changed && !self.pending.contains(&event) {
            self.pending.push(event);
        }
    }
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    // VERIFIED (PTR R85 and Retail WM83): IsRecipeFavorite and SetRecipeFavorite match.
    if !supports_recipe_favorite_apis() {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_TradeSkillUI")?;
    table_set_rust_fn_static(state, namespace, "IsRecipeFavorite", is_recipe_favorite)?;
    table_set_rust_fn_static(state, namespace, "SetRecipeFavorite", set_recipe_favorite)
}

pub(crate) fn register_has_favorite_order_recipes(state: &mut LuaState) -> LuaResult<()> {
    // INFERRED (PTR R85; publication UNKNOWN): retain for PTR 70493 consumers
    // TradeSkillUIDocumentation.lua:897 and ProfessionsCrafterOrderPage.lua:434,523.
    if !supports_recipe_favorite_apis() {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_TradeSkillUI")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "HasFavoriteOrderRecipes",
        has_favorite_order_recipes,
    )
}

fn supports_recipe_favorite_apis() -> bool {
    matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    ) && cfg!(feature = "retail-12-1-5")
}

fn recipe_id(state: &LuaState, usage: &str) -> LuaResult<i32> {
    let value = stack_val(state, 1);
    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .and_then(|string| super::helpers::parse_numeric_string(string.data())),
        _ => None,
    }
    .ok_or_else(|| runtime_error(usage))?;
    if !number.is_finite() || number < f64::from(i32::MIN) || number > f64::from(i32::MAX) {
        return Err(runtime_error(format!(
            "integer overflow attempting to store {number:.6}"
        )));
    }
    Ok(number.trunc() as i32)
}

fn is_recipe_favorite(state: &mut LuaState) -> LuaResult<u32> {
    let recipe_id = recipe_id(state, "Usage: IsRecipeFavorite(recipeID)")?;
    if profession_data::get_recipe(recipe_id).is_none() {
        return Ok(0);
    }
    let favorite = borrow_state(state)?
        .crafting
        .recipe_favorites
        .contains(recipe_id);
    state.push(Val::Bool(favorite));
    Ok(1)
}

fn set_recipe_favorite(state: &mut LuaState) -> LuaResult<u32> {
    let recipe_id = recipe_id(state, "Usage: SetRecipeFavorite(recipeID, favorite)")?;
    let favorite = !matches!(stack_val(state, 2), Val::Nil | Val::Bool(false));
    let Some(profession) = profession_data::get_profession_for_recipe(recipe_id) else {
        return Ok(0);
    };
    let mut sim = borrow_state_mut(state)?;
    if sim.crafting.selected_profession_id == Some(profession.profession_id) {
        sim.crafting.recipe_favorites.set(recipe_id, favorite);
    }
    Ok(0)
}

#[path = "crafting_order_recipe_data.rs"]
mod crafting_order_recipe_data;

/// WM87: existence of associated CraftingOrder metadata makes a row eligible.
fn is_order_eligible(recipe_id: i32) -> bool {
    crafting_order_recipe_data::ORDER_RECIPE_IDS
        .binary_search(&recipe_id)
        .is_ok()
}

/// WM87: any loaded row with both the favorite and order-eligibility flags.
fn has_favorite_order_recipes(state: &mut LuaState) -> LuaResult<u32> {
    let has_favorites = {
        let favorites = &borrow_state(state)?.crafting.recipe_favorites;
        profession_data::get_all_recipe_ids()
            .into_iter()
            .any(|recipe_id| favorites.contains(recipe_id) && is_order_eligible(recipe_id))
    };
    state.push(Val::Bool(has_favorites));
    Ok(1)
}

#[cfg(test)]
mod tests {
    use super::is_order_eligible;

    #[test]
    fn crafting_order_recipe_data_matches_70391_export() {
        // CraftingOrder 36 -> SkillLineAbility 46625 -> Spell 365729.
        assert!(is_order_eligible(365729));
        // CraftingOrder 21 -> SkillLineAbility 46644 -> Spell 365730.
        assert!(is_order_eligible(365730));
        // CraftingOrder 854 -> SkillLineAbility 49927 -> Spell 438914.
        // Its Field_10_0_2_46479_002 is zero; existence still qualifies.
        assert!(is_order_eligible(438914));
        assert!(!is_order_eligible(2662));
        assert!(!is_order_eligible(99_999_999));
    }
}

pub(crate) fn drain_pending(state: &mut LuaState) -> LuaResult<()> {
    let events = std::mem::take(&mut borrow_state_mut(state)?.crafting.recipe_favorites.pending);
    for (favorite, recipe_id) in events {
        fire_named_event_state(
            state,
            "TRADE_SKILL_FAVORITES_CHANGED",
            &[Val::Bool(favorite), Val::Num(f64::from(recipe_id))],
        );
    }
    Ok(())
}
