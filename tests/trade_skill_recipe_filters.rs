#![cfg(feature = "profile-retail")]
//! Recipe-list filters driven through the original Professions filter helpers.

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

fn professions_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Lua environment");
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path().expect("Blizzard UI cache");
    for addon in ["Blizzard_Colors", "Blizzard_ProfessionsTemplates"] {
        let toc = ui.join(addon).join(format!("{addon}.toc"));
        load_addon(&env.loader_env(), &toc).unwrap_or_else(|err| panic!("{addon}: {err}"));
    }
    env
}

#[test]
fn default_filters_show_learned_and_unlearned_recipes() {
    let env = professions_env();
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
    assert_eq!(result, "false true true 10");
}

#[test]
fn category_and_item_level_filters_narrow_the_recipe_list() {
    let env = professions_env();
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
    assert_eq!(result, "8 false 100001,100002,100004,100005 10");
}

#[test]
fn filter_set_round_trips_through_apply() {
    let env = professions_env();
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
    assert_eq!(result, "10 true false 0");
}
