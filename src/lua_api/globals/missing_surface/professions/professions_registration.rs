use super::{
    CRAFTING_ORDERS_NAMESPACE, TRADE_SKILL_NAMESPACE, c_crafting_orders_get_order_claim_info,
    c_trade_skill_ui_close_trade_skill, c_trade_skill_ui_craft_recipe,
    c_trade_skill_ui_get_all_profession_trade_skill_lines, c_trade_skill_ui_get_all_recipe_ids,
    c_trade_skill_ui_get_base_profession_info, c_trade_skill_ui_get_categories,
    c_trade_skill_ui_get_category_info, c_trade_skill_ui_get_child_profession_info,
    c_trade_skill_ui_get_child_profession_infos, c_trade_skill_ui_get_crafting_order_count,
    c_trade_skill_ui_get_dependent_reagents, c_trade_skill_ui_get_num_recipes,
    c_trade_skill_ui_get_num_trade_skills, c_trade_skill_ui_get_profession_by_inventory_slot,
    c_trade_skill_ui_get_profession_info_by_recipe_id,
    c_trade_skill_ui_get_profession_inventory_slots, c_trade_skill_ui_get_profession_skill_line_id,
    c_trade_skill_ui_get_profession_slots, c_trade_skill_ui_get_professions,
    c_trade_skill_ui_get_recipe_description, c_trade_skill_ui_get_recipe_info,
    c_trade_skill_ui_get_recipe_item_link, c_trade_skill_ui_get_recipe_num_reagents,
    c_trade_skill_ui_get_recipe_output_item_data, c_trade_skill_ui_get_recipe_reagent_info,
    c_trade_skill_ui_get_recipe_reagent_item_link, c_trade_skill_ui_get_recipe_schematic,
    c_trade_skill_ui_get_recipes_tracked, c_trade_skill_ui_get_trade_skill_display_name,
    c_trade_skill_ui_get_trade_skill_line, c_trade_skill_ui_get_trade_skill_list_link,
    c_trade_skill_ui_get_trade_skill_texture, c_trade_skill_ui_is_data_source_changing,
    c_trade_skill_ui_is_npc_crafting, c_trade_skill_ui_is_recipe_craftable,
    c_trade_skill_ui_is_recipe_in_skill_line, c_trade_skill_ui_is_recipe_learned,
    c_trade_skill_ui_is_recipe_tracked, c_trade_skill_ui_is_runeforging,
    c_trade_skill_ui_is_trade_skill_guild, c_trade_skill_ui_is_trade_skill_guild_member,
    c_trade_skill_ui_is_trade_skill_linked, c_trade_skill_ui_is_trade_skill_ready,
    c_trade_skill_ui_open_trade_skill, c_trade_skill_ui_set_profession_child_skill_line_id,
    c_trade_skill_ui_set_recipe_tracked, ensure_namespace,
};
use crate::lua_api::methods::create_table;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

type NamespaceMethod = (&'static str, fn(&mut LuaState) -> LuaResult<u32>);

const TRADE_SKILL_METHODS: &[NamespaceMethod] = &[
    (
        "GetAllProfessionTradeSkillLines",
        c_trade_skill_ui_get_all_profession_trade_skill_lines,
    ),
    ("GetAllRecipeIDs", c_trade_skill_ui_get_all_recipe_ids),
    (
        "GetBaseProfessionInfo",
        c_trade_skill_ui_get_base_profession_info,
    ),
    ("GetCategoryInfo", c_trade_skill_ui_get_category_info),
    (
        "GetChildProfessionInfo",
        c_trade_skill_ui_get_child_profession_info,
    ),
    (
        "GetChildProfessionInfos",
        c_trade_skill_ui_get_child_profession_infos,
    ),
    ("GetConcentrationCurrencyID", stub_zero),
    (
        "GetCraftingOrderCount",
        c_trade_skill_ui_get_crafting_order_count,
    ),
    (
        "GetDependentReagents",
        c_trade_skill_ui_get_dependent_reagents,
    ),
    ("GetCraftableCount", stub_zero),
    ("GetRecraftRemovalWarnings", stub_empty_table),
    ("GetRemainingRecasts", stub_zero),
    ("IsRecraftItemEquipped", stub_false),
    ("RecraftLimitCategoryValid", stub_true),
    ("CanStoreEnchantInItem", stub_false),
    ("CanTradeSkillListLink", stub_false),
    ("GetCategories", c_trade_skill_ui_get_categories),
    (
        "GetProfessionInfoByRecipeID",
        c_trade_skill_ui_get_profession_info_by_recipe_id,
    ),
    (
        "GetProfessionByInventorySlot",
        c_trade_skill_ui_get_profession_by_inventory_slot,
    ),
    (
        "GetProfessionInventorySlots",
        c_trade_skill_ui_get_profession_inventory_slots,
    ),
    (
        "GetProfessionSkillLineID",
        c_trade_skill_ui_get_profession_skill_line_id,
    ),
    ("GetProfessionSlots", c_trade_skill_ui_get_profession_slots),
    ("GetProfessions", c_trade_skill_ui_get_professions),
    ("GetNumRecipes", c_trade_skill_ui_get_num_recipes),
    ("GetNumTradeSkills", c_trade_skill_ui_get_num_trade_skills),
    (
        "GetRecipeDescription",
        c_trade_skill_ui_get_recipe_description,
    ),
    ("GetRecipeInfo", c_trade_skill_ui_get_recipe_info),
    ("GetRecipeItemLink", c_trade_skill_ui_get_recipe_item_link),
    (
        "GetRecipeOutputItemData",
        c_trade_skill_ui_get_recipe_output_item_data,
    ),
    ("GetRecipeQualityItemIDs", stub_empty_table),
    (
        "GetRecipeNumReagents",
        c_trade_skill_ui_get_recipe_num_reagents,
    ),
    ("GetRecipeRequirements", stub_empty_table),
    (
        "GetRecipeReagentInfo",
        c_trade_skill_ui_get_recipe_reagent_info,
    ),
    (
        "GetRecipeReagentItemLink",
        c_trade_skill_ui_get_recipe_reagent_item_link,
    ),
    ("GetRecipeSchematic", c_trade_skill_ui_get_recipe_schematic),
    ("GetRecipesTracked", c_trade_skill_ui_get_recipes_tracked),
    (
        "GetTradeSkillDisplayName",
        c_trade_skill_ui_get_trade_skill_display_name,
    ),
    ("GetTradeSkillLine", c_trade_skill_ui_get_trade_skill_line),
    (
        "GetTradeSkillListLink",
        c_trade_skill_ui_get_trade_skill_list_link,
    ),
    ("GetFactionSpecificOutputItem", stub_nil),
    (
        "GetTradeSkillTexture",
        c_trade_skill_ui_get_trade_skill_texture,
    ),
    (
        "IsDataSourceChanging",
        c_trade_skill_ui_is_data_source_changing,
    ),
    ("IsNPCCrafting", c_trade_skill_ui_is_npc_crafting),
    ("IsRecipeCraftable", c_trade_skill_ui_is_recipe_craftable),
    (
        "IsRecipeInSkillLine",
        c_trade_skill_ui_is_recipe_in_skill_line,
    ),
    ("IsRecipeLearned", c_trade_skill_ui_is_recipe_learned),
    ("IsRecipeTracked", c_trade_skill_ui_is_recipe_tracked),
    ("IsRuneforging", c_trade_skill_ui_is_runeforging),
    ("IsTradeSkillGuild", c_trade_skill_ui_is_trade_skill_guild),
    (
        "IsTradeSkillGuildMember",
        c_trade_skill_ui_is_trade_skill_guild_member,
    ),
    ("IsTradeSkillLinked", c_trade_skill_ui_is_trade_skill_linked),
    ("IsTradeSkillReady", c_trade_skill_ui_is_trade_skill_ready),
    ("OpenTradeSkill", c_trade_skill_ui_open_trade_skill),
    ("CloseTradeSkill", c_trade_skill_ui_close_trade_skill),
    (
        "SetProfessionChildSkillLineID",
        c_trade_skill_ui_set_profession_child_skill_line_id,
    ),
    ("SetRecipeTracked", c_trade_skill_ui_set_recipe_tracked),
    ("CraftRecipe", c_trade_skill_ui_craft_recipe),
];

const CRAFTING_ORDER_METHODS: &[NamespaceMethod] = &[
    ("AreOrderNotesDisabled", stub_false),
    ("CanOrderSkillAbility", stub_false),
    ("CloseCrafterCraftingOrders", stub_noop),
    ("GetClaimedOrder", stub_nil),
    ("GetCrafterBuckets", stub_empty_table),
    ("GetCrafterOrders", stub_empty_table),
    ("GetCraftingOrderTime", stub_zero),
    ("GetOrderClaimInfo", c_crafting_orders_get_order_claim_info),
    ("GetPersonalOrdersInfo", stub_empty_table),
    ("OpenCrafterCraftingOrders", stub_noop),
    ("OrderCanBeRecrafted", stub_false),
    ("ShouldShowCraftingOrderTab", stub_false),
    ("SkillLineHasOrders", stub_false),
    ("UpdateIgnoreList", stub_noop),
];

pub(super) fn register_trade_skill_namespace(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, TRADE_SKILL_NAMESPACE)?;
    register_namespace_methods(state, table_ref, TRADE_SKILL_METHODS)
}

pub(super) fn register_crafting_order_namespace(state: &mut LuaState) -> LuaResult<()> {
    let orders_ref = ensure_namespace(state, CRAFTING_ORDERS_NAMESPACE)?;
    register_namespace_methods(state, orders_ref, CRAFTING_ORDER_METHODS)
}

fn register_namespace_methods(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
    methods: &[NamespaceMethod],
) -> LuaResult<()> {
    for &(name, func) in methods {
        table_set_rust_fn_static(state, table_ref, name, func)?;
    }
    Ok(())
}

fn stub_noop(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn stub_nil(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Nil);
    Ok(1)
}

fn stub_false(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

fn stub_true(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

fn stub_zero(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

fn stub_empty_table(state: &mut LuaState) -> LuaResult<u32> {
    let table = create_table(state);
    state.push(table);
    Ok(1)
}
