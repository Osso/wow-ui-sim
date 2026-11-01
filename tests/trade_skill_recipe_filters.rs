#![cfg(feature = "client-retail")]
//! Recipe-list filters driven through the original Professions filter helpers.

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

fn load_professions_templates(env: &WowLuaEnv) {
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path().expect("Blizzard UI cache");
    let toc = ui
        .join("Blizzard_ProfessionsTemplates")
        .join("Blizzard_ProfessionsTemplates.toc");
    load_addon(&env.loader_env(), &toc).expect("Blizzard_ProfessionsTemplates loads");
}

prefork_full_ui_case! {
    fn default_filters_show_learned_and_unlearned_recipes(env: &WowLuaEnv) {
    load_professions_templates(env);
    let result: String = env
        .eval(
            r#"
            local before = tostring(Professions.IsUsingDefaultFilters(true))
            Professions.SetDefaultFilters(true)
            return before .. " " .. tostring(Professions.IsUsingDefaultFilters(true)) .. " "
                .. tostring(C_TradeSkillUI.GetShowUnlearned()) .. " "
                .. #C_TradeSkillUI.GetFilteredRecipeIDs()
            "#,
        )
        .unwrap();
    assert_eq!(result, "false true true 34");
    }
}

prefork_full_ui_case! {
    fn category_and_item_level_filters_narrow_the_recipe_list(env: &WowLuaEnv) {
    load_professions_templates(env);
    let result: String = env
        .eval(
            r#"
            Professions.SetDefaultFilters(true)
            C_TradeSkillUI.SetRecipeCategoryFilter(3, true)
            local withoutIngots = #C_TradeSkillUI.GetFilteredRecipeIDs()
            local changed = Professions.IsUsingDefaultFilters(true)
            C_TradeSkillUI.ClearRecipeCategoryFilter()
            C_TradeSkillUI.SetRecipeItemLevelFilter(590, 610)
            local ids = C_TradeSkillUI.GetFilteredRecipeIDs()
            C_TradeSkillUI.SetRecipeItemLevelFilter(0, 0)
            return withoutIngots .. " " .. tostring(changed) .. " "
                .. table.concat(ids, ",") .. " " .. #C_TradeSkillUI.GetFilteredRecipeIDs()
            "#,
        )
        .unwrap();
    assert_eq!(result, "32 false 100001,100002,100004,100005 34");
    }
}

prefork_full_ui_case! {
    fn filter_set_round_trips_through_apply(env: &WowLuaEnv) {
    load_professions_templates(env);
    let result: String = env
        .eval(
            r#"
            Professions.SetDefaultFilters(true)
            C_TradeSkillUI.SetOnlyShowMakeableRecipes(true)
            C_TradeSkillUI.SetShowLearned(false)
            local saved = Professions.GetCurrentFilterSet()
            Professions.SetDefaultFilters(true)
            local reset = #C_TradeSkillUI.GetFilteredRecipeIDs()
            Professions.ApplyfilterSet(saved)
            return reset .. " " .. tostring(C_TradeSkillUI.GetOnlyShowMakeableRecipes()) .. " "
                .. tostring(C_TradeSkillUI.GetShowLearned()) .. " "
                .. #C_TradeSkillUI.GetFilteredRecipeIDs()
            "#,
        )
        .unwrap();
    assert_eq!(result, "34 true false 0");
    }
}
