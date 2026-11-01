use crate::common;

use common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use common::panel_fixtures::{blizzard_ui_dir, clear_recorded_lua_errors, recorded_lua_errors};
use wow_ui_sim::toc::TocFile;

const ROOT: &str = "Blizzard_AuctionHouseUI";
const SORTS_BY_SEARCH_CONTEXT: &str = "g_auctionHouseSortsBySearchContext";
const ACTIVE_BID_AUCTION_IDS: &str = "g_activeBidAuctionIDs";
const SAVED_VARIABLE_DEFAULTS_PROBE_LUA: &str = r#"
local failures = {}

local function expect(condition, message)
    if not condition then
        table.insert(failures, message)
    end
end

local function expectDefaultSort(contextSorts, firstOrder, secondOrder, contextName)
    expect(type(contextSorts) == "table", contextName .. " sorts must be a table")
    if type(contextSorts) ~= "table" then
        return
    end

    expect(#contextSorts == 2, contextName .. " must have two default sort rows")
    expect(contextSorts[1] and contextSorts[1].sortOrder == firstOrder,
           contextName .. " first sort order mismatch")
    expect(contextSorts[1] and contextSorts[1].reverseSort == false,
           contextName .. " first sort must not be reversed")
    expect(contextSorts[2] and contextSorts[2].sortOrder == secondOrder,
           contextName .. " second sort order mismatch")
    expect(contextSorts[2] and contextSorts[2].reverseSort == false,
           contextName .. " second sort must not be reversed")
end

expect(type(g_auctionHouseSortsBySearchContext) == "table",
       "g_auctionHouseSortsBySearchContext must be a table")
expect(type(g_activeBidAuctionIDs) == "table",
       "g_activeBidAuctionIDs must be a table")

local sorts = g_auctionHouseSortsBySearchContext
if type(sorts) == "table" then
    expect(sorts.auctionHouseSortVersion == 2,
           "auctionHouseSortVersion must be 2 after OnLoad")
    expectDefaultSort(sorts[AuctionHouseSearchContext.BrowseAll],
                      Enum.AuctionHouseSortOrder.Price,
                      Enum.AuctionHouseSortOrder.Name,
                      "BrowseAll")
    expectDefaultSort(sorts[AuctionHouseSearchContext.BuyItems],
                      Enum.AuctionHouseSortOrder.Buyout,
                      Enum.AuctionHouseSortOrder.Bid,
                      "BuyItems")
    expectDefaultSort(sorts[AuctionHouseSearchContext.AllAuctions],
                      Enum.AuctionHouseSortOrder.Name,
                      Enum.AuctionHouseSortOrder.Price,
                      "AllAuctions")
end

expect(#g_activeBidAuctionIDs == 0,
       "g_activeBidAuctionIDs must start as an empty array")
expect(next(g_activeBidAuctionIDs) == nil,
       "g_activeBidAuctionIDs must have no keyed entries")

return table.concat(failures, "\n")
"#;

#[test]
fn auction_house_saved_variables_exist_with_onload_defaults() {
    let toc = TocFile::from_file(&auction_house_toc()).expect("AuctionHouse TOC should parse");
    assert_eq!(
        toc.saved_variables(),
        [SORTS_BY_SEARCH_CONTEXT],
        "`{ROOT}` must declare its account-level auction-house sort saved variable"
    );
    assert_eq!(
        toc.saved_variables_per_character(),
        [ACTIVE_BID_AUCTION_IDS],
        "`{ROOT}` must declare its per-character active-bid saved variable"
    );

    common::with_perf_lock(|| {
        common::with_timeout(240, || {
            with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
                clear_recorded_lua_errors(env);

                let (loaded, reason): (bool, Option<String>) = env
                    .eval(r#"return C_AddOns.LoadAddOn("Blizzard_AuctionHouseUI")"#)
                    .expect("C_AddOns.LoadAddOn should return");
                assert!(loaded, "`{ROOT}` should load: {reason:?}");

                assert_saved_variable_defaults(env);

                let errors = recorded_lua_errors(env);
                assert!(
                    errors.is_empty(),
                    "`{ROOT}` saved-variable load emitted Lua errors:\n{}",
                    errors.join("\n")
                );
            });
        });
    });
}

fn auction_house_toc() -> std::path::PathBuf {
    wow_ui_sim::loader::find_toc_file(&blizzard_ui_dir().join(ROOT))
        .expect("Blizzard_AuctionHouseUI TOC must resolve")
}

fn assert_saved_variable_defaults(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let failures: String = env
        .eval(SAVED_VARIABLE_DEFAULTS_PROBE_LUA)
        .expect("AuctionHouse saved-variable defaults probe should run");

    assert!(
        failures.is_empty(),
        "AuctionHouse saved-variable defaults mismatch:\n{failures}"
    );
}
