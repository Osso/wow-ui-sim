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
fn typed_setters_clear_first_and_report_the_previous_mode() {
    let env = env();
    env.exec(
        r#"
        -- WM54 Q1/Q2: typed pickups run the shared clear, then report
        -- (false, mode, mode before the clear, 0).
        local log = record(function() PickupPlayerMoney(10) end)
        assert(log == 'CURSOR_CHANGED,false,2,0,0', log)
        assert(CursorHasMoney() and not CursorHasItem() and not CursorHasSpell())
        log = record(function() C_Spell.PickupSpell(853) end)
        assert(log == 'CURSOR_CHANGED,true,0,2,0;ACTIONBAR_SHOWGRID;CURSOR_CHANGED,false,3,2,0', log)
        assert(CursorHasSpell() and not CursorHasMoney())
        log = record(function() PickupMacro(1) end)
        assert(log == 'ACTIONBAR_HIDEGRID;CURSOR_CHANGED,true,0,3,0;ACTIONBAR_SHOWGRID;CURSOR_CHANGED,false,7,3,0', log)
        assert(CursorHasMacro() and not CursorHasSpell())
        log = record(function() PickupPetAction(1) end)
        assert(log == 'ACTIONBAR_HIDEGRID;CURSOR_CHANGED,true,0,7,0;ACTIONBAR_SHOWGRID;CURSOR_CHANGED,false,4,7,0', log)
        assert(not CursorHasMacro())
        log = record(ClearCursor)
        assert(log == 'ACTIONBAR_HIDEGRID;CURSOR_CHANGED,true,0,4,0', log)
    "#,
    )
    .unwrap();
}

#[test]
fn owned_items_use_mode_one() {
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
        -- WM54 Q4: the Hearthstone's use spell shows the grid.
        local log = record(function() C_Container.PickupContainerItem(0, 1) end)
        assert(log == 'ACTIONBAR_SHOWGRID;CURSOR_CHANGED,false,1,0,0', log)
        assert(CursorHasItem())
        -- Dropping into an empty slot runs the shared clear; the default
        -- backpack fills slots 1-4.
        log = record(function() C_Container.PickupContainerItem(0, 5) end)
        assert(log == 'ACTIONBAR_HIDEGRID;CURSOR_CHANGED,true,0,1,0', log)
        assert(not CursorHasItem())
        assert(C_Container.GetContainerItemID(0, 5) == 6948)
    "#,
    )
    .unwrap();
}

fn bag_item(item_id: u32) -> wow_ui_sim::lua_api::state::BagItem {
    wow_ui_sim::lua_api::state::BagItem {
        item_id,
        stack_count: 1,
        hyperlink: None,
        durability: None,
        applied_transmog: None,
    }
}

#[test]
fn same_mode_replacement_reports_only_the_clear_except_owned_items() {
    let env = env();
    env.exec(
        r#"
        C_Spell.PickupSpell(853)
        -- WM54 Q1: a typed Spell-to-Spell replacement emits no set-side event.
        local log = record(function() C_Spell.PickupSpell(642) end)
        assert(log == 'ACTIONBAR_HIDEGRID;CURSOR_CHANGED,true,0,3,0;ACTIONBAR_SHOWGRID', log)
        assert(CursorHasSpell())
        ClearCursor()
        -- Owned Item-to-Item still fires: its previous mode is read after the
        -- clear. Default backpack: Hearthstone in slot 1, water in slot 2.
        C_Container.PickupContainerItem(0, 1)
        log = record(function() C_Container.PickupContainerItem(0, 2) end)
        assert(log == 'ACTIONBAR_HIDEGRID;CURSOR_CHANGED,true,0,1,0;ACTIONBAR_SHOWGRID;CURSOR_CHANGED,false,1,0,0', log)
        local kind, item = GetCursorInfo()
        assert(kind == 'item' and item == 159, tostring(item))
        assert(C_Container.GetContainerItemID(0, 2) == 6948)
    "#,
    )
    .unwrap();
}

#[test]
fn owned_item_grid_needs_a_use_spell_or_inventory_type_outside_class_17() {
    let env = env();
    {
        let mut sim = env.state().borrow_mut();
        // Worn Shortsword: InventoryType 21, no item effects.
        sim.bag_items.insert((0, 5), bag_item(25));
        // Linen Cloth: class 7, InventoryType 0, no item effects.
        sim.bag_items.insert((0, 6), bag_item(2589));
        // Pet Cage: class 17 with use spell 123302.
        sim.bag_items.insert((0, 7), bag_item(82800));
    }
    env.exec(
        r#"
        local function pickup(slot)
            ClearCursor()
            local log = record(function() C_Container.PickupContainerItem(0, slot) end)
            C_Container.PickupContainerItem(0, slot)
            return log
        end
        local shown = 'ACTIONBAR_SHOWGRID;CURSOR_CHANGED,false,1,0,0'
        local hidden = 'CURSOR_CHANGED,false,1,0,0'
        -- Hearthstone: use spell 8690, InventoryType 0.
        assert(pickup(1) == shown, 'use spell')
        assert(pickup(5) == shown, 'inventory type')
        assert(pickup(6) == hidden, 'neither')
        assert(pickup(7) == hidden, 'class 17 wins over its use spell')
    "#,
    )
    .unwrap();
}

#[test]
fn owned_item_grid_uses_profile_item_effect_associations() {
    for (item_id, spell_id) in [
        (278743, 55884),
        (281372, 1310965),
        (282657, 55884),
        (283360, 1314712),
        (284572, 1316093),
    ] {
        let env = env();
        env.state()
            .borrow_mut()
            .bag_items
            .insert((0, 5), bag_item(item_id));
        let (_, spell): (String, i64) = env
            .eval(&format!("return C_Item.GetItemSpell({item_id})"))
            .unwrap();
        assert_eq!(spell, spell_id, "item {item_id}");

        let log: String = env
            .eval("return record(function() C_Container.PickupContainerItem(0, 5) end)")
            .unwrap();
        assert_eq!(
            log, "ACTIONBAR_SHOWGRID;CURSOR_CHANGED,false,1,0,0",
            "item {item_id}"
        );
        let cursor: (String, i64) = env.eval("return GetCursorInfo()").unwrap();
        assert_eq!(cursor, ("item".to_string(), i64::from(item_id)));
        let log: String = env.eval("return record(ClearCursor)").unwrap();
        assert_eq!(
            log, "ACTIONBAR_HIDEGRID;CURSOR_CHANGED,true,0,1,0",
            "item {item_id}"
        );
    }
}

#[test]
fn notification_reads_the_cursor_when_it_fires() {
    let env = env();
    env.exec(
        r#"
        local reentered = false
        local clearer = CreateFrame('Frame')
        clearer:RegisterEvent('ACTIONBAR_SHOWGRID')
        clearer:SetScript('OnEvent', function()
            if not reentered then
                reentered = true
                ClearCursor()
            end
        end)
        local log = record(function() C_Spell.PickupSpell(853) end)
        -- The set-side event keeps its saved previous mode 0 but reports the
        -- cursor the SHOWGRID handler emptied (WM54 Q1).
        assert(log:sub(-#'CURSOR_CHANGED,true,0,0,0') == 'CURSOR_CHANGED,true,0,0,0', log)
        assert(log:find('CURSOR_CHANGED,true,0,3,0', 1, true), log)
        assert(GetCursorInfo() == nil)
    "#,
    )
    .unwrap();
}
