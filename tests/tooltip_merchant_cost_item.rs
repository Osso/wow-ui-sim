//! `C_TooltipInfo.GetMerchantCostItem(slot, costIndex)`.
//!
//! Evidence: decomp Round 6 §2, Retail 12.1.5 (70688). The getter selects one
//! entry of the merchant offer's extended-cost record: the surviving item costs
//! first, then the surviving currency costs, and `costIndex` is a zero-based
//! ordinal over that sequence, not a record position.

#![cfg(all(
    feature = "retail-12-1-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::MerchantCostRecord;

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

/// Slot 1 is an offer whose record holds two selectable item costs (the zero
/// identity between them must not consume an ordinal) and two currency costs.
fn seed_offer_costs(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.merchant_items = vec![229181];
    state.merchant_offer_costs.insert(
        0,
        MerchantCostRecord {
            item_costs: [6948, 0, 159, 0, 0],
            currency_costs: [2245, 0, 1792, 0, 0],
        },
    );
}

#[test]
fn original_money_frame_cost_hover_publishes_the_cost_ordinal() {
    let env = tooltip_full_env_helpers::setup_full_env();
    seed_offer_costs(&env);

    // MoneyFrame.lua:448-455 SmallDenominationTemplate_OnEnter is the original
    // owner of the merchant cost-item tooltip path; it forwards `self.index`
    // and `self.item` to the `SetMerchantCostItem` accessor declared in
    // TooltipDataHandler.lua:581.
    env.exec(
        r#"
        assert(type(SmallDenominationTemplate_OnEnter) == 'function',
            'MoneyFrame cost consumer missing')
        local button = CreateFrame('Button', nil, UIParent)
        button.index = 1
        button.item = 2
        SmallDenominationTemplate_OnEnter(button)
        local info = GameTooltip:GetPrimaryTooltipInfo()
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(info and info.getterName == 'GetMerchantCostItem',
            'cost-item accessor must run the GetMerchantCostItem getter')
        assert(data, 'cost tooltip data missing')
        assert(data.type == Enum.TooltipDataType.Item,
            'item cost identity type, got ' .. tostring(data.type))
        assert(data.id == 159,
            'cost index 2 must skip the empty identity, got ' .. tostring(data.id))
        assert(data.lines[1].leftText == 'Refreshing Spring Water',
            'item cost must publish the item lines')
        assert(GameTooltip:GetLeftLine(1):GetText() == 'Refreshing Spring Water',
            'item cost lines must reach the frame')
        "#,
    )
    .unwrap();
}

#[test]
fn merchant_cost_ordinals_skip_absent_and_unresolvable_entries() {
    let env = tooltip_full_env_helpers::setup_full_env();
    let mut state = env.state().borrow_mut();
    state.merchant_items = vec![229181];
    state.merchant_offer_costs.insert(
        0,
        MerchantCostRecord {
            // 999999 has no resident item, so it never consumes an ordinal.
            item_costs: [0, 6948, 999999, 159, 0],
            currency_costs: [0, 2245, 0, 0, 0],
        },
    );
    drop(state);

    env.exec(
        r#"
        local first = C_TooltipInfo.GetMerchantCostItem(1, 1)
        local second = C_TooltipInfo.GetMerchantCostItem(1, 2)
        local third = C_TooltipInfo.GetMerchantCostItem(1, 3)
        assert(first.id == 6948 and first.lines[1].leftText == 'Hearthstone',
            'ordinal 1 must be the first surviving item cost')
        assert(second.id == 159 and second.lines[1].leftText == 'Refreshing Spring Water',
            'ordinal 2 must skip the empty and the unresolvable identities')
        assert(third.type == Enum.TooltipDataType.Currency,
            'currency cost identity type, got ' .. tostring(third.type))
        assert(third.id == 2245, 'ordinal 3 must be the first currency cost')
        assert(third.lines[1].leftText == 'Valorstones',
            'currency cost must publish the currency lines')
        assert(third.quantity == 0,
            'merchant currency quantity must be zero, got ' .. tostring(third.quantity))
        assert(select('#', C_TooltipInfo.GetMerchantCostItem(1, 4)) == 0,
            'an ordinal past the selectable costs must produce no tooltip data')
        "#,
    )
    .unwrap();
}

#[test]
fn merchant_cost_misses_produce_no_tooltip_data() {
    let env = tooltip_full_env_helpers::setup_full_env();
    let mut state = env.state().borrow_mut();
    state.merchant_items = vec![229181, 229181, 229181];
    state
        .merchant_offer_costs
        .insert(0, MerchantCostRecord {
            item_costs: [6948, 0, 0, 0, 0],
            currency_costs: [0, 0, 0, 0, 0],
        });
    // Slot 2 is a purely monetary offer: no item or currency entry survives, so
    // it has no ordinal at all.
    state
        .merchant_offer_costs
        .insert(1, MerchantCostRecord::default());
    drop(state);

    env.exec(
        r#"
        local misses = {
            ['no currency ordinal'] = { 1, 2 },
            ['ordinal zero'] = { 1, 0 },
            ['pure money offer'] = { 2, 1 },
            ['offer without a record'] = { 3, 1 },
            ['slot without an offer'] = { 4, 1 },
        }
        for name, args in pairs(misses) do
            assert(select('#', C_TooltipInfo.GetMerchantCostItem(args[1], args[2])) == 0,
                name .. ' must produce no tooltip data')
        end
        GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
        GameTooltip:SetMerchantCostItem(2, 1)
        assert(GameTooltip:GetPrimaryTooltipData() == nil,
            'a missed cost must not replace the frame tooltip data')
        "#,
    )
    .unwrap();
}
