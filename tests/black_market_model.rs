#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::paths::default_blizzard_ui_addons_path;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;

const TWO_LISTINGS_STATE: &str = r#"{"items":[
    {"marketID":701,"itemID":19019,"name":"Supplied Relic","texture":"Interface\\Icons\\INV_Misc_Gem_01","quantity":1,"itemType":"Armor","usable":true,"level":80,"levelType":"item level","sellerName":"Seller One","minBid":10000,"minIncrement":10000,"currBid":20000,"youHaveHighBid":false,"numBids":1,"timeLeft":4,"link":"item:19019","quality":4},
    {"marketID":702,"itemID":19020,"name":"Supplied Blade","texture":"Interface\\Icons\\INV_Sword_04","quantity":2,"itemType":"Weapon","usable":true,"level":90,"levelType":"item level","sellerName":"Seller Two","minBid":30000,"minIncrement":5000,"currBid":10000,"youHaveHighBid":false,"numBids":1,"timeLeft":3,"link":"item:19020","quality":3},
    {"marketID":703,"itemID":19021,"name":"Expired Artifact","texture":"Interface\\Icons\\INV_Misc_QuestionMark","quantity":1,"itemType":"Misc","usable":true,"level":1,"levelType":"item level","sellerName":"Seller Three","minBid":1000,"minIncrement":100,"currBid":500,"youHaveHighBid":false,"numBids":2,"timeLeft":0,"link":"item:19021","quality":2}
],"hotItemMarketID":702,"viewOnly":false}"#;

const DUPLICATE_HOT_ITEM_STATE: &str = r#"{"items":[{"marketID":711,"itemID":19022,"name":"Shared Relic","texture":"Interface\\Icons\\INV_Misc_Gem_01","quantity":1,"itemType":"Armor","usable":true,"level":80,"levelType":"item level","sellerName":"Shared Seller","minBid":20000,"minIncrement":10000,"currBid":10000,"youHaveHighBid":false,"numBids":1,"timeLeft":4,"link":"item:19022","quality":4}],"hotItemMarketID":711,"viewOnly":false}"#;

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
    assert_eq!(rows, "Supplied Relic|Supplied Blade|Expired Artifact");
    assert_eq!(hot_name, "Supplied Blade");
    let bid_state: (bool, bool, String, f64, bool, f64, bool, bool, String, bool, bool) = env
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
                submitBid(703, 100000)
                local _, _, _, _, _, _, _, _, _, _, expiredBid, expiredHigh, expiredCount =
                    C_BlackMarket.GetItemInfoByID(703)
                return rejectedIncrementBid == 20000 and not rejectedIncrementHigh
                        and rejectedIncrementCount == 1,
                    row701.YourBid:IsShown(), row701.CurrentBid.GoldButton.Text:GetText(),
                    currentBid, highBid, bidCount,
                    rejectedMinimumBid == 10000 and not rejectedMinimumHigh
                        and rejectedMinimumCount == 1,
                    row702.YourBid:IsShown(), row702.CurrentBid.GoldButton.Text:GetText(),
                    C_BlackMarket.GetItemInfoByIndex(0) == nil,
                    expiredBid == 500 and not expiredHigh and expiredCount == 2
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
            true,
            true
        )
    );
}

#[test]
fn black_market_queries_return_no_values_without_interaction() {
    let env = load_game_ui_with_black_market();
    let closed: (i64, i64, i64, i64, i64, i64) = env
        .eval("return select('#', C_BlackMarket.GetNumItems()), select('#', C_BlackMarket.IsViewOnly()), select('#', C_BlackMarket.GetHotItem()), select('#', C_BlackMarket.GetItemInfoByID(701)), select('#', C_BlackMarket.GetItemInfoByIndex(1)), select('#', C_BlackMarket.RequestItems())")
        .unwrap();
    assert_eq!(closed, (0, 0, 0, 0, 0, 0));

    open_black_market(&env, TWO_LISTINGS_STATE);
    let open: (i64, i64, i64, i64, i64) = env
        .eval("return select('#', C_BlackMarket.GetNumItems()), select('#', C_BlackMarket.IsViewOnly()), select('#', C_BlackMarket.GetHotItem()), select('#', C_BlackMarket.GetItemInfoByID(701)), select('#', C_BlackMarket.GetItemInfoByIndex(2))")
        .unwrap();
    assert_eq!(open, (1, 1, 17, 17, 17));

    let close_values: i64 = env
        .eval("return select('#', C_BlackMarket.Close())")
        .unwrap();
    assert_eq!(close_values, 0);
    let closed_again: (i64, i64) = env
        .eval("return select('#', C_BlackMarket.GetNumItems()), select('#', C_BlackMarket.GetHotItem())")
        .unwrap();
    assert_eq!(closed_again, (0, 0));
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
    open_black_market(
        &env,
        r#"{"items":[],"hotItemMarketID":null,"viewOnly":true}"#,
    );
    let controls: (bool, bool) = env
        .eval("return C_BlackMarket.IsViewOnly(), BlackMarketFrame.BidButton:IsShown()")
        .expect("view-only state and bid control should be queryable");
    assert_eq!(controls, (true, false));
}
