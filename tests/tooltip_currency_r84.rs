//! R84 Retail currency payloads through the original GameTooltip consumers.
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

fn retail_env() -> Option<WowLuaEnv> {
    (ACTIVE == ClientProfile::Retail).then(tooltip_full_env_helpers::setup_full_env)
}

fn xp_inputs(local_currency_record: bool) -> CurrencyTooltipInputs {
    CurrencyTooltipInputs {
        flags_0: Some(1 << 3),
        flags_1: Some(1 << 1),
        field_11_xp_quest_difficulty: Some(1),
        local_currency_record: Some(local_currency_record),
        ..CurrencyTooltipInputs::default()
    }
}

fn seed_currency_tooltip_inputs(env: &WowLuaEnv, inputs: CurrencyTooltipInputs) {
    env.state()
        .borrow_mut()
        .currency_tooltip_inputs
        .insert(2245, inputs);
}

/// `UIPanelTemplates.lua:176` calls this method; `TooltipDataHandler.lua:556` selects its getter.
#[test]
fn set_currency_by_id_preserves_amount_and_defaults_absent_to_zero() {
    let Some(env) = retail_env() else {
        return;
    };
    seed_currency_tooltip_inputs(&env, xp_inputs(true));
    env.exec(
        r#"
        local button = CreateFrame('Frame', nil, UIParent)
        button.currencyID = 2245
        CurrencyLayoutFrameIconMixin.OnEnter(button)

        local info = GameTooltip:GetPrimaryTooltipInfo()
        local data = GameTooltip:GetPrimaryTooltipData()
        local currencyName = C_CurrencyInfo.GetCurrencyInfo(2245).name
        assert(info and info.getterName == 'GetCurrencyByID', 'currency consumer must select GetCurrencyByID')
        assert(data and data.type == Enum.TooltipDataType.Currency, 'currency data type')
        assert(data and data.id == 2245, 'currency id')
        assert(data and data.dataInstanceID ~= nil, 'currency data instance id')
        assert(data.lines[1].leftText == currencyName and currencyName ~= '', 'currency title must use the resolved CurrencyTypes name')
        assert(data.lines[1].leftColor ~= nil and data.lines[1].wrapText == nil, 'currency title shape')
        assert(#data.lines == 3, 'XP row must end the producer')
        assert(data.lines[3].type == 14, 'currency XP row type')
        assert(data.lines[3].leftText == XP .. ': 0', 'absent currency amount must reach the producer as zero')
        assert(data.lines[3].wrapText == nil, 'XP row must not wrap')
        assert(data.quantity == nil, 'R84 §Q4: no top-level quantity')
        for _, line in ipairs(data.lines) do
            assert(line.rightText == nil and line.rightColor == nil, 'currency rows are left-only')
        end

        GameTooltip:SetCurrencyByID(2245, 99)
        data = GameTooltip:GetPrimaryTooltipData()
        assert(data.lines[3].leftText == XP .. ': 99', 'GetCurrencyByID must pass amount unchanged')
        assert(#data.lines == 3, 'XP row must prevent spacer rows')
        "#,
    )
    .unwrap();
}

/// `TooltipDataHandler.lua:590` maps SetHyperlink to GetHyperlink. No native currency-link caller was found.
#[test]
fn set_hyperlink_resolves_currency_links() {
    let Some(env) = retail_env() else {
        return;
    };
    seed_currency_tooltip_inputs(&env, xp_inputs(false));
    env.exec(
        r#"
        GameTooltip:SetHyperlink('currency:2245')
        local info = GameTooltip:GetPrimaryTooltipInfo()
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(info and info.getterName == 'GetHyperlink', 'SetHyperlink must select GetHyperlink')
        assert(data and data.type == Enum.TooltipDataType.Currency, 'currency hyperlink data type')
        assert(data and data.id == 2245, 'currency hyperlink id')
        assert(data and data.dataInstanceID ~= nil, 'currency hyperlink data instance id')
        assert(data and data.quantity == nil, 'R84 §Q4: no top-level quantity')
        assert(data.lines[1] and data.lines[1].leftText, 'currency hyperlink title')
        assert(#data.lines == 3, 'currency hyperlink XP row')
        assert(data.lines[3].leftText == XP .. ': 0', 'currency hyperlink without amount must pass zero')
        "#,
    )
    .unwrap();
}

/// AlertFrameSystems.lua:125 calls the original consumer. The map supplies the exact query result;
/// it does not model the unresolved reward/required/choice selector mapping.
#[test]
fn set_quest_log_currency_uses_selected_quest_and_normalizes_supplied_amount() {
    let Some(env) = retail_env() else {
        return;
    };
    let quest_id = 81_234;
    {
        let mut state = env.state().borrow_mut();
        state.selected_quest_log_id = Some(quest_id as u32);
        state.currency_tooltip_inputs.insert(2245, xp_inputs(true));
        state
            .quest_currency_tooltip_results
            .insert((quest_id, "reward".to_owned(), 1), (2245, 12_345));
    }
    env.exec(
        r#"
        GameTooltip:SetQuestLogCurrency('reward', 1)
        local info = GameTooltip:GetPrimaryTooltipInfo()
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(info and info.getterName == 'GetQuestLogCurrency', 'quest consumer must select GetQuestLogCurrency')
        assert(data and data.type == Enum.TooltipDataType.Currency, 'quest currency data type')
        assert(data and data.id == 2245, 'quest currency id')
        assert(#data.lines == 3, 'XP row must end the producer')
        assert(data.lines[3].type == 14, 'quest currency XP row type')
        assert(data.lines[3].leftText == XP .. ': 123', 'quest currency hundredths must divide with signed truncation')
        assert(data.quantity == nil, 'R84 §Q4: no top-level quantity')
        "#,
    )
    .unwrap();
}

#[test]
fn currency_account_labels_and_bit_22_control_the_spacer() {
    let Some(env) = retail_env() else {
        return;
    };
    seed_currency_tooltip_inputs(
        &env,
        CurrencyTooltipInputs {
            flags_0: Some(0),
            flags_1: Some(1 << 3),
            field_6_low_byte: Some(1 << 2),
            field_16_transfer_percentage: Some(1.0),
            local_currency_record: Some(true),
            ..CurrencyTooltipInputs::default()
        },
    );
    env.exec(
        r#"
        GameTooltip:SetCurrencyByID(2245, 0)
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(data and #data.lines >= 4, 'account labels are missing')
        assert(data.lines[2].leftText == ACCOUNT_LEVEL_CURRENCY, 'account-level row')
        assert(data.lines[3].leftText == ACCOUNT_TRANSFERRABLE_CURRENCY, 'transferable row')
        assert(data.lines[4].leftText == REPUTATION_TOOLTIP_ACCOUNT_WIDE_LABEL, 'reputation account-wide row')
        for index = 2, 4 do
            assert(data.lines[index].type == 0 and data.lines[index].leftColor ~= nil, 'account label row shape')
        end
        local hasSpacer = false
        for _, line in ipairs(data.lines) do
            if line.type == 1 then hasSpacer = true end
        end
        assert(hasSpacer, 'supplied local currency record should emit the spacer when bit 22 is clear')
        "#,
    )
    .unwrap();

    {
        let mut state = env.state().borrow_mut();
        let inputs = state
            .currency_tooltip_inputs
            .get_mut(&2245)
            .expect("currency tooltip input");
        inputs.flags_0 = Some(1 << 24);
        inputs.flags_1 = Some(0);
    }
    env.exec(
        r#"
        GameTooltip:SetCurrencyByID(2245, 0)
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(data.lines[2].leftText == ACCOUNT_LEVEL_CURRENCY, 'Flags_0 bit 24 should emit the account-level row')
        "#,
    )
    .unwrap();

    env.state()
        .borrow_mut()
        .currency_tooltip_inputs
        .get_mut(&2245)
        .expect("currency tooltip input")
        .flags_0 = Some(1 << 22);
    env.state()
        .borrow_mut()
        .currency_tooltip_inputs
        .get_mut(&2245)
        .expect("currency tooltip input")
        .flags_1 = Some(1 << 3);
    env.exec(
        r#"
        GameTooltip:SetCurrencyByID(2245, 0)
        local data = GameTooltip:GetPrimaryTooltipData()
        for _, line in ipairs(data.lines) do
            assert(line.type ~= 1, 'Flags_0 bit 22 must suppress the spacer')
        end
        "#,
    )
    .unwrap();
}
