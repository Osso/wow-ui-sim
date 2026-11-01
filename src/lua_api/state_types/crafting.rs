//! Crafting / profession dynamic state.

use std::collections::{BTreeSet, HashMap, HashSet};

/// Mutable crafting state. Static profession/recipe catalogue still
/// lives in `globals::profession_data`; this struct only carries
/// what the player has actually done with it.
#[derive(Debug, Clone, Default)]
pub struct CraftingState {
    pub reagents: crate::c_api::crafting_reagents::CraftingInputs,
    /// Recipe-list search text; an empty string disables name filtering.
    pub recipe_name_filter: String,
    /// Recipe-list display filters other than the name search.
    pub recipe_filters: RecipeListFilters,
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
    /// Wrath selection is a recipe ID, independent of filtered display order.
    pub wrath_selected_trade_skill_recipe: Option<i32>,
    pub wrath_trade_skill_name_filter: String,
    pub wrath_trade_skill_level_filter: (i32, i32),
    pub wrath_trade_skill_makeable_only: bool,
    pub wrath_trade_skill_repeat_count: u32,
    pub wrath_trade_skill_linked: bool,
    pub wrath_trade_skill_cooldowns: HashMap<i32, f64>,
    pub wrath_trade_skill_tools: HashMap<i32, Vec<String>>,
    /// Currently selected legacy ClassTrainerFrame service index.
    pub selected_trainer_service_index: Option<i32>,
}

/// Recipe-list display filters for `C_TradeSkillUI`; see
/// `docs/specs/trade-skill-recipe-filters.md`.
#[derive(Clone, Debug, PartialEq)]
pub struct RecipeListFilters {
    pub show_learned: bool,
    /// INFERRED: unlearned recipes start hidden, preserving the catalogue's
    /// learned-only list until the UI applies `Professions.SetDefaultFilters`.
    pub show_unlearned: bool,
    pub only_makeable: bool,
    /// Stored only: the catalogue records no relative difficulty.
    pub only_skill_up: bool,
    /// Stored only: the catalogue records no first-craft state.
    pub only_first_craft: bool,
    /// Filtered-out `Enum.BattlePetSources` indices.
    pub filtered_source_types: BTreeSet<i32>,
    /// Filtered-out filterable inventory-slot indices.
    pub filtered_inventory_slots: BTreeSet<i32>,
    /// Filtered-out recipe category IDs.
    pub filtered_category_ids: BTreeSet<i32>,
    /// Inclusive `(min, max)` recipe item level; `None` disables the filter.
    pub item_level_range: Option<(i32, i32)>,
}

impl Default for RecipeListFilters {
    fn default() -> Self {
        Self {
            show_learned: true,
            show_unlearned: false,
            only_makeable: false,
            only_skill_up: false,
            only_first_craft: false,
            filtered_source_types: BTreeSet::new(),
            filtered_inventory_slots: BTreeSet::new(),
            filtered_category_ids: BTreeSet::new(),
            item_level_range: None,
        }
    }
}
