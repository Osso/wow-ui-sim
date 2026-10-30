//! Focused Auction House panel load/show coverage.

use crate::common;

use wow_ui_sim::loader::load_addon;

#[test]
fn auction_house_panel_loads_and_shows_seeded_browse_results() {
    test_timeout! {
        let env = common::panel_fixtures::setup_env();
        let ui = common::panel_fixtures::blizzard_ui_dir();
        let toc_path = ui.join("Blizzard_AuctionHouseUI/Blizzard_AuctionHouseUI.toc");
        load_addon(&env.loader_env(), &toc_path)
            .unwrap_or_else(|err| panic!("failed to load Blizzard_AuctionHouseUI: {err}"));

        let result: String = env.eval(
            r#"
            A_Admin.ClearAuctionBrowseResults()
            A_Admin.AddAuctionBrowseResult(210935, 70, 25000, 400, false)

            if not AuctionHouseFrame then
                return "missing_frame"
            end

            ShowUIPanel(AuctionHouseFrame)
            if not AuctionHouseFrame:IsShown() then
                return "frame_hidden"
            end

            AuctionHouseFrame.BrowseResultsFrame:UpdateBrowseResults()

            local rows = AuctionHouseFrame.BrowseResultsFrame.browseResults
            if type(rows) ~= "table" then
                return "missing_rows"
            end
            if #rows ~= 1 then
                return "row_count=" .. tostring(#rows)
            end
            if rows[1].itemKey.itemID ~= 210935 then
                return "row_item=" .. tostring(rows[1].itemKey and rows[1].itemKey.itemID)
            end

            return "ok"
            "#,
        ).unwrap();

        assert_eq!(
            result,
            "ok",
            "Auction House panel should load, show, and expose the seeded browse row: {result}"
        );
    }
}

#[test]
fn auction_house_auctions_tabs_show_seeded_owned_and_bid_groups() {
    test_timeout! {
        let env = common::panel_fixtures::setup_env();
        let ui = common::panel_fixtures::blizzard_ui_dir();
        // Retail manifest pins the unqualified TOC.
        let toc_path = ui.join("Blizzard_AuctionHouseUI/Blizzard_AuctionHouseUI.toc");
        load_addon(&env.loader_env(), &toc_path)
            .unwrap_or_else(|err| panic!("failed to load Blizzard_AuctionHouseUI: {err}"));

        let result: String = env.eval(
            r#"
            A_Admin.ClearOwnedAuctions()
            A_Admin.ClearAuctionBids()
            local playerGuid = UnitGUID("player")

            A_Admin.AddOwnedAuction(101, 210935, 70, 200, 0, 50000, 0, 4, 86400)
            A_Admin.AddOwnedAuction(102, 210935, 70, 50, 12500, 25000, 0, 3, 7200)
            A_Admin.AddOwnedAuction(103, 224072, 80, 1, 9000000, 9900000, 0, 2, 1800)

            A_Admin.AddAuctionBid(201, 210935, 70, 25, 41000, 50000, 4, 86400, playerGuid)
            A_Admin.AddAuctionBid(202, 224072, 80, 1, 9000000, 9900000, 2, 1800, nil)

            ShowUIPanel(AuctionHouseFrame)
            if not AuctionHouseFrame:IsShown() then
                return "frame_hidden"
            end

            AuctionHouseFrame:SetDisplayMode(AuctionHouseFrameDisplayMode.Auctions)
            local auctionsFrame = AuctionHouseFrame.AuctionsFrame
            if not auctionsFrame:IsShown() then
                return "auctions_frame_hidden"
            end

            auctionsFrame:SetTab(1)
            A_Admin.FireEvent("OWNED_AUCTIONS_UPDATED")
            local ownedProvider = auctionsFrame.SummaryList.ScrollBox:GetDataProvider()
            if ownedProvider:GetSize() ~= 3 then
                return "owned_provider_size=" .. tostring(ownedProvider:GetSize())
            end
            local ownedType = C_AuctionHouse.GetOwnedAuctionType(1)
            if ownedType.itemID ~= 210935 then
                return "owned_first_type=" .. tostring(ownedType.itemID)
            end
            if AuctionHouseFrame:GetMaxBidPriceWidthForAllAuctions(PriceFontWhite) <= 0 then
                return "owned_bid_width"
            end
            if AuctionHouseFrame:GetMaxBuyoutPriceWidthForAllAuctions(PriceFontWhite) <= 0 then
                return "owned_buyout_width"
            end

            auctionsFrame:SetTab(2)
            A_Admin.FireEvent("BIDS_UPDATED")
            local bidProvider = auctionsFrame.SummaryList.ScrollBox:GetDataProvider()
            if bidProvider:GetSize() ~= 3 then
                return "bid_provider_size=" .. tostring(bidProvider:GetSize())
            end
            local bidType = C_AuctionHouse.GetBidType(2)
            if bidType.itemID ~= 224072 then
                return "bid_second_type=" .. tostring(bidType.itemID)
            end
            local firstBid = C_AuctionHouse.GetBidInfo(1)
            if AuctionHouseFrame:GetBidStatus(firstBid) ~= AuctionHouseBidStatus.PlayerBid then
                return "first_bid_status=" .. tostring(AuctionHouseFrame:GetBidStatus(firstBid))
            end
            if AuctionHouseFrame:GetMaxBidPriceWidthForAllBids(PriceFontWhite) <= 0 then
                return "bid_width"
            end
            if AuctionHouseFrame:GetMaxBuyoutPriceWidthForAllBids(PriceFontWhite) <= 0 then
                return "bid_buyout_width"
            end

            return "ok"
            "#,
        ).unwrap();

        assert_eq!(
            result,
            "ok",
            "Auction House Auctions/Bids tabs should group seeded rows and expose price helpers: {result}"
        );
    }
}
