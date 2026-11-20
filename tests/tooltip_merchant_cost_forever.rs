#![cfg(any(feature = "profile-retail", feature = "client-wowforever"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::MerchantCostRecord;

fn seed_currency_cost(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.merchant_items = vec![229181];
    state.merchant_offer_costs.insert(
        0,
        MerchantCostRecord {
            currency_costs: [2245, 1792, 0, 0, 0],
            ..MerchantCostRecord::default()
        },
    );
}

#[test]
fn original_money_frame_currency_cost_hover_publishes_the_cost() {
    let env = tooltip_full_env_helpers::setup_full_env();
    seed_currency_cost(&env);

    env.exec(
        r#"
        assert(type(SmallDenominationTemplate_OnEnter) == 'function',
            'MoneyFrame cost consumer missing')
        local button = CreateFrame('Button', nil, UIParent)
        button.index = 1
        for ordinal, currencyID in ipairs({2245, 1792}) do
            button.item = ordinal
            SmallDenominationTemplate_OnEnter(button)
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(data, 'currency cost tooltip data missing')
            assert(data.type == Enum.TooltipDataType.Currency,
                'currency cost identity type, got ' .. tostring(data.type))
            assert(data.id == currencyID,
                'currency identity must follow the selected offer cost')
            local name = C_CurrencyInfo.GetCurrencyInfo(currencyID).name
            assert(data.lines[1].leftText == name,
                'currency cost must publish selected currency text')
            assert(GameTooltip:GetLeftLine(1):GetText() == name,
                'currency cost lines must reach the frame')
        end
        "#,
    )
    .unwrap();
}
