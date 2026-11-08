#![cfg(feature = "client-mists")]

#[path = "common/lua_error_output.rs"]
mod lua_error_output;

use std::path::PathBuf;
use std::process::Command;

fn wow_sim_binary() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_wow-sim")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target")
                .join("debug")
                .join("wow-sim")
        })
}

#[test]
fn mists_auction_house_supports_browse_bid_post_and_cancel() {
    let output = Command::new("timeout")
        .arg("90")
        .arg(wow_sim_binary())
        .args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            A_Admin.ClearAuctionBrowseResults()
            A_Admin.ClearAuctionBids()
            A_Admin.ClearOwnedAuctions()
            A_Admin.ClearBags()

            A_Admin.AddAuctionBrowseResult(6948, 1, 2500, 1, false, 7001)
            A_Admin.AddBagItem(0, 1, 6948, 1)

            AuctionFrame_LoadUI()
            AuctionFrame_Show()
            if not (AuctionFrame and AuctionFrame:IsShown()) then
                error("AuctionFrame did not open")
            end

            QueryAuctionItems("", nil, nil, 0, 0, 0, 0, 0, 0)
            BrowseSearchButton:Click()
            BrowseSearchCountText:SetText("")
            AuctionFrameBrowse_Update()

            local browseCount, browseTotal = GetNumAuctionItems("list")
            if browseCount ~= 1 or browseTotal ~= 1 then
                error("browse count=" .. tostring(browseCount) .. "/" .. tostring(browseTotal))
            end

            local name, texture, count, quality, canUse, level, levelColHeader,
                minBid, minIncrement, buyoutPrice, bidAmount, highBidder,
                bidderFullName, owner, ownerFullName, saleStatus, itemID =
                GetAuctionItemInfo("list", 1)
            if name ~= "Hearthstone" then
                error("browse name=" .. tostring(name))
            end
            if itemID ~= 6948 then
                error("browse itemID=" .. tostring(itemID))
            end
            if buyoutPrice ~= 2500 then
                error("browse buyout=" .. tostring(buyoutPrice))
            end

            PlaceAuctionBid("list", 1, buyoutPrice)
            if C_AuctionHouse.GetNumBids() ~= 1 then
                error("bid count=" .. tostring(C_AuctionHouse.GetNumBids()))
            end

            PickupContainerItem(0, 1)
            ClickAuctionSellItemButton()
            local sellName, sellTexture, sellCount = GetAuctionSellItemInfo()
            if sellName ~= "Hearthstone" then
                error("sell name=" .. tostring(sellName))
            end
            if sellCount ~= 1 then
                error("sell count=" .. tostring(sellCount))
            end

            StartAuction(1000, 4000, 1, 1, 1)
            local ownedCount = GetNumAuctionItems("owner")
            if ownedCount ~= 1 then
                error("owned count after post=" .. tostring(ownedCount))
            end

            local canCancel = CanCancelAuction(1)
            if not canCancel then
                error("posted auction cannot cancel")
            end
            CancelAuction(1)
            local _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, saleStatus =
                GetAuctionItemInfo("owner", 1)
            if saleStatus ~= 1 then
                error("cancel saleStatus=" .. tostring(saleStatus))
            end
            "#,
            "lua-errors",
        ])
        .output()
        .expect("failed to run wow-sim");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "wow-sim failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    lua_error_output::assert_no_lua_errors("auction house flow", &stdout, &stderr);
}

