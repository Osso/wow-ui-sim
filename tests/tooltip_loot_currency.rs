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

use wow_ui_sim::c_api::c_tooltip_info_currency::CurrencyTooltipInputs;
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
        assert(data and data.lines[1].leftText == 'Valorstones', 'loot currency name mismatch')
        "#,
    )
    .unwrap();
    if ACTIVE == ClientProfile::Retail {
        let r84_payload: bool = env
            .eval(
                "local data = GameTooltip:GetPrimaryTooltipData(); return data.id == 2245 and data.dataInstanceID ~= nil and data.quantity == nil and data.lines[1].rightText == nil and data.lines[1].rightColor == nil",
            )
            .unwrap();
        assert!(
            r84_payload,
            "R84 §Q4: loot currency identity must omit top-level quantity"
        );
    }
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

/// LootFrame.lua:348 routes the currency slot through SetLootCurrency.
#[test]
fn tooltip_loot_currency_normalizes_hundredths_for_the_xp_row() {
    if ACTIVE != ClientProfile::Retail {
        return;
    }
    let env = loot_fixture();
    {
        let mut state = env.state().borrow_mut();
        state.loot_slots[1] = LootSlot::Currency {
            currency_id: 2245,
            quantity: 12_345,
        };
        state.currency_tooltip_inputs.insert(
            2245,
            CurrencyTooltipInputs {
                flags_0: Some(1 << 3),
                flags_1: Some(1 << 1),
                field_11_xp_quest_difficulty: Some(1),
                local_currency_record: Some(true),
                ..CurrencyTooltipInputs::default()
            },
        );
    }
    env.exec(
        r#"
        local button = CreateFrame('Button', nil, UIParent)
        button.HighlightNameFrame = CreateFrame('Frame', nil, button)
        button.GetItemSlotType = LootFrameElementMixin.GetItemSlotType
        function button:GetSlotIndex() return self.slot end
        button.slot = 2
        LootFrameItemElementMixin.OnEnter(button)
        local info = GameTooltip:GetPrimaryTooltipInfo()
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(info and info.getterName == 'GetLootCurrency', 'loot hover must select GetLootCurrency')
        assert(#data.lines == 3, 'XP row must end the producer')
        assert(data.lines[3].type == 14, 'currency XP row type')
        assert(data.lines[3].leftText == XP .. ': 123', 'loot amount must divide by 100 with signed truncation')
        assert(data.quantity == nil, 'R84 §Q4: no top-level quantity')
        "#,
    )
    .unwrap();

    env.state()
        .borrow_mut()
        .currency_tooltip_inputs
        .get_mut(&2245)
        .expect("currency tooltip input")
        .flags_0 = Some(0);
    env.exec(
        r#"
        local button = CreateFrame('Button', nil, UIParent)
        button.HighlightNameFrame = CreateFrame('Frame', nil, button)
        button.GetItemSlotType = LootFrameElementMixin.GetItemSlotType
        function button:GetSlotIndex() return self.slot end
        button.slot = 2
        LootFrameItemElementMixin.OnEnter(button)
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(data.lines[3].leftText == XP .. ': 12345', 'loot amount must stay unchanged when bit 3 is clear')
        "#,
    )
    .unwrap();
}
