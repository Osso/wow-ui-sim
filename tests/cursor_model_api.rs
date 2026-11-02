//! API-level coverage of the Retail shared cursor model (WM46): exact clear and
//! placement payloads, stored item IDs and mode-tested predicates.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

const RECORDER: &str = r#"
    local events = {}
    local listener = CreateFrame('Frame')
    for _, event in ipairs({'CURSOR_CHANGED', 'ACTIONBAR_SHOWGRID', 'ACTIONBAR_HIDEGRID'}) do
        listener:RegisterEvent(event)
    end
    listener:SetScript('OnEvent', function(_, event, ...)
        local parts = {event}
        for index = 1, select('#', ...) do
            parts[#parts + 1] = tostring((select(index, ...)))
        end
        events[#events + 1] = table.concat(parts, ',')
    end)
    function record(action)
        events = {}
        action()
        return table.concat(events, ';')
    end
"#;

fn env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().player.money = 1_000;
    env.exec(RECORDER).unwrap();
    env
}

#[test]
fn item_setters_report_stored_ids_and_clears_zero_them() {
    let env = env();
    env.exec(
        r#"
        local log = record(function() C_Item.PickupItem(6948) end)
        assert(log == 'ACTIONBAR_SHOWGRID;CURSOR_CHANGED,false,6,0,0', log)
        -- The merchant setter reports S after clearing mode 6, which zeros it.
        log = record(function() PickupMerchantItem(3) end)
        assert(log == 'ACTIONBAR_HIDEGRID;CURSOR_CHANGED,true,0,6,6948;CURSOR_CHANGED,false,5,0,0', log)
        assert(select('#', GetCursorInfo()) == 2)
        local kind, index = GetCursorInfo()
        assert(kind == 'merchant' and index == 3)
        assert(not CursorHasItem())
        -- Merchant row 3 stores synthesized item 100003 in S.
        log = record(ClearCursor)
        assert(log == 'CURSOR_CHANGED,true,0,5,100003', log)
        assert(record(ClearCursor) == '')
    "#,
    )
    .unwrap();
}

#[test]
fn typed_setters_clear_first_and_report_zero() {
    let env = env();
    env.exec(
        r#"
        -- INFERRED (WM54 pending): typed pickups run the shared clear, then
        -- report (false, mode, 0, 0), from an empty and a nonempty cursor.
        local log = record(function() PickupPlayerMoney(10) end)
        assert(log == 'CURSOR_CHANGED,false,2,0,0', log)
        assert(CursorHasMoney() and not CursorHasItem() and not CursorHasSpell())
        log = record(function() C_Spell.PickupSpell(853) end)
        assert(log == 'CURSOR_CHANGED,true,0,2,0;ACTIONBAR_SHOWGRID;CURSOR_CHANGED,false,3,0,0', log)
        assert(CursorHasSpell() and not CursorHasMoney())
        log = record(function() PickupMacro(1) end)
        assert(log == 'ACTIONBAR_HIDEGRID;CURSOR_CHANGED,true,0,3,0;ACTIONBAR_SHOWGRID;CURSOR_CHANGED,false,7,0,0', log)
        assert(CursorHasMacro() and not CursorHasSpell())
        log = record(function() PickupPetAction(1) end)
        assert(log == 'ACTIONBAR_HIDEGRID;CURSOR_CHANGED,true,0,7,0;ACTIONBAR_SHOWGRID;CURSOR_CHANGED,false,4,0,0', log)
        assert(not CursorHasMacro())
        log = record(ClearCursor)
        assert(log == 'ACTIONBAR_HIDEGRID;CURSOR_CHANGED,true,0,4,0', log)
    "#,
    )
    .unwrap();
}

#[test]
fn owned_items_use_mode_one_without_the_grid() {
    let env = env();
    env.state().borrow_mut().bag_items.insert(
        (0, 1),
        wow_ui_sim::lua_api::state::BagItem {
            item_id: 6948,
            stack_count: 1,
            hyperlink: None,
            durability: None,
            applied_transmog: None,
        },
    );
    env.exec(
        r#"
        local log = record(function() C_Container.PickupContainerItem(0, 1) end)
        assert(log == 'CURSOR_CHANGED,false,1,0,0', log)
        assert(CursorHasItem())
        -- Dropping into the empty slot runs the shared clear.
        log = record(function() C_Container.PickupContainerItem(0, 2) end)
        assert(log == 'CURSOR_CHANGED,true,0,1,0', log)
        assert(not CursorHasItem())
        assert(C_Container.GetContainerItemID(0, 2) == 6948)
    "#,
    )
    .unwrap();
}
