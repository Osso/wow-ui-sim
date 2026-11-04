#![cfg(any(feature = "profile-retail", feature = "client-ptr"))]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::globals::real::merchant_buyback::BuybackItem;

fn fixture() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.merchant_frame_open = true;
        state.merchant_repair_capable = true;
        state.player.money = 100;
        state.merchant_buyback_items = vec![BuybackItem {
            item_id: 6948,
            quantity: 1,
            price: 50,
            num_available: 1,
            is_usable: true,
            is_bound: false,
        }];
    }
    env
}

#[test]
fn merchant_buyback_cursor_tracks_affordability_and_preserves_invalid() {
    let env = fixture();
    env.exec("ShowBuybackSellCursor(1)").unwrap();
    assert_eq!(env.state().borrow().cursor_storage.presentation_display, 3);
    env.state().borrow_mut().player.money = 10;
    env.exec("ShowBuybackSellCursor('1.9')").unwrap();
    assert_eq!(env.state().borrow().cursor_storage.presentation_display, 49);
    env.exec("ShowBuybackSellCursor(0); ShowBuybackSellCursor(99)")
        .unwrap();
    assert_eq!(env.state().borrow().cursor_storage.presentation_display, 49);
    let err = env.exec("ShowBuybackSellCursor({})").unwrap_err();
    assert!(
        err.to_string()
            .contains("Usage: ShowBuybackSellCursor(index)")
    );
    env.state().borrow_mut().cursor_storage.presentation_gate_a = true;
    env.exec("ShowBuybackSellCursor({})").unwrap();
    assert_eq!(env.state().borrow().cursor_storage.presentation_display, 49);
    {
        let mut state = env.state().borrow_mut();
        state.cursor_storage.presentation_gate_a = false;
        state.cursor_storage.presentation_gate_b = true;
    }
    env.exec("ShowBuybackSellCursor({})").unwrap();
    assert_eq!(env.state().borrow().cursor_storage.presentation_display, 49);
    env.state().borrow_mut().cursor_storage.presentation_gate_b = false;
    env.exec("PickupPlayerMoney(1); ShowBuybackSellCursor(1); assert(GetCursorInfo() == 'money')")
        .unwrap();
}

#[test]
fn repair_base_survives_overrides_and_clears_held_item() {
    let env = fixture();
    env.state().borrow_mut().merchant_repair_capable = false;
    env.exec("PickupPlayerMoney(1); ShowRepairCursor(); assert(GetCursorInfo() == 'money'); assert(not InRepairMode())")
        .unwrap();
    env.state().borrow_mut().merchant_repair_capable = true;
    env.exec("ShowRepairCursor(); assert(GetCursorInfo() == nil); assert(InRepairMode())")
        .unwrap();
    {
        let state = env.state().borrow();
        assert_eq!(state.cursor_storage.presentation_base, 17);
        assert_eq!(state.cursor_storage.presentation_display, 17);
    }
    env.exec("SetCursor('INSPECT_CURSOR'); assert(InRepairMode())")
        .unwrap();
    assert_eq!(env.state().borrow().cursor_storage.presentation_display, 7);
    env.exec("ResetCursor(); assert(InRepairMode())").unwrap();
    assert_eq!(env.state().borrow().cursor_storage.presentation_display, 17);
    env.exec("SetCursor('POINT_CURSOR'); SetCursor(nil); assert(InRepairMode())")
        .unwrap();
    assert_eq!(env.state().borrow().cursor_storage.presentation_display, 17);
}

#[test]
fn presentation_change_notifies_with_held_content_not_display() {
    let env = fixture();
    {
        let mut state = env.state().borrow_mut();
        state.cursor_storage.presentation_refresh = true;
        state.cursor_storage.presentation_notifications = true;
    }
    env.exec(
        r#"
        events = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('CURSOR_CHANGED')
        listener:SetScript('OnEvent', function(_, _, ...)
            events[#events + 1] = {...}
        end)
        ShowBuybackSellCursor(1)
        assert(#events == 1)
        assert(events[1][1] == true and events[1][2] == 0 and events[1][3] == 0 and events[1][4] == 0)
        ShowBuybackSellCursor(1)
        assert(#events == 1)
        ResetCursor()
        PickupPlayerMoney(1)
        events = {}
        ShowBuybackSellCursor(1)
        assert(#events == 1)
        assert(events[1][1] == false and events[1][2] == 2 and events[1][3] == 2 and events[1][4] == 0)
        "#,
    )
    .unwrap();
}
