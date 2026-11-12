//! Round 6-B `GetLootCurrency`: the original loot-frame hover selects the
//! currency getter from the loot-slot kind, and the two loot getters are
//! mutually exclusive per slot.
#![cfg(any(
    feature = "profile-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{BagItem, LootSlot};

fn loot_fixture() -> WowLuaEnv {
    let env = tooltip_full_env_helpers::setup_full_env();
    env.state().borrow_mut().loot_slots = vec![
        LootSlot::Item(BagItem {
            owned_identity: None,
            item_id: 6948,
            stack_count: 1,
            hyperlink: None,
            durability: None,
            applied_transmog: None,
        }),
        LootSlot::Currency {
            currency_id: 2245,
            quantity: 42,
        },
    ];
    env.state().borrow_mut().loot_frame_open = true;
    env
}

/// `Blizzard_UIPanels_Game/Mainline/LootFrame.lua` (`LootFrameItemElementMixin:OnEnter`,
/// Retail :348, PTR/Forever :620) reads `GetLootSlotType` and routes a currency
/// slot to `GameTooltip:SetLootCurrency`, which the handler map at
/// `Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua:557` maps to this
/// getter.
#[test]
fn tooltip_loot_currency_original_hover_selects_the_currency_slot() {
    if ACTIVE != ClientProfile::Retail {
        return;
    }
    let env = loot_fixture();
    env.exec(
        r#"
        assert(type(LootFrameItemElementMixin.OnEnter) == 'function', 'loot consumer missing')
        assert(type(LootFrameElementMixin.GetItemSlotType) == 'function', 'slot-type consumer missing')
        local button = CreateFrame('Button', nil, UIParent)
        button.HighlightNameFrame = CreateFrame('Frame', nil, button)
        button.GetItemSlotType = LootFrameElementMixin.GetItemSlotType
        function button:GetSlotIndex() return self.slot end
        button.slot = 2
        LootFrameItemElementMixin.OnEnter(button)
        local info = GameTooltip:GetPrimaryTooltipInfo()
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(info and info.getterName == 'GetLootCurrency', 'original hover must select GetLootCurrency')
        assert(data and data.type == Enum.TooltipDataType.Currency, 'loot currency data type')
        assert(data and data.id == 2245, 'loot currency id mismatch')
        assert(data and data.dataInstanceID ~= nil, 'loot currency instance id missing')
        assert(data and data.quantity == nil, 'R84 §Q4: loot currency must not publish quantity')
        local currencyName = C_CurrencyInfo.GetCurrencyInfo(2245).name
        assert(currencyName ~= nil and currencyName ~= '', 'resolved currency name missing')
        assert(data.lines[1].leftText == currencyName, 'currency title must use the resolved CurrencyTypes name')
        for _, line in ipairs(data.lines) do
            assert(line.rightText == nil and line.rightColor == nil, 'currency rows are left-only')
        end
        "#,
    )
    .unwrap();
}

/// Round 6-B §1(b): the loot-item reader requires kind `0` and the
/// loot-currency reader requires a non-zero kind, so each getter takes its miss
/// arm on the other kind. `GetLootSlotType` reports the kind the original
/// consumer checks.
#[test]
fn tooltip_loot_currency_and_item_are_mutually_exclusive_per_slot() {
    let env = loot_fixture();
    env.exec(
        r#"
        assert(GetLootSlotType(1) == Enum.LootSlotType.Item, 'item slot type')
        assert(GetLootSlotType(2) == Enum.LootSlotType.Currency, 'currency slot type')
        assert(select('#', C_TooltipInfo.GetLootItem(2)) == 0, 'GetLootItem must miss on a currency slot')
        assert(select('#', C_TooltipInfo.GetLootCurrency(1)) == 0, 'GetLootCurrency must miss on an item slot')
        local currency = C_TooltipInfo.GetLootCurrency(2)
        assert(currency and currency.type == Enum.TooltipDataType.Currency, 'currency identity type')
        assert(currency and currency.id == 2245, 'currency identity id')
        assert(currency and #currency.lines > 0, 'currency tooltip lines')
        assert(currency and currency.lines[1].leftText == 'Valorstones', 'currency name mismatch')
        local item = C_TooltipInfo.GetLootItem(1)
        assert(item and item.lines[1].leftText == 'Hearthstone', 'item slot still selects GetLootItem')
        "#,
    )
    .unwrap();
}

/// Round 6-B §1(b): an entry whose currency row does not resolve takes the
/// same miss arm, and the argument reader carries the getter's own usage text.
#[test]
fn tooltip_loot_currency_miss_and_argument_text() {
    let env = loot_fixture();
    env.state().borrow_mut().loot_slots.push(LootSlot::Currency {
        currency_id: 0,
        quantity: 1,
    });
    env.exec(
        r#"
        local getter = C_TooltipInfo.GetLootCurrency
        local usage = 'Usage: local data = C_TooltipInfo.GetLootCurrency(slot)'
        local range = 'outside of expected range 0 to 4294967295'
        assert(select('#', getter(3)) == 0, 'unresolvable currency row must miss')
        assert(select('#', getter(0)) == 0, 'slot 0 must miss')
        assert(select('#', getter(4)) == 0, 'past-count slot must miss')
        local function error_text(...)
            local ok, err = pcall(getter, ...)
            assert(not ok, 'expected an argument error')
            return err
        end
        assert(error_text(-1):find(range, 1, true), 'negative slot must report the range')
        assert(error_text(4294967296):find(range, 1, true), 'above the domain must report the range')
        assert(error_text('1'):find(usage, 1, true), 'wrong type must report the usage text')
        assert(error_text():find(usage, 1, true), 'missing argument must report the usage text')
        "#,
    )
    .unwrap();
}
