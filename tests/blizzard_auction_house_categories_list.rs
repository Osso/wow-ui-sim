//! Categories-list coverage for `Blizzard_AuctionHouseUI`.

use crate::common;

use common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use common::panel_fixtures::{clear_recorded_lua_errors, recorded_lua_errors};

const ROOT: &str = "Blizzard_AuctionHouseUI";
const CATEGORIES_LIST_PROBE_LUA: &str = r#"
local failures = {}

local function expect(condition, message)
    if not condition then
        table.insert(failures, message)
    end
end

local categoriesList = AuctionHouseFrame and AuctionHouseFrame.CategoriesList
expect(categoriesList ~= nil, "CategoriesList must exist")

if categoriesList then
    expect(type(categoriesList.GetNumElementsForRefresh) == "function",
           "GetNumElementsForRefresh must be a function")

    if type(categoriesList.GetNumElementsForRefresh) == "function" then
        local expectedCategoryCount = #AuctionCategories
        local actualCategoryCount = categoriesList:GetNumElementsForRefresh()
        expect(actualCategoryCount == expectedCategoryCount,
               "category count " .. tostring(actualCategoryCount) ..
               " did not match AuctionCategories count " .. tostring(expectedCategoryCount))
    end

    categoriesList:OnShow()
    local provider = categoriesList.ScrollBox:GetDataProvider()
    expect(provider ~= nil, "categories ScrollBox data provider must exist")
    if provider then
        expect(provider:GetSize() == #AuctionCategories,
               "provider size " .. tostring(provider:GetSize()) ..
               " did not match AuctionCategories count " .. tostring(#AuctionCategories))
    end
end

return table.concat(failures, "\n")
"#;

#[test]
fn auction_house_categories_list_refresh_count_matches_categories() {
    common::with_perf_lock(|| {
        common::with_timeout(240, || {
            with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
                clear_recorded_lua_errors(env);

                let (loaded, reason): (bool, Option<String>) = env
                    .eval(r#"return C_AddOns.LoadAddOn("Blizzard_AuctionHouseUI")"#)
                    .expect("C_AddOns.LoadAddOn should return");
                assert!(loaded, "`{ROOT}` should load: {reason:?}");

                #[cfg(feature = "client-ptr")]
                {
                    let should_display: bool = env
                        .eval("return C_WowTokenUI.ShouldDisplayInAuctionHouse()")
                        .expect("PTR token display query should return a boolean");
                    assert!(should_display, "PTR should display the WoW Token category");
                }

                let failures: String = env
                    .eval(CATEGORIES_LIST_PROBE_LUA)
                    .expect("AuctionHouse categories-list probe should run");
                assert!(
                    failures.is_empty(),
                    "`{ROOT}` categories-list failures:\n{failures}"
                );

                let errors = recorded_lua_errors(env);
                assert!(
                    errors.is_empty(),
                    "`{ROOT}` categories-list probe emitted Lua errors:\n{}",
                    errors.join("\n")
                );
            });
        });
    });
}
