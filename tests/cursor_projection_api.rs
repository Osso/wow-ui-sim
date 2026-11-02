//! API-level coverage of Retail GetCursorInfo projections (WM46) and the
//! equipment-set and currency setters (WM46, WM54). No original Retail consumer
//! reads the owned-item link or the currency tuple.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{BagItem, CurrencyInfo, PetActionSlot};

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
    env.exec(RECORDER).unwrap();
    env
}

#[test]
fn owned_item_projects_id_and_link_or_nil() {
    let env = env();
    env.state().borrow_mut().bag_items.insert(
        (0, 5),
        BagItem {
            item_id: 999_999,
            stack_count: 3,
            hyperlink: None,
            durability: None,
            applied_transmog: None,
        },
    );
    env.exec(
        r#"
        -- WM46 mode 1: ("item", owned item ID, generated link or nil).
        C_Container.PickupContainerItem(0, 1)
        assert(select('#', GetCursorInfo()) == 3)
        local kind, id, link = GetCursorInfo()
        assert(kind == 'item' and id == 6948, tostring(id))
        assert(link == select(2, C_Item.GetItemInfo(6948)), tostring(link))
        C_Container.PickupContainerItem(0, 1)
        -- No item metadata: the link is nil, not the stack count.
        C_Container.PickupContainerItem(0, 5)
        assert(select('#', GetCursorInfo()) == 3)
        kind, id, link = GetCursorInfo()
        assert(kind == 'item' and id == 999999 and link == nil, tostring(link))
    "#,
    )
    .unwrap();
}

#[test]
fn equipment_set_cursor_projects_the_record_name() {
    let env = env();
    env.exec(
        r#"
        C_EquipmentSet.CreateEquipmentSet('Tank', 'Interface\\Icons\\INV_Shield_06')
        C_EquipmentSet.CreateEquipmentSet('Heal', 'Interface\\Icons\\Spell_Holy_HolyBolt')
        local tank = C_EquipmentSet.GetEquipmentSetID('Tank')
        local heal = C_EquipmentSet.GetEquipmentSetID('Heal')
        -- WM54: EquipmentSet 12 shows the grid and reports (false, 12, O, 0).
        local log = record(function() C_EquipmentSet.PickupEquipmentSet(tank) end)
        assert(log == 'ACTIONBAR_SHOWGRID;CURSOR_CHANGED,false,12,0,0', log)
        -- INFERRED: the projected text is the set name.
        local kind, name = GetCursorInfo()
        assert(kind == 'equipmentset' and name == 'Tank', tostring(name))
        assert(not CursorHasItem())
        -- Same-mode replacement emits only the clear's event.
        log = record(function() C_EquipmentSet.PickupEquipmentSet(heal) end)
        assert(log == 'ACTIONBAR_HIDEGRID;CURSOR_CHANGED,true,0,12,0;ACTIONBAR_SHOWGRID', log)
        kind, name = GetCursorInfo()
        assert(kind == 'equipmentset' and name == 'Heal', tostring(name))
        -- WM46: a missing record projects zero values without clearing.
        C_EquipmentSet.DeleteEquipmentSet(heal)
        assert(select('#', GetCursorInfo()) == 0)
        log = record(ClearCursor)
        assert(log == 'ACTIONBAR_HIDEGRID;CURSOR_CHANGED,true,0,12,0', log)
    "#,
    )
    .unwrap();
}

#[test]
fn currency_cursor_projects_id_and_icon_or_nil() {
    let env = env();
    {
        let mut sim = env.state().borrow_mut();
        for (id, icon) in [(1792, 1_455_894), (1166, 0)] {
            sim.currency_info.insert(
                id,
                CurrencyInfo {
                    currency_id: id,
                    icon_file_id: icon,
                    ..Default::default()
                },
            );
        }
        sim.currency_info.remove(&999_999);
    }
    env.exec(
        r#"
        -- WM54: Currency 13 shows the grid and reports (false, 13, O, 0).
        local log = record(function() C_CurrencyInfo.PickupCurrency(1792) end)
        assert(log == 'ACTIONBAR_SHOWGRID;CURSOR_CHANGED,false,13,0,0', log)
        -- INFERRED: the metadata integer is the icon; zero projects nil.
        assert(select('#', GetCursorInfo()) == 3)
        local kind, id, icon = GetCursorInfo()
        assert(kind == 'currency' and id == 1792 and icon == 1455894, tostring(icon))
        C_CurrencyInfo.PickupCurrency(1166)
        kind, id, icon = GetCursorInfo()
        assert(kind == 'currency' and id == 1166 and icon == nil, tostring(icon))
        assert(select('#', GetCursorInfo()) == 3)
        -- WM46: a missing currency record projects zero values.
        C_CurrencyInfo.PickupCurrency(999999)
        assert(select('#', GetCursorInfo()) == 0)
        log = record(ClearCursor)
        assert(log == 'ACTIONBAR_HIDEGRID;CURSOR_CHANGED,true,0,13,0', log)
    "#,
    )
    .unwrap();
}

#[test]
fn spell_cursor_projects_main_book_slot_type_and_id() {
    let env = env();
    env.state().borrow_mut().action_bars.insert(1, 853);
    env.exec(
        r#"
        -- WM46/WM56: ("spell", book slot, book type, stored spell ID, override
        -- or nil). Hammer of Justice is slot 7 of the Paladin main book.
        C_Spell.PickupSpell(853)
        assert(select('#', GetCursorInfo()) == 5)
        local kind, slot, book, id, override = GetCursorInfo()
        assert(kind == 'spell' and slot == 7 and book == 'spell', tostring(slot))
        assert(id == 853 and override == nil, tostring(id))
        -- WM56: a spell in neither book projects slot 0 and "spell".
        C_Spell.PickupSpell(118)
        kind, slot, book, id = GetCursorInfo()
        assert(kind == 'spell' and slot == 0 and book == 'spell' and id == 118, tostring(slot))
        -- Spell action content installs the same mode 3 payload.
        ClearCursor()
        PickupAction(1)
        kind, slot, book, id = GetCursorInfo()
        assert(kind == 'spell' and slot == 7 and book == 'spell' and id == 853, tostring(slot))
    "#,
    )
    .unwrap();
}

#[test]
fn pet_action_cursor_projects_the_packed_word_category() {
    let env = env();
    {
        let mut sim = env.state().borrow_mut();
        // Supplied packed words (WM56): category 1 with and without the
        // high flags, reaction category 6 and command category 7.
        let words = [
            0x0100_0000 | 16_827,
            0xC100_0000 | 17_253,
            0x0600_0002,
            0x0700_0001,
        ];
        for (slot, word) in words.into_iter().enumerate() {
            sim.pet_actions[slot] = PetActionSlot {
                has_action: true,
                packed_action: word,
                ..Default::default()
            };
        }
        sim.pet_spells.clear();
    }
    env.exec(
        r#"
        -- WM46/WM56: ("petaction", low 24 bits, pet-book slot or nil,
        -- override or nil). Only category (word >> 24) & 63 == 1 looks up the
        -- pet book, where a miss is 0; the supplied pet book is empty.
        local function project(slot)
            PickupPetAction(slot)
            assert(select('#', GetCursorInfo()) == 4)
            local kind, action, book, override = GetCursorInfo()
            assert(kind == 'petaction', tostring(kind))
            return action, book, override
        end
        local action, book, override = project(1)
        assert(action == 16827 and book == 0 and override == nil, tostring(book))
        action, book, override = project(2)
        assert(action == 17253 and book == 0 and override == nil, tostring(book))
        action, book, override = project(3)
        assert(action == 2 and book == nil and override == nil, tostring(book))
        action, book, override = project(4)
        assert(action == 1 and book == nil and override == nil, tostring(book))
    "#,
    )
    .unwrap();
}
