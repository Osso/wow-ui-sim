//! Round 6-B `GetLootItem`: the original loot-frame hover reaches the getter,
//! the slot index is one-based and truncating, and the live loot slot count
//! bounds it.
#![cfg(any(
    feature = "profile-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::BagItem;

fn loot_fixture() -> WowLuaEnv {
    let env = tooltip_full_env_helpers::setup_full_env();
    env.state().borrow_mut().loot_slots = vec![
        BagItem {
            owned_identity: None,
            item_id: 6948,
            stack_count: 1,
            hyperlink: None,
            durability: None,
            applied_transmog: None,
        },
        BagItem {
            owned_identity: None,
            item_id: 2840,
            stack_count: 1,
            hyperlink: None,
            durability: None,
            applied_transmog: None,
        },
    ];
    env.state().borrow_mut().loot_frame_open = true;
    env
}

/// `Blizzard_UIPanels_Game/Mainline/LootFrame.lua` (`LootFrameItemElementMixin:OnEnter`,
/// Retail :353, Forever :631) routes the item slot type to `GameTooltip:SetLootItem`,
/// which the handler map at `Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua:537`
/// maps to this getter.
#[test]
fn tooltip_loot_item_original_hover_selects_the_slot() {
    let env = loot_fixture();
    env.exec(
        r#"
        assert(type(LootFrameItemElementMixin.OnEnter) == 'function', 'loot consumer missing')
        local button = CreateFrame('Button', nil, UIParent)
        button.HighlightNameFrame = CreateFrame('Frame', nil, button)
        function button:GetItemSlotType() return Enum.LootSlotType.Item end
        function button:GetSlotIndex() return self.slot end
        for slot, name in ipairs({'Hearthstone', 'Copper Bar'}) do
            button.slot = slot
            LootFrameItemElementMixin.OnEnter(button)
            local info = GameTooltip:GetPrimaryTooltipInfo()
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(info and info.getterName == 'GetLootItem', 'GetLootItem info missing')
            assert(data and data.type == Enum.TooltipDataType.Item, 'loot data type')
            assert(data and #data.lines > 0, 'loot item must compose the item lines')
            assert(data and data.lines[1].leftText == name, 'loot name mismatch')
            assert(GameTooltip:GetLeftLine(1):GetText() == name, 'loot text mismatch')
        end
        "#,
    )
    .unwrap();
}

/// Round 6-B §1(a): the reader accepts any number in 0..=2^32-1, subtracts 1.0
/// and truncates toward zero before the live-count bound test.
#[test]
fn tooltip_loot_item_index_truncates_without_an_integral_test() {
    let env = loot_fixture();
    env.exec(
        r#"
        local getter = C_TooltipInfo.GetLootItem
        assert(getter(1).lines[1].leftText == 'Hearthstone', 'index 1')
        assert(getter(1.9).lines[1].leftText == 'Hearthstone', '1.9 must truncate to slot 1')
        assert(getter(2.9).lines[1].leftText == 'Copper Bar', '2.9 must truncate to slot 2')
        assert(getter(0.5).lines[1].leftText == 'Hearthstone', '0.5 must truncate toward zero to slot 1')
        "#,
    )
    .unwrap();
}

/// Round 6-B §1(b): a slot at or past the live count takes the miss arm, which
/// returns no values and never publishes an empty tooltip.
#[test]
fn tooltip_loot_item_miss_arm_returns_no_values() {
    let env = loot_fixture();
    env.exec(
        r#"
        local getter = C_TooltipInfo.GetLootItem
        assert(select('#', getter(0)) == 0, 'slot 0 must miss')
        assert(select('#', getter(3)) == 0, 'past-count slot must miss')
        assert(select('#', getter(4294967295)) == 0, 'top of the domain must miss')
        GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
        GameTooltip:SetLootItem(3)
        assert(GameTooltip:GetPrimaryTooltipData() == nil, 'miss must not publish a tooltip')
        "#,
    )
    .unwrap();
}

/// Round 6-B §1(b): the getters never consult loot window state, so a closed
/// window does not gate a slot the list still holds.
#[test]
fn tooltip_loot_item_ignores_loot_window_state() {
    let env = loot_fixture();
    env.exec(
        r#"
        CloseLoot()
        local data = C_TooltipInfo.GetLootItem(1)
        assert(data and data.lines[1].leftText == 'Hearthstone', 'closed window must not gate the slot')
        "#,
    )
    .unwrap();
}

/// Round 6-B §1(a): the argument error carries the getter's own usage text, and
/// an out-of-domain number carries the reader's range reason.
#[test]
fn tooltip_loot_item_argument_errors_carry_native_text() {
    let env = loot_fixture();
    env.exec(
        r#"
        local getter = C_TooltipInfo.GetLootItem
        local usage = 'Usage: local data = C_TooltipInfo.GetLootItem(slot)'
        local range = 'outside of expected range 0 to 4294967295'
        local function error_text(...)
            local ok, err = pcall(getter, ...)
            assert(not ok, 'expected an argument error')
            return err
        end
        assert(error_text(-1):find(range, 1, true), 'negative slot must report the range')
        assert(error_text(4294967296):find(range, 1, true), 'above the domain must report the range')
        assert(error_text(0/0):find(range, 1, true), 'non-finite must report the range')
        assert(error_text('1'):find(usage, 1, true), 'wrong type must report the usage text')
        assert(error_text():find(usage, 1, true), 'missing argument must report the usage text')
        "#,
    )
    .unwrap();
}
