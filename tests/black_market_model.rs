#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::paths::default_blizzard_ui_addons_path;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;

const TWO_LISTINGS_STATE: &str = r#"{"items":[
    {"marketID":701,"itemID":19019,"name":"Supplied Relic","texture":"Interface\\Icons\\INV_Misc_Gem_01","quantity":1,"itemType":"Armor","usable":true,"level":80,"levelType":"item level","sellerName":"Seller One","minBid":10000,"minIncrement":10000,"currBid":20000,"youHaveHighBid":false,"numBids":1,"timeLeft":4,"link":"item:19019","quality":4},
    {"marketID":702,"itemID":19020,"name":"Supplied Blade","texture":"Interface\\Icons\\INV_Sword_04","quantity":2,"itemType":"Weapon","usable":true,"level":90,"levelType":"item level","sellerName":"Seller Two","minBid":30000,"minIncrement":5000,"currBid":10000,"youHaveHighBid":false,"numBids":1,"timeLeft":3,"link":"item:19020","quality":3}
],"hotItem":{"marketID":703,"itemID":19021,"name":"Hot Deal Supplied","texture":"Interface\\Icons\\INV_Misc_QuestionMark","quantity":1,"itemType":"Misc","usable":true,"level":1,"levelType":"item level","sellerName":"Hot Seller","minBid":3000,"minIncrement":300,"currBid":0,"youHaveHighBid":false,"numBids":0,"timeLeft":2,"link":"item:19021","quality":2},"viewOnly":false}"#;

const DUPLICATE_HOT_ITEM_STATE: &str = r#"{"items":[{"marketID":711,"itemID":19022,"name":"Shared Relic","texture":"Interface\\Icons\\INV_Misc_Gem_01","quantity":1,"itemType":"Armor","usable":true,"level":80,"levelType":"item level","sellerName":"Shared Seller","minBid":20000,"minIncrement":10000,"currBid":10000,"youHaveHighBid":false,"numBids":1,"timeLeft":4,"link":"item:19022","quality":4}],"hotItem":{"marketID":711,"itemID":19022,"name":"Shared Relic","texture":"Interface\\Icons\\INV_Misc_Gem_01","quantity":1,"itemType":"Armor","usable":true,"level":80,"levelType":"item level","sellerName":"Shared Seller","minBid":20000,"minIncrement":10000,"currBid":10000,"youHaveHighBid":false,"numBids":1,"timeLeft":4,"link":"item:19022","quality":4},"viewOnly":false}"#;

fn blizzard_ui_dir() -> PathBuf {
    default_blizzard_ui_addons_path().expect("Blizzard UI cache should be available")
}

fn load_game_ui(env: &WowLuaEnv) {
    wow_ui_sim::xml::register_intrinsic_templates();
    let ui = blizzard_ui_dir();
    for (name, toc) in discover_blizzard_addons_for_screen(&ui, ScreenKind::Game) {
        load_addon(&env.loader_env(), &toc)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }
    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(env, ScreenKind::Game);
}

fn load_black_market_ui(env: &WowLuaEnv) {
    let toc = blizzard_ui_dir().join("Blizzard_BlackMarketUI/Blizzard_BlackMarketUI.toc");
    load_addon(&env.loader_env(), &toc).expect("BlackMarketUI should load on demand");
}

fn load_game_ui_with_black_market() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    load_game_ui(&env);
    load_black_market_ui(&env);
    env
}

fn open_black_market(env: &WowLuaEnv, state_json: &str) {
    let code = format!("A_Admin.SetBlackMarketState([[{state_json}]])\nBlackMarketFrame_Show()");
    env.exec(&code)
        .expect("supplied Black Market data should open the Blizzard panel");
}

#[test]
fn blizzard_black_market_bid_results_enforce_minimums_and_refresh_rows() {
    let env = load_game_ui_with_black_market();
    open_black_market(&env, TWO_LISTINGS_STATE);
    let (rows, hot_name): (String, String) = env
        .eval("local n = {}; BlackMarketFrame.ScrollBox:ForEachFrame(function(b) n[#n + 1] = b.Name:GetText() end); return table.concat(n, '|'), BlackMarketFrame.HotDeal.Name:GetText()")
        .expect("Blizzard Black Market rows should be readable");
    assert_eq!(rows, "Supplied Relic|Supplied Blade");
    assert_eq!(hot_name, "Hot Deal Supplied");
    let bid_state: (bool, bool, String, f64, bool, f64, bool, bool, String, bool) = env
        .eval(
            r#"
                local function submitBid(marketID, amount)
                    C_BlackMarket.ItemPlaceBid(marketID, amount)
                end
                submitBid(701, 20000)
                local _, _, _, _, _, _, _, _, _, _, rejectedIncrementBid,
                    rejectedIncrementHigh, rejectedIncrementCount = C_BlackMarket.GetItemInfoByID(701)
                submitBid(701, 40000)
                local _, _, _, _, _, _, _, _, _, _, currentBid, highBid, bidCount =
                    C_BlackMarket.GetItemInfoByID(701)
                local row701
                BlackMarketFrame.ScrollBox:ForEachFrame(function(button)
                    if button.marketID == 701 then row701 = button end
                end)
                submitBid(702, 20000)
                local _, _, _, _, _, _, _, _, _, _, rejectedMinimumBid,
                    rejectedMinimumHigh, rejectedMinimumCount = C_BlackMarket.GetItemInfoByID(702)
                local row702
                BlackMarketFrame.ScrollBox:ForEachFrame(function(button)
                    if button.marketID == 702 then row702 = button end
                end)
                return rejectedIncrementBid == 20000 and not rejectedIncrementHigh
                        and rejectedIncrementCount == 1,
                    row701.YourBid:IsShown(), row701.CurrentBid.GoldButton.Text:GetText(),
                    currentBid, highBid, bidCount,
                    rejectedMinimumBid == 10000 and not rejectedMinimumHigh
                        and rejectedMinimumCount == 1,
                    row702.YourBid:IsShown(), row702.CurrentBid.GoldButton.Text:GetText(),
                    C_BlackMarket.GetItemInfoByIndex(0) == nil
            "#,
        )
        .expect("Blizzard bid confirmation should enforce minimums and refresh rows");
    assert_eq!(
        bid_state,
        (
            true,
            true,
            "4".to_string(),
            40000.0,
            true,
            2.0,
            true,
            false,
            "1".to_string(),
            true
        )
    );
}

#[test]
fn setting_black_market_bid_enables_bid_button_after_selecting_listing() {
    let env = load_game_ui_with_black_market();
    open_black_market(&env, TWO_LISTINGS_STATE);
    env.exec(
        r#"
            A_Admin.SetMoney(1000000)
            BlackMarketTextChanges = {}
            BlackMarketBidPrice.gold:HookScript('OnTextChanged', function(_, userInput)
                table.insert(BlackMarketTextChanges, userInput)
            end)
            BlackMarketFrame.ScrollBox:ForEachFrame(function(button)
                if button.marketID == 701 then button:Click() end
            end)
        "#,
    )
    .expect("selecting a listing should run the original Black Market handler");
    env.fire_on_update(0.016).unwrap();
    let first_change: (f64, bool, bool) = env
        .eval(
            "return #BlackMarketTextChanges, BlackMarketTextChanges[1], BlackMarketFrame.BidButton:IsEnabled()",
        )
        .expect("the first Black Market bid amount should notify the EditBox consumer");
    assert_eq!(first_change, (1.0, false, true));

    env.exec("MoneyInputFrame_SetCopper(BlackMarketBidPrice, 40000)")
        .expect("setting a second bid amount should run Blizzard handlers");
    env.fire_on_update(0.016).unwrap();
    let second_change: (f64, bool, bool) = env
        .eval(
            "return #BlackMarketTextChanges, BlackMarketTextChanges[2], BlackMarketFrame.BidButton:IsEnabled()",
        )
        .expect("the second Black Market bid amount should notify the EditBox consumer");
    assert_eq!(second_change, (2.0, false, true));

    env.exec("BlackMarketBidPrice.gold:SetNumber(4)")
        .expect("setting the current number again should remain valid");
    env.fire_on_update(0.016).unwrap();
    let unchanged_change_count: f64 = env
        .eval("return #BlackMarketTextChanges")
        .expect("the unchanged number callback count should be readable");
    assert_eq!(unchanged_change_count, 2.0);
}

#[test]
fn fresh_money_input_frame_set_copper_dispatches_after_return() {
    let env = WowLuaEnv::new().expect("simulator should initialize");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    load_game_ui(&env);
    env.fire_on_update(0.016).unwrap();
    env.exec(
        r#"
            FreshMoneyInputFrame = CreateFrame("Frame", nil, UIParent, "MoneyInputFrameTemplate")
            FreshMoneyInputFrame.changes = {}
            FreshMoneyInputFrame.callReturned = false
            FreshMoneyInputFrame.gold:HookScript("OnTextChanged", function()
                table.insert(FreshMoneyInputFrame.changes, FreshMoneyInputFrame.callReturned)
            end)
            MoneyInputFrame_SetCopper(FreshMoneyInputFrame, 40000)
            FreshMoneyInputFrame.callReturned = true
        "#,
    )
    .expect("SetCopper on a new MoneyInputFrame should return without an error");
    let before_update: (f64, f64, bool) = env
        .eval(
            "return #FreshMoneyInputFrame.changes, FreshMoneyInputFrame.expectChanges, FreshMoneyInputFrame.callReturned",
        )
        .expect("the MoneyInputFrame should retain the caller's expected-change count");
    assert_eq!(before_update, (0.0, 3.0, true));

    env.fire_on_update(0.016).unwrap();
    let after_update: (f64, bool, bool) = env
        .eval(
            "return #FreshMoneyInputFrame.changes, FreshMoneyInputFrame.changes[1], FreshMoneyInputFrame.expectChanges == nil",
        )
        .expect("the queued text handler should run after SetCopper returns");
    assert_eq!(after_update, (1.0, true, true));
}

#[test]
fn blizzard_black_market_hot_item_can_match_listing_market_id() {
    let env = load_game_ui_with_black_market();
    open_black_market(&env, DUPLICATE_HOT_ITEM_STATE);
    let copies: (bool, bool, bool) = env
        .eval(
            r#"
                C_BlackMarket.ItemPlaceBid(711, 30000)
                local _, _, _, _, _, _, _, _, _, _, listingBid, listingHigh, listingCount,
                    _, _, listingMarketID = C_BlackMarket.GetItemInfoByID(711)
                local _, _, _, _, _, _, _, _, _, _, hotBid, hotHigh, hotCount,
                    _, _, hotMarketID = C_BlackMarket.GetHotItem()
                local row
                BlackMarketFrame.ScrollBox:ForEachFrame(function(button)
                    if button.marketID == 711 then row = button end
                end)
                return listingMarketID == hotMarketID and listingBid == hotBid
                    and listingHigh == hotHigh and listingCount == hotCount,
                    row.YourBid:IsShown(),
                    BlackMarketFrame.HotDeal.BlackMarketHotItemBidPrice.YourBid:IsShown()
            "#,
        )
        .expect("listing and hot-item copies should stay synchronized after bidding");
    assert_eq!(copies, (true, true, true));
}

#[test]
fn blizzard_black_market_view_only_hides_bid_controls() {
    let env = load_game_ui_with_black_market();
    open_black_market(&env, r#"{"items":[],"hotItem":null,"viewOnly":true}"#);
    let controls: (bool, bool) = env
        .eval("return C_BlackMarket.IsViewOnly(), BlackMarketFrame.BidButton:IsShown()")
        .expect("view-only state and bid control should be queryable");
    assert_eq!(controls, (true, false));
}
