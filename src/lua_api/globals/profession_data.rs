//! Profession and recipe data for the WoW UI simulator.
//!
//! Provides static profession info and Blacksmithing recipes for the crafting panel UI.

/// A profession known by the player.
pub struct ProfessionInfo {
    pub profession_id: i32,
    pub profession: i32,
    pub skill_line_id: i32,
    pub name: &'static str,
    pub parent_profession_name: &'static str,
    pub skill_level: i32,
    pub max_skill_level: i32,
    pub skill_modifier: i32,
    pub icon: i32,
}

/// A recipe category.
pub struct RecipeCategory {
    pub category_id: i32,
    pub name: &'static str,
    pub parent_category_id: i32,
    pub ui_order: i32,
}

/// A reagent requirement for a recipe.
///
/// `dependent_reagents` lists items that must also be allocated when this
/// reagent is allocated — e.g. a Crest's required Spark of Omens. The
/// crafting form refuses to craft when any dependent is missing
/// (`ProfessionsRecipeTransactionMixin:HasMissingDependentReagents`).
pub struct ReagentSlot {
    pub item_id: u32,
    pub quantity: i32,
    pub dependent_reagents: &'static [ReagentSlot],
}

/// A crafting recipe.
pub struct RecipeEntry {
    pub recipe_id: i32,
    pub name: &'static str,
    pub learned: bool,
    pub craftable: bool,
    pub difficulty: i32,
    pub category_id: i32,
    pub item_level: i32,
    pub output_item_id: u32,
    pub output_quantity: i32,
    pub reagents: &'static [ReagentSlot],
}

pub static PROFESSIONS: &[ProfessionInfo] = &[
    ProfessionInfo {
        profession_id: 164,
        profession: 1,
        skill_line_id: 164,
        name: "Blacksmithing",
        parent_profession_name: "",
        skill_level: 80,
        max_skill_level: 100,
        skill_modifier: 0,
        icon: 136241,
    },
    ProfessionInfo {
        profession_id: 186,
        profession: 6,
        skill_line_id: 186,
        name: "Mining",
        parent_profession_name: "",
        skill_level: 90,
        max_skill_level: 100,
        skill_modifier: 0,
        icon: 136248,
    },
];

const fn recipe_category(category_id: i32, name: &'static str, ui_order: i32) -> RecipeCategory {
    RecipeCategory {
        category_id,
        name,
        parent_category_id: 0,
        ui_order,
    }
}

pub static RECIPE_CATEGORIES: &[RecipeCategory] = &[
    recipe_category(101, "Classic Blacksmithing", 1),
    recipe_category(102, "Outland Blacksmithing", 2),
    recipe_category(103, "Northrend Blacksmithing", 3),
    recipe_category(104, "Cataclysm Blacksmithing", 4),
    recipe_category(105, "Pandaria Blacksmithing", 5),
    recipe_category(106, "Draenor Blacksmithing", 6),
    recipe_category(107, "Legion Blacksmithing", 7),
    recipe_category(108, "Kul Tiran Blacksmithing", 8),
    recipe_category(109, "Shadowlands Blacksmithing", 9),
    recipe_category(110, "Dragon Isles Blacksmithing", 10),
    recipe_category(111, "Khaz Algar Blacksmithing", 11),
    recipe_category(112, "Midnight Blacksmithing", 12),
    recipe_category(1, "Armor", 20),
    recipe_category(2, "Weapons", 21),
    recipe_category(3, "Reagents", 22),
    recipe_category(4, "Miscellaneous", 23),
];

const fn wago_blacksmithing_recipe(
    recipe_id: i32,
    name: &'static str,
    category_id: i32,
    output_item_id: u32,
    output_quantity: i32,
    reagents: &'static [ReagentSlot],
) -> RecipeEntry {
    RecipeEntry {
        recipe_id,
        name,
        learned: true,
        craftable: true,
        difficulty: 1,
        category_id,
        item_level: 1,
        output_item_id,
        output_quantity,
        reagents,
    }
}

const fn reagent(item_id: u32, quantity: i32) -> ReagentSlot {
    ReagentSlot {
        item_id,
        quantity,
        dependent_reagents: &[],
    }
}

#[allow(dead_code)]
const fn reagent_with_deps(
    item_id: u32,
    quantity: i32,
    dependent_reagents: &'static [ReagentSlot],
) -> ReagentSlot {
    ReagentSlot {
        item_id,
        quantity,
        dependent_reagents,
    }
}

static ROUGH_SHARPENING_STONE_REAGENTS: &[ReagentSlot] = &[reagent(2835, 1)];
static COPPER_CHAIN_PANTS_REAGENTS: &[ReagentSlot] = &[reagent(2840, 4)];
static FEL_IRON_PLATE_GLOVES_REAGENTS: &[ReagentSlot] = &[reagent(23445, 4)];
static FEL_IRON_PLATE_BELT_REAGENTS: &[ReagentSlot] = &[reagent(23445, 4)];
static COBALT_LEGPLATES_REAGENTS: &[ReagentSlot] = &[reagent(36916, 5)];
static COBALT_BELT_REAGENTS: &[ReagentSlot] = &[reagent(36916, 4)];
static FOLDED_OBSIDIUM_REAGENTS: &[ReagentSlot] = &[reagent(54849, 2)];
static HARDENED_OBSIDIUM_BRACERS_REAGENTS: &[ReagentSlot] = &[reagent(65365, 3), reagent(18567, 1)];
static SPIRITGUARD_HELM_REAGENTS: &[ReagentSlot] = &[reagent(72096, 12)];
static SPIRITGUARD_SHOULDERS_REAGENTS: &[ReagentSlot] = &[reagent(72096, 7)];
static TRUESTEEL_INGOT_REAGENTS: &[ReagentSlot] = &[reagent(109119, 20), reagent(109118, 10)];
static SMOLDERING_HELM_REAGENTS: &[ReagentSlot] = &[reagent(109118, 60)];
static LEYSTONE_ARMGUARDS_REAGENTS: &[ReagentSlot] = &[reagent(123918, 18)];
static LEYSTONE_WAISTGUARD_REAGENTS: &[ReagentSlot] = &[reagent(123918, 24)];
static MONEL_HARDENED_HOOFPLATES_REAGENTS: &[ReagentSlot] =
    &[reagent(152512, 25), reagent(160298, 2)];
static MONEL_HARDENED_STIRRUPS_REAGENTS: &[ReagentSlot] =
    &[reagent(152512, 25), reagent(160298, 2)];
static SHADOWGHAST_INGOT_REAGENTS: &[ReagentSlot] = &[
    reagent(171829, 1),
    reagent(171832, 1),
    reagent(171831, 1),
    reagent(171830, 1),
    reagent(180733, 4),
];
static CEREMONIOUS_BREASTPLATE_REAGENTS: &[ReagentSlot] =
    &[reagent(171828, 12), reagent(180733, 2)];
static PRIMAL_MOLTEN_WEAPON_REAGENTS: &[ReagentSlot] = &[reagent(189541, 17)];
static ALGARI_COMPETITOR_BREASTPLATE_REAGENTS: &[ReagentSlot] = &[reagent(222426, 6)];
static ALGARI_COMPETITOR_SABATONS_REAGENTS: &[ReagentSlot] = &[reagent(222426, 4)];
static SUN_BLESSED_TOOL_REAGENTS: &[ReagentSlot] = &[reagent(238528, 1), reagent(237366, 2)];

static HELM_REAGENTS: &[ReagentSlot] = &[reagent(210934, 12), reagent(210937, 2)];
static CHEST_REAGENTS: &[ReagentSlot] = &[reagent(210934, 16), reagent(210937, 3)];
static GAUNTLETS_REAGENTS: &[ReagentSlot] = &[reagent(210934, 8), reagent(210937, 2)];
static GREAVES_REAGENTS: &[ReagentSlot] = &[reagent(210934, 14), reagent(210937, 2)];
static GREATSWORD_REAGENTS: &[ReagentSlot] =
    &[reagent(210934, 20), reagent(210937, 4), reagent(210935, 2)];
static MACE_REAGENTS: &[ReagentSlot] = &[reagent(210934, 10), reagent(210937, 2)];
static INGOT_REAGENTS: &[ReagentSlot] = &[reagent(210930, 3)];
static AQIRITE_INGOT_REAGENTS: &[ReagentSlot] = &[reagent(210931, 3)];
static SPIKE_REAGENTS: &[ReagentSlot] = &[reagent(210934, 6), reagent(210937, 1)];
static CHAIN_REAGENTS: &[ReagentSlot] = &[reagent(210934, 8), reagent(210937, 1)];

pub static BLACKSMITHING_RECIPES: &[RecipeEntry] = &[
    wago_blacksmithing_recipe(
        2660,
        "Rough Sharpening Stone",
        101,
        2862,
        1,
        ROUGH_SHARPENING_STONE_REAGENTS,
    ),
    wago_blacksmithing_recipe(
        2662,
        "Copper Chain Pants",
        101,
        2852,
        1,
        COPPER_CHAIN_PANTS_REAGENTS,
    ),
    wago_blacksmithing_recipe(
        29545,
        "Fel Iron Plate Gloves",
        102,
        23482,
        1,
        FEL_IRON_PLATE_GLOVES_REAGENTS,
    ),
    wago_blacksmithing_recipe(
        29547,
        "Fel Iron Plate Belt",
        102,
        23484,
        1,
        FEL_IRON_PLATE_BELT_REAGENTS,
    ),
    wago_blacksmithing_recipe(
        52567,
        "Cobalt Legplates",
        103,
        39086,
        1,
        COBALT_LEGPLATES_REAGENTS,
    ),
    wago_blacksmithing_recipe(52568, "Cobalt Belt", 103, 39087, 1, COBALT_BELT_REAGENTS),
    wago_blacksmithing_recipe(
        76178,
        "Folded Obsidium",
        104,
        65365,
        1,
        FOLDED_OBSIDIUM_REAGENTS,
    ),
    wago_blacksmithing_recipe(
        76179,
        "Hardened Obsidium Bracers",
        104,
        54850,
        1,
        HARDENED_OBSIDIUM_BRACERS_REAGENTS,
    ),
    wago_blacksmithing_recipe(
        122568,
        "Spiritguard Helm",
        105,
        80811,
        1,
        SPIRITGUARD_HELM_REAGENTS,
    ),
    wago_blacksmithing_recipe(
        122569,
        "Spiritguard Shoulders",
        105,
        82896,
        1,
        SPIRITGUARD_SHOULDERS_REAGENTS,
    ),
    wago_blacksmithing_recipe(
        171690,
        "Truesteel Ingot",
        106,
        108257,
        1,
        TRUESTEEL_INGOT_REAGENTS,
    ),
    wago_blacksmithing_recipe(
        171691,
        "Smoldering Helm",
        106,
        116426,
        1,
        SMOLDERING_HELM_REAGENTS,
    ),
    wago_blacksmithing_recipe(
        182928,
        "Leystone Armguards",
        107,
        123898,
        1,
        LEYSTONE_ARMGUARDS_REAGENTS,
    ),
    wago_blacksmithing_recipe(
        182929,
        "Leystone Waistguard",
        107,
        123897,
        1,
        LEYSTONE_WAISTGUARD_REAGENTS,
    ),
    wago_blacksmithing_recipe(
        253110,
        "Monel-Hardened Hoofplates",
        108,
        152812,
        1,
        MONEL_HARDENED_HOOFPLATES_REAGENTS,
    ),
    wago_blacksmithing_recipe(
        253112,
        "Monel-Hardened Stirrups",
        108,
        152813,
        1,
        MONEL_HARDENED_STIRRUPS_REAGENTS,
    ),
    wago_blacksmithing_recipe(
        307611,
        "Shadowghast Ingot",
        109,
        171428,
        2,
        SHADOWGHAST_INGOT_REAGENTS,
    ),
    wago_blacksmithing_recipe(
        307663,
        "Ceremonious Breastplate",
        109,
        171374,
        1,
        CEREMONIOUS_BREASTPLATE_REAGENTS,
    ),
    wago_blacksmithing_recipe(
        365729,
        "Primal Molten Warglaive",
        110,
        190508,
        1,
        PRIMAL_MOLTEN_WEAPON_REAGENTS,
    ),
    wago_blacksmithing_recipe(
        365730,
        "Primal Molten Shortblade",
        110,
        190505,
        1,
        PRIMAL_MOLTEN_WEAPON_REAGENTS,
    ),
    wago_blacksmithing_recipe(
        438914,
        "Algari Competitor's Plate Breastplate",
        111,
        217143,
        1,
        ALGARI_COMPETITOR_BREASTPLATE_REAGENTS,
    ),
    wago_blacksmithing_recipe(
        438915,
        "Algari Competitor's Plate Sabatons",
        111,
        217144,
        1,
        ALGARI_COMPETITOR_SABATONS_REAGENTS,
    ),
    wago_blacksmithing_recipe(
        1229598,
        "Sun-Blessed Blacksmith's Hammer",
        112,
        238018,
        1,
        SUN_BLESSED_TOOL_REAGENTS,
    ),
    wago_blacksmithing_recipe(
        1229599,
        "Sun-Blessed Leatherworker's Knife",
        112,
        238017,
        1,
        SUN_BLESSED_TOOL_REAGENTS,
    ),
    RecipeEntry {
        recipe_id: 100001,
        name: "Khaz Algar Helm",
        learned: true,
        craftable: true,
        difficulty: 3,
        category_id: 1,
        item_level: 590,
        output_item_id: 211993,
        output_quantity: 1,
        reagents: HELM_REAGENTS,
    },
    RecipeEntry {
        recipe_id: 100002,
        name: "Khaz Algar Breastplate",
        learned: true,
        craftable: true,
        difficulty: 3,
        category_id: 1,
        item_level: 590,
        output_item_id: 211996,
        output_quantity: 1,
        reagents: CHEST_REAGENTS,
    },
    RecipeEntry {
        recipe_id: 100003,
        name: "Khaz Algar Gauntlets",
        learned: true,
        craftable: true,
        difficulty: 2,
        category_id: 1,
        item_level: 580,
        output_item_id: 211994,
        output_quantity: 1,
        reagents: GAUNTLETS_REAGENTS,
    },
    RecipeEntry {
        recipe_id: 100004,
        name: "Khaz Algar Greaves",
        learned: true,
        craftable: true,
        difficulty: 3,
        category_id: 1,
        item_level: 590,
        output_item_id: 211992,
        output_quantity: 1,
        reagents: GREAVES_REAGENTS,
    },
    RecipeEntry {
        recipe_id: 100005,
        name: "Greatsword of Radiant Dawn",
        learned: true,
        craftable: true,
        difficulty: 4,
        category_id: 2,
        item_level: 610,
        output_item_id: 229181,
        output_quantity: 1,
        reagents: GREATSWORD_REAGENTS,
    },
    RecipeEntry {
        recipe_id: 100006,
        name: "Khaz Algar Mace",
        learned: true,
        craftable: true,
        difficulty: 2,
        category_id: 2,
        item_level: 580,
        output_item_id: 0,
        output_quantity: 1,
        reagents: MACE_REAGENTS,
    },
    RecipeEntry {
        recipe_id: 100007,
        name: "Khaz Algar Ingot",
        learned: true,
        craftable: true,
        difficulty: 1,
        category_id: 3,
        item_level: 1,
        output_item_id: 210934,
        output_quantity: 1,
        reagents: INGOT_REAGENTS,
    },
    RecipeEntry {
        recipe_id: 100008,
        name: "Aqirite Ingot",
        learned: true,
        craftable: true,
        difficulty: 2,
        category_id: 3,
        item_level: 1,
        output_item_id: 210935,
        output_quantity: 1,
        reagents: AQIRITE_INGOT_REAGENTS,
    },
    RecipeEntry {
        recipe_id: 100009,
        name: "Khaz Algar Shield Spike",
        learned: true,
        craftable: true,
        difficulty: 2,
        category_id: 4,
        item_level: 1,
        output_item_id: 0,
        output_quantity: 1,
        reagents: SPIKE_REAGENTS,
    },
    RecipeEntry {
        recipe_id: 100010,
        name: "Khaz Algar Weapon Chain",
        learned: true,
        craftable: true,
        difficulty: 2,
        category_id: 4,
        item_level: 1,
        output_item_id: 0,
        output_quantity: 1,
        reagents: CHAIN_REAGENTS,
    },
];

pub fn get_profession(profession_id: i32) -> Option<&'static ProfessionInfo> {
    PROFESSIONS
        .iter()
        .find(|p| p.profession_id == profession_id)
}

pub fn get_profession_by_skill_line_id(skill_line_id: i32) -> Option<&'static ProfessionInfo> {
    PROFESSIONS
        .iter()
        .find(|p| p.skill_line_id == skill_line_id)
}

pub fn get_profession_by_index(index: usize) -> Option<&'static ProfessionInfo> {
    PROFESSIONS.get(index)
}

pub fn get_recipe(recipe_id: i32) -> Option<&'static RecipeEntry> {
    BLACKSMITHING_RECIPES
        .iter()
        .find(|r| r.recipe_id == recipe_id)
}

/// The current recipe catalog contains only Blacksmithing entries.
pub fn get_profession_for_recipe(recipe_id: i32) -> Option<&'static ProfessionInfo> {
    get_recipe(recipe_id).and_then(|_| get_profession_by_index(0))
}

pub fn get_all_recipe_ids() -> Vec<i32> {
    BLACKSMITHING_RECIPES.iter().map(|r| r.recipe_id).collect()
}

pub fn get_filtered_recipe_ids() -> Vec<i32> {
    BLACKSMITHING_RECIPES
        .iter()
        .filter(|r| r.learned)
        .map(|r| r.recipe_id)
        .collect()
}

pub fn get_category_ids() -> Vec<i32> {
    RECIPE_CATEGORIES
        .iter()
        .map(|category| category.category_id)
        .collect()
}

pub fn get_category(category_id: i32) -> Option<&'static RecipeCategory> {
    RECIPE_CATEGORIES
        .iter()
        .find(|c| c.category_id == category_id)
}

/// Find the dependent reagents declared for `item_id` across all recipes.
///
/// `C_TradeSkillUI.GetDependentReagents(reagent)` is the API surface; the
/// dependency is intrinsic to the reagent item (not the recipe slot), so the
/// first matching slot wins. Returns `&[]` for items with no declared
/// dependents (the common case).
pub fn find_reagent_dependents(item_id: u32) -> &'static [ReagentSlot] {
    BLACKSMITHING_RECIPES
        .iter()
        .flat_map(|r| r.reagents.iter())
        .find(|slot| slot.item_id == item_id)
        .map(|slot| slot.dependent_reagents)
        .unwrap_or(&[])
}
