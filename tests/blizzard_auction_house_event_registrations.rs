//! Event-registration coverage for `Blizzard_AuctionHouseUI`.

use crate::common;

use common::panel_fixtures::{clear_recorded_lua_errors, recorded_lua_errors};

const ROOT: &str = "Blizzard_AuctionHouseUI";
const EVENT_REGISTRATIONS_PROBE_LUA: &str = r#"
local failures = {}

local function expect(condition, message)
    if not condition then
        table.insert(failures, message)
    end
end

local requiredFrameEvents = {
    "ITEM_SEARCH_RESULTS_UPDATED",
    "COMMODITY_SEARCH_RESULTS_UPDATED",
    "OWNED_AUCTIONS_UPDATED",
    "BIDS_UPDATED",
    "AUCTION_HOUSE_POST_WARNING",
    "AUCTION_HOUSE_POST_ERROR",
    "PLAYER_MONEY",
}

AuctionHouseFrame:Show()
local browseResults = AuctionHouseFrame.BrowseResultsFrame
local itemBuy = AuctionHouseFrame.ItemBuyFrame
browseResults:Show()
itemBuy:Show()

for _, eventName in ipairs(requiredFrameEvents) do
    expect(AuctionHouseFrame:IsEventRegistered(eventName),
           "AuctionHouseFrame must register " .. eventName .. " after Show")
end

local browseEvent = "AUCTION_HOUSE_BROWSE_RESULTS_UPDATED"
expect(not AuctionHouseFrame:IsEventRegistered(browseEvent),
       "AuctionHouseFrame must not own browse-results updates")
expect(browseResults:IsEventRegistered(browseEvent),
       "BrowseResultsFrame must register browse-results updates on load")
expect(itemBuy:IsEventRegistered(browseEvent),
       "ItemBuyFrame must register browse-results updates while shown")

itemBuy:Hide()
expect(not itemBuy:IsEventRegistered(browseEvent),
       "ItemBuyFrame must unregister browse-results updates when hidden")

return table.concat(failures, "\n")
"#;

#[test]
#[cfg(all(feature = "optional-tests", feature = "client-retail"))]
fn auction_house_browse_updates_are_registered_by_the_subframes() {
    common::with_perf_lock(|| {
        common::with_timeout(240, || {
            let env = common::panel_fixtures::setup_env();
            load_auction_house_through_runtime(&env);
            load_show_prerequisites(&env);
            clear_recorded_lua_errors(&env);

            let failures: String = env
                .eval(EVENT_REGISTRATIONS_PROBE_LUA)
                .expect("AuctionHouseFrame event-registration probe should run");
            assert!(
                failures.is_empty(),
                "`{ROOT}` missing event registrations:\n{failures}"
            );

            let errors = recorded_lua_errors(&env);
            assert!(
                errors.is_empty(),
                "`{ROOT}` event-registration probe emitted Lua errors:\n{}",
                errors.join("\n")
            );
        });
    });
}

fn load_show_prerequisites(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.exec(
        r#"
        local tokenLoaded, tokenReason = C_AddOns.LoadAddOn("Blizzard_TokenUI")
        assert(tokenLoaded, "Blizzard_TokenUI should load: " .. tostring(tokenReason))
        if ContainerFrameSettingsManager and not ContainerFrameSettingsManager.TokenTracker then
            ContainerFrameSettingsManager:OnAddonLoaded("Blizzard_TokenUI")
        end

        local wowTokenLoaded, wowTokenReason = C_AddOns.LoadAddOn("Blizzard_WowTokenUI")
        assert(wowTokenLoaded, "Blizzard_WowTokenUI should load: " .. tostring(wowTokenReason))

        if type(WowToken_IsWowTokenAuctionDialogShown) ~= "function" then
            function WowToken_IsWowTokenAuctionDialogShown()
                return false
            end
        end
        "#,
    )
    .expect("show prerequisite addons should load");
    env.apply_post_load_workarounds();
}

fn load_auction_house_through_runtime(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let (loaded, reason): (bool, Option<String>) = env
        .eval(r#"return C_AddOns.LoadAddOn("Blizzard_AuctionHouseUI")"#)
        .expect("C_AddOns.LoadAddOn should return");
    assert!(loaded, "`{ROOT}` should load: {reason:?}");
}
