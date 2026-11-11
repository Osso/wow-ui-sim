#![cfg(any(feature = "client-ptr", feature = "client-retail"))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn env_with_merchant_items(items: &[u32]) -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().merchant_items = items.to_vec();
    env
}

#[test]
fn pickup_merchant_item_returns_the_selected_row() {
    let env = env_with_merchant_items(&[6948, 6256]);
    env.exec(
        r#"
        local cursorChanges = {}
        local gridShown = false
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('CURSOR_CHANGED')
        listener:RegisterEvent('ACTIONBAR_SHOWGRID')
        listener:SetScript('OnEvent', function(_, event, ...)
            if event == 'ACTIONBAR_SHOWGRID' then
                gridShown = true
                return
            end
            local empty, mode, previous = ...
            cursorChanges[#cursorChanges + 1] = { empty, mode, previous }
        end)

        PickupMerchantItem(2)
        local kind, merchantIndex = GetCursorInfo()
        assert(select('#', GetCursorInfo()) == 2)
        assert(kind == 'merchant' and merchantIndex == 2)
        assert(not CursorHasItem())
        assert(not gridShown)
        assert(#cursorChanges == 1)
        assert(cursorChanges[1][1] == false)
        assert(cursorChanges[1][2] == 5 and cursorChanges[1][3] == 0)
        "#,
    )
    .unwrap();
    let cursor = env.state().borrow().cursor_item.clone();
    assert!(matches!(
        cursor,
        Some(wow_ui_sim::lua_api::state::CursorInfo::Item {
            item_id: 6256,
            origin: wow_ui_sim::lua_api::state::CursorItemOrigin::Merchant { index: 2 },
            ..
        })
    ));
}

#[test]
fn pickup_merchant_item_clears_for_invalid_inputs() {
    let env = env_with_merchant_items(&[6948, 6256]);
    env.exec(
        r#"
        local invalidCalls = {
            function() PickupMerchantItem() end,
            function() PickupMerchantItem(nil) end,
            function() PickupMerchantItem('1') end,
            function() PickupMerchantItem(0) end,
            function() PickupMerchantItem(3) end,
        }
        for index, pickup in ipairs(invalidCalls) do
            ClearCursor()
            PickupSpell(853)
            local ok, err = pcall(pickup)
            assert(ok, tostring(err))
            assert(select('#', GetCursorInfo()) == 0, 'invalid pickup ' .. index .. ' kept cursor')
        end
        "#,
    )
    .unwrap();
}

#[test]
fn pickup_merchant_item_truncates_fractional_index_and_errors_on_overflow() {
    let env = env_with_merchant_items(&[6948, 6256]);
    env.exec(
        r#"
        PickupMerchantItem(1.9)
        local kind, merchantIndex = GetCursorInfo()
        assert(select('#', GetCursorInfo()) == 2)
        assert(kind == 'merchant' and merchantIndex == 1)

        ClearCursor()
        for _, overflow in ipairs({2147483648, -2147483649}) do
            local ok, err = pcall(PickupMerchantItem, overflow)
            assert(not ok and err:find('integer overflow attempting to store', 1, true), tostring(err))
        end
        "#,
    )
    .unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn place_action_leaves_normal_merchant_cursor_and_slot_unchanged() {
    let env = env_with_merchant_items(&[6948]);
    {
        let mut sim = env.state().borrow_mut();
        sim.action_bars.remove(&202);
        sim.action_outfits.remove(&202);
        sim.action_macros.remove(&202);
    }
    env.exec(
        r#"
        local events = {}
        local listener = CreateFrame('Frame')
        for _, event in ipairs({
            'ACTIONBAR_HIDEGRID',
            'ACTIONBAR_SHOWGRID',
            'ACTIONBAR_SLOT_CHANGED',
            'ACTIONBAR_UPDATE_COOLDOWN',
            'CURSOR_CHANGED',
        }) do
            listener:RegisterEvent(event)
        end
        listener:SetScript('OnEvent', function(_, event)
            events[#events + 1] = event
        end)

        PickupMerchantItem(1)
        events = {}
        PlaceAction(202)
        local kind, merchantIndex = GetCursorInfo()
        assert(select('#', GetCursorInfo()) == 2)
        assert(kind == 'merchant' and merchantIndex == 1)
        assert(GetActionInfo(202) == nil)
        assert(#events == 0, table.concat(events, ','))
        "#,
    )
    .unwrap();
}
