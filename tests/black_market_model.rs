#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn blizzard_black_market_displays_supplied_listings_and_hot_deal(env: &WowLuaEnv) {
    env.exec(r#"
        A_Admin.SetBlackMarketState([[{"items":[
            {"marketID":701,"itemID":19019,"name":"Supplied Relic","texture":"Interface\\Icons\\INV_Misc_Gem_01","quantity":1,"itemType":"Armor","usable":true,"level":80,"levelType":"item level","sellerName":"Seller One","minBid":1000,"minIncrement":100,"currBid":0,"youHaveHighBid":false,"numBids":0,"timeLeft":4,"link":"item:19019","quality":4},
            {"marketID":702,"itemID":19020,"name":"Supplied Blade","texture":"Interface\\Icons\\INV_Sword_04","quantity":2,"itemType":"Weapon","usable":true,"level":90,"levelType":"item level","sellerName":"Seller Two","minBid":2000,"minIncrement":200,"currBid":1500,"youHaveHighBid":false,"numBids":1,"timeLeft":3,"link":"item:19020","quality":3}
        ],"hotItem":{"marketID":703,"itemID":19021,"name":"Hot Deal Supplied","texture":"Interface\\Icons\\INV_Misc_QuestionMark","quantity":1,"itemType":"Misc","usable":true,"level":1,"levelType":"item level","sellerName":"Hot Seller","minBid":3000,"minIncrement":300,"currBid":0,"youHaveHighBid":false,"numBids":0,"timeLeft":2,"link":"item:19021","quality":2},"viewOnly":false}]])
        BlackMarket_LoadUI()
        ShowUIPanel(BlackMarketFrame)
    "#).expect("supplied Black Market data should open the Blizzard panel");

    let (rows, hot_name): (String, String) = env.eval(r#"
        local names = {}
        BlackMarketFrame.ScrollBox:ForEachFrame(function(button)
            names[#names + 1] = button.Name:GetText()
        end)
        return table.concat(names, "|"), BlackMarketFrame.HotDeal.Name:GetText()
    "#).expect("Blizzard Black Market rows should be readable");
    assert_eq!(rows, "Supplied Relic|Supplied Blade");
    assert_eq!(hot_name, "Hot Deal Supplied");
}
}
