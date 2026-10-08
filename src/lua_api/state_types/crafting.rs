//! Crafting / profession dynamic state.

use std::collections::HashSet;

/// Mutable crafting state. Static profession/recipe catalogue still
/// lives in `globals::profession_data`; this struct only carries
/// what the player has actually done with it.
#[derive(Debug, Clone, Default)]
pub struct CraftingState {
    pub reagents: crate::c_api::crafting_reagents::CraftingInputs,
    /// Recipe-list search text; an empty string disables name filtering.
    pub recipe_name_filter: String,
    /// Currently-selected profession id (Skill Line ID — matches the
    /// values in `profession_data`). `None` until `C_TradeSkillUI.
    /// SetProfessionChildSkillLineID` is called or the player opens
    /// a trainer.
    pub selected_profession_id: Option<i32>,
    /// Recipes the player knows. Populated by
    /// `A_Admin.LearnRecipe(id)` / `UnlearnRecipe(id)`. Drives
    /// `C_TradeSkillUI.IsRecipeLearned(id)`.
    pub known_recipe_ids: HashSet<i32>,
    /// Professions the player has abandoned. Populated by `AbandonSkill(skillLine)`
    /// and `A_Admin.UnlearnProfession(skillLineId)`. Filters all profession queries.
    pub unlearned_profession_ids: HashSet<i32>,
    /// Currently selected legacy CraftFrame list index.
    pub selected_craft_index: Option<i32>,
    /// Currently selected legacy TradeSkillFrame list index.
    pub selected_trade_skill_index: Option<i32>,
    /// Currently selected legacy ClassTrainerFrame service index.
    pub selected_trainer_service_index: Option<i32>,
}
