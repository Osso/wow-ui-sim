//! Integration tests for `src/lua_api/globals/trade_verbs.rs`.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{CursorInfo, CursorItemOrigin};

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

fn fired(env: &WowLuaEnv, name: &str) -> bool {
    env.state()
        .borrow()
        .events
        .pending()
        .iter()
        .any(|e| e.name == name)
}

// ── InitiateTrade ─────────────────────────────────────────────────────────────

#[test]
fn initiate_trade_opens_slot_and_fires_trade_show() {
    let env = env();
    env.exec(r#"InitiateTrade("Jaina")"#).unwrap();
    let st = env.state().borrow();
    let trade = st.active_trade.as_ref().expect("trade must be open");
    assert_eq!(trade.target, "Jaina");
    drop(st);
    assert!(fired(&env, "TRADE_SHOW"));
}

#[test]
fn initiate_trade_without_target_is_noop() {
    let env = env();
    env.exec(r#"InitiateTrade("")"#).unwrap();
    assert!(env.state().borrow().active_trade.is_none());
}

#[test]
fn initiate_trade_while_already_open_is_silent_noop() {
    let env = env();
    env.exec(r#"InitiateTrade("Jaina")"#).unwrap();
    // Drain the initial TRADE_SHOW observation.
    let before_events = env.state().borrow().events.pending().len();
    env.exec(r#"InitiateTrade("Thrall")"#).unwrap();
    let st = env.state().borrow();
    // First target is preserved; no new event queued.
    assert_eq!(st.active_trade.as_ref().unwrap().target, "Jaina");
    assert_eq!(st.events.pending().len(), before_events);
}

// ── AcceptTrade ───────────────────────────────────────────────────────────────

#[test]
fn accept_trade_flags_player_accepted_and_fires_accept_update() {
    let env = env();
    env.exec(r#"InitiateTrade("Jaina")"#).unwrap();
    env.exec("AcceptTrade()").unwrap();
    let st = env.state().borrow();
    let trade = st.active_trade.as_ref().expect("trade still open");
    assert!(trade.player_accepted);
    drop(st);
    assert!(fired(&env, "TRADE_ACCEPT_UPDATE"));
}

#[test]
fn accept_trade_finalizes_when_both_sides_accepted() {
    let env = env();
    env.exec(r#"InitiateTrade("Jaina")"#).unwrap();
    // Simulate opponent accept.
    env.state()
        .borrow_mut()
        .active_trade
        .as_mut()
        .unwrap()
        .target_accepted = true;
    env.exec("AcceptTrade()").unwrap();
    let st = env.state().borrow();
    assert!(st.active_trade.is_none(), "trade must finalize");
    drop(st);
    assert!(fired(&env, "TRADE_CLOSED"));
}

#[test]
fn accept_trade_without_open_trade_is_noop() {
    let env = env();
    env.exec("AcceptTrade()").unwrap();
    assert!(!fired(&env, "TRADE_ACCEPT_UPDATE"));
}

// ── CancelTrade ───────────────────────────────────────────────────────────────

#[test]
fn cancel_trade_clears_slot_and_fires_closed() {
    let env = env();
    env.exec(
        r#"InitiateTrade("Jaina")
               CancelTrade()"#,
    )
    .unwrap();
    let st = env.state().borrow();
    assert!(st.active_trade.is_none());
    drop(st);
    assert!(fired(&env, "TRADE_CLOSED"));
}

// ── SetTradeCurrency ──────────────────────────────────────────────────────────

#[test]
fn set_trade_currency_writes_player_money() {
    let env = env();
    env.exec(
        r#"InitiateTrade("Jaina")
               SetTradeCurrency(10000)"#,
    )
    .unwrap();
    let st = env.state().borrow();
    assert_eq!(st.active_trade.as_ref().unwrap().player_money, 10000);
}

#[test]
fn set_trade_currency_without_trade_is_noop() {
    let env = env();
    // Must not panic or crash.
    env.exec("SetTradeCurrency(500)").unwrap();
    assert!(env.state().borrow().active_trade.is_none());
}

#[test]
fn trade_money_getters_read_player_and_target_money() {
    let env = env();
    env.exec(
        r#"InitiateTrade("Jaina")
               SetTradeCurrency(10000)"#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .active_trade
        .as_mut()
        .unwrap()
        .target_money = 2500;

    let (player_money, target_money): (i64, i64) = env
        .eval("return GetPlayerTradeMoney(), GetTargetTradeMoney()")
        .unwrap();

    assert_eq!(player_money, 10000);
    assert_eq!(target_money, 2500);
}

// ── SetCursorItemSlot ─────────────────────────────────────────────────────────

#[test]
fn set_cursor_item_slot_moves_cursor_item_into_slot() {
    let env = env();
    env.exec(r#"InitiateTrade("Jaina")"#).unwrap();
    {
        let mut st = env.state().borrow_mut();
        st.cursor_item = Some(CursorInfo::Item {
            item_id: 6948,
            stack_count: 1,
            origin: CursorItemOrigin::Unknown,
            applied_transmog: None,
        });
    }
    env.exec("SetCursorItemSlot(3)").unwrap();
    let st = env.state().borrow();
    assert_eq!(st.active_trade.as_ref().unwrap().player_slots[2], 6948);
    assert!(st.cursor_item.is_none());
}

#[test]
fn set_cursor_item_slot_out_of_range_is_noop() {
    let env = env();
    env.exec(r#"InitiateTrade("Jaina")"#).unwrap();
    {
        let mut st = env.state().borrow_mut();
        st.cursor_item = Some(CursorInfo::Item {
            item_id: 123,
            stack_count: 1,
            origin: CursorItemOrigin::Unknown,
            applied_transmog: None,
        });
    }
    env.exec("SetCursorItemSlot(99)").unwrap();
    let st = env.state().borrow();
    // Cursor still holds the item; no slot was touched.
    assert!(matches!(
        st.cursor_item,
        Some(CursorInfo::Item { item_id: 123, .. })
    ));
    for slot in &st.active_trade.as_ref().unwrap().player_slots {
        assert_eq!(*slot, 0);
    }
}

#[test]
fn set_cursor_item_slot_without_item_cursor_is_noop() {
    let env = env();
    env.exec(
        r#"InitiateTrade("Jaina")
               SetCursorItemSlot(1)"#,
    )
    .unwrap();
    let st = env.state().borrow();
    assert_eq!(st.active_trade.as_ref().unwrap().player_slots[0], 0);
}
