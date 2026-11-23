#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::paths::default_blizzard_ui_addons_path;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;

const TWO_LISTINGS_STATE: &str = r#"{"items":[
    {"marketID":701,"itemID":19019,"name":"Supplied Relic","texture":"Interface\\Icons\\INV_Misc_Gem_01","quantity":1,"itemType":"Armor","usable":true,"level":80,"levelType":"item level","sellerName":"Seller One","minBid":1000,"minIncrement":100,"currBid":0,"youHaveHighBid":false,"numBids":0,"timeLeft":4,"link":"item:19019","quality":4},
    {"marketID":702,"itemID":19020,"name":"Supplied Blade","texture":"Interface\\Icons\\INV_Sword_04","quantity":2,"itemType":"Weapon","usable":true,"level":90,"levelType":"item level","sellerName":"Seller Two","minBid":2000,"minIncrement":200,"currBid":1500,"youHaveHighBid":false,"numBids":1,"timeLeft":3,"link":"item:19020","quality":3}
],"hotItem":{"marketID":701,"itemID":19019,"name":"Supplied Relic","texture":"Interface\\Icons\\INV_Misc_Gem_01","quantity":1,"itemType":"Armor","usable":true,"level":80,"levelType":"item level","sellerName":"Seller One","minBid":1000,"minIncrement":100,"currBid":0,"youHaveHighBid":false,"numBids":0,"timeLeft":4,"link":"item:19019","quality":4},"viewOnly":false}"#;

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
fn blizzard_black_market_displays_supplied_listings_and_hot_deal() {
    let env = load_game_ui_with_black_market();
    open_black_market(&env, TWO_LISTINGS_STATE);
    let (rows, hot_name): (String, String) = env
        .eval("local n = {}; BlackMarketFrame.ScrollBox:ForEachFrame(function(b) n[#n + 1] = b.Name:GetText() end); return table.concat(n, '|'), BlackMarketFrame.HotDeal.Name:GetText()")
        .expect("Blizzard Black Market rows should be readable");
    assert_eq!(rows, "Supplied Relic|Supplied Blade");
    assert_eq!(hot_name, "Supplied Relic");
    let bid_state: (bool, bool, String, f64, bool, bool, bool) = env
        .eval(
            r#"
                A_Admin.SetMoney(50000)
                local row
                BlackMarketFrame.ScrollBox:ForEachFrame(function(button)
                    if button.marketID == 701 then row = button end
                end)
                assert(row, "selected listing should be a Blizzard row")
                row:Click("LeftButton")
                MoneyInputFrame_SetCopper(BlackMarketBidPrice, 999)
                BlackMarketFrame.BidButton:Click()
                StaticPopup1Button1:Click()
                local _, _, _, _, _, _, _, _, _, _, rejectedBid, rejectedHighBid, rejectedBids =
                    C_BlackMarket.GetItemInfoByID(701)
                MoneyInputFrame_SetCopper(BlackMarketBidPrice, 20000)
                BlackMarketFrame.BidButton:Click()
                StaticPopup1Button1:Click()
                local updatedRow
                BlackMarketFrame.ScrollBox:ForEachFrame(function(button)
                    if button.marketID == 701 then updatedRow = button end
                end)
                local _, _, _, _, _, _, _, _, _, _, currentBid, highBid =
                    C_BlackMarket.GetItemInfoByID(701)
                local invalidIndex = C_BlackMarket.GetItemInfoByIndex(0)
                return rejectedBid == 0 and not rejectedHighBid and rejectedBids == 0,
                    updatedRow.YourBid:IsShown(), updatedRow.CurrentBid.GoldButton.Text:GetText(),
                    currentBid, highBid,
                    BlackMarketFrame.HotDeal.BlackMarketHotItemBidPrice.YourBid:IsShown(),
                    invalidIndex == nil
            "#,
        )
        .expect("Blizzard bid confirmation should update the listing and hot deal");
    assert_eq!(
        bid_state,
        (true, true, "2".to_string(), 20000.0, true, true, true)
    );
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
