//! Concrete bounded contracts; not historical/native Legion parity.
#[path = "common/publication_sweep.rs"]
mod sweep;

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_7_0_3_publication_probe_constructs_line_and_alpha() {
    let env = WowLuaEnv::new().unwrap();
    for symbol in ["Line", "Alpha:SetFromAlpha"] {
        let entry = sweep::Entry {
            id: symbol.into(),
            section: "widgets".into(),
            direction: "added".into(),
            symbol: symbol.into(),
            page_default: None,
            kind: None,
        };
        let result = sweep::probe_entry(&env, &entry, false, &std::collections::BTreeMap::new());
        assert!(result.2, "{symbol}: {}", result.1);
    }
}

pub(crate) const RECIPE_FILTER_ASSERTIONS: &str = r#"
local api = C_TradeSkillUI
local all = api.GetFilteredRecipeIDs()
assert(#all > 2, 'fixture must have enough recipes to prove filtering')
local calls = 0
local listener = CreateFrame('Frame')
listener:RegisterEvent('TRADE_SKILL_LIST_UPDATE')
listener:SetScript('OnEvent', function() calls = calls + 1 end)
api.SetRecipeItemNameFilter('rough sharpening')
assert(api.GetRecipeItemNameFilter() == 'rough sharpening')
local filtered = api.GetFilteredRecipeIDs()
assert(#filtered == 1 and filtered[1] == 2660, 'concrete output recipe search')
assert(calls == 1, 'filter changes publish list update')
api.SetRecipeItemNameFilter('ROUGH SHARPENING')
assert(api.GetFilteredRecipeIDs()[1] == 2660, 'case-insensitive name match')
api.SetRecipeItemNameFilter('does not name any recipe')
assert(#api.GetFilteredRecipeIDs() == 0, 'no match must not return all recipes')
api.SetRecipeItemNameFilter(nil)
assert(api.GetRecipeItemNameFilter() == '', 'nil clears search from cached UI')
local reset = api.GetFilteredRecipeIDs()
assert(#reset == #all)
for index, recipeID in ipairs(all) do assert(reset[index] == recipeID) end
assert(not pcall(api.SetRecipeItemNameFilter, {}), 'invalid input must fail')
assert(api.GetRecipeItemNameFilter() == '', 'invalid input leaves filter unchanged')
listener:UnregisterAllEvents()
"#;

pub(crate) const MOUNT_RETIREMENT_ASSERTIONS: &str = r#"
for _, name in ipairs({'GetMountInfo', 'GetMountInfoExtra'}) do
    assert(rawget(C_MountJournal, name) == nil)
    assert(C_MountJournal[name] == nil, 'retired lookup must not synthesize a function')
    assert(C_MountJournal[name] == nil, 'repeat lookup stays absent')
end
assert(type(C_MountJournal.GetMountInfoByID) == 'function')
assert(type(C_MountJournal.GetMountInfoExtraByID) == 'function')
assert(type(C_MountJournal.Summon) == 'function', 'live legacy caller retained')
"#;

#[test]
fn patch_7_0_3_unused_mount_members_stay_absent() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(MOUNT_RETIREMENT_ASSERTIONS).unwrap();
}

prefork_full_ui_case! {
fn patch_7_0_3_cached_unused_mount_members_stay_absent(env: &WowLuaEnv) {
    env.exec(MOUNT_RETIREMENT_ASSERTIONS).unwrap();
}
}

#[test]
fn patch_7_0_3_recipe_name_filter_changes_catalog_results() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(RECIPE_FILTER_ASSERTIONS).unwrap();
}

prefork_full_ui_case! {
fn patch_7_0_3_cached_recipe_name_filter_changes_catalog_results(env: &WowLuaEnv) {
    env.exec(RECIPE_FILTER_ASSERTIONS).unwrap();
}
}
