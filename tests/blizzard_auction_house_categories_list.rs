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

    local totalCategoryCount = #AuctionCategories
    local expectedProviderSize = totalCategoryCount
    if C_WowTokenUI and type(C_WowTokenUI.ShouldDisplayInAuctionHouse) == "function"
       and not C_WowTokenUI.ShouldDisplayInAuctionHouse() then
        for _, categoryInfo in ipairs(AuctionCategories) do
            if categoryInfo:HasFlag("WOW_TOKEN_FLAG") then
                expectedProviderSize = expectedProviderSize - 1
            end
        end
    end

    if type(categoriesList.GetNumElementsForRefresh) == "function" then
        local refreshCount = categoriesList:GetNumElementsForRefresh()
        expect(refreshCount == totalCategoryCount,
               "refresh count " .. tostring(refreshCount) ..
               " did not match total AuctionCategories count " .. tostring(totalCategoryCount))
    end

    categoriesList:OnShow()
    local provider = categoriesList.ScrollBox:GetDataProvider()
    expect(provider ~= nil, "categories ScrollBox data provider must exist")
    if provider then
        expect(provider:GetSize() == expectedProviderSize,
               "provider size " .. tostring(provider:GetSize()) ..
               " did not match filtered AuctionCategories count " .. tostring(expectedProviderSize))
    end
end

return table.concat(failures, "\n")
"#;

#[test]
#[cfg(feature = "client-retail")]
fn retail_does_not_register_token_auction_house_visibility() {
    common::with_perf_lock(|| {
        common::with_timeout(240, || {
            with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
                let registered: bool = env
                    .eval("return C_WowTokenUI ~= nil and C_WowTokenUI.ShouldDisplayInAuctionHouse ~= nil")
                    .expect("Retail token API surface should be queryable");
                assert!(!registered);
            });
        });
    });
}

#[test]
#[cfg(any(feature = "client-ptr", feature = "client-wowforever"))]
fn auction_house_token_category_follows_native_status_policy() {
    common::with_perf_lock(|| {
        common::with_timeout(240, || {
            with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
                let (loaded, reason): (bool, Option<String>) = env
                    .eval(r#"return C_AddOns.LoadAddOn("Blizzard_AuctionHouseUI")"#)
                    .expect("C_AddOns.LoadAddOn should return");
                assert!(loaded, "`{ROOT}` should load: {reason:?}");

                let ptr_override = cfg!(feature = "client-ptr");
                let default_visibility: bool = env
                    .eval("return C_WowTokenUI.ShouldDisplayInAuctionHouse()")
                    .expect("token category visibility should be queryable");
                assert_eq!(default_visibility, ptr_override);

                env.exec(
                    r#"
                    tokenStatusEvents = 0
                    local statusProbe = CreateFrame("Frame")
                    statusProbe:RegisterEvent("TOKEN_STATUS_CHANGED")
                    statusProbe:SetScript("OnEvent", function()
                        tokenStatusEvents = tokenStatusEvents + 1
                    end)

                    local tokenCategory = {
                        name = "Token probe",
                        subCategories = {},
                        HasFlag = function(_, flag)
                            return flag == "WOW_TOKEN_FLAG"
                        end,
                    }
                    table.insert(AuctionCategories, tokenCategory)

                    local function categoryCount()
                        AuctionHouseFrame.CategoriesList:OnShow()
                        return AuctionHouseFrame.CategoriesList.ScrollBox:GetDataProvider():GetSize()
                    end

                    local tokenCategoryCount = 0
                    for _, categoryInfo in ipairs(AuctionCategories) do
                        if categoryInfo:HasFlag("WOW_TOKEN_FLAG") then
                            tokenCategoryCount = tokenCategoryCount + 1
                        end
                    end
                    tokenCategoryCountBeforeStatus = categoryCount()
                    tokenCategoryTotal = tokenCategoryCount
                    "#,
                )
                .expect("token category probe should initialize");

                let getter_event_count: i32 = env
                    .eval("return tokenStatusEvents")
                    .expect("visibility getter should not dispatch status events");
                assert_eq!(getter_event_count, 0);

                let before_status: i32 = env
                    .eval("return tokenCategoryCountBeforeStatus")
                    .expect("initial category count should be readable");
                let (total_categories, token_categories): (i32, i32) = env
                    .eval("return #AuctionCategories, tokenCategoryTotal")
                    .expect("category catalog should be readable");
                let expected_initial_count = if default_visibility {
                    total_categories
                } else {
                    total_categories - token_categories
                };
                assert_eq!(
                    before_status, expected_initial_count,
                    "the original AuctionHouse consumer should hide the token category only when the getter is false"
                );

                wow_ui_sim::c_api::c_wow_token_ui::set_feature_status(env, true)
                    .expect("token status update should fire its event");
                let visible_after_update: bool = env
                    .eval("return C_WowTokenUI.ShouldDisplayInAuctionHouse()")
                    .expect("token visibility should reflect native status policy");
                assert!(visible_after_update);

                let after_enable: i32 = env
                    .eval(
                        "AuctionHouseFrame.CategoriesList:OnShow(); return AuctionHouseFrame.CategoriesList.ScrollBox:GetDataProvider():GetSize()",
                    )
                    .expect("original AuctionHouse category consumer should refresh");
                assert_eq!(after_enable, total_categories);

                let event_count: i32 = env
                    .eval("return tokenStatusEvents")
                    .expect("TOKEN_STATUS_CHANGED listener should run synchronously");
                assert_eq!(event_count, 1);

                if !ptr_override {
                    wow_ui_sim::c_api::c_wow_token_ui::set_feature_status(env, false)
                        .expect("second token status update should fire its event");
                    let hidden_after_update: bool = env
                        .eval("return C_WowTokenUI.ShouldDisplayInAuctionHouse()")
                        .expect("token visibility should clear with status");
                    assert!(!hidden_after_update);

                    let after_disable: i32 = env
                        .eval(
                            "AuctionHouseFrame.CategoriesList:OnShow(); return AuctionHouseFrame.CategoriesList.ScrollBox:GetDataProvider():GetSize()",
                        )
                        .expect("original AuctionHouse category consumer should refresh");
                    assert_eq!(after_disable, total_categories - token_categories);
                    let event_count: i32 = env
                        .eval("return tokenStatusEvents")
                        .expect("second token status event should be observable");
                    assert_eq!(event_count, 2);
                }
            });
        });
    });
}

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
