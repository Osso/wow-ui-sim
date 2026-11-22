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
],"hotItem":{"marketID":703,"itemID":19021,"name":"Hot Deal Supplied","texture":"Interface\\Icons\\INV_Misc_QuestionMark","quantity":1,"itemType":"Misc","usable":true,"level":1,"levelType":"item level","sellerName":"Hot Seller","minBid":3000,"minIncrement":300,"currBid":0,"youHaveHighBid":false,"numBids":0,"timeLeft":2,"link":"item:19021","quality":2},"viewOnly":false}"#;

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
    assert_eq!(hot_name, "Hot Deal Supplied");
    let bid_state: (f64, bool, f64, bool, f64) = env
        .eval("C_BlackMarket.ItemPlaceBid(701, 1200); local _,_,_,_,_,_,_,_,_,_,bid,high,bids=C_BlackMarket.GetItemInfoByID(701); return C_BlackMarket.GetNumItems(),C_BlackMarket.IsViewOnly(),bid,high,bids")
        .expect("Black Market state should reflect the supplied bid");
    assert_eq!(bid_state, (2.0, false, 1200.0, true, 1.0));
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
